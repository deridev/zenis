use std::sync::Arc;

use zenis_commands::COMMANDS;
use zenis_common::config;
use zenis_database::ZenisDatabase;
use zenis_discord::{
    application_command::CommandOptionValue,
    twilight_http::client::InteractionClient,
    twilight_model::{
        application::command::CommandOptionType,
        gateway::payload::incoming::InteractionCreate,
        id::{
            marker::{ApplicationMarker, GuildMarker},
            Id,
        },
    },
    ApiCommand, InteractionData,
};
use zenis_framework::{
    watcher::Watcher, CommandBuilder, CommandContext, CommandOptionBuilder, ZenisClient,
};

pub async fn execute_command(
    interaction: Box<InteractionCreate>,
    client: Arc<ZenisClient>,
    watcher: Arc<Watcher>,
    database: Arc<ZenisDatabase>,
) -> anyhow::Result<()> {
    let data = interaction
        .data
        .clone()
        .and_then(|d| match d {
            InteractionData::ApplicationCommand(data) => Some(data),
            _ => None,
        })
        .ok_or(anyhow::anyhow!("Data not found"))?;

    let mut options = data.options.clone();

    let subcommand = match data.options.first() {
        Some(option) => match &option.value {
            CommandOptionValue::SubCommand(suboptions) => {
                options = suboptions.clone();
                Some(option.name.clone())
            }
            _ => None,
        },
        None => None,
    };

    let command_key = match subcommand {
        Some(subcommand) => format!("{} {subcommand}", data.name),
        None => data.name.to_owned(),
    };

    let ctx = CommandContext::new(
        client.clone(),
        Box::new(interaction.0),
        watcher,
        database,
        options,
    );
    let command = COMMANDS
        .get(&command_key)
        .ok_or(anyhow::anyhow!("Command not found"))?;

    let result = command.run(ctx).await;
    if result.is_err() {
        eprintln!("[ERROR]\n{}", result.unwrap_err());
    }

    Ok(())
}

pub async fn register_commands(application_id: Id<ApplicationMarker>, client: Arc<ZenisClient>) {
    let commands: Vec<CommandBuilder> = {
        let mut parent_commands: Vec<(String, CommandBuilder)> = Vec::new();
        let mut commands = Vec::new();

        for (name, command) in COMMANDS.iter() {
            let splitted_name = name
                .split_ascii_whitespace()
                .map(str::to_owned)
                .collect::<Vec<_>>();
            if splitted_name.len() == 1 {
                commands.push(command.build_command(application_id));
            } else {
                parent_commands.push((
                    splitted_name[0].clone(),
                    command.build_command(application_id),
                ));
            }
        }

        for (parent_name, command) in parent_commands.into_iter() {
            let parent_command = match commands
                .iter_mut()
                .find(|cmd| cmd.command.name == parent_name)
            {
                Some(command) => command,
                None => {
                    commands.push(CommandBuilder::new(application_id, &parent_name, "Group"));
                    commands
                        .iter_mut()
                        .find(|cmd| cmd.command.name == parent_name)
                        .unwrap()
                }
            };

            let builder = parent_command.clone();
            let subname = command
                .command
                .name
                .split_ascii_whitespace()
                .collect::<Vec<_>>();
            let subname = subname[1].to_owned();
            *parent_command = builder.add_option(
                CommandOptionBuilder::new(
                    subname,
                    command.command.description.clone(),
                    CommandOptionType::SubCommand,
                )
                .set_options(command.command.options),
            );
        }

        commands
    };

    let guild_id = match config::DEBUG {
        true => Some(Id::new(config::DEBUG_GUILD_ID)),
        false => None,
    };

    register_http_commands(
        guild_id,
        commands
            .into_iter()
            .map(|mut c| {
                if let Some(guild_id) = guild_id {
                    c = c.clone().set_guild_id(guild_id);
                }

                let build = c.build();
                println!(
                    "Registering command {}{}",
                    build.name,
                    if config::DEBUG { " (DEBUG)" } else { "" }
                );

                build
            })
            .collect::<Vec<ApiCommand>>()
            .as_slice(),
        client.http.interaction(application_id),
    )
    .await;
}

async fn register_http_commands<'a>(
    guild_id: Option<Id<GuildMarker>>,
    commands: &[ApiCommand],
    interaction: InteractionClient<'a>,
) {
    match guild_id {
        Some(guild_id) => {
            interaction
                .set_guild_commands(guild_id, commands)
                .await
                .expect("Failed to register guild commands");
        }
        _ => {
            interaction
                .set_global_commands(commands)
                .await
                .expect("Failed to register global commands");
        }
    };
}
