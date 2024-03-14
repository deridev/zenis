use zenis_discord::twilight_model::channel::message::component::ButtonStyle;
use zenis_framework::util::make_multiple_rows;

use crate::prelude::*;

#[command("Entre no servidor oficial do ZenisAI!")]
#[name("servidoroficial")]
pub async fn officialguild(mut ctx: CommandContext) -> anyhow::Result<()> {
    let author = ctx.author().await?;
    let embed = EmbedBuilder::new_common()
        .set_color(Color::CYAN)
        .set_author_to_user(&author)
        .set_description("## Servidor oficial do ZenisAI!\nEntre para participar da comunidade de ZenisAI ou receber suporte.");

    ctx.reply(
        Response::from(embed).set_components(make_multiple_rows(vec![ButtonBuilder::new()
            .set_url("https://discord.gg/NaPXxajzSG")
            .set_label("Entrar no servidor")
            .set_style(ButtonStyle::Link)])),
    )
    .await?;

    Ok(())
}
