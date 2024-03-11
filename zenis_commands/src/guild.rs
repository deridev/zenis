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
        );

    ctx.reply(embed).await?;

    Ok(())
}
