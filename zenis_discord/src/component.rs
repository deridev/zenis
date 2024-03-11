#![allow(unused)]
use twilight_model::channel::message::{
    component::{ActionRow, Button, ButtonStyle},
    Component, ReactionType,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionRowBuilder {
    data: ActionRow,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ButtonBuilder {
    pub data: Button,
}

impl From<Component> for ButtonBuilder {
    fn from(component: Component) -> Self {
        if let Component::Button(data) = component {
            Self { data }
        } else {
            Self::new()
        }
    }
}

impl Default for ButtonBuilder {
    fn default() -> Self {
        Self {
            data: Button {
                custom_id: None,
                style: ButtonStyle::Secondary,
                disabled: false,
                emoji: None,
                label: None,
                url: None,
            },
        }
    }
}

impl ButtonBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_custom_id(mut self, custom_id: impl ToString) -> Self {
        self.data.custom_id = Some(custom_id.to_string());
        self
    }

    pub fn set_label(mut self, label: impl ToString) -> Self {
        self.data.label = Some(label.to_string());
        self
    }

    pub fn set_style(mut self, style: ButtonStyle) -> Self {
        self.data.style = style;
        self
    }

    pub fn set_emoji(mut self, emoji: impl Into<ReactionType>) -> Self {
        self.data.emoji = Some(emoji.into());
        self
    }

    pub fn set_disabled(mut self, disabled: bool) -> Self {
        self.data.disabled = disabled;
        self
    }

    pub fn build(self) -> Component {
        Component::Button(self.data)
    }
}

impl Default for ActionRowBuilder {
    fn default() -> Self {
        Self {
            data: ActionRow { components: vec![] },
        }
    }
}

impl ActionRowBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_button(mut self, button: ButtonBuilder) -> Self {
        self.data.components.push(button.build());
        self
    }

    pub fn add_buttons(mut self, buttons: Vec<ButtonBuilder>) -> Self {
        let mut buttons: Vec<Component> = buttons.iter().cloned().map(|b| b.build()).collect();
        self.data.components.append(&mut buttons);
        self
    }

    pub fn build(self) -> Component {
        Component::ActionRow(self.data)
    }
}

impl From<ActionRowBuilder> for Component {
    fn from(value: ActionRowBuilder) -> Self {
        value.build()
    }
}
