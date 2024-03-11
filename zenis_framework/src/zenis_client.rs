use anyhow::bail;
use base64::Engine;
use std::{
    collections::{HashMap, HashSet},
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc,
    },
};
use tokio::sync::RwLock;
use zenis_ai::{claude_brain::ClaudeBrain, Agent, CreditsPaymentMethod};
use zenis_common::Color;
use zenis_database::{
    agent_model::{AgentModel, AgentPricing},
    ZenisDatabase,
};
use zenis_discord::{
    twilight_model::{
        guild::Guild,
        id::{
            marker::{ChannelMarker, GuildMarker, UserMarker},
            Id,
        },
        user::{CurrentUser, User},
    },
    DiscordHttpClient, EmbedBuilder,
};
use zenis_payment::mp::client::{MercadoPagoClient, Transaction, TransactionId};

async fn load_png_from_url(url: &str) -> anyhow::Result<String> {
    let client = reqwest::Client::new();
    let response = client.get(url).send().await?;

    let mime_type = response
        .headers()
        .get("Content-Type")
        .and_then(|ct| ct.to_str().ok())
        .unwrap_or("image/png")
        .to_owned();

    let response_bytes = response.bytes().await?;

    let engine = base64::engine::general_purpose::STANDARD;
    let base64_encoded = engine.encode(&response_bytes);
    let data_uri = format!("data:{};base64,{}", mime_type, base64_encoded);

    Ok(data_uri)
}

#[derive(Debug, Clone)]
pub struct ZenisAgent(pub usize, pub Arc<RwLock<Agent<ClaudeBrain>>>);

#[derive(Debug)]
pub struct ZenisClient {
    pub http: Arc<DiscordHttpClient>,
    pub is_ready: AtomicBool,
    pub users_fighting: RwLock<HashSet<Id<UserMarker>>>,

    pub agents: RwLock<HashMap<Id<ChannelMarker>, Vec<ZenisAgent>>>,
    pub mp_client: Arc<MercadoPagoClient>,
}

impl ZenisClient {
    pub fn new(token: String, mp: Arc<MercadoPagoClient>) -> Self {
        Self {
            http: Arc::new(DiscordHttpClient::new(token)),
            is_ready: AtomicBool::new(false),
            users_fighting: RwLock::new(HashSet::new()),

            agents: RwLock::new(HashMap::new()),
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

    pub async fn get_agents(&self, id: Id<ChannelMarker>) -> Option<Vec<ZenisAgent>> {
        self.agents.read().await.get(&id).cloned()
    }

    pub async fn get_transaction(&self, id: TransactionId) -> Option<Transaction> {
        self.mp_client.get_transaction(id).await
    }

    pub async fn delete_transaction(&self, id: TransactionId) -> anyhow::Result<()> {
        self.mp_client.delete_transaction(id).await
    }

    pub async fn create_agent(
        &self,
        channel_id: Id<ChannelMarker>,
        agent: AgentModel,
        pricing: AgentPricing,
        payment_method: CreditsPaymentMethod,
    ) -> anyhow::Result<()> {
        let image = match &agent.agent_url_image {
            Some(url) => load_png_from_url(url).await.ok(),
            None => None,
        };

        let webhook = self.http.create_webhook(channel_id, &agent.name)?;
        let webhook = match image {
            None => webhook.await?.model().await?,
            Some(image) => webhook.avatar(&image).await?.model().await?,
        };

        let Some(token) = webhook.token.clone() else {
            self.http.delete_webhook(webhook.id).await?;
            bail!("Failed to create a webhook")
        };

        let agent = Agent::new(
            channel_id,
            (token, webhook.id),
            agent,
            pricing,
            payment_method,
            ClaudeBrain,
        );
        self.insert_agent(channel_id, agent).await;

        Ok(())
    }

    pub async fn insert_agent(&self, channel_id: Id<ChannelMarker>, agent: Agent<ClaudeBrain>) {
        static AGENTS_COUNT: AtomicUsize = AtomicUsize::new(0);
        let id = AGENTS_COUNT.fetch_add(1, Ordering::Relaxed);

        let mut agents = self.agents.write().await;
        let agents = agents.entry(channel_id).or_default();

        agents.push(ZenisAgent(id, Arc::new(RwLock::new(agent))));
    }

    pub async fn delete_inactive_agents(&self) -> anyhow::Result<()> {
        let mut deletion_list = vec![];

        for agents in self.agents.read().await.values() {
            for agent in agents {
                let agent_id = agent.0;
                let agent = agent.1.read().await;
                if let Some(exit_reason) = agent.exit_reason.clone() {
                    deletion_list.push((agent_id, agent.webhook_id));

                    let embeds = vec![EmbedBuilder::new_common()
                        .set_color(Color::RED)
                        .set_description(format!(
                            "**{}** foi desligado.\n**Motivo:** `{exit_reason}`",
                            agent.agent_name
                        ))
                        .build()];
                    let embeds = &embeds;

                    if let Ok(create_message) =
                        self.http.create_message(agent.channel_id).embeds(embeds)
                    {
                        create_message.await.ok();
                    }
                }
            }
        }

        // Inner scope to drop the lock after deleting the webhooks and the agents
        {
            let mut agents = self.agents.write().await;
            for (agent_id, webhook_id) in deletion_list {
                self.http.delete_webhook(webhook_id).await.ok();

                for channel_agents in agents.values_mut() {
                    channel_agents.retain(|agent| agent.0 != agent_id);
                }
            }
        }

        self.agents
            .write()
            .await
            .retain(|_, agents| !agents.is_empty());

        Ok(())
    }

    pub async fn init(&self, _db: Arc<ZenisDatabase>) -> anyhow::Result<()> {
        self.is_ready.swap(true, Ordering::Relaxed);
        let current_user = self.current_user().await?;

        println!("-> Client initialized. Username: {}", current_user.name);

        Ok(())
    }

    pub async fn mark_user_as_fighter(&self, id: Id<UserMarker>) {
        self.users_fighting.write().await.insert(id);
    }

    pub async fn remove_user_fighting_mark(&self, id: Id<UserMarker>) {
        self.users_fighting.write().await.remove(&id);
    }

    pub async fn is_user_fighting(&self, id: Id<UserMarker>) -> bool {
        self.users_fighting.read().await.contains(&id)
    }
}
