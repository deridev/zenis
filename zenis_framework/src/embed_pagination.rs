use std::time::Duration;

use tokio_stream::StreamExt;
use zenis_common::Pagination;
use zenis_discord::{
    twilight_model::{
        channel::message::{
            component::{TextInput, TextInputStyle},
            Component,
        },
        id::{marker::UserMarker, Id},
    },
    ActionRowBuilder, ButtonBuilder, EmbedBuilder, Emoji, InteractionData, ModalBuilder,
};

use crate::{watcher::WatcherOptions, CommandContext, Response};
pub struct EmbedPagination {
    ctx: CommandContext,
    pagination: Pagination<EmbedBuilder>,
    timeout: Duration,
    ephemeral: bool,
    allowed_users: Vec<Id<UserMarker>>,
}

impl EmbedPagination {
    pub fn new(ctx: CommandContext, pages: Vec<EmbedBuilder>) -> Self {
        Self {
            allowed_users: vec![ctx.author_id()],
            ctx,
            pagination: Pagination::new(pages),
            ephemeral: false,
            timeout: Duration::from_secs(300),
        }
    }

    pub fn _stop(&mut self) {
        self.pagination.active = false;
    }

    pub fn set_allowed_users(mut self, allowed_users: Vec<Id<UserMarker>>) -> Self {
        self.allowed_users = allowed_users;
        self
    }

    pub fn set_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn set_ephemeral(mut self) -> Self {
        self.ephemeral = true;
        self
    }

    fn generate_embed(&self) -> EmbedBuilder {
        if self.pagination.pages.is_empty() {
            return self.pagination.get_current_page().clone();
        }

        let embed = self.pagination.get_current_page().clone();

        embed.add_footer_text(format!(
            "Página {} de {}",
            self.pagination.page + 1,
            self.pagination.pages.len()
        ))
    }

    fn generate_components(&self) -> Vec<Component> {
        if self.pagination.pages.len() < 2 {
            return vec![];
        }

        let previous = ButtonBuilder::new()
            .set_custom_id("previous")
            .set_emoji(Emoji::from_unicode("◀️"));

        let next = ButtonBuilder::new()
            .set_custom_id("next")
            .set_emoji(Emoji::from_unicode("▶️"));

        let specific = ButtonBuilder::new()
            .set_custom_id("specific")
            .set_emoji(Emoji::from_unicode("🔍"));

        if self.pagination.pages.len() > 8 {
            return vec![ActionRowBuilder::new()
                .add_button(previous)
                .add_button(next)
                .add_button(specific)
                .build()];
        }

        vec![ActionRowBuilder::new()
            .add_button(previous)
            .add_button(next)
            .build()]
    }

    fn generate_response(&self) -> Response {
        let mut response = Response {
            embeds: Some(vec![self.generate_embed()]),
            components: Some(self.generate_components()),
            ..Default::default()
        };

        if self.ephemeral {
            response = response.set_ephemeral();
        }

        response
    }

    pub async fn send(&mut self) -> anyhow::Result<()> {
        let allowed_users = self.allowed_users.clone();
        let message = self.ctx.send(self.generate_response()).await?;

        let stream = self.ctx.watcher.create_component_stream(
            message.id,
            move |interaction| {
                interaction
                    .author_id()
                    .is_some_and(|id| allowed_users.contains(&id))
            },
            WatcherOptions {
                timeout: self.timeout,
            },
        );
        tokio::pin!(stream);

        while let Some(Ok(collected)) = stream.next().await {
            let Some(InteractionData::MessageComponent(data)) = collected.data.clone() else {
                break;
            };

            let mut ctx = CommandContext::from_with_interaction(&self.ctx, Box::new(collected));

            if data.custom_id == "next" {
                self.pagination.goto_next_page();
            } else if data.custom_id == "previous" {
                self.pagination.goto_previous_page();
            } else if data.custom_id == "specific" {
                let modal = ModalBuilder::new("Escolha a Página", "pagination_choose_page")
                    .add_text_input(TextInput {
                        custom_id: "page".to_string(),
                        label: "Página".to_string(),
                        style: TextInputStyle::Short,
                        required: Some(true),
                        max_length: None,
                        min_length: None,
                        placeholder: None,
                        value: None,
                    });

                let modal_response = ctx
                    .helper()
                    .show_and_await_modal(
                        modal,
                        WatcherOptions {
                            timeout: Duration::from_secs(60),
                        },
                    )
                    .await?;

                let page = match modal_response {
                    Some(modal_response) => {
                        let page = modal_response.get_text_input("page").unwrap_or_default();
                        ctx = CommandContext::from_with_interaction(
                            &self.ctx,
                            modal_response.interaction(),
                        );

                        page.parse::<usize>().unwrap_or(self.pagination.page + 1)
                    }
                    None => self.pagination.page + 1,
                };

                self.pagination.page = page.clamp(0, self.pagination.pages.len()).saturating_sub(1);

                ctx.update_message(self.generate_response()).await?;
                self.ctx = ctx;
                continue;
            }

            if !self.pagination.active {
                break;
            }

            ctx.update_message(self.generate_response()).await?;
        }

        Ok(())
    }
}
