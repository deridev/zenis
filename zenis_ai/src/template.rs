use std::collections::HashMap;

fn parse_string_to_hashmap(input: &str) -> HashMap<String, String> {
    let mut result = HashMap::new();
    let mut key = String::new();
    let mut value = String::new();
    let mut is_key = false;

    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '<' if chars.peek() == Some(&'!') => {
                chars.next(); // consume '!'
                if !key.is_empty() {
                    result.insert(key.clone(), value.clone());
                    key.clear();
                    value.clear();
                }
                is_key = true;
            }
            '/' if is_key && chars.peek() == Some(&'>') => {
                chars.next(); // consume '>'
                is_key = false;
            }
            '\n' => {
                if !key.is_empty() {
                    result.insert(key.clone(), value.clone());
                    key.clear();
                    value.clear();
                }
            }
            _ if is_key => key.push(c),
            _ => value.push(c),
        }
    }

    if !key.is_empty() {
        result.insert(key, value);
    }

    result
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AssistantObject {
    pub thinking: String,
    pub message: Option<String>,
    pub quote: Option<u64>,
    pub is_noreply: bool,
    pub exit_reason: Option<String>,
}

pub fn to_assistant_object(input: &str) -> AssistantObject {
    let hashmap = parse_string_to_hashmap(input);
    AssistantObject {
        thinking: hashmap
            .get("thinking")
            .unwrap_or(&String::new())
            .to_string(),
        message: hashmap.get("message").map(|m| m.to_string()),
        quote: hashmap.get("quote").map(|q| q.parse::<u64>().unwrap_or(0)),
        is_noreply: hashmap.get("noreply").is_some(),
        exit_reason: hashmap.get("exit").map(|e| e.to_string()),
    }
}
