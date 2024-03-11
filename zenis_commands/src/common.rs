use crate::prelude::*;

#[command("ping")]
pub async fn ping(mut ctx: CommandContext) -> anyhow::Result<()> {
    let before = chrono::Utc::now();

    ctx.reply("Pong!").await?;

    let now = chrono::Utc::now();
    let ping = now.timestamp_millis() - before.timestamp_millis();

    let embed = EmbedBuilder::new_common()
        .set_color(if ping < 200 {
            Color::GREEN
        } else if ping < 400 {
            Color::YELLOW
        } else {
            Color::RED
        })
        .set_title("Pong! 🏓")
        .add_inlined_field("Latência", format!("{ping}ms"));

    ctx.update_interaction_reply(Response::from(embed)).await?;

    Ok(())
}
