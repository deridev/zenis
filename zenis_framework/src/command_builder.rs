#![allow(unused)]
use std::collections::HashMap;

use zenis_discord::{
    twilight_model::{
        application::command::{CommandOption, CommandOptionType, CommandType},
        id::{
            marker::{ApplicationMarker, GuildMarker},
            Id,
        },
    },
    ApiCommand,
};

#[derive(Debug, Clone, PartialEq)]
pub struct CommandBuilder {
    pub command: ApiCommand,
}

impl CommandBuilder {
    pub fn new(
        application_id: Id<ApplicationMarker>,
        name: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            command: ApiCommand {
                application_id: Some(application_id),
                name: name.into(),
                description: description.into(),
                default_member_permissions: None,
                name_localizations: None,
                description_localizations: None,
                dm_permission: None,
                guild_id: None,
                id: None,
                nsfw: None,
                kind: CommandType::ChatInput,
                options: vec![],
                version: Id::new(1),
            },
        }
    }

    pub fn set_guild_id(mut self, guild_id: Id<GuildMarker>) -> Self {
        self.command.guild_id = Some(guild_id);
        self
    }

    pub fn add_option(mut self, option: CommandOptionBuilder) -> Self {
        self.command.options.push(option.build());
        self
    }

    pub fn set_options(mut self, options: Vec<CommandOption>) -> Self {
        self.command.options = options;
        self
    }

    pub fn build(self) -> ApiCommand {
        self.command
    }
}

pub struct CommandOptionBuilder {
    option: CommandOption,
}

impl CommandOptionBuilder {
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        kind: CommandOptionType,
    ) -> Self {
        Self {
            option: CommandOption {
                name: name.into(),
                description: description.into(),
                kind,
                autocomplete: None,
                channel_types: None,
                choices: None,
                name_localizations: None,
                description_localizations: None,
                min_length: None,
                max_length: None,
                min_value: None,
                max_value: None,
                options: None,
                required: None,
            },
        }
    }

    pub fn set_required(mut self, required: bool) -> Self {
        self.option.required = Some(required);
        self
    }

    pub fn set_min_max_length(mut self, min: u16, max: u16) -> Self {
        self.option.min_length = Some(min);
        self.option.max_length = Some(max);
        self
    }

    pub fn set_options(mut self, options: Vec<CommandOption>) -> Self {
        self.option.options = Some(options);
        self
    }

    pub fn build(self) -> CommandOption {
        self.option
    }
}
