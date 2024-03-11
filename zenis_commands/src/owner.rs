use zenis_payment::mp::common::Item;

use crate::prelude::*;

type IdString = String;

#[derive(Debug, Clone)]
pub enum Command {
    Help,
    ResetCache(Option<IdString>),
    AddCredits(IdString, i64),
    RemoveCredits(IdString, i64),
    Test,
}

impl Command {
    pub const HELP_EXAMPLES: &'static [&'static str] = &[
        "help",
        "reset cache [id]",
        "add credits <id> [quantity]",
        "remove credits <id> [quantity]",
    ];

    pub fn parse(input: &str) -> Option<Command> {
        let mut splitted = input.split(' ');

        let command = splitted.next()?.to_lowercase();

        match command.as_str() {
            "help" => Some(Command::Help),
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
            "test" => Some(Command::Test),
            _ => None,
        }
    }
}

#[command("Comando restrito.")]
#[name("owner")]
#[character_required(true)]
pub async fn owner(
    mut ctx: CommandContext,
    #[rename("comando")]
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
        Command::Test => {
            let preference = ctx
                .client
                .mp_client
                .create_preference(vec![Item::simple(
                    10.0,
                    "1000 Créditos",
                    "Créditos são usados para interagir com agentes IA",
                    1,
                )])
                .await?;

            ctx.reply(format!("```json\n{:?}\n```", preference)).await?;
        }
    }

    Ok(())
}
