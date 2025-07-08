use std::time::Duration;

use zenis_discord::{
    twilight_model::{
        channel::message::component::ButtonStyle,
        id::{marker::UserMarker, Id},
    },
    ActionRowBuilder, ButtonBuilder, Emoji, InteractionExtension, ModalBuilder, ModalResponse,
};

use crate::{watcher::WatcherOptions, CommandContext, Response};

pub struct CommandContextHelper<'a> {
    pub ctx: &'a mut CommandContext,
}

impl<'a> CommandContextHelper<'a> {
    pub async fn create_confirmation(
        &mut self,
        user_id: Id<UserMarker>,
        delete_after_interaction: bool,
        response: impl Into<Response>,
    ) -> anyhow::Result<bool> {
        let response: Response = response.into();

        let buttons = vec![
            ButtonBuilder::new()
                .set_custom_id("yes")
                .set_style(ButtonStyle::Secondary)
                .set_emoji(Emoji::from_unicode("✅")),
            ButtonBuilder::new()
                .set_custom_id("no")
                .set_style(ButtonStyle::Secondary)
                .set_emoji(Emoji::from_unicode("❌")),
        ];

        let response =
            response.set_components(vec![ActionRowBuilder::new().add_buttons(buttons.clone())]);

        let message = self.ctx.send(response.clone()).await?;
        let component = self
            .ctx
            .watcher
            .await_single_component(
                message.id,
                move |interaction| interaction.author_id() == Some(user_id),
                WatcherOptions {
                    timeout: Duration::from_secs(30),
                },
            )
            .await;
        let Ok(Some(component)) = component else {
            return Ok(false);
        };

        let data = component.parse_message_component_data()?;

        let mut component_context =
            CommandContext::from_with_interaction(self.ctx, Box::new(component));
        component_context
            .update_message(response.set_components(vec![
                    ActionRowBuilder::new().add_buttons(
                        buttons
                            .into_iter()
                            .map(|button| {
                                if button.data.custom_id.as_ref() == Some(&data.custom_id) {
                                    button.set_style(ButtonStyle::Success)
                                } else {
                                    button.set_style(ButtonStyle::Secondary)
                                }
                                .set_disabled(true)
                            })
                            .collect(),
                    ),
                ]))
            .await?;

        if delete_after_interaction {
            self.ctx
                .client
                .http
                .delete_message(message.channel_id, message.id)
                .await
                .ok();
        } else {
            *self.ctx = component_context;
        }

        Ok(data.custom_id == "yes")
    }

    pub async fn show_and_await_modal(
        &mut self,
        modal: ModalBuilder,
        options: WatcherOptions,
    ) -> anyhow::Result<Option<ModalResponse>> {
        let user_id = self.ctx.author_id();
        let modal_custom_id = modal.custom_id().to_string();
        self.ctx.show_modal(modal).await?;

        let Ok(Some(modal_response)) = self
            .ctx
            .watcher
            .await_single_modal(
                modal_custom_id,
                move |interaction| interaction.author_id() == Some(user_id),
                options,
            )
            .await
        else {
            return Ok(None);
        };

        Ok(Some(modal_response))
    }
}
