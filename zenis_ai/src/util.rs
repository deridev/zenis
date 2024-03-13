use chrono::Utc;
use regex::Regex;
use zenis_database::instance_model::InstanceModel;

use crate::{
    common::{ChatMessage, ChatResponse},
    BrainType,
};

pub fn remove_italic_actions(input: &str) -> String {
    let re = Regex::new(r"[_*][^_*]+[_*]").unwrap();
    let output = re.replace_all(input, "");
    output.trim().to_string()
}

pub async fn process_instance_message_queue(
    instance: &mut InstanceModel,
    brain_type: BrainType,
    messages: Vec<ChatMessage>,
) -> anyhow::Result<ChatResponse> {
    let brain = brain_type.get();
    let mut parameters = brain.default_parameters();
    parameters.system_prompt = instance.agent_description.clone();

    let response = brain.prompt_chat(parameters, messages.clone()).await?;
    instance.push_message(response.clone());

    instance.last_sent_message_timestamp = Utc::now().timestamp() + 3;

    Ok(response)
}
