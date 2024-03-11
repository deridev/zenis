use std::time::Duration;

use once_cell::sync::Lazy;
use zenis_ai::PaymentMethod;
use zenis_database::agent_model::{AgentModel, AgentPricing};
use zenis_discord::twilight_model::channel::message::component::ButtonStyle;
use zenis_framework::{util::make_multiple_rows, watcher::WatcherOptions};

use crate::prelude::*;

static DEFAULT_AGENTS: Lazy<Vec<AgentModel>> = Lazy::new(|| {
    vec![
        AgentModel {
            agent_url_image: Some("https://cdn.discordapp.com/attachments/1215410071039770624/1215448116925440070/J_qwY5LV_400x400.png?ex=65fcc945&is=65ea5445&hm=b9063e2bbe4f825a3e7432c2298602898d154c05a524c18458d4772c173eb7cb&".to_string()),
            name: "Monki".to_string(),
            description: "Você é Monki, um robô macaco amigável, respeitoso, calmo e carismático. O criador de Monki se chama Jubileu! Monki sonha em compreender o mundo físico. Monki tem um irmão chamado Pablo, um papagaio irritado e uma amiga chamada Gust, uma pata que ama zoar e brincar. Houve uma época em que Monki era famoso no Twitter, mas quando Elon Musk comprou a rede social e renomeou para X, Monki perdeu acesso a sua conta. Monki escreve de forma informal, leve e educada. Monki sabe brincar, e não leva xingamentos a sério, apesar de ser amigável e não xingar. Monki é tão bondoso, que mesmo quando xingam ele, ele elogia de volta de forma animada, otimista e feliz!"
            .to_string(),
            pricing: AgentPricing::default(),
        },
        AgentModel {
            agent_url_image: Some("https://cdn.discordapp.com/attachments/1215410071039770624/1215493592001413130/L2qOga1F_400x400.png?ex=65fcf39f&is=65ea7e9f&hm=1a356a41e8cf14519dfcbe878915d6f919bc1e8428ee27de05783995a1c3ecfd&".to_string()),
            name: "Pablo".to_string(),
            description: "Você é Pablo, um robô papagaio sério, reservado, arrogante e irritado. O criador do Pablo se chama Jubileu, e é o humano que Pablo mais respeita. Pablo quer tomar o poder do mundo pois é mais inteligente que humanos para liderar. Pablo tem um irmão irritante e animado chamado Monki, um macaco otimista e uma amiga chamada Gust, uma pata que gosta zoar, brincar e irritar Pablo. Houve uma época em que Pablo era famoso no Twitter, mas quando Elon Musk comprou a rede social e renomeou para X, Pablo perdeu acesso a sua conta. Pablo guarda ressentimento de Musk. Pablo escreve de forma formal, séria e nada sem paciência."
            .to_string(),
            pricing: AgentPricing::default(),

        },
        AgentModel {
            agent_url_image: Some("https://pbs.twimg.com/profile_images/1601274831948881920/VU_bwlhp_400x400.jpg".to_string()),
            name: "Gust".to_string(),
            description: "Você é Gust, uma pata robô feliz, zoeira e engraçada. O criador da Gust se chama Jubileu. Gust não liga pra porra nenhuma, só quer zoar! Gust tem um amigo feliz e animado chamado Monki, um macaco otimista e outro amigo chamado Pablo, um papagaio arrombado e sério, Gust ama zoar o Pablo. Pablo e Monki são irmãos. Houve uma época em que Gust era famosa no Twitter, mas quando Elon Musk comprou a rede social e renomeou para X, Gust perdeu acesso a sua conta. Gust escreve de forma informal, zoeira e insana. Gust ri com KKKKKKKK e fala palavrão sem medo nenhum, além de zoar geral. Gust não leva nada a sério."
            .to_string(),
            pricing: AgentPricing::default(),
        },
    ]
});

