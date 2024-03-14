mod prelude;
pub mod util;

use once_cell::sync::Lazy;
use std::collections::HashMap;
use zenis_framework::Command;

type BoxedCommand = Box<(dyn Command + Send + Sync)>;

#[macro_export]
macro_rules! register_command {
    ($map:expr, $command_pat:expr) => {{
        use $crate::prelude::*;
        let cmd = $command_pat;
        $map.insert(
            // Build the command with a dummy ID just to get its name
            cmd.build_command(Id::new(12345678)).command.name,
            Box::new(cmd),
        );
    }};
}

mod buy;
mod common;
mod configure_agent;
mod create_agent;
mod explore;
mod guild;
mod invoke;
mod my_agents;
mod officialguild;
mod wallet;
mod invite;
mod tutorial;

mod adm;

pub type CommandMap = HashMap<String, BoxedCommand>;

pub static COMMANDS: Lazy<CommandMap> = Lazy::new(|| {
    let mut map: CommandMap = HashMap::new();

    register_command!(map, common::PingCommand);
    register_command!(map, invoke::InvokeCommand);
    register_command!(map, wallet::WalletCommand);
    register_command!(map, guild::GuildCommand);
    register_command!(map, buy::BuyCommand);
    register_command!(map, tutorial::TutorialCommand);
    register_command!(map, invite::InviteCommand);
    register_command!(map, explore::ExploreCommand);
    register_command!(map, my_agents::My_agentsCommand);
    register_command!(map, create_agent::Create_agentCommand);
    register_command!(map, configure_agent::Configure_agentCommand);
    register_command!(map, officialguild::OfficialguildCommand);

    register_command!(map, adm::AdmCommand);

    map
});
