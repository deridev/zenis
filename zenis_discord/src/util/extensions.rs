use twilight_model::{
    application::interaction::{
        message_component::MessageComponentInteractionData, Interaction, InteractionData,
    },
    guild::Guild,
    user::User,
};

pub trait UserExtension {
    fn avatar_url(&self) -> String;
    fn mention(&self) -> String;
    fn display_name(&self) -> String;
}

impl UserExtension for User {
    fn avatar_url(&self) -> String {
        let Some(avatar) = self.avatar else {
            return "https://external-preview.redd.it/fauTrGFvbnTjWM6A6AC8sGqohLQxKHQTfZjhtPbWY7g.jpg?auto=webp&s=5d8e36356dead73ec2e624e41659d411b5fbca53".into();
        };

        format!(
            "https://cdn.discordapp.com/avatars/{}/{}.png",
            self.id, avatar
        )
    }

    fn mention(&self) -> String {
        format!("<@{}>", self.id)
    }

    fn display_name(&self) -> String {
        self.global_name.clone().unwrap_or(self.name.clone())
    }
}

pub trait GuildExtension {
    fn icon_url(&self) -> String;
}

impl GuildExtension for Guild {
    fn icon_url(&self) -> String {
        let Some(icon) = self.icon else {
            return "https://external-preview.redd.it/fauTrGFvbnTjWM6A6AC8sGqohLQxKHQTfZjhtPbWY7g.jpg?auto=webp&s=5d8e36356dead73ec2e624e41659d411b5fbca53".into();
        };

        format!("https://cdn.discordapp.com/icons/{}/{}.png", self.id, icon)
    }
}

pub trait InteractionExtension {
    fn parse_message_component_data(&self) -> anyhow::Result<MessageComponentInteractionData>;
}

impl InteractionExtension for Interaction {
    fn parse_message_component_data(&self) -> anyhow::Result<MessageComponentInteractionData> {
        if let Some(InteractionData::MessageComponent(data)) = self.data.clone() {
            Ok(data)
        } else {
            anyhow::bail!("invalid message component");
        }
    }
}
