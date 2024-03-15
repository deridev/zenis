use chrono::Utc;
use regex::Regex;
use zenis_database::instance_model::{InstanceBrain, InstanceModel};

use crate::{
    brain::Brain,
    claude_brain::ClaudeBrain,
    cohere_brain::CohereBrain,
    common::{ChatMessage, ChatResponse},
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
    parameters.system_prompt = instance.agent_description.clone();

    let response = brain.prompt_chat(parameters, messages.clone()).await?;
    instance.push_message(response.clone());

    instance.last_sent_message_timestamp = Utc::now().timestamp() + 3;

    Ok(response)
}

pub fn get_brain(brain: InstanceBrain) -> Box<dyn Brain + Send + Sync + 'static> {
    match brain {
        InstanceBrain::CohereCommandR => Box::new(CohereBrain),
        InstanceBrain::ClaudeHaiku => Box::new(ClaudeBrain),
    }
}
