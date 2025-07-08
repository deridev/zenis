use crate::{CommandBuilder, CommandContext};
use zenis_discord::twilight_model::id::{marker::ApplicationMarker, Id};

#[derive(Debug, Clone)]
pub struct CommandConfig;

#[async_trait::async_trait]
pub trait Command {
    fn command_config(&self) -> CommandConfig;
    fn build_command(&self, application_id: Id<ApplicationMarker>) -> CommandBuilder;
    async fn run(&self, ctx: CommandContext) -> anyhow::Result<()>;
}
