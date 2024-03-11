use twilight_model::{
    channel::message::{
        embed::{
            EmbedAuthor as APIEmbedAuthor, EmbedField as APIEmbedField,
            EmbedFooter as APIEmbedFooter, EmbedImage as APIEmbedImage,
            EmbedThumbnail as APIEmbedThumbnail,
        },
        Embed as APIEmbed,
    },
    user::User,
    util::Timestamp,
};

use crate::UserExtension;

use zenis_common::Color;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmbedField {
    pub name: String,
    pub value: String,
    pub inline: bool,
}

impl From<(&str, &str, bool)> for EmbedField {
    fn from(value: (&str, &str, bool)) -> Self {
        Self {
            name: value.0.to_string(),
            value: value.1.to_string(),
            inline: value.2,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmbedAuthor {
    pub name: String,
    pub icon_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmbedFooter {
    pub text: String,
    pub icon_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Embed {
    author: Option<EmbedAuthor>,
    color: Option<Color>,
    title: Option<String>,
    image: Option<String>,
    thumbnail: Option<String>,
    description: Option<String>,
    fields: Vec<EmbedField>,
    timestamp: Option<Timestamp>,
    footer: Option<EmbedFooter>,
}

impl Embed {
    fn build(self) -> APIEmbed {
        APIEmbed {
            author: self.author.map(|a| APIEmbedAuthor {
                name: a.name,
                icon_url: a.icon_url,
                proxy_icon_url: None,
                url: None,
            }),
            color: self.color.map(|c| c.to_u32()),
            title: self.title,
            description: self.description,
            fields: self
                .fields
                .iter()
                .map(|f| APIEmbedField {
                    inline: f.inline,
                    name: f.name.clone(),
                    value: f.value.clone(),
                })
                .collect(),
            footer: self.footer.map(|f| APIEmbedFooter {
                text: f.text,
                icon_url: f.icon_url,
                proxy_icon_url: None,
            }),
            image: self.image.map(|img| APIEmbedImage {
                url: img,
                height: None,
                width: None,
                proxy_url: None,
            }),
            kind: "Embed".to_string(),
            provider: None,
            thumbnail: self.thumbnail.map(|t| APIEmbedThumbnail {
                url: t,
                width: None,
                height: None,
                proxy_url: None,
            }),
            timestamp: self.timestamp,
            url: None,
            video: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EmbedBuilder {
    pub embed: Embed,
}

impl EmbedBuilder {
    pub fn new() -> EmbedBuilder {
        Self {
            embed: Embed::default(),
        }
    }

    pub fn new_common() -> EmbedBuilder {
        Self::new().set_color(Color::BLUE).set_current_timestamp()
    }

    pub fn set_author(mut self, author: EmbedAuthor) -> Self {
        self.embed.author = Some(author);
        self
    }

    pub fn set_author_to_user(self, user: &User) -> Self {
        self.set_author(EmbedAuthor {
            name: format!("{}#{}", user.name, user.discriminator),
            icon_url: Some(user.avatar_url()),
        })
    }

    pub fn set_color(mut self, color: Color) -> Self {
        self.embed.color = Some(color);
        self
    }

    pub fn set_title(mut self, title: impl ToString) -> Self {
        self.embed.title = Some(title.to_string());
        self
    }

    pub fn set_image(mut self, image: impl ToString) -> Self {
        self.embed.image = Some(image.to_string());
        self
    }

    pub fn set_thumbnail(mut self, thumbnail: impl ToString) -> Self {
        self.embed.thumbnail = Some(thumbnail.to_string());
        self
    }

    pub fn set_description(mut self, description: impl ToString) -> Self {
        self.embed.description = Some(description.to_string());
        self
    }

    pub fn set_timestamp(mut self, timestamp: Timestamp) -> Self {
        self.embed.timestamp = Some(timestamp);
        self
    }

    pub fn set_current_timestamp(self) -> Self {
        let timestamp = Timestamp::parse(chrono::Utc::now().to_rfc3339().as_str());

        if let Ok(timestamp) = timestamp {
            return self.set_timestamp(timestamp);
        }

        self
    }

    pub fn set_footer(mut self, footer: EmbedFooter) -> Self {
        self.embed.footer = Some(footer);
        self
    }

    pub fn add_description_text(mut self, description: impl ToString) -> Self {
        if let Some(embed_description) = &mut self.embed.description {
            embed_description.push_str(&description.to_string());
        } else {
            self = self.set_description(description);
        }

        self
    }

    pub fn add_footer_text(mut self, text: impl ToString) -> Self {
        if let Some(footer) = &self.embed.footer {
            let new_footer = EmbedFooter {
                text: format!("{} | {}", footer.text, text.to_string()),
                ..footer.clone()
            };
            self.embed.footer = Some(new_footer);
        } else {
            return self.set_footer(EmbedFooter {
                text: text.to_string(),
                icon_url: None,
            });
        }

        self
    }

    pub fn add_not_inlined_field(mut self, name: impl ToString, value: impl ToString) -> Self {
        self.embed.fields.push(EmbedField {
            name: name.to_string(),
            value: value.to_string(),
            inline: false,
        });
        self
    }

    pub fn add_inlined_field(mut self, name: impl ToString, value: impl ToString) -> Self {
        self.embed.fields.push(EmbedField {
            name: name.to_string(),
            value: value.to_string(),
            inline: true,
        });
        self
    }

    pub fn add_field(mut self, field: EmbedField) -> Self {
        self.embed.fields.push(field);
        self
    }

    pub fn add_field_with_emoji(self, emoji: impl Into<String>, mut field: EmbedField) -> Self {
        field.name = format!("{} {}", emoji.into(), field.name);
        self.add_field(field)
    }

    pub fn add_fields(mut self, fields: &mut Vec<EmbedField>) -> Self {
        self.embed.fields.append(fields);
        self
    }

    pub fn build(self) -> APIEmbed {
        self.embed.build()
    }
}

impl From<EmbedBuilder> for APIEmbed {
    fn from(value: EmbedBuilder) -> Self {
        value.build()
    }
}
