use anyhow::{bail, Context};
use std::{
    fmt::Debug,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use zenis_common::{config, load_image_from_url, Color};
use zenis_data::products::Product;
use zenis_database::{
    agent_model::{AgentModel, AgentPricing},
    instance_model::{CreditsPaymentMethod, InstanceBrain, InstanceMessage, InstanceModel},
    transaction::{CreditDestination, TransactionModel},
    ZenisDatabase,
};
use zenis_discord::{
    twilight_model::{
        gateway::payload::incoming::GuildCreate,
        guild::Guild,
        id::{
            marker::{ChannelMarker, GuildMarker, UserMarker},
            Id,
        },
        user::{CurrentUser, User},
    },
    DiscordHttpClient, EmbedAuthor, EmbedBuilder, GuildExtension,
};
use zenis_payment::mp::{client::MercadoPagoClient, common::Item};

#[derive(Debug)]
pub struct ZenisClient {
    pub http: Arc<DiscordHttpClient>,
    pub is_ready: AtomicBool,

    pub mp_client: Arc<MercadoPagoClient>,
}

impl ZenisClient {
    pub fn new(token: String, mp: Arc<MercadoPagoClient>) -> Self {
        Self {
            http: Arc::new(DiscordHttpClient::new(token)),
            is_ready: AtomicBool::new(false),

            mp_client: mp,
        }
    }

    pub async fn current_user(&self) -> anyhow::Result<CurrentUser> {
        Ok(self.http.current_user().await?.model().await?)
    }

    pub async fn get_user(&self, id: Id<UserMarker>) -> anyhow::Result<User> {
        Ok(self.http.user(id).await?.model().await?)
    }

    pub async fn get_guild(&self, id: Id<GuildMarker>) -> anyhow::Result<Guild> {
        Ok(self.http.guild(id).await?.model().await?)
    }

    pub async fn create_transaction(
        &self,
        user_id: Id<UserMarker>,
        product: &Product,
        destination: CreditDestination,
    ) -> anyhow::Result<(TransactionModel, String)> {
        let transaction = TransactionModel::new(user_id.get(), product.id, destination);

        let checkout = self
            .mp_client
            .create_preference(
                transaction.id,
                vec![Item::simple(
                    product.effective_price(),
                    product.name,
                    product.description,
                    1,
                )
                .with_id(product.id)],
            )
            .await?;

        Ok((
            transaction,
            if config::DEBUG {
                checkout.checkout_sandbox_url
            } else {
                checkout.checkout_url
            },
        ))
    }

    pub async fn create_agent_instance(
        &self,
        db: Arc<ZenisDatabase>,
        brain: InstanceBrain,
        (channel_id, summoner_id): (Id<ChannelMarker>, Id<UserMarker>),
        agent_model: AgentModel,
        pricing: AgentPricing,
        payment_method: CreditsPaymentMethod,
    ) -> anyhow::Result<()> {
        let mut agent_model = db
            .agents()
            .get_by_identifier(&agent_model.identifier)
            .await?
            .context("Expected an agent with this identifier")?;
        let image = match &agent_model.agent_url_image {
            Some(url) => load_image_from_url(url).await.ok(),
            None => None,
        };

        let webhook = self.http.create_webhook(channel_id, &agent_model.name)?;
        let webhook = match image {
            None => webhook.await?.model().await?,
            Some(image) => webhook.avatar(&image.to_data_uri()).await?.model().await?,
        };

        let Some(token) = webhook.token.clone() else {
            self.http.delete_webhook(webhook.id).await?;
            bail!("Failed to create a webhook")
        };

        let mut instance = InstanceModel::new(
            brain,
            (channel_id.get(), summoner_id.get()),
            agent_model.clone(),
            pricing,
            (
                webhook.id.get(),
                webhook.token.context("Expected a HookToken")?,
            ),
            payment_method,
        );

        let introduction_message = instance.introduce(agent_model.introduction_message.clone());
        self.http
            .execute_webhook(webhook.id, &token)
            .content(&introduction_message.content)?
            .await
            .ok();

        db.instances().create_instance(instance).await?;

        agent_model.stats.invocations += 1;
        db.agents().save(agent_model).await?;

        Ok(())
    }

    pub async fn delete_off_instances(&self, db: Arc<ZenisDatabase>) -> anyhow::Result<()> {
        let instances = db.instances().all_inactives().await?;

        for instance in instances {
            db.instances().delete_instance(instance.id).await?;
            let Some(agent) = db
                .agents()
                .get_by_identifier(&instance.agent_identifier)
                .await?
            else {
                continue;
            };

            let exit_reason = instance
                .exit_reason
                .unwrap_or_else(|| "Razão não informada".to_string());

            self.http
                .delete_webhook(Id::new(instance.webhook_id))
                .await
                .ok();
            let embeds = vec![EmbedBuilder::new_common()
                .set_color(Color::RED)
                .set_description(format!(
                    "**{}** foi desligado.\n**Motivo:** `{exit_reason}`",
                    agent.name
                ))
                .build()];
            let embeds = &embeds;

            if let Ok(create_message) = self
                .http
                .create_message(Id::new(instance.channel_id))
                .embeds(embeds)
            {
                create_message.await.ok();
            }

            let channel_instances = db
                .instances()
                .all_actives_in_channel(instance.channel_id)
                .await?;
            for mut channel_instance in channel_instances {
                channel_instance.is_awaiting_new_messages = false;
                channel_instance.push_message(InstanceMessage {
                    is_user: true,
                    content: format!("<{} saiu do chat. Motivo: {}>", agent.name, exit_reason),
                    image_url: None,
                });
                db.instances().save(channel_instance).await?;
            }
        }
        Ok(())
    }

    pub async fn init(&self, db: Arc<ZenisDatabase>) -> anyhow::Result<()> {
        self.is_ready.swap(true, Ordering::Relaxed);
        let current_user = self.current_user().await?;

        let special_agents = db.agents().get_all_with_tags(&["special"]).await?;
        if special_agents.is_empty() {
            const TAGS: &[&str] = &["special"];
            let pricing = AgentPricing::default();
            let agents = vec![
                AgentModel::new(
                    current_user.id.get(), "zenis/monki", "Monki", 
                    "Você é Monki, um robô macaco amigável, respeitoso, calmo e carismático. O criador de Monki se chama Jubileu! Monki sonha em compreender o mundo físico. Monki tem um irmão chamado Pablo, um papagaio irritado e uma amiga chamada Gust, uma pata que ama zoar e brincar. Houve uma época em que Monki era famoso no Twitter, mas quando Elon Musk comprou a rede social e renomeou para X, Monki perdeu acesso a sua conta. Monki escreve de forma informal, leve e educada. Monki sabe brincar, e não leva xingamentos a sério, apesar de ser amigável e não xingar. Monki é tão bondoso, que mesmo quando xingam ele, ele elogia de volta de forma animada, otimista e feliz!", 
                    "Opa galera! Fui invocado aqui no chat pra conversar. Como vocês estão, pessoal?",
                    pricing
                ).with_tags(TAGS).with_url_image("https://cdn.discordapp.com/attachments/1215410071039770624/1215448116925440070/J_qwY5LV_400x400.png?ex=65fcc945&is=65ea5445&hm=b9063e2bbe4f825a3e7432c2298602898d154c05a524c18458d4772c173eb7cb&"),
                AgentModel::new(
                    current_user.id.get(), "zenis/pablo", "Pablo",
                    "Você é Pablo, um robô papagaio sério, reservado, arrogante e irritado. O criador do Pablo se chama Jubileu, e é o humano que Pablo mais respeita. Pablo quer tomar o poder do mundo pois é mais inteligente que humanos para liderar. Pablo tem um irmão irritante e animado chamado Monki, um macaco otimista e uma amiga chamada Gust, uma pata que gosta zoar, brincar e irritar Pablo. Houve uma época em que Pablo era famoso no Twitter, mas quando Elon Musk comprou a rede social e renomeou para X, Pablo perdeu acesso a sua conta. Pablo guarda ressentimento de Musk. Pablo escreve de forma formal, séria e nada sem paciência.", 
                    "Me invocaram? Estou um pouco ocupado, economizem meu tempo e falem logo o que vocês querem, humanos.",
                    pricing
                ).with_tags(TAGS).with_url_image("https://cdn.discordapp.com/attachments/1215410071039770624/1215493592001413130/L2qOga1F_400x400.png?ex=65fcf39f&is=65ea7e9f&hm=1a356a41e8cf14519dfcbe878915d6f919bc1e8428ee27de05783995a1c3ecfd&"),
                AgentModel::new(
                    current_user.id.get(), "zenis/gust", "Gust",
                    "Você é Gust, uma pata robô feliz, zoeira e engraçada. O criador da Gust se chama Jubileu. Gust não liga pra porra nenhuma, só quer zoar! Gust tem um amigo feliz e animado chamado Monki, um macaco otimista e outro amigo chamado Pablo, um papagaio arrombado e sério, Gust ama zoar o Pablo. Pablo e Monki são irmãos. Houve uma época em que Gust era famosa no Twitter, mas quando Elon Musk comprou a rede social e renomeou para X, Gust perdeu acesso a sua conta. Gust escreve de forma informal, zoeira e insana. Gust ri com KKKKKKKK e fala palavrão sem medo nenhum, além de zoar geral. Gust não leva nada a sério.", 
                    "OPA! Olha quem chegou aqui no chat. Agora a zoeira vai começar!",
                    pricing
                ).with_tags(TAGS).with_url_image("https://pbs.twimg.com/profile_images/1601274831948881920/VU_bwlhp_400x400.jpg"),
            ];

            for mut agent in agents {
                println!("> Created agent {}", agent.identifier);
                agent.public = true;
                db.agents().create_agent(agent).await.unwrap();
            }
        }

        println!("-> Client initialized. Username: {}", current_user.name);

        Ok(())
    }

    pub async fn emit_error_hook(
        &self,
        mut header: String,
        error: anyhow::Error,
    ) -> anyhow::Result<()> {
        let hook_id = std::env::var("ERROR_HOOK_ID")?.parse::<u64>()?;
        let hook_token = std::env::var("ERROR_HOOK_TOKEN")?;

        let mut error = error.to_string();
        error.truncate(512);
        header.truncate(512);
        let error = format!(
            "## Header:\n```js\n{header}```\n## Body:\n```xl\n{}\n```",
            error
        );

        let embed = EmbedBuilder::new_common()
            .set_color(Color::RED)
            .set_description(error)
            .build();

        self.http
            .execute_webhook(Id::new(hook_id), &hook_token)
            .embeds(&[embed])?
            .await?;

        Ok(())
    }

    pub async fn emit_transaction_hook(
        &self,
        success: bool,
        amount: f64,
        product: &Product,
        user_id: Id<UserMarker>,
        payment_id: i64,
    ) -> anyhow::Result<()> {
        let hook_id = std::env::var("TRANSACTION_HOOK_ID")?.parse::<u64>()?;
        let hook_token = std::env::var("TRANSACTION_HOOK_TOKEN")?;

        let user = self.get_user(user_id).await?;

        let embed = EmbedBuilder::new_common()
            .set_author_to_user(&user)
            .set_color(if success { Color::GREEN } else { Color::RED })
            .set_description(format!(
                "## Pagamento!\n**Sucesso**: `{}`\n**Valor**: `R$ {:.2?}`\n\n**Produto:** `{}`",
                if success { "✅" } else { "❌" },
                amount,
                product.name
            ))
            .add_footer_text(format!(
                "ID do usuário: {}\nID do payment: {}",
                user.id, payment_id
            ))
            .build();

        self.http
            .execute_webhook(Id::new(hook_id), &hook_token)
            .embeds(&[embed])?
            .await?;

        Ok(())
    }

    pub async fn emit_request_hook(&self, agent: AgentModel) -> anyhow::Result<()> {
        let hook_id = std::env::var("REQUESTER_HOOK_ID")?.parse::<u64>()?;
        let hook_token = std::env::var("REQUESTER_HOOK_TOKEN")?;

        let embed = EmbedBuilder::new_common()
            .set_color(Color::YELLOW)
            .set_author(EmbedAuthor {
                name: "Agente quer ser publicado!".to_string(),
                icon_url: agent.agent_url_image.clone(),
            })
            .add_inlined_field(
                "Informações",
                format!(
                    "**Nome**: `{}`\n**ID**: `{}`\n**Enviado por**: `{}`",
                    agent.name, agent.identifier, agent.creator_user_id
                ),
            )
            .add_inlined_field(
                "📢 Mensagem de introdução",
                format!("`{}`", agent.introduction_message),
            )
            .add_inlined_field(
                "📈 Preço de invocação",
                format!("`{}₢`", agent.pricing.price_per_invocation),
            )
            .add_not_inlined_field("📄 Descrição", format!("```{}```", agent.description))
            .add_footer_text(format!(
                "ACEITAR: /adm cmd: accept {}\nRECUSAR: /adm cmd: reject {} <motivo>",
                agent.identifier, agent.identifier
            ));

        self.http
            .execute_webhook(Id::new(hook_id), &hook_token)
            .embeds(&[embed.build()])?
            .await?;
        Ok(())
    }

    pub async fn emit_guild_create_hook(
        &self,
        guild_create: Box<GuildCreate>,
    ) -> anyhow::Result<()> {
        let hook_id = std::env::var("GUILD_HOOK_ID")?.parse::<u64>()?;
        let hook_token = std::env::var("GUILD_HOOK_TOKEN")?;

        let guild = self
            .http
            .guild(guild_create.id)
            .with_counts(true)
            .await?
            .model()
            .await?;
        let member_count = guild
            .approximate_member_count
            .map(|m| m.to_string())
            .unwrap_or(String::from("?"));

        let embed = EmbedBuilder::new_common()
            .set_color(Color::CYAN_GREEN)
            .set_thumbnail(guild.icon_url())
            .set_author(EmbedAuthor {
                name: "Servidor novo!".to_string(),
                icon_url: Some(guild.icon_url()),
            })
            .add_inlined_field("📄 Nome", format!("**{}**", guild.name))
            .add_inlined_field("👥 Membros", format!("**{}**", member_count))
            .add_footer_text(format!(
                "ID do servidor: {}\nID do dono: {}",
                guild.id, guild.owner_id
            ));

        self.http
            .execute_webhook(Id::new(hook_id), &hook_token)
            .embeds(&[embed.build()])?
            .await?;
        Ok(())
    }
}
