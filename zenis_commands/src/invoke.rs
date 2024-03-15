#![allow(clippy::len_zero)]
use std::time::Duration;

use zenis_database::{
    agent_model::{AgentModel, AgentPricing},
    instance_model::{CreditsPaymentMethod, InstanceBrain},
};
use zenis_discord::twilight_model::channel::message::component::ButtonStyle;
use zenis_framework::{util::make_multiple_rows, watcher::WatcherOptions};

use crate::prelude::*;

#[command("Invoque um agente de IA no chat para conversar com você!")]
#[name("invocar")]
pub async fn invoke(mut ctx: CommandContext) -> anyhow::Result<()> {
    let author = ctx.author().await?;
    let author_id = author.id;

    if ctx.interaction.guild_id.is_none() {
        ctx.reply(
            Response::new_user_reply(
                &author,
                "você precisa estar em um servidor para usar esse comando!",
            )
            .add_emoji_prefix(emojis::ERROR),
        )
        .await?;
        return Ok(());
    }

    let Some(channel) = ctx.interaction.channel.clone() else {
        return Ok(());
    };

    if ctx
        .db()
        .instances()
        .get_all_by_channel(channel.id.get())
        .await?
        .len()
        > 1
    {
        ctx.reply(
            Response::new_user_reply(&author, "já há muitos agentes neste chat!")
                .add_emoji_prefix(emojis::ERROR),
        )
        .await?;
        return Ok(());
    }

    let special_agents = ctx.db().agents().get_all_with_tags(&["special"]).await?;

    let mut buttons = vec![];
    for agent in special_agents.iter() {
        buttons.push(
            ButtonBuilder::new()
                .set_custom_id(&agent.identifier)
                .set_label(&agent.name)
                .set_style(ButtonStyle::Primary),
        );
    }

    buttons.push(
        ButtonBuilder::new()
            .set_custom_id("custom")
            .set_label("Agente personalizado"),
    );

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

    let mut agent_identifier = data.custom_id.clone();

    if data.custom_id == "custom" {
        ctx.send(
            Response::new_user_reply(
                &author,
                "escreva o ID do agente personalizado que você quer invocar:",
            )
            .add_emoji_prefix("⁉️"),
        )
        .await?;

        let Ok(Some(message)) = ctx
            .watcher
            .await_single_message(
                channel.id,
                move |message| message.author.id == author_id,
                WatcherOptions {
                    timeout: Duration::from_secs(60),
                },
            )
            .await
        else {
            return Ok(());
        };

        agent_identifier = message.content.to_lowercase().trim().to_owned();
    }

    let Some(agent) = ctx
        .db()
        .agents()
        .get_by_identifier(&agent_identifier)
        .await?
    else {
        ctx.send(
            Response::new_user_reply(&author, "agente inválido ou inexistente")
                .add_emoji_prefix(emojis::ERROR),
        )
        .await?;
        return Ok(());
    };

    if !agent.public
        && agent.creator_user_id != author.id.get()
        && agent.guild_id != channel.guild_id.map(|g| g.get())
    {
        ctx.send(
            Response::new_user_reply(
                &author,
                "você não tem permissão para invocar este agente privado!",
            )
            .add_emoji_prefix(emojis::ERROR),
        )
        .await?;
        return Ok(());
    }

    let brain = ask_for_brain(&mut ctx).await?;
    let mut pricing = agent.pricing;
    pricing.price_per_reply += brain.extra_price_per_reply();

    let mut payment_method = CreditsPaymentMethod::UserCredits(author_id.get());

    if let Some(guild_id) = channel.guild_id {
        if let Some(method) =
            ask_for_payment_method(&mut ctx, &agent, pricing, author_id, guild_id).await?
        {
            payment_method = method;
        } else {
            return Ok(());
        }
    }

    let minimum_credits = pricing.price_per_invocation + pricing.price_per_reply;

    match payment_method {
        CreditsPaymentMethod::UserCredits(user_id) => {
            let mut user_data = ctx.db().users().get_by_user(Id::new(user_id)).await?;
            if user_data.credits < minimum_credits {
                ctx.send(Response::new_user_reply(
                    &author,
                    "créditos insuficientes na carteira do usuário para invocar o agente!",
                ))
                .await?;
                return Ok(());
            }

            user_data.remove_credits(pricing.price_per_invocation);
        }
        CreditsPaymentMethod::GuildPublicCredits(guild_id) => {
            let mut guild_data = ctx.db().guilds().get_by_guild(Id::new(guild_id)).await?;
            if guild_data.public_credits < minimum_credits {
                ctx.send(Response::new_user_reply(
                    &author,
                    "créditos insuficientes na carteira pública do servidor para invocar o agente!",
                ))
                .await?;
                return Ok(());
            }

            guild_data.remove_public_credits(pricing.price_per_invocation);
        }
    }

    if pricing.price_per_invocation > 0 {
        let mut creator_data = ctx
            .db()
            .users()
            .get_by_user(Id::new(agent.creator_user_id))
            .await?;
        let profit = pricing.price_per_invocation as f64 * 0.025;
        let profit = profit as i64;
        creator_data.add_credits(profit);
        ctx.db().users().save(creator_data).await.ok();
    }

    if ctx
        .db()
        .instances()
        .get_all_by_channel(channel.id.get())
        .await?
        .len()
        > 1
    {
        ctx.send(
            Response::new_user_reply(&author, "já há muitos agentes neste chat!")
                .add_emoji_prefix(emojis::ERROR),
        )
        .await?;
        return Ok(());
    }

    let mut embed = EmbedBuilder::new_common()
        .set_color(Color::GREEN)
        .set_author(EmbedAuthor {
            name: format!("Invocado por {}", author.display_name()),
            icon_url: Some(author.avatar_url()),
        })
        .set_description(format!(
            "## {} invocado neste chat!\nEnvie mensagens e o agente responderá.",
            agent.name
        ));

    if let Some(image_url) = &agent.agent_url_image {
        embed = embed.set_thumbnail(image_url);
    }

    let message = ctx.send(embed).await?;

    let result = ctx
        .client
        .create_agent_instance(
            ctx.db(),
            brain,
            (channel.id, author.id),
            agent.clone(),
            pricing,
            payment_method,
        )
        .await;

    if let Some(e) = result.err() {
        ctx.client
            .http
            .delete_message(message.channel_id, message.id)
            .await?;

        ctx.client
            .emit_error_hook(
                format!("Invocation failed. Agent ID: {}", agent.identifier),
                e,
            )
            .await
            .ok();

        ctx.send(
            Response::new_user_reply(&author, "**algo deu errado ao invocar o agente!**\nVerifique se o agente tem um link de imagem PNG válido. Se não for isso, talvez eu não tenha permissão de criar webhooks aqui.\nSe o erro persistir, entre em **/servidoroficial** e busque suporte!")
                .add_emoji_prefix(emojis::ERROR),
        )
        .await?;
        return Ok(());
    }

    Ok(())
}

