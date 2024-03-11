use std::sync::{atomic::Ordering, Arc};

use zenis_common::{config, Probability};
use zenis_database::ZenisDatabase;
use zenis_discord::{
    twilight_gateway::Event,
    twilight_model::gateway::payload::incoming::{InteractionCreate, MessageCreate, Ready},
};
use zenis_framework::{watcher::Watcher, ZenisClient};

use crate::command_handler;

pub struct EventHandler {
    client: Arc<ZenisClient>,
    watcher: Arc<Watcher>,
    database: Arc<ZenisDatabase>,
}

impl EventHandler {
    pub fn new(
        client: Arc<ZenisClient>,
        watcher: Arc<Watcher>,
        database: Arc<ZenisDatabase>,
    ) -> Self {
        Self {
            client,
            watcher,
            database,
        }
    }

    pub async fn handle(self, event: Event) {
        self.watcher.process(&event);

        match event {
            Event::Ready(ready) => {
                if self.client.is_ready.load(Ordering::Relaxed) {
                    return;
                }

                let client = self.client.clone();
                let database = self.database.clone();

                self.ready(ready).await.unwrap();
                client.init(database).await.unwrap();
            }
            Event::InteractionCreate(interaction_create) => {
                self.interaction_create(interaction_create).await.ok();
            }
            Event::MessageCreate(message) => {
                self.message_create(message).await.ok();
            }
            _ => {}
        };
    }

    pub async fn ready(self, ready: Box<Ready>) -> anyhow::Result<()> {
        let current_user = self.client.current_user().await?;
        println!("{} is ready!", current_user.name);

        command_handler::register_commands(ready.application.id, self.client).await;

        Ok(())
    }

    pub async fn interaction_create(
        self,
        interaction: Box<InteractionCreate>,
    ) -> anyhow::Result<()> {
        command_handler::execute_command(interaction, self.client, self.watcher, self.database)
            .await
    }

    pub async fn message_create(self, message: Box<MessageCreate>) -> anyhow::Result<()> {
        let channel = self
            .client
            .http
            .channel(message.channel_id)
            .await?
            .model()
            .await?;

        if config::BOT_IDS.contains(&message.author.id.get()) {
            return Ok(());
        }

        let author = message.author.clone();
        let Some(agents) = self.client.get_agents(channel.id).await else {
            return Ok(());
        };

        for agent in agents {
            let mut agent = agent.1.write().await;

            #[allow(clippy::needless_bool_assign)]
            if message.author.bot {
                agent.awaiting_message = Probability::new(80).generate_random_bool();
            } else {
                agent.awaiting_message = false;
            }

            agent
                .enqueue_message(author.clone(), message.content.clone())
                .await;
            agent.last_received_message_timestamp = message.timestamp.as_secs();
            if message.author.bot {
                agent.last_received_message_timestamp += 3;
            }
        }

        Ok(())
    }
}
