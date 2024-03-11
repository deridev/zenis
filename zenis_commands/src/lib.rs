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

mod common;
mod guild;
mod invoke;
mod wallet;

mod owner;

pub type CommandMap = HashMap<String, BoxedCommand>;

pub static COMMANDS: Lazy<CommandMap> = Lazy::new(|| {
    let mut map: CommandMap = HashMap::new();

    register_command!(map, common::PingCommand);
    register_command!(map, invoke::InvokeCommand);
    register_command!(map, wallet::WalletCommand);
    register_command!(map, guild::GuildCommand);

    register_command!(map, owner::OwnerCommand);

    map
});
