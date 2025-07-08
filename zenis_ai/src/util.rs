use chrono::Utc;
use regex::Regex;
use zenis_database::instance_model::{InstanceBrain, InstanceMessage, InstanceModel};

use crate::{
    brain::Brain,
    claude_brain::ClaudeBrain,
    common::{ChatMessage, ChatResponse},
    gemini_brain::{GeminiBrain, GeminiModel},
    openai_brain::{OpenAIBrain, OpenAIModel},
};

pub fn remove_italic_actions(input: &str) -> String {
    let re = Regex::new(r"[_*][^_*]+[_*]").unwrap();
    let output = re.replace_all(input, "");
    output.trim().to_string()
}

pub async fn process_instance_message_queue(
    instance: &mut InstanceModel,
    messages: Vec<ChatMessage>,
    debug: bool,
) -> anyhow::Result<ChatResponse> {
    let brain = get_brain(instance.brain);
    let mut parameters = brain.default_parameters();
    parameters.debug = debug;
    parameters.system_prompt = instance.system_prompt.clone();

    let response = brain.prompt_raw(parameters, messages.clone()).await?;
    instance.push_message(InstanceMessage {
        image_url: None,
        is_assistant: true,
        user_id: instance.webhook_id,
        text: response.message.content.clone(),
    });

    instance.last_sent_message_timestamp = Utc::now().timestamp() + 3;

    Ok(response)
}

pub fn get_brain(brain: InstanceBrain) -> Box<dyn Brain + Send + Sync + 'static> {
    match brain {
        InstanceBrain::GeminiFlash => Box::new(GeminiBrain {
            model: GeminiModel::Flash25,
        }),
        InstanceBrain::GeminiPro => Box::new(GeminiBrain {
            model: GeminiModel::Pro25,
        }),
        InstanceBrain::ClaudeHaiku => Box::new(ClaudeBrain),
        InstanceBrain::ZenisFinetuned => Box::new(OpenAIBrain {
            model: OpenAIModel::Finetuned,
        }),
    }
}
