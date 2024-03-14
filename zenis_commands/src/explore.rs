use std::time::Duration;

use zenis_database::agent_model::AgentModel;
use zenis_discord::twilight_model::channel::message::component::ButtonStyle;
use zenis_framework::{util::make_multiple_rows, watcher::WatcherOptions};

use crate::prelude::*;

const AGENTS_PER_PAGE: usize = 4;

#[command("Veja todos os agentes que você pode invocar!")]
#[name("explorar")]
pub async fn explore(mut ctx: CommandContext) -> anyhow::Result<()> {
    let author = ctx.author().await?;
    let guild_id = ctx.interaction.guild_id;
    let author_id = author.id;

    let buttons = vec![
        ButtonBuilder::new()
            .set_custom_id("public")
            .set_label("Agentes públicos")
            .set_style(ButtonStyle::Secondary),
        ButtonBuilder::new()
            .set_custom_id("private")
            .set_label("Agentes privados")
            .set_style(ButtonStyle::Secondary),
    ];
    let message = ctx
        .send(
            Response::new_user_reply(
                &author,
                "você quer explorar **agentes públicos** ou **agentes privados** do servidor?",
            )
            .add_emoji_prefix("❔")
            .set_components(make_multiple_rows(buttons.clone())),
        )
        .await?;

    let Ok(Some(interaction)) = ctx
        .watcher
        .await_single_component(
            message.id,
            move |interaction| interaction.author_id() == Some(author_id),
            WatcherOptions {
                timeout: Duration::from_secs(20),
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

    let db = ctx.db();
    if data.custom_id == "public" {
        generate_pagination(&mut ctx, &author, db.agents().get_all_public().await?).await?;
    } else if data.custom_id == "private" {
        let mut private = ctx
            .db()
            .agents()
            .get_all_by_creator(author_id.get())
            .await?;

        let mut guild_private_agents = match guild_id {
            Some(guild_id) => {
                ctx.db()
                    .agents()
                    .get_all_private_by_guild(guild_id.get())
                    .await?
            }
            None => vec![],
        };

        private.append(&mut guild_private_agents);

        generate_pagination(&mut ctx, &author, private).await?;
    }

    Ok(())
}

async fn generate_pagination(
    ctx: &mut CommandContext,
    author: &User,
    mut agents: Vec<AgentModel>,
) -> anyhow::Result<()> {
    if agents.is_empty() {
        ctx.reply(
            Response::new_user_reply(
                author,
                "você não encontrou nenhum agente! Crie agentes com **/criar agente**!",
            )
            .add_emoji_prefix(emojis::ERROR),
        )
        .await?;
        return Ok(());
    }

    agents.sort_unstable_by_key(|agent| agent.stats.invocations);
    agents.retain(|agent| !agent.tags.contains("special"));

    let mut pages = vec![];
    for i in (0..agents.len()).step_by(AGENTS_PER_PAGE) {
        let mut page = EmbedBuilder::new_common()
            .set_color(Color::LIGHT_ORANGE)
            .set_author(EmbedAuthor {
                name: format!("Agentes invocáveis por {}", author.display_name()),
                icon_url: Some(author.avatar_url()),
            });

        for j in 0..AGENTS_PER_PAGE {
            let Some(agent) = agents.get(i + j) else {
                break;
            };

            let mut display_description = agent.description.clone();
            display_description.truncate(80);

            page = page.add_not_inlined_field(
                &agent.name,
                format!(
                    "**ID**: `{}`\n**Descrição**: `{}...`\n**Preço por invocação**: `{}₢`",
                    agent.identifier,
                    display_description,
                    if agent.pricing.price_per_invocation > 0 {
                        agent.pricing.price_per_invocation.to_string()
                    } else {
                        "GRÁTIS ".to_string()
                    }
                ),
            );

            if let Some(image_url) = &agent.agent_url_image {
                page = page.set_thumbnail(image_url);
            }
        }

        pages.push(page);
    }

    EmbedPagination::new(ctx.clone(), pages).send().await?;

    Ok(())
}
