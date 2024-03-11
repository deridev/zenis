use std::time::Duration;

use tokio_stream::StreamExt;
use zenis_discord::{
    twilight_gateway::Event,
    twilight_model::{
        channel::Message,
        gateway::payload::incoming::MessageCreate,
        id::{
            marker::{ChannelMarker, MessageMarker},
            Id,
        },
    },
    twilight_standby::{
        future::{WaitForComponentStream, WaitForMessageStream},
        Standby,
    },
    Interaction,
};

#[derive(Debug, Clone)]
pub struct WatcherOptions {
    pub timeout: Duration,
}

impl Default for WatcherOptions {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(60),
        }
    }
}

#[derive(Debug, Default)]
pub struct Watcher {
    standby: Standby,
}

impl Watcher {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn process(&self, event: &Event) {
        self.standby.process(event);
    }

    pub async fn await_single_message<F: Fn(&MessageCreate) -> bool + Sync + Send + 'static>(
        &self,
        channel_id: Id<ChannelMarker>,
        filter: F,
        options: WatcherOptions,
    ) -> anyhow::Result<Option<Message>> {
        let stream = self
            .standby
            .wait_for_message_stream(channel_id, filter)
            .timeout(options.timeout);
        tokio::pin!(stream);

        let Some(message) = stream.next().await else {
            return Ok(None);
        };

        let message = message?.0;

        Ok(Some(message))
    }

    pub async fn await_single_component<F: Fn(&Interaction) -> bool + Sync + Send + 'static>(
        &self,
        message_id: Id<MessageMarker>,
        filter: F,
        options: WatcherOptions,
    ) -> anyhow::Result<Option<Interaction>> {
        let stream = self
            .standby
            .wait_for_component_stream(message_id, filter)
            .timeout(options.timeout);
        tokio::pin!(stream);

        let Some(interaction) = stream.next().await else {
            return Ok(None);
        };

        let interaction = interaction?;

        Ok(Some(interaction))
    }

    pub fn create_message_stream<F: Fn(&MessageCreate) -> bool + Sync + Send + 'static>(
        &self,
        channel_id: Id<ChannelMarker>,
        filter: F,
        options: WatcherOptions,
    ) -> tokio_stream::Timeout<WaitForMessageStream> {
        self.standby
            .wait_for_message_stream(channel_id, filter)
            .timeout(options.timeout)
    }

    pub fn create_component_stream<F: Fn(&Interaction) -> bool + Sync + Send + 'static>(
        &self,
        message_id: Id<MessageMarker>,
        filter: F,
        options: WatcherOptions,
    ) -> tokio_stream::Timeout<WaitForComponentStream> {
        self.standby
            .wait_for_component_stream(message_id, filter)
            .timeout(options.timeout)
    }
}