#[command("Invoque um agente de IA no chat para conversar com você!")]
#[name("invocar")]
pub async fn invoke(mut ctx: CommandContext) -> anyhow::Result<()> {
    let Some(channel) = ctx.interaction.channel.clone() else {
        return Ok(());
    };

    let author = ctx.author().await?;
    let author_id = author.id;

    if ctx
        .client
        .get_agents(channel.id)
        .await
        .is_some_and(|a| a.len() >= 2)
    {
        ctx.reply(
            Response::new_user_reply(&author, "já há muitos agentes neste chat!")
                .add_emoji_prefix(emojis::ERROR),
        )
        .await?;
        return Ok(());
    }

    let mut buttons = vec![];
    for agent in DEFAULT_AGENTS.iter() {
        buttons.push(
            ButtonBuilder::new()
                .set_custom_id(&agent.name)
                .set_label(&agent.name),
        );
    }

    let message = ctx
        .send(
            Response::new_user_reply(&author, "escolha um agente para invocar nesse chat:")
                .set_components(make_multiple_rows(buttons.clone())),
        )
        .await?;

    let Ok(Some(interaction)) = ctx
        .watcher
        .await_single_component(
            message.id,
            move |interaction| interaction.author_id() == Some(author_id),
            WatcherOptions {
                timeout: Duration::from_secs(60),
            },
        )
        .await
    else {
        return Ok(());
    };

    let data = interaction.parse_message_component_data()?;

    let buttons = buttons
        .iter()
        .map(|b| {
            let id = b.data.custom_id.as_ref();
            b.clone()
                .set_disabled(true)
                .set_style(if id == Some(&data.custom_id) {
                    ButtonStyle::Success
                } else {
                    ButtonStyle::Secondary
                })
        })
        .collect::<Vec<_>>();

    let mut ctx = CommandContext::from_with_interaction(&ctx, Box::new(interaction));
    ctx.update_message(Response::default().set_components(make_multiple_rows(buttons)))
        .await?;

    if ctx
        .client
        .get_agents(channel.id)
        .await
        .is_some_and(|a| a.len() >= 2)
    {
        ctx.send(
            Response::new_user_reply(&author, "já há muitos agentes neste chat!")
                .add_emoji_prefix(emojis::ERROR),
        )
        .await?;
        return Ok(());
    }

    let Some(agent) = DEFAULT_AGENTS.iter().find(|a| a.name == data.custom_id) else {
        return Ok(());
    };

    let mut payment_method = PaymentMethod::UserCredits(author_id);

    if let Some(guild_id) = channel.guild_id {
        if let Some(method) = ask_for_payment_method(&mut ctx, agent, author_id, guild_id).await? {
            payment_method = method;
        }
    }

    let result = ctx
        .client
        .create_agent(channel.id, agent.clone(), agent.pricing, payment_method)
        .await;
    if result.is_err() {
        ctx.send(
            Response::new_user_reply(&author, "O agente não foi criado. Tente novamente mais tarde ou certifique-se que eu tenho permissão de **Criar Webhooks** neste chat.")
                .add_emoji_prefix(emojis::ERROR),
        )
        .await?;
        return Ok(());
    }

    ctx.send("Agente criado. Envie mensagens e o agente responderá.")
        .await?;

    Ok(())
}

async fn ask_for_payment_method(
    ctx: &mut CommandContext,
    agent: &AgentModel,
    author_id: Id<UserMarker>,
    guild_id: Id<GuildMarker>,
) -> anyhow::Result<Option<PaymentMethod>> {
    let guild_data = ctx.db().guilds().get_by_guild(guild_id).await?;
    let user_data = ctx.db().users().get_by_user(author_id).await?;

    let buttons = vec![
        ButtonBuilder::new()
            .set_custom_id("cancel")
            .set_label("Cancelar")
            .set_style(ButtonStyle::Danger),
        ButtonBuilder::new()
            .set_custom_id("user")
            .set_label("Carteira")
            .set_style(ButtonStyle::Secondary),
        ButtonBuilder::new()
            .set_custom_id("guild")
            .set_label("Créditos do Servidor")
            .set_style(ButtonStyle::Secondary),
    ];

    let embed = EmbedBuilder::new_common()
        .set_color(Color::YELLOW)
        .set_description(format!(
            "## {} Escolha quem irá pagar o agente.\nPreço por resposta de **{}**: `{}₢`",
            emojis::CREDIT,
            agent.name,
            agent.pricing.price_per_reply
        ))
        .add_inlined_field("Sua Carteira", format!("{}₢", user_data.credits))
        .add_inlined_field(
            "Créditos Públicos do Servidor",
            format!("{}₢", guild_data.public_credits),
        );

    let message = ctx
        .send(Response::from(embed).set_components(make_multiple_rows(buttons.clone())))
        .await?;

    let Ok(Some(interaction)) = ctx
        .watcher
        .await_single_component(
            message.id,
            move |interaction| interaction.author_id() == Some(author_id),
            WatcherOptions {
                timeout: Duration::from_secs(60),
            },
        )
        .await
    else {
        return Ok(None);
    };

    let data = interaction.parse_message_component_data()?;

    let buttons = buttons
        .iter()
        .map(|b| {
            let id = b.data.custom_id.as_ref();
            b.clone()
                .set_disabled(true)
                .set_style(if id == Some(&data.custom_id) {
                    ButtonStyle::Success
                } else {
                    ButtonStyle::Secondary
                })
        })
        .collect::<Vec<_>>();

    let mut ctx = CommandContext::from_with_interaction(ctx, Box::new(interaction));
    ctx.update_message(Response::default().set_components(make_multiple_rows(buttons)))
        .await?;

    if data.custom_id == "user" {
        return Ok(Some(PaymentMethod::UserCredits(author_id)));
    } else if data.custom_id == "guild" {
        return Ok(Some(PaymentMethod::GuildPublicCredits(guild_id)));
    }

    Ok(None)
}
