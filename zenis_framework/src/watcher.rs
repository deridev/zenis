use std::time::Duration;

use tokio_stream::StreamExt;
use zenis_discord::{
    modal::ModalInteractionData,
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
    Interaction, InteractionData, InteractionType, ModalResponse,
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

    pub async fn await_single_modal<F: Fn(&Interaction) -> bool + Sync + Send + 'static>(
        &self,
        custom_id: impl Into<String>,
        filter: F,
        options: WatcherOptions,
    ) -> anyhow::Result<Option<ModalResponse>> {
        let custom_id: String = custom_id.into();

        let stream = self
            .standby
            .wait_for_event_stream(move |event: &Event| match event {
                Event::InteractionCreate(interaction) => {
                    let Some(modal_submit) = get_modal_submit(&interaction) else {
                        return false;
                    };

                    modal_submit.custom_id == custom_id && filter(&interaction)
                }
                _ => false,
            })
            .timeout(options.timeout);
        tokio::pin!(stream);

        let Some(event) = stream.next().await else {
            return Ok(None);
        };

        let event = event?;
        match event {
            Event::InteractionCreate(interaction) => {
                let Some(modal_submit) = get_modal_submit(&interaction) else {
                    return Ok(None);
                };

                Ok(Some(ModalResponse::new(
                    interaction.0.clone(),
                    modal_submit,
                )))
            }
            _ => Ok(None),
        }
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

fn get_modal_submit(interaction: &Interaction) -> Option<ModalInteractionData> {
    if interaction.kind != InteractionType::ModalSubmit {
        return None;
    }

    match &interaction.data {
        Some(InteractionData::ModalSubmit(modal_submit)) => Some(modal_submit.clone()),
        _ => None,
    }
}
