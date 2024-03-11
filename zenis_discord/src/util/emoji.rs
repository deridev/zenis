use twilight_model::{
    channel::message::ReactionType,
    id::{marker::EmojiMarker, Id},
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Emoji<'a> {
    Unicode(&'a str),
    Custom {
        animated: bool,
        id: Id<EmojiMarker>,
        name: Option<&'a str>,
    },
}

impl<'a> From<Emoji<'a>> for String {
    fn from(val: Emoji<'a>) -> Self {
        val.to_string()
    }
}

impl<'a> From<Emoji<'a>> for ReactionType {
    fn from(val: Emoji<'a>) -> Self {
        match val {
            Emoji::Unicode(emoji) => ReactionType::Unicode {
                name: emoji.to_string(),
            },
            Emoji::Custom { animated, id, name } => ReactionType::Custom {
                animated,
                id,
                name: name.map(ToString::to_string),
            },
        }
    }
}

impl<'a> std::fmt::Display for Emoji<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Emoji::Unicode(emoji) => write!(f, "{}", emoji),
            Emoji::Custom { animated, id, name } => {
                let name = (*name).unwrap_or_default();
                if *animated {
                    write!(f, "<a:{}:{id}>", name)
                } else {
                    write!(f, "<:{}:{id}>", name)
                }
            }
        }
    }
}

impl<'a> Emoji<'a> {
    pub fn new(emoji: &'a str) -> Self {
        if emoji.is_empty() {
            return Emoji::from_unicode("🟩");
        }

        let s = emoji.trim_matches(|c| c == '<' || c == '>');
        if !s.contains(':') {
            return Emoji::from_unicode(s);
        }

        if let Some(s) = s.strip_prefix("a:") {
            let parts: Vec<&str> = s[2..].split(':').collect();
            let id: u64 = parts[1].parse().unwrap_or(1);

            Emoji::from_animated_emote(Some(parts[0]), id)
        } else {
            let parts: Vec<&str> = s[1..].split(':').collect();
            let id: u64 = parts[1].parse().unwrap_or(1);

            Emoji::from_emote(Some(parts[0]), id)
        }
    }

    pub const fn from_unicode(unicode: &'a str) -> Self {
        Self::Unicode(unicode)
    }

    pub const fn from_emote(name: Option<&'a str>, id: u64) -> Self {
        Self::Custom {
            name,
            animated: false,
            id: Id::new(id),
        }
    }

    pub const fn from_animated_emote(name: Option<&'a str>, id: u64) -> Self {
        Self::Custom {
            name,
            animated: true,
            id: Id::new(id),
        }
    }
}
