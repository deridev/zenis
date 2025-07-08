use twilight_model::{
    application::interaction::{Interaction, modal::ModalInteractionData},
    channel::message::component::ComponentType,
};

#[derive(Debug, Clone, PartialEq)]
pub struct ModalResponse {
    interaction: Interaction,
    data: ModalInteractionData,
}

impl ModalResponse {
    pub fn new(interaction: Interaction, data: ModalInteractionData) -> Self {
        Self { interaction, data }
    }

    pub fn data(&self) -> &ModalInteractionData {
        &self.data
    }

    pub fn interaction(&self) -> Box<Interaction> {
        self.interaction.clone().into()
    }

    pub fn get_text_input(&self, custom_id: &str) -> Option<String> {
        let components = self
            .data
            .components
            .iter()
            .map(|c| &c.components)
            .flatten()
            .collect::<Vec<_>>();
        for component in &components {
            if component.custom_id == custom_id && component.kind == ComponentType::TextInput {
                return component.value.clone();
            }
        }

        None
    }
}
