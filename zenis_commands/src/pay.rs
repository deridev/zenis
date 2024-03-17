use crate::prelude::*;

#[command("Envie créditos para outro usuário!")]
#[name("pagar")]
pub async fn pay(
    mut ctx: CommandContext,
    #[rename("usuário")]
    #[description("O usuário que você quer ver a carteira")]
    user: User,
    #[rename("créditos")]
    #[description("Quantia de créditos que você quer enviar")]
    credits: i64,
) -> anyhow::Result<()> {
    let author = ctx.author().await?;
    let credits = credits.clamp(1, i64::MAX / 2);

    let author_data = ctx.db().users().get_by_user(author.id).await?;
    if author_data.credits < credits {
        ctx.send(
            Response::new_user_reply(
                &author,
                "você não tem suficientes créditos para enviar para esse usuário!",
            )
            .add_emoji_prefix(emojis::ERROR),
        )
        .await?;
        return Ok(());
    }

    let confirmation = ctx
        .helper()
        .create_confirmation(
            author.id,
            false,
            Response::new_user_reply(
                &author,
                format!(
                    "você quer mesmo enviar **{credits}₢** para **{}**?",
                    user.display_name()
                ),
            )
            .add_emoji_prefix("⁉️"),
        )
        .await?;
    if !confirmation {
        return Ok(());
    }

    let mut author_data = ctx.db().users().get_by_user(author.id).await?;
    if author_data.credits < credits {
        ctx.send(
            Response::new_user_reply(
                &author,
                "você não tem suficientes créditos para enviar para esse usuário!",
            )
            .add_emoji_prefix(emojis::ERROR),
        )
        .await?;
        return Ok(());
    }

    author_data.credits -= credits;
    ctx.db().users().save(author_data).await?;

    let mut user_data = ctx.db().users().get_by_user(user.id).await?;
    user_data.credits += credits;
    ctx.db().users().save(user_data).await?;

    ctx.send(
        Response::new_user_reply(&author, "créditos enviados com sucesso!")
            .add_emoji_prefix(emojis::SUCCESS),
    )
    .await?;
    Ok(())
}
