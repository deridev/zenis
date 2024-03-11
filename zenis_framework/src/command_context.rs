use std::sync::Arc;

use anyhow::Context;
use zenis_database::ZenisDatabase;
use zenis_discord::{
    application_command::CommandDataOption,
    twilight_http::{client::InteractionClient, Response as ApiResponse},
    twilight_model::{
        channel::Message,
        http::{
            attachment::Attachment,
            interaction::{InteractionResponse, InteractionResponseType},
        },
        id::{marker::UserMarker, Id},
        user::User,
    },
    Interaction, InteractionData,
};

use crate::{watcher::Watcher, CommandContextHelper, OptionHandler, Response, ZenisClient};

#[derive(Debug, Clone)]
pub struct CommandContext {
    pub client: Arc<ZenisClient>,
    pub interaction: Box<Interaction>,
    pub watcher: Arc<Watcher>,
    database: Arc<ZenisDatabase>,
    pub(crate) options: Vec<CommandDataOption>,

    pub already_replied: bool,
}

impl CommandContext {
    pub fn new(
        client: Arc<ZenisClient>,
        interaction: Box<Interaction>,
        watcher: Arc<Watcher>,
        database: Arc<ZenisDatabase>,
        options: Vec<CommandDataOption>,
    ) -> Self {
        Self {
            client,
            interaction,
            watcher,
            database,
            options,

            already_replied: false,
        }
    }

    pub fn from_with_interaction(ctx: &CommandContext, interaction: Box<Interaction>) -> Self {
        Self {
            client: ctx.client.clone(),
            watcher: ctx.watcher.clone(),
            database: ctx.database.clone(),
            already_replied: false,

            options: interaction
                .data
                .clone()
                .map(|data| match data {
                    InteractionData::ApplicationCommand(data) => data.options.clone(),
                    _ => vec![],
                })
                .unwrap_or_default(),

            interaction,
        }
    }

    pub fn interaction_client(&self) -> InteractionClient<'_> {
        self.client
            .http
            .interaction(self.interaction.application_id)
    }

    pub fn author_id(&self) -> Id<UserMarker> {
        self.interaction.author_id().unwrap()
    }

    pub fn options(&self) -> OptionHandler {
        OptionHandler { ctx: self }
    }

    pub fn helper(&mut self) -> CommandContextHelper {
        CommandContextHelper { ctx: self }
    }

    pub fn db(&self) -> Arc<ZenisDatabase> {
        self.database.clone()
    }

    pub async fn author(&self) -> anyhow::Result<User> {
        let id = self.author_id();
        let user = self.client.http.user(id).await?.model().await?;

        Ok(user)
    }

    pub async fn fetch_interaction_reply(&self) -> anyhow::Result<Message> {
        if !self.already_replied {
            anyhow::bail!("This CommandContext didn't replied yet");
        }

        let response = self
            .interaction_client()
            .response(&self.interaction.token)
            .await?;

        Ok(response.model().await?)
    }

    pub async fn reply(&mut self, response: impl Into<Response>) -> anyhow::Result<()> {
        if self.already_replied {
            self.followup_interaction(response).await?;
        } else {
            self.reply_interaction(response).await?;
        }

        Ok(())
    }

    pub async fn send(&mut self, response: impl Into<Response>) -> anyhow::Result<Message> {
        if self.already_replied {
            Ok(self.send_in_channel(response).await?.model().await?)
        } else {
            self.reply_interaction(response).await?;
            self.fetch_interaction_reply().await
        }
    }

    pub async fn send_in_channel(
        &mut self,
        response: impl Into<Response>,
    ) -> anyhow::Result<ApiResponse<Message>> {
        let channel = self
            .interaction
            .channel
            .clone()
            .context("Channel not found")?;
        let response = response.into();
        let json = response.to_json();

        Ok(self
            .client
            .http
            .create_message(channel.id)
            .payload_json(&json)
            .await?)
    }

    pub async fn reply_interaction(&mut self, response: impl Into<Response>) -> anyhow::Result<()> {
        self.already_replied = true;
        let response = response.into();

        self.interaction_client()
            .create_response(
                self.interaction.id,
                &self.interaction.token,
                &InteractionResponse {
                    kind: InteractionResponseType::ChannelMessageWithSource,
                    data: Some(response.into()),
                },
            )
            .await?;

        Ok(())
    }

    pub async fn followup_interaction(
        &mut self,
        response: impl Into<Response>,
    ) -> anyhow::Result<ApiResponse<Message>> {
        let response = response.into();
        let json = response.to_json();

        Ok(self
            .interaction_client()
            .create_followup(&self.interaction.token)
            .payload_json(&json)
            .await?)
    }

    pub async fn update_interaction_reply(
        &self,
        response: impl Into<Response>,
    ) -> anyhow::Result<ApiResponse<Message>> {
        if !self.already_replied {
            anyhow::bail!("This CommandContext didn't replied yet");
        }

        let response = response.into();
        let json = response.to_json();

        Ok(self
            .interaction_client()
            .update_response(&self.interaction.token)
            .payload_json(&json)
            .await?)
    }

    pub async fn delete_reply_message(&self) -> anyhow::Result<()> {
        self.interaction_client()
            .delete_response(&self.interaction.token)
            .await?;

        Ok(())
    }

    pub async fn update_message(&mut self, response: impl Into<Response>) -> anyhow::Result<()> {
        self.already_replied = true;
        let response = response.into();

        self.interaction_client()
            .create_response(
                self.interaction.id,
                &self.interaction.token,
                &InteractionResponse {
                    kind: InteractionResponseType::UpdateMessage,
                    data: Some(response.into()),
                },
            )
            .await?;

        Ok(())
    }

    pub async fn update_specific_message(
        &self,
        message: &Message,
        response: impl Into<Response>,
    ) -> anyhow::Result<()> {
        let response = response.into();

        const EMPTY_ARRAY: &[Attachment] = &[];
        let attachments = match response.attachments.as_ref() {
            Some(s) => s,
            None => EMPTY_ARRAY,
        };

        self.client
            .http
            .update_message(message.channel_id, message.id)
            .attachments(attachments)?
            .payload_json(&response.clone().to_json())
            .await?;

        Ok(())
    }
}
