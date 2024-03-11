use zenis_discord::{
    twilight_model::{
        channel::message::{Component, MessageFlags},
        http::{attachment::Attachment, interaction::InteractionResponseData},
        user::User,
    },
    EmbedBuilder, UserExtension,
};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Response {
    pub content: Option<String>,
    pub embeds: Option<Vec<EmbedBuilder>>,
    pub flags: Option<MessageFlags>,
    pub components: Option<Vec<Component>>,
    pub attachments: Option<Vec<Attachment>>,

    pub prefixed: bool,
}

impl From<Response> for InteractionResponseData {
    fn from(response: Response) -> Self {
        Self {
            content: response.content,
            embeds: response
                .embeds
                .map(|vec| vec.iter().cloned().map(|e| e.build()).collect()),
            flags: response.flags,
            components: response.components,
            attachments: response.attachments,
            ..Default::default()
        }
    }
}

impl Response {
    pub fn new_user_reply(user: &User, string: impl Into<String>) -> Response {
        Response::from_string(format!("**{}**, {}", user.mention(), string.into()))
    }

    pub fn from_string(string: impl Into<String>) -> Response {
        Response {
            content: Some(string.into()),
            ..Default::default()
        }
    }

    pub fn from_embeds(embeds: Vec<EmbedBuilder>) -> Response {
        Response {
            embeds: Some(embeds),
            ..Default::default()
        }
    }

    pub fn remove_all_components(self) -> Response {
        Response {
            components: Some(vec![]),
            ..self
        }
    }

    pub fn remove_all_attachments(self) -> Response {
        Response {
            attachments: Some(vec![]),
            ..self
        }
    }

    pub fn set_attachments(self, attachments: Vec<Attachment>) -> Response {
        Response {
            attachments: Some(attachments),
            ..self
        }
    }

    pub fn set_ephemeral(self) -> Response {
        Response {
            flags: Some(MessageFlags::EPHEMERAL),
            ..self
        }
    }

    pub fn set_components(self, components: Vec<impl Into<Component>>) -> Response {
        Response {
            components: Some(components.into_iter().map(|c| c.into()).collect::<Vec<_>>()),
            ..self
        }
    }

    pub fn add_embed(self, embed: EmbedBuilder) -> Response {
        let mut embeds = self.embeds.unwrap_or_default();
        embeds.push(embed);

        Response {
            embeds: Some(embeds),
            ..self
        }
    }

    pub fn add_string_content(self, content: impl Into<String>) -> Response {
        let content: String = content.into();
        Response {
            content: Some(
                self.content
                    .map_or(content.clone(), |c| format!("{}{}", c, content)),
            ),
            ..self
        }
    }

    pub fn add_emoji_prefix(self, emoji: impl Into<String>) -> Response {
        if self.content.is_none() {
            return Response {
                content: Some(emoji.into()),
                ..self
            };
        }

        Response {
            content: self.content.map(|c| {
                if self.prefixed {
                    c.replace('|', &format!("{} |", emoji.into()))
                } else {
                    format!("{} **|** {}", emoji.into(), c)
                }
            }),
            prefixed: true,
            ..self
        }
    }

    pub fn add_emoji_suffix(self, emoji: impl Into<String>) -> Response {
        if self.content.is_none() {
            return Response {
                content: Some(emoji.into()),
                ..self
            };
        }

        Response {
            content: self.content.map(|c| format!("{} {}", c, emoji.into())),
            ..self
        }
    }

    pub fn error_response(self) -> Response {
        self.add_emoji_prefix(":x:")
    }

    pub fn success_response(self) -> Response {
        self.add_emoji_prefix("✅")
    }

    pub fn to_json(self) -> Vec<u8> {
        let data = InteractionResponseData::from(self);

        serde_json::to_vec(&data).unwrap_or_default()
    }
}

impl From<EmbedBuilder> for Response {
    fn from(value: EmbedBuilder) -> Self {
        Self::from_embeds(vec![value])
    }
}

impl From<String> for Response {
    fn from(value: String) -> Self {
        Self::from_string(value)
    }
}

impl From<&str> for Response {
    fn from(value: &str) -> Self {
        Self::from_string(value.to_string())
    }
}
