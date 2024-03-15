use std::time::Duration;

use zenis_database::instance_model::InstanceModel;
use zenis_discord::twilight_model::channel::message::component::ButtonStyle;
use zenis_framework::{util::make_multiple_rows, watcher::WatcherOptions};

use crate::prelude::*;

#[command("Desligue os agentes nesse chat!")]
#[name("desligar")]
pub async fn shutdown(mut ctx: CommandContext) -> anyhow::Result<()> {
    let author = ctx.author().await?;
    let channel_id = ctx
        .interaction
        .channel
        .clone()
        .context("Expected a channel")?
        .id;
    let instances = ctx
        .db()
        .instances()
        .all_actives_in_channel(channel_id.get())
        .await?;

    if instances.is_empty() {
        ctx.send(
            Response::new_user_reply(
                &author,
                "nenhum agente está invocado neste chat atualmente!",
            )
            .add_emoji_prefix(emojis::ERROR)
            .set_ephemeral(),
        )
        .await?;
        return Ok(());
    }

    let mut buttons = vec![ButtonBuilder::new()
        .set_custom_id("cancel")
        .set_label("Cancelar")
        .set_style(ButtonStyle::Danger)];

    for instance in instances.iter() {
        buttons.push(
            ButtonBuilder::new()
                .set_custom_id(&instance.agent_identifier)
                .set_label(&instance.agent_name)
                .set_style(ButtonStyle::Secondary),
        );
    }

    let message = ctx
        .send(
            Response::new_user_reply(&author, "escolha um agente para desligar:")
                .add_emoji_prefix("🤖")
                .set_components(make_multiple_rows(buttons.clone())),
        )
        .await?;

    let Ok(Some(interaction)) = ctx
        .watcher
        .await_single_component(
            message.id,
            move |interaction| interaction.author_id() == Some(author.id),
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

    if data.custom_id == "cancel" {
        return Ok(());
    }

    let Some(instance) = instances
        .iter()
        .find(|instance| instance.agent_identifier == data.custom_id)
    else {
        return Ok(());
    };

    shutdown_instance(&mut ctx, &author, instance.clone()).await?;

    Ok(())
}

async fn shutdown_instance(
    ctx: &mut CommandContext,
    author: &User,
    instance: InstanceModel,
) -> anyhow::Result<()> {
    let Some(mut instance) = ctx.db().instances().get_by_id(instance.id).await? else {
        ctx.send(
            Response::new_user_reply(author, "esse agente não foi encontrado!")
                .add_emoji_prefix(emojis::ERROR),
        )
        .await?;
        return Ok(());
    };

    if instance.summoner_id != author.id.get() {
        let summoner = ctx.client.get_user(Id::new(instance.summoner_id)).await?;
        let confirmation = ctx
            .helper()
            .create_confirmation(
                summoner.id,
                false,
                Response::new_user_reply(
                    &summoner,
                    format!(
                        "você autoriza o desligamento do agente **{}**?",
                        instance.agent_name
                    ),
                )
                .add_emoji_prefix("⁉️"),
            )
            .await?;
        if !confirmation {
            return Ok(());
        }
    }

    instance.exit_reason = Some(format!("Desligado por {}", author.display_name()));
    ctx.db().instances().save(instance).await?;

    ctx.send(
        Response::new_user_reply(author, "agente desligado com sucesso!")
            .add_emoji_prefix(emojis::SUCCESS),
    )
    .await?;

    Ok(())
}
