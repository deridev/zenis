use zenis_discord::twilight_model::{
    application::interaction::application_command::CommandOptionValue, user::User,
};

use crate::CommandContext;

pub struct OptionHandler<'a> {
    pub ctx: &'a CommandContext,
}

impl<'a> OptionHandler<'a> {
    pub fn get_option_value(
        &self,
        option_name: impl Into<String>,
    ) -> anyhow::Result<Option<CommandOptionValue>> {
        let option_name: String = option_name.into();
        let Some(option) = self.ctx.options.iter().find(|o| o.name == option_name) else {
            return Ok(None);
        };

        Ok(Some(option.value.clone()))
    }

    pub fn get_string(&self, option_name: impl Into<String>) -> anyhow::Result<Option<String>> {
        let Some(value) = self.get_option_value(option_name)? else {
            return Ok(None);
        };

        Ok(match value {
            CommandOptionValue::String(value) => Some(value),
            _ => None,
        })
    }

    pub fn get_integer(&self, option_name: impl Into<String>) -> anyhow::Result<Option<i64>> {
        let Some(value) = self.get_option_value(option_name)? else {
            return Ok(None);
        };

        Ok(match value {
            CommandOptionValue::Integer(value) => Some(value),
            _ => None,
        })
    }

    pub fn get_number(&self, option_name: impl Into<String>) -> anyhow::Result<Option<f64>> {
        let Some(value) = self.get_option_value(option_name)? else {
            return Ok(None);
        };

        Ok(match value {
            CommandOptionValue::Number(value) => Some(value),
            _ => None,
        })
    }

    pub async fn get_user(&self, option_name: impl Into<String>) -> anyhow::Result<Option<User>> {
        let Some(value) = self.get_option_value(option_name)? else {
            return Ok(None);
        };

        Ok(match value {
            CommandOptionValue::User(user_id) => {
                let user = self.ctx.client.get_user(user_id).await?;

                Some(user)
            }
            _ => None,
        })
    }

    pub fn get_boolean(&self, option_name: impl Into<String>) -> anyhow::Result<Option<bool>> {
        let Some(value) = self.get_option_value(option_name)? else {
            return Ok(None);
        };

        Ok(match value {
            CommandOptionValue::Boolean(value) => Some(value),
            _ => None,
        })
    }
}