async fn ask_for_payment_method(
    ctx: &mut CommandContext,
    agent: &AgentModel,
    pricing: AgentPricing,
    author_id: Id<UserMarker>,
    guild_id: Id<GuildMarker>,
) -> anyhow::Result<Option<CreditsPaymentMethod>> {
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

    let invocation_price_str = if pricing.price_per_invocation > 0 {
        format!(
            "\nPreço por invocação de **{}**: `{}₢`",
            agent.name, pricing.price_per_invocation
        )
    } else {
        "".to_string()
    };

    let embed = EmbedBuilder::new_common()
        .set_color(Color::YELLOW)
        .set_description(format!(
            "## {} Escolha quem irá pagar o agente.\nPreço por resposta de **{}**: `{}₢`{invocation_price_str}",
            emojis::CREDIT,
            agent.name,
            pricing.price_per_reply
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

    ctx.client
        .http
        .delete_message(message.channel_id, message.id)
        .await
        .ok();

    if data.custom_id == "user" {
        return Ok(Some(CreditsPaymentMethod::UserCredits(author_id.get())));
    } else if data.custom_id == "guild" {
        return Ok(Some(CreditsPaymentMethod::GuildPublicCredits(
            guild_id.get(),
        )));
    }

    Ok(None)
}

pub async fn ask_for_brain(ctx: &mut CommandContext) -> anyhow::Result<InstanceBrain> {
    let author = ctx.author().await?;

    let buttons = vec![
        ButtonBuilder::new()
            .set_custom_id("cohere_command_r")
            .set_label("Command-R")
            .set_style(ButtonStyle::Secondary),
        ButtonBuilder::new()
            .set_custom_id("claude_haiku")
            .set_label("Haiku")
            .set_style(ButtonStyle::Secondary),
    ];

    let embed = EmbedBuilder::new_common()
        .set_color(Color::LIGHT_RED)
        .set_author(EmbedAuthor {
            name: "Seleção de Cérebro".to_string(),
            icon_url: Some(author.avatar_url()),
        })
        .set_description(format!("## {} Escolha qual cérebro você quer no seu agente:\n\n**Command-R**: cérebro normal. Preço padrão. Menos carismático, mais rápido.\n\n**Haiku**: mais carismático, mais lento e consegue ver imagens. 2 créditos mais caro por mensagem, 3 créditos extra pra cada imagem.", "🧠"));

    let message = ctx
        .send(
            Response::new_user_reply(&author, "escolha o cérebro do seu agente:")
                .add_emoji_prefix("🧠")
                .add_embed(embed)
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
        return Ok(InstanceBrain::ClaudeHaiku);
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

    match data.custom_id.as_str() {
        "cohere_command_r" => Ok(InstanceBrain::CohereCommandR),
        "claude_haiku" => Ok(InstanceBrain::ClaudeHaiku),
        _ => Ok(InstanceBrain::ClaudeHaiku),
    }
}
