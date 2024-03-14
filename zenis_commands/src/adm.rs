use crate::prelude::*;

type IdString = String;

#[derive(Debug, Clone)]
pub enum Command {
    Help,
    ServerCount,
    ResetCache(Option<IdString>),
    AddCredits(IdString, i64),
    RemoveCredits(IdString, i64),
    ClearChannelInstances(u64, String),
    ClearInstances(String),
    Accept(String),
    Reject(String, String),
}

impl Command {
    pub const HELP_EXAMPLES: &'static [&'static str] = &[
        "help",
        "servercount",
        "reset cache [id]",
        "add credits <id> [quantity]",
        "remove credits <id> [quantity]",
        "clear instances [reason]",
        "accept <id>",
        "reject <id> <reason>",
    ];

    pub fn parse(input: &str) -> Option<Command> {
        let mut splitted = input.split(' ');

        let command = splitted.next()?.to_lowercase();

        match command.as_str() {
            "help" => Some(Command::Help),
            "servercount" => Some(Command::ServerCount),
            "reset" => {
                let subcommand = splitted.next()?.to_lowercase();
                match subcommand.as_str() {
                    "cache" => {
                        let id = splitted.next().map(str::to_owned);
                        Some(Command::ResetCache(id))
                    }
                    _ => None,
                }
            }
            "add" | "remove" => {
                let subcommand = splitted.next()?.to_lowercase();

                match subcommand.as_str() {
                    "credits" => {
                        let id = splitted.next()?.to_lowercase();
                        let quantity = splitted.next().unwrap_or("1").parse::<i64>().ok()?;

                        match command.as_str() {
                            "add" => Some(Command::AddCredits(id, quantity)),
                            "remove" => Some(Command::RemoveCredits(id, quantity)),
                            _ => None,
                        }
                    }
                    _ => None,
                }
            }
            "clear" => {
                let subcommand = splitted.next()?.to_lowercase();

                match subcommand.as_str() {
                    "channelinstances" => {
                        let id = splitted.next()?.to_lowercase().parse::<u64>().ok()?;
                        let reason = splitted.collect::<Vec<_>>().join(" ");
                        Some(Command::ClearChannelInstances(id, reason))
                    }
                    "instances" => {
                        let reason = splitted.collect::<Vec<_>>().join(" ");
                        Some(Command::ClearInstances(reason))
                    }
                    _ => None,
                }
            }
            "accept" => {
                let id = splitted.next()?.to_lowercase();
                Some(Command::Accept(id))
            }
            "reject" => {
                let id = splitted.next()?.to_lowercase();
                let reason = splitted.collect::<Vec<_>>().join(" ");
                Some(Command::Reject(id, reason))
            }
            _ => None,
        }
    }
}

