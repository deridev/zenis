use crate::prelude::*;

const AGENTS_PER_PAGE: usize = 3;

#[command("Veja todos os agentes que você criou!")]
#[name("meus agentes")]
pub async fn my_agents(mut ctx: CommandContext) -> anyhow::Result<()> {
    let author = ctx.author().await?;
    let author_id = author.id;

    let agents = ctx
        .db()
        .agents()
        .get_all_by_creator(author_id.get())
        .await?;
    if agents.is_empty() {
        ctx.send(
            Response::new_user_reply(&author, "você não criou nenhum agente ainda!")
                .add_emoji_prefix(emojis::ERROR),
        )
        .await?;
        return Ok(());
    }

    let mut pages = vec![];
    for i in (0..agents.len()).step_by(AGENTS_PER_PAGE) {
        let mut page = EmbedBuilder::new_common()
            .set_color(Color::LIGHT_GRAY)
            .set_author(EmbedAuthor {
                name: format!("Agentes de {}", author.display_name()),
                icon_url: Some(author.avatar_url()),
            });

        for j in 0..AGENTS_PER_PAGE {
            let Some(agent) = agents.get(i + j) else {
                break;
            };

            let mut display_description = agent.description.clone();
            display_description.truncate(30);

            page = page.add_not_inlined_field(
                &agent.name,
                format!(
                    "ID: `{}`\nDescrição: `{}...`\nPreço por invocação: `{}₢`\n\nInvocações totais: `{}`\nRespostas totais: `{}`",
                    agent.identifier, display_description, agent.pricing.price_per_invocation, agent.stats.invocations, agent.stats.replies
                )
            );
        }

        pages.push(page);
    }

    EmbedPagination::new(ctx, pages)
        .set_ephemeral()
        .send()
        .await?;

    Ok(())
}
