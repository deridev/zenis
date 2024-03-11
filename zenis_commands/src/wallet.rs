use crate::prelude::*;

#[command("Veja a carteira global de créditos de algum usuário!")]
#[name("carteira")]
pub async fn wallet(
    mut ctx: CommandContext,
    #[rename("usuário")]
    #[description("O usuário que você quer ver a carteira")]
    user: Option<User>,
) -> anyhow::Result<()> {
    let user = match user {
        Some(user) => user,
        None => ctx.author().await?,
    };

    let user_data = ctx.db().users().get_by_user(user.id).await?;

    let embed = EmbedBuilder::new_common()
        .set_color(Color::YELLOW)
        .set_author(EmbedAuthor {
            name: format!("Carteira de {}", user.display_name()),
            icon_url: Some(user.avatar_url()),
        })
        .add_inlined_field(
            format!("{} Créditos", emojis::CREDIT),
            format!("{}₢", user_data.credits),
        );

    ctx.reply(embed).await?;

    Ok(())
}
