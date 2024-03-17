use std::time::Duration;

use zenis_discord::twilight_model::channel::message::component::ButtonStyle;
use zenis_framework::{util::make_multiple_rows, watcher::WatcherOptions};

use crate::prelude::*;

use super::{common::ArenaPaymentMethod, controller::ArenaFighter};

pub async fn select_payment_method(
    author: &User,
    ctx: &mut CommandContext,
) -> anyhow::Result<Option<ArenaPaymentMethod>> {
    let author_id = author.id;
    let buttons = vec![
        ButtonBuilder::new()
            .set_custom_id("user")
            .set_label("Pagar tudo")
            .set_style(ButtonStyle::Secondary),
        ButtonBuilder::new()
            .set_custom_id("half")
            .set_label("Cada um paga metade")
            .set_style(ButtonStyle::Secondary),
    ];

    let message = ctx
        .send(
            Response::new_user_reply(author, "escolha o método de pagamento da arena:")
                .add_emoji_prefix(emojis::CREDIT)
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
        return Ok(Some(ArenaPaymentMethod::User(author.id)));
    } else if data.custom_id == "half" {
        return Ok(Some(ArenaPaymentMethod::EveryoneHalf));
    }

    Ok(None)
}

pub async fn create_arena_fighters(
    ctx: &mut CommandContext,
    users: Vec<User>,
) -> anyhow::Result<Vec<ArenaFighter>> {
    let mut fighters = Vec::new();

    for user in users {
        let message = ctx.send(Response::new_user_reply(&user, "escolha um nome para o seu personagem na arena ou envie um **.** para usar o mesmo do Discord:").add_emoji_prefix("📋")).await?;

        let Ok(Some(message)) = ctx
            .watcher
            .await_single_message(
                message.channel_id,
                move |message| message.author.id == user.id,
                WatcherOptions {
                    timeout: Duration::from_secs(60),
                },
            )
            .await
        else {
            return Ok(vec![]);
        };

        let mut name = message.content.trim().to_owned();
        name = name.replace('\n', " ");

        if name == "." {
            name = user.display_name().clone();
        }

        if !config::ARENA_NAME_SIZE.contains(&name.len()) {
            ctx.send(
                Response::new_user_reply(
                    &user,
                    format!(
                        "o nome do personagem deve ter no máximo {} caracteres!",
                        config::ARENA_NAME_SIZE.end()
                    ),
                )
                .add_emoji_prefix(emojis::ERROR),
            )
            .await?;
            return Ok(vec![]);
        }

        let message = ctx.send(Response::new_user_reply(&user, "escreva uma descrição para o personagem (ex: `Um mafioso com uma faca no bolso e muita força física`):").add_emoji_prefix("📝")).await?;

        let Ok(Some(message)) = ctx
            .watcher
            .await_single_message(
                message.channel_id,
                move |message| message.author.id == user.id,
                WatcherOptions {
                    timeout: Duration::from_secs(300),
                },
            )
            .await
        else {
            return Ok(vec![]);
        };

        let description = message.content.trim().to_owned();
        if !config::ARENA_DESCRIPTION_SIZE.contains(&description.len()) {
            ctx.send(
                Response::new_user_reply(
                    &user,
                    format!(
                        "a descrição do personagem deve ter no máximo {} caracteres!",
                        config::ARENA_DESCRIPTION_SIZE.end()
                    ),
                )
                .add_emoji_prefix(emojis::ERROR),
            )
            .await?;
            return Ok(vec![]);
        }

        fighters.push(ArenaFighter {
            user_id: user.id,
            user,
            name,
            description,
        });
    }

    Ok(fighters)
}
