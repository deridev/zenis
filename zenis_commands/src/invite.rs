use zenis_discord::twilight_model::channel::message::component::ButtonStyle;
use zenis_framework::util::make_multiple_rows;

use crate::prelude::*;

#[command("Convide ZenisAI para o seu servidor!")]
#[name("convidar")]
pub async fn invite(mut ctx: CommandContext) -> anyhow::Result<()> {
    let author = ctx.author().await?;
    let embed = EmbedBuilder::new_common()
        .set_color(Color::YELLOW)
        .set_author_to_user(&author)
        .set_description("Convide ZenisAI para o seu servidor!");

    ctx.reply(
        Response::from(embed).set_components(make_multiple_rows(vec![ButtonBuilder::new()
            .set_url("https://discord.com/oauth2/authorize?client_id=1215409249262379018&permissions=277562354704&scope=bot+applications.commands")
            .set_label("Convidar")
            .set_style(ButtonStyle::Link)])),
    )
    .await?;

    Ok(())
}
