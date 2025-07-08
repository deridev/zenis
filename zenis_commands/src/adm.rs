use rand::{prelude::SliceRandom, rngs::StdRng, SeedableRng};
use zenis_database::user_model::AdminPermission;

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
    AddPerm(IdString, AdminPermission),
    RemovePerm(IdString, AdminPermission),
    SuperGiveaway(u32, i64, u64),
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
        "add perm <id> <permission>",
        "remove perm <id> <permission>",
        "supergiveaway <amount of guilds> <credits>, <min_members>",
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
                    "perm" => {
                        let id = splitted.next()?.to_lowercase();
                        let permission = splitted.next()?.to_lowercase();

                        match command.as_str() {
                            "add" => Some(Command::AddPerm(id, permission.try_into().ok()?)),
                            "remove" => Some(Command::RemovePerm(id, permission.try_into().ok()?)),
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
            "supergiveaway" => {
                let amount_of_guilds = splitted.next()?.parse::<u32>().ok()?;
                let credits = splitted.next()?.parse::<i64>().ok()?;
                let min_members = splitted.next()?.parse::<u64>().ok()?;
                Some(Command::SuperGiveaway(
                    amount_of_guilds,
                    credits,
                    min_members,
                ))
            }
            _ => None,
        }
    }

    pub const fn required_permissions(&self) -> &[AdminPermission] {
        match self {
            Command::Help => &[],
            Command::ServerCount => &[],
            Command::ResetCache(_) => &[],
            Command::AddCredits(_, _) => &[AdminPermission::ManageCredits],
            Command::RemoveCredits(_, _) => &[AdminPermission::ManageCredits],
            Command::ClearChannelInstances(_, _) => &[AdminPermission::ManageInstances],
            Command::ClearInstances(_) => &[AdminPermission::ManageInstances],
            Command::Accept(_) => &[AdminPermission::ManageAgents],
            Command::Reject(_, _) => &[AdminPermission::ManageAgents],
            Command::AddPerm(_, _) => &[AdminPermission::All],
            Command::RemovePerm(_, _) => &[AdminPermission::All],
            Command::SuperGiveaway(..) => &[AdminPermission::God],
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
    let author_data = ctx.db().users().get_by_user(author.id).await?;

    macro_rules! check_permission {
        ($permission:expr) => {{
            if !author_data.has_admin_permission($permission) {
                ctx.send(
                    Response::new_user_reply(
                        &author,
                        format!("você precisa da permissão de administrador **{}** para executar essa ação!", $permission),
                    )
                    .add_emoji_prefix("⛔"),
                )
                .await?;
                return Ok(());
            }
        }};
    }

    check_permission!(AdminPermission::UseAdmCommand);

    let Some(command) = Command::parse(&command) else {
        ctx.reply(
            Response::new_user_reply(
                &author,
                "comando ou argumentos inválidos! Use **/adm cmd: help**.",
            )
            .add_emoji_prefix(emojis::ERROR),
        )
        .await?;
        return Ok(());
    };

    for perm in command.required_permissions() {
        check_permission!(*perm);
    }

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
                .embeds(&[embed.build()])
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
                .embeds(&[embed.build()])
                .await
                .ok();
        }
        Command::AddPerm(id, permission) => {
            let id = parse_id!(Some(id));
            if let Ok(user) = ctx.client.get_user(id_to_discord_id!(id)).await {
                let mut user_data = ctx.db().users().get_by_user(user.id).await?;
                user_data.insert_admin_permission(permission);
                ctx.db().users().save(user_data).await?;

                ctx.send(
                    Response::new_user_reply(
                        &author,
                        format!("permissão `{}` adicionada com sucesso!", permission),
                    )
                    .add_emoji_prefix(emojis::SUCCESS),
                )
                .await
                .ok();
            }
        }
        Command::RemovePerm(id, permission) => {
            let id = parse_id!(Some(id));
            if let Ok(user) = ctx.client.get_user(id_to_discord_id!(id)).await {
                let mut user_data = ctx.db().users().get_by_user(user.id).await?;
                user_data.remove_admin_permission(permission);
                ctx.db().users().save(user_data).await?;

                ctx.send(
                    Response::new_user_reply(
                        &author,
                        format!("permissão `{}` removida com sucesso!", permission),
                    )
                    .add_emoji_prefix(emojis::SUCCESS),
                )
                .await
                .ok();
            }
        }
        Command::SuperGiveaway(amount_of_guilds, credits, min_members) => {
            ctx.send("Sorteando...").await?;
            let client_user = ctx.client.current_user().await?;

            let all_guild_models = ctx.db().guilds().get_all_guilds().await?;

            let mut all_guilds = {
                let mut guilds = Vec::with_capacity(amount_of_guilds as usize);
                for data in all_guild_models.iter() {
                    let Ok(guild) = ctx.client.get_guild(data.guild_id.parse().unwrap()).await
                    else {
                        continue;
                    };

                    if guild.approximate_member_count.unwrap_or(0) < min_members {
                        continue;
                    }

                    guilds.push((guild, data.clone()));
                }

                guilds
            };

            all_guilds.shuffle(&mut StdRng::from_os_rng());

            let guilds = all_guilds.into_iter().take(amount_of_guilds as usize);

            let mut counter = 0;
            for (guild, mut data) in guilds {
                data.add_public_credits(credits);
                ctx.db().guilds().save(data).await?;

                counter += 1;

                'f: for channel in guild.channels.iter() {
                    let embed = EmbedBuilder::new_common()
                        .set_color(Color::YELLOW)
                        .set_author(EmbedAuthor {
                            name: "O servidor ganhou créditos!".to_string(),
                            icon_url: Some(client_user.avatar_url()),
                        })
                        .set_description(format!("## {} Parabéns! O administrador {} sorteou alguns créditos, e o seu servidor `{}` venceu!\n\n**{}₢ créditos** foram adicionados à carteira pública do seu servidor. Use **/invocar** e se divirtam!\n**Não sabe usar Zenis? Use `/tutorial` e vai testando os comandos!**", emojis::CREDIT, author.display_name(), guild.name, credits));

                    if ctx
                        .client
                        .http
                        .create_message(channel.id)
                        .embeds(&[embed.build()])
                        .await
                        .is_ok()
                    {
                        break 'f;
                    }
                }
            }

            ctx.send(Response::new_user_reply(&author, format!("**{} servidores com mais de {} membros** foram sorteados e receberam **{}₢ créditos**!", counter, min_members, credits)).add_emoji_prefix(emojis::SUCCESS)).await?;
        }
    }

    Ok(())
}