#[command("Comando restrito para administradores Zenis.")]
#[name("adm")]
#[character_required(true)]
pub async fn adm(
    mut ctx: CommandContext,
    #[rename("cmd")]
    #[description("Comando a ser executado")]
    command: String,
) -> anyhow::Result<()> {
    let author = ctx.author().await?;
    if author.id.get() != 518830049949122571 {
        ctx.reply(
            Response::new_user_reply(&author, "você não tem permissão para usar esse comando!")
                .add_emoji_prefix(emojis::ERROR),
        )
        .await?;
        return Ok(());
    }

    let Some(command) = Command::parse(&command) else {
        ctx.reply(
            Response::new_user_reply(&author, "comando inválido!").add_emoji_prefix(emojis::ERROR),
        )
        .await?;
        return Ok(());
    };

    macro_rules! parse_id {
        ($id:expr) => {{
            let id = $id.clone().unwrap_or(author.id.to_string());
            if id.as_str() == "self" {
                author.id.to_string()
            } else {
                id
            }
        }};
    }

    macro_rules! id_to_discord_id {
        ($id:expr) => {{
            let id = $id.clone().parse::<u64>().unwrap_or(12345678);
            Id::new(id)
        }};
    }

    match command {
        Command::Help => {
            ctx.reply(Response::new_user_reply(
                &author,
                format!(
                    "exemplos de comandos:\n```\n{}\n```",
                    Command::HELP_EXAMPLES.join("\n")
                ),
            ))
            .await?;
        }
        Command::ServerCount => {
            let guilds = ctx
                .client
                .http
                .current_user_guilds()
                .await?
                .models()
                .await?
                .len();
            ctx.reply(Response::new_user_reply(
                &author,
                format!("**{}** servidores", guilds),
            ))
            .await?;
        }
        Command::ResetCache(id) => {
            let id = parse_id!(id);
            if let Ok(user) = ctx.client.get_user(id_to_discord_id!(id)).await {
                let user_data = ctx.db().users().get_by_user(user.id).await?;
                ctx.db().users().remove_from_cache(&user_data);

                ctx.reply(Response::new_user_reply(
                    &author,
                    format!(
                        "você resetou o cache de {} com sucesso.",
                        user.display_name()
                    ),
                ))
                .await?;
            } else if let Ok(guild) = ctx.client.get_guild(id_to_discord_id!(id)).await {
                let guild_data = ctx.db().guilds().get_by_guild(guild.id).await?;
                ctx.db().guilds().remove_from_cache(&guild_data);

                ctx.reply(Response::new_user_reply(
                    &author,
                    format!(
                        "você resetou o cache do servidor {} com sucesso.",
                        guild.name
                    ),
                ))
                .await?;
            }
        }
        Command::AddCredits(id, quantity) => {
            let id = parse_id!(Some(id));
            if let Ok(user) = ctx.client.get_user(id_to_discord_id!(id)).await {
                let mut user_data = ctx.db().users().get_by_user(user.id).await?;
                user_data.add_credits(quantity);
                ctx.db().users().save(user_data).await?;

                ctx.reply(Response::new_user_reply(
                    &author,
                    format!(
                        "você adicionou **{quantity}₢** ao usuário {} com sucesso.",
                        user.display_name()
                    ),
                ))
                .await?;
            } else if let Ok(guild) = ctx.client.get_guild(id_to_discord_id!(id)).await {
                let mut guild_data = ctx.db().guilds().get_by_guild(guild.id).await?;
                guild_data.add_credits(quantity);
                ctx.db().guilds().save(guild_data).await?;

                ctx.reply(Response::new_user_reply(
                    &author,
                    format!(
                        "você adicionou **{quantity}₢** ao servidor {} com sucesso.",
                        guild.name
                    ),
                ))
                .await?;
            }
        }
        Command::RemoveCredits(id, quantity) => {
            let id = parse_id!(Some(id));
            if let Ok(user) = ctx.client.get_user(id_to_discord_id!(id)).await {
                let mut user_data = ctx.db().users().get_by_user(user.id).await?;
                user_data.remove_credits(quantity);
                ctx.db().users().save(user_data).await?;

                ctx.reply(Response::new_user_reply(
                    &author,
                    format!(
                        "você removeu **{quantity}₢** do usuário {} com sucesso.",
                        user.display_name()
                    ),
                ))
                .await?;
            } else if let Ok(guild) = ctx.client.get_guild(id_to_discord_id!(id)).await {
                let mut guild_data = ctx.db().guilds().get_by_guild(guild.id).await?;
                guild_data.remove_credits(quantity);
                ctx.db().guilds().save(guild_data).await?;

                ctx.reply(Response::new_user_reply(
                    &author,
                    format!(
                        "você removeu **{quantity}₢** do servidor {} com sucesso.",
                        guild.name
                    ),
                ))
                .await?;
            }
        }
        Command::ClearChannelInstances(id, reason) => {
            let instances = ctx.db().instances().all_actives_in_channel(id).await?;
            let mut counter = 0;
            for mut instance in instances {
                counter += 1;
                instance.exit_reason = Some(reason.clone());
                ctx.db().instances().save(instance).await?;
            }

            ctx.reply(Response::new_user_reply(
                &author,
                format!(
                    "**{}** instâncias foram desligadas com sucesso no canal **{}**.",
                    counter, id
                ),
            ))
            .await?;
        }
        Command::ClearInstances(reason) => {
            let reason = if reason.is_empty() {
                "Desligado por administrador Zenis.".to_string()
            } else {
                reason.to_string()
            };

            let instances = ctx.db().instances().all_actives().await?;
            let mut counter = 0;
            for mut instance in instances {
                counter += 1;
                instance.exit_reason = Some(reason.clone());
                ctx.db().instances().save(instance).await?;
            }

            ctx.reply(Response::new_user_reply(
                &author,
                format!("**{}** instâncias foram desligadas com sucesso.", counter),
            ))
            .await?;
        }
        Command::Accept(id) => {
            let Some(mut agent) = ctx.db().agents().get_by_identifier(id).await? else {
                ctx.send(
                    Response::new_user_reply(&author, "agente inválido ou inexistente")
                        .add_emoji_prefix(emojis::ERROR),
                )
                .await?;
                return Ok(());
            };

            if !agent.is_waiting_for_approval {
                ctx.send(
                    Response::new_user_reply(
                        &author,
                        "o agente não está em processo de verificação.",
                    )
                    .add_emoji_prefix(emojis::ERROR),
                )
                .await?;
                return Ok(());
            }

            agent.is_waiting_for_approval = false;
            agent.public = true;
            ctx.db().agents().save(agent.clone()).await?;

            ctx.send(
                Response::new_user_reply(&author, "agente aceito com sucesso!")
                    .add_emoji_prefix(emojis::SUCCESS),
            )
            .await
            .ok();

            let dm = ctx
                .client
                .http
                .create_private_channel(Id::new(agent.creator_user_id))
                .await?
                .model()
                .await?;
            let embed = EmbedBuilder::new_common()
                .set_color(Color::GREEN)
                .set_author(EmbedAuthor {
                    name: "Agente aceito!".to_string(),
                    icon_url: agent.agent_url_image.clone(),
                })
                .set_description(format!("## {} Agente aceito!\nO seu agente **{}** (`{}`) foi aceito e agora é PÚBLICO. Parabéns!", emojis::SUCCESS, agent.name, agent.identifier));

            ctx.client
                .http
                .create_message(dm.id)
                .embeds(&[embed.build()])?
                .await
                .ok();
        }
        Command::Reject(id, reason) => {
            let Some(mut agent) = ctx.db().agents().get_by_identifier(id).await? else {
                ctx.send(
                    Response::new_user_reply(&author, "agente inválido ou inexistente")
                        .add_emoji_prefix(emojis::ERROR),
                )
                .await?;
                return Ok(());
            };

            if !agent.is_waiting_for_approval {
                ctx.send(
                    Response::new_user_reply(
                        &author,
                        "o agente não está em processo de verificação.",
                    )
                    .add_emoji_prefix(emojis::ERROR),
                )
                .await?;
                return Ok(());
            }

            agent.is_waiting_for_approval = false;
            agent.public = false;
            ctx.db().agents().save(agent.clone()).await?;

            ctx.send(
                Response::new_user_reply(&author, "agente rejeitado com sucesso!")
                    .add_emoji_prefix(emojis::SUCCESS),
            )
            .await
            .ok();

            let dm = ctx
                .client
                .http
                .create_private_channel(Id::new(agent.creator_user_id))
                .await?
                .model()
                .await?;
            let embed = EmbedBuilder::new_common()
                .set_color(Color::RED)
                .set_author(EmbedAuthor {
                    name: "Agente rejeitado!".to_string(),
                    icon_url: agent.agent_url_image.clone(),
                })
                .set_description(format!("## {} Agente rejeitado!\nO seu agente **{}** (`{}`) foi rejeitado.\n**Motivo da recusa:** `{}`", emojis::ERROR, agent.name, agent.identifier, reason))
                .add_footer_text("Não acha que a recusa foi justa? Entre no /servidoroficial e fale com o suporte do bot.");

            ctx.client
                .http
                .create_message(dm.id)
                .embeds(&[embed.build()])?
                .await
                .ok();
        }
    }

    Ok(())
}
