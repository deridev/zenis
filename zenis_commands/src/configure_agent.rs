use std::time::Duration;

use zenis_discord::twilight_model::channel::message::component::ButtonStyle;
use zenis_framework::{util::make_multiple_rows, watcher::WatcherOptions};

use crate::prelude::*;

#[command("Configure um dos seus agentes!")]
#[name("configurar agente")]
pub async fn configure_agent(
    mut ctx: CommandContext,
    #[rename("id")]
    #[description("O ID do agente que você quer configurar")]
    identifier: String,
) -> anyhow::Result<()> {
    let author = ctx.author().await?;

    let Some(agent) = ctx.db().agents().get_by_identifier(&identifier).await? else {
        ctx.send(
            Response::new_user_reply(&author, "agente inválido ou inexistente")
                .add_emoji_prefix(emojis::ERROR),
        )
        .await?;
        return Ok(());
    };

    if agent.creator_user_id != author.id.get() {
        ctx.send(
            Response::new_user_reply(
                &author,
                "você não pode configurar esse agente pois não é seu!",
            )
            .add_emoji_prefix(emojis::ERROR),
        )
        .await?;
        return Ok(());
    }

    let mut description = agent.description.clone();
    description.truncate(128);

    let mut introduction_message = agent.introduction_message.clone();
    introduction_message.truncate(128);

    let mut agent_preview = EmbedBuilder::new_common()
        .set_color(Color::YELLOW)
        .set_author(EmbedAuthor {
            name: format!("Configuração do agente {}", agent.name),
            icon_url: agent.agent_url_image.clone(),
        })
        .add_inlined_field("📄 Nome", format!("**{}**", agent.name))
        .add_inlined_field("😀 Descrição", format!("`{}`", description))
        .add_inlined_field(
            "📢 Mensagem de introdução",
            format!("`{}`", introduction_message),
        )
        .add_inlined_field(
            format!("{} Preços", emojis::CREDIT),
            format!(
                "Preço por invocação: **{}₢**\nPreço por resposta: **{}₢**",
                agent.pricing.price_per_invocation, agent.pricing.price_per_reply
            ),
        )
        .add_inlined_field(
            "👥 Agente Público?",
            if agent.public { "Sim" } else { "Não" },
        )
        .add_footer_text(format!("ID do agente: {}", agent.identifier));

    if let Some(agent_url_image) = &agent.agent_url_image {
        agent_preview = agent_preview.set_thumbnail(agent_url_image);
    }

    let buttons = vec![
        ButtonBuilder::new()
            .set_custom_id("cancel")
            .set_label("Cancelar")
            .set_style(ButtonStyle::Danger),
        ButtonBuilder::new()
            .set_custom_id("change_description")
            .set_label("Alterar descrição")
            .set_style(ButtonStyle::Secondary),
        ButtonBuilder::new()
            .set_custom_id("change_introduction")
            .set_label("Alterar Mensagem de Introdução")
            .set_style(ButtonStyle::Secondary),
        ButtonBuilder::new()
            .set_custom_id("change_image")
            .set_label("Alterar URL de Imagem")
            .set_style(ButtonStyle::Secondary),
        ButtonBuilder::new()
            .set_custom_id("change_invocation_price")
            .set_label("Alterar Preço de Invocação")
            .set_style(ButtonStyle::Secondary),
        ButtonBuilder::new()
            .set_custom_id("change_public")
            .set_label(if !agent.public { "Publicar" } else { "Privar" })
            .set_style(ButtonStyle::Secondary),
    ];

    let message = ctx
        .send(Response::from(agent_preview).set_components(make_multiple_rows(buttons.clone())))
        .await?;

    let Ok(Some(interaction)) = ctx
        .watcher
        .await_single_component(
            message.id,
            move |interaction| interaction.author_id() == Some(author.id),
            WatcherOptions {
                timeout: Duration::from_secs(120),
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

    if data.custom_id == "change_description" {
        let Ok(Some(description)) = get_input(
            &mut ctx,
            &author,
            Response::new_user_reply(&author, "escreva a nova descrição do agente:"),
        )
        .await
        else {
            return Ok(());
        };

        let Some(mut agent) = ctx.db().agents().get_by_identifier(identifier).await? else {
            ctx.send(
                Response::new_user_reply(&author, "agente inválido ou inexistente")
                    .add_emoji_prefix(emojis::ERROR),
            )
            .await?;
            return Ok(());
        };

        if !config::DESCRIPTION_SIZE.contains(&description.len()) {
            ctx.send(
                Response::new_user_reply(
                    &author,
                    format!(
                        "a descrição deve ter no máximo {} caracteres!",
                        config::DESCRIPTION_SIZE.end()
                    ),
                )
                .add_emoji_prefix(emojis::ERROR),
            )
            .await?;
            return Ok(());
        }

        agent.description = description;
        ctx.db().agents().save(agent).await?;

        ctx.send(
            Response::new_user_reply(&author, "descrição alterada com sucesso!")
                .add_emoji_prefix(emojis::SUCCESS),
        )
        .await?;
    } else if data.custom_id == "change_introduction" {
        let Ok(Some(introduction_message)) = get_input(
            &mut ctx,
            &author,
            Response::new_user_reply(&author, "escreva a nova introdução do agente:"),
        )
        .await
        else {
            return Ok(());
        };

        let Some(mut agent) = ctx.db().agents().get_by_identifier(identifier).await? else {
            ctx.send(
                Response::new_user_reply(&author, "agente inválido ou inexistente")
                    .add_emoji_prefix(emojis::ERROR),
            )
            .await?;
            return Ok(());
        };

        if !config::INTRODUCTION_MESSAGE_SIZE.contains(&introduction_message.len()) {
            ctx.send(
                Response::new_user_reply(
                    &author,
                    format!(
                        "a introdução deve ter no máximo {} caracteres!",
                        config::INTRODUCTION_MESSAGE_SIZE.end()
                    ),
                )
                .add_emoji_prefix(emojis::ERROR),
            )
            .await?;
            return Ok(());
        }

        agent.introduction_message = introduction_message;
        ctx.db().agents().save(agent).await?;

        ctx.send(
            Response::new_user_reply(&author, "introdução alterada com sucesso!")
                .add_emoji_prefix(emojis::SUCCESS),
        )
        .await?;
    } else if data.custom_id == "change_invocation_price" {
        let Ok(Some(invocation_price)) = get_input(
            &mut ctx,
            &author,
            Response::new_user_reply(&author, "escreva o novo preço de invocação do agente:"),
        )
        .await
        else {
            return Ok(());
        };

        let Some(mut agent) = ctx.db().agents().get_by_identifier(identifier).await? else {
            ctx.send(
                Response::new_user_reply(&author, "agente inválido ou inexistente")
                    .add_emoji_prefix(emojis::ERROR),
            )
            .await?;
            return Ok(());
        };

        let price = invocation_price
            .parse::<u8>()
            .ok()
            .unwrap_or(0)
            .clamp(0, 100);

        agent.pricing.price_per_invocation = price as i64;
        ctx.db().agents().save(agent).await?;

        ctx.send(
            Response::new_user_reply(&author, "preço alterado com sucesso!")
                .add_emoji_prefix(emojis::SUCCESS),
        )
        .await?;
    } else if data.custom_id == "change_image" {
        let Ok(Some(image)) = get_input(
            &mut ctx,
            &author,
            Response::new_user_reply(&author, "escreva a nova URL da imagem do agente:"),
        )
        .await
        else {
            return Ok(());
        };

        let Some(mut agent) = ctx.db().agents().get_by_identifier(identifier).await? else {
            ctx.send(
                Response::new_user_reply(&author, "agente inválido ou inexistente")
                    .add_emoji_prefix(emojis::ERROR),
            )
            .await?;
            return Ok(());
        };

        agent.agent_url_image = Some(image);
        ctx.db().agents().save(agent).await?;

        ctx.send(
            Response::new_user_reply(&author, "imagem alterada com sucesso! Se o URL não for um PNG válido, o agente vai dar erro sempre que for invocado. Tome cuidado!")
                .add_emoji_prefix(emojis::SUCCESS)
        ).await?;
    } else if data.custom_id == "change_public" {
        if agent.public {
            let confirmation = ctx.helper().create_confirmation(
                author.id, false,
                Response::new_user_reply(&author, "você quer MESMO privar o agente? Você precisará pagar novamente para publicar de novo.")
                .add_emoji_prefix(emojis::WARNING)).await?;
            if !confirmation {
                return Ok(());
            }

            let Some(mut agent) = ctx.db().agents().get_by_identifier(identifier).await? else {
                ctx.send(
                    Response::new_user_reply(&author, "agente inválido ou inexistente")
                        .add_emoji_prefix(emojis::ERROR),
                )
                .await?;
                return Ok(());
            };

            agent.public = false;
            ctx.db().agents().save(agent).await?;

            ctx.send(
                Response::new_user_reply(&author, "agente privado com sucesso!")
                    .add_emoji_prefix(emojis::SUCCESS),
            )
            .await?;
        } else {
            let confirmation = ctx.helper().create_confirmation(
                author.id, false,
                Response::new_user_reply(&author, "você quer publicar o agente? Vai custar **20₢** e seu agente passará por um processo de verificação antes de ser publicado.\n**Se o seu agente for RECUSADO, você não terá reembolso dos créditos. Não envie agentes ofensivos, NSFW ou de cunho preconceituoso.**")
                .add_emoji_prefix(emojis::WARNING)).await?;
            if !confirmation {
                return Ok(());
            }

            let Some(mut agent) = ctx.db().agents().get_by_identifier(identifier).await? else {
                ctx.send(
                    Response::new_user_reply(&author, "agente inválido ou inexistente")
                        .add_emoji_prefix(emojis::ERROR),
                )
                .await?;
                return Ok(());
            };

            if agent.is_waiting_for_approval {
                ctx.send(
                    Response::new_user_reply(&author, "o agente já está em processo de verificação. Você precisa esperar até que ele seja analisado.")
                        .add_emoji_prefix(emojis::ERROR),
                )
                .await?;
                return Ok(());
            }

            let mut user_data = ctx.db().users().get_by_user(author.id).await?;
            if user_data.credits < 20 {
                ctx.send(
                    Response::new_user_reply(
                        &author,
                        "você não tem suficientes créditos para publicar o agente!",
                    )
                    .add_emoji_prefix(emojis::ERROR),
                )
                .await?;
                return Ok(());
            }

            user_data.remove_credits(20);
            ctx.db().users().save(user_data).await?;
            agent.is_waiting_for_approval = true;
            ctx.db().agents().save(agent.clone()).await?;

            ctx.client.emit_request_hook(agent).await?;

            ctx.send(
                Response::new_user_reply(&author, "agente enviado para verificação com sucesso! Abra sua DM para receber notificações sobre o status do processo.")
                    .add_emoji_prefix(emojis::SUCCESS),
            ).await?;
        }
    }

    Ok(())
}
