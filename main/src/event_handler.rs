use std::sync::{atomic::Ordering, Arc};

use zenis_common::{config, Probability};
use zenis_database::{instance_model::InstanceMessage, user_model::UserFlags, ZenisDatabase};
use zenis_discord::{
    twilight_gateway::Event,
    twilight_model::gateway::payload::incoming::{
        GuildCreate, InteractionCreate, MessageCreate, Ready,
    },
    UserExtension,
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
            Event::GuildCreate(guild_create) => {
                self.guild_create(guild_create).await.ok();
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
        let mut instances = self
            .database
            .instances()
            .get_all_by_channel(channel.id.get())
            .await?;

        for instance in instances.iter_mut() {
            if instance.agent_name == author.display_name() && author.bot {
                continue;
            }

            let mut content = message.content.clone();
            content.truncate(1024);

            instance.push_message(InstanceMessage {
                is_user: true,
                content: format!(
                    "<{} (@{})>: {content}",
                    message.author.display_name(),
                    message.author.name
                ),
            });
            instance.is_awaiting_new_messages = false;

            if author.bot && Probability::new(20).generate_random_bool() {
                instance.is_awaiting_new_messages = true;
            }

            self.database.instances().save(instance.clone()).await?;
        }

        Ok(())
    }

    pub async fn guild_create(self, guild_create: Box<GuildCreate>) -> anyhow::Result<()> {
        let mut guild_data = self.database.guilds().get_by_guild(guild_create.id).await?;

        let mut owner_data = self
            .database
            .users()
            .get_by_user(guild_create.owner_id)
            .await?;

        if guild_data.public_credits == 0
            && !owner_data.has_flag(UserFlags::AlreadyReceivedFreeGuildCredits)
        {
            owner_data.insert_flag(UserFlags::AlreadyReceivedFreeGuildCredits);
            guild_data.add_public_credits(250);
            self.database.guilds().save(guild_data).await?;
            self.database.users().save(owner_data).await?;
        }

        self.client.emit_guild_create_hook(guild_create).await?;

        Ok(())
    }
}
