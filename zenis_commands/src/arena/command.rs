use std::time::Duration;

use crate::prelude::*;

use super::{
    common::{ArenaPaymentMethod, PRICE_PER_ACTION, PRICE_PER_ARENA},
    controller, helper,
};

#[command("Em uma arena AI, lute contra usuários!")]
#[name("arena")]
pub async fn arena(
    mut ctx: CommandContext,
    #[rename("oponente")]
    #[description("O usuário que você quer ver lutar na arena IA")]
    user: User,
) -> anyhow::Result<()> {
    let author = ctx.author().await?;
    let pricing_embed = EmbedBuilder::new_common()
        .set_color(Color::YELLOW)
        .set_author(EmbedAuthor {
            name: "Preço da Arena".to_string(),
            icon_url: Some(user.avatar_url()),
        })
        .set_description(format!(
            "## {} Custos da Arena:\n**Preço pra criar arena**: {}₢\n**Preço por ação**: {}₢",
            emojis::CREDIT,
            PRICE_PER_ARENA,
            PRICE_PER_ACTION
        ))
        .add_footer_text("Não tem créditos? Use /comprar para aproveitar ZenisAI!");

    let confirmation = ctx
        .helper()
        .create_confirmation(
            user.id,
            false,
            Response::new_user_reply(
                &user,
                format!(
                    "você foi convidado para uma arena AI por **{}**! Quer lutar?",
                    author.name
                ),
            )
            .add_embed(pricing_embed)
            .add_emoji_prefix("🤫"),
        )
        .await?;
    if !confirmation {
        return Ok(());
    }

    let Some(arena_payment_method) = helper::select_payment_method(&author, &mut ctx).await? else {
        return Ok(());
    };

    let users = vec![author.clone(), user.clone()];

    // Check for credits
    let initial_cost = PRICE_PER_ARENA + PRICE_PER_ACTION;
    match arena_payment_method {
        ArenaPaymentMethod::User(user_id) => {
            let user_data = ctx.db().users().get_by_user(user_id).await?;
            let user = ctx.client.get_user(user_id).await?;

            if user_data.credits < initial_cost {
                ctx.send(
                    Response::new_user_reply(
                        &user,
                        "você não tem suficientes créditos para pagar a arena! Use **/comprar** para adquirir mais créditos e aproveitar ZenisAI!",
                    )
                    .add_emoji_prefix(emojis::ERROR),
                )
                .await?;
                return Ok(());
            }
        }
        ArenaPaymentMethod::EveryoneHalf => {
            let cost = initial_cost / users.len() as i64;
            for user in users.iter() {
                let user_data = ctx.db().users().get_by_user(user.id).await?;
                if user_data.credits < cost {
                    ctx.send(
                        Response::new_user_reply(
                            user,
                            "você não tem suficientes créditos para pagar a arena de forma dividida! Use **/comprar** para adquirir mais créditos e aproveitar ZenisAI!",
                        )
                        .add_emoji_prefix(emojis::ERROR),
                    )
                    .await?;
                    return Ok(());
                }
            }
        }
    }

    // Create arena fighters
    let fighters = helper::create_arena_fighters(&mut ctx, users.clone()).await?;
    if fighters.len() < 2 {
        return Ok(());
    }

    // Get authorization
    let mut authorization_embed = EmbedBuilder::new_common()
        .set_color(Color::CYAN)
        .set_author(EmbedAuthor {
            name: "Resumo da Arena".to_string(),
            icon_url: Some(author.avatar_url()),
        })
        .add_footer_text(match arena_payment_method {
            ArenaPaymentMethod::User(id) => {
                let user = ctx.client.get_user(id).await?;
                format!(
                    "{} pagará todos os custos da arena com sua carteira.",
                    user.name
                )
            }
            ArenaPaymentMethod::EveryoneHalf => {
                "Preço dividido entre todos os participantes da arena.".to_string()
            }
        });

    for fighter in fighters.iter() {
        let mut description = fighter.description.clone();
        if description.len() > 200 {
            description.truncate(200);
            description.push_str("...");
        }

        authorization_embed = authorization_embed.add_not_inlined_field(
            format!("{} ({})", fighter.name, fighter.user.name),
            format!("`{}`", description),
        );
    }

    for user in users.iter() {
        let authorization = ctx
            .helper()
            .create_confirmation(
                user.id,
                true,
                Response::new_user_reply(user, "você confirma a criação da arena?")
                    .add_embed(authorization_embed.clone())
                    .add_emoji_prefix("⁉️"),
            )
            .await?;
        if !authorization {
            ctx.send_in_channel(format!(
                "**{}** não autorizou a criação da arena!",
                user.name
            ))
            .await?;
            return Ok(());
        }

        tokio::time::sleep(Duration::from_millis(400)).await;
    }

    let result = controller::run_arena(&mut ctx, None, arena_payment_method, fighters).await;
    if let Err(e) = result {
        ctx.client
            .emit_error_hook("Arena failed".to_string(), e)
            .await?;
        ctx.send(
            Response::new_user_reply(
                &author,
                "a arena foi encerrada devido a um erro. Por favor, tente novamente mais tarde. O desenvolvedor já foi notificado sobre o erro.",
            )
            .add_emoji_prefix(emojis::ERROR),
        )
        .await?;
        return Ok(());
    }

    Ok(())
}
