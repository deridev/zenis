use std::time::Duration;

use zenis_discord::twilight_model::{channel::message::component::ButtonStyle, guild::Permissions};
use zenis_framework::{util::make_multiple_rows, watcher::WatcherOptions};

use crate::prelude::*;

#[command("Veja informações do servidor que você usou o comando!")]
#[name("servidor")]
pub async fn guild(mut ctx: CommandContext) -> anyhow::Result<()> {
    let author = ctx.author().await?;

    let Some(guild_id) = ctx.interaction.guild_id else {
        ctx.reply(
            Response::new_user_reply(
                &author,
                "você deve estar em um servidor para usar este comando!",
            )
            .add_emoji_prefix(emojis::ERROR),
        )
        .await?;

        return Ok(());
    };

    let guild = ctx.client.get_guild(guild_id).await?;
    let guild_data = ctx.db().guilds().get_by_guild(guild_id).await?;

    let embed = EmbedBuilder::new_common()
        .set_color(Color::CYAN)
        .set_author(EmbedAuthor {
            name: format!("Servidor {}", guild.name),
            icon_url: Some(guild.icon_url()),
        })
        .add_inlined_field(
            format!("{} Créditos", emojis::CREDIT),
            format!("{}₢", guild_data.credits),
        )
        .add_inlined_field(
            "💸 Créditos Públicos",
            format!("{}₢", guild_data.public_credits),
        )
        .add_footer_text(format!("ID do servidor: {}", guild_id));

    ctx.reply(embed).await?;

    let member = ctx
        .client
        .http
        .guild_member(guild_id, author.id)
        .await?
        .model()
        .await?;
    let guild_roles = ctx.client.http.roles(guild_id).await?.model().await?;
    let mut is_adm = false;

    for role in member.roles.iter() {
        let Some(role) = guild_roles.iter().find(|r| r.id == *role) else {
            continue;
        };

        if role.permissions.contains(Permissions::MANAGE_GUILD)
            || role.permissions.contains(Permissions::ADMINISTRATOR)
        {
            is_adm = true;
            break;
        }
    }

    if is_adm {
        admin_dashboard(&mut ctx, &author, guild_id).await?;
    }

    Ok(())
}

/////////////////////
/////////////////////
/////////////////////
/////////////////////
/////////////////////
/////////////////////
// DASHBOARD
/////////////////////

async fn admin_dashboard(
    ctx: &mut CommandContext,
    author: &User,
    guild_id: Id<GuildMarker>,
) -> anyhow::Result<()> {
    let buttons = vec![
        ButtonBuilder::new()
            .set_custom_id("cancel")
            .set_label("Cancelar")
            .set_style(ButtonStyle::Danger),
        ButtonBuilder::new()
            .set_custom_id("realoc_credits")
            .set_label("Realocar Créditos"),
    ];

    let message = ctx
        .followup_interaction(
            Response::new_user_reply(
                author,
                "**bem vindo ao dashboard do servidor, administrador!**\nPor favor, escolha uma opção, caso você queira alterar algo no servidor:",
            )
            .add_emoji_prefix("🤫")
            .set_ephemeral()
            .set_components(make_multiple_rows(buttons.clone())),
        )
        .await?.model().await?;

    let author_id = author.id;
    let Ok(Some(interaction)) = ctx
        .watcher
        .await_single_component(
            message.id,
            move |interaction| interaction.author_id() == Some(author_id),
            WatcherOptions {
                timeout: Duration::from_secs(30),
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

    let mut ctx = CommandContext::from_with_interaction(ctx, Box::new(interaction));
    ctx.update_message(Response::default().set_components(make_multiple_rows(buttons)))
        .await?;

    if data.custom_id == "cancel" {
        return Ok(());
    } else if data.custom_id == "realoc_credits" {
        let Ok(Some(public_credits)) = get_input(
            &mut ctx, author,
            Response::new_user_reply(author,
                "quantos créditos PÚBLICOS você quer? Os créditos que restarem ficarão privados. Se não quiser nenhum crédito público, envie zero:"
            ).add_emoji_prefix(emojis::CREDIT)
        ).await else {
            return Ok(());
        };

        let Ok(public_credits) = public_credits.parse::<u32>() else {
            ctx.send(
                Response::new_user_reply(
                    author,
                    "o número de créditos públicos deve ser um número inteiro positivo válido!",
                )
                .add_emoji_prefix(emojis::ERROR),
            )
            .await?;
            return Ok(());
        };

        let public_credits = public_credits as i64;

        let mut guild_data = ctx.db().guilds().get_by_guild(guild_id).await?;
        let total_credits = guild_data.credits + guild_data.public_credits;

        if public_credits > total_credits {
            ctx.send(
                Response::new_user_reply(
                    author,
                    "esse servidor não tem créditos suficientes para realizar essa ação!",
                )
                .add_emoji_prefix(emojis::ERROR),
            )
            .await?;
            return Ok(());
        }

        let private_credits = total_credits - public_credits;

        guild_data.public_credits = public_credits;
        guild_data.credits = private_credits;

        ctx.db().guilds().save(guild_data).await?;

        ctx.send(
            Response::new_user_reply(
                author,
                format!(
                    "**créditos realocados com sucesso!**! O servidor agora tem **{}₢** créditos públicos e **{}₢** créditos privados.",
                    public_credits,
                    private_credits
                ),
            )
            .add_emoji_prefix(emojis::SUCCESS),
        )
        .await?;
    }

    Ok(())
}
