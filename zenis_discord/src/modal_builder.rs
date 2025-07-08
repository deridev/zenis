use twilight_model::{
    channel::message::{
        Component,
        component::{ActionRow, TextInput},
    },
    http::interaction::InteractionResponseData,
};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Modal {
    title: String,
    custom_id: String,
    components: Vec<Component>,
}

impl Modal {
    fn build(self) -> InteractionResponseData {
        InteractionResponseData {
            title: Some(self.title),
            custom_id: Some(self.custom_id),
            components: Some(self.components),
            ..InteractionResponseData::default()
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ModalBuilder {
    custom_id: String,
    pub modal: Modal,
}

impl ModalBuilder {
    pub fn new(title: impl Into<String>, custom_id: impl Into<String>) -> ModalBuilder {
        let custom_id: String = custom_id.into();
        Self {
            custom_id: custom_id.clone(),
            modal: Modal {
                title: title.into(),
                custom_id: custom_id,
                components: Vec::new(),
            },
        }
    }

    pub fn custom_id(&self) -> &str {
        &self.custom_id
    }

    pub fn add_text_input(mut self, text_input: TextInput) -> Self {
        self.modal.components.push(Component::ActionRow(ActionRow {
            components: vec![Component::TextInput(text_input)],
        }));
        self
    }

    pub fn build(self) -> InteractionResponseData {
        self.modal.build()
    }
}
