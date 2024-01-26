pub mod chatgpt;
use chatgpt::ChatGpt;
use serde::{Deserialize, Serialize};

use crate::Tool;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LLM {
    ChatGpt(String),
}

impl LLM {
    pub fn chatgpt(version: &str) -> Self {
        Self::ChatGpt(version.to_string())
    }
}

#[derive(Debug, Clone)]
pub struct ToolCall {
    id: String,
    name: String,
    arguments: String,
}

impl ToolCall {
    pub fn name_is_equal(&self, name: &str) -> bool {
        self.name == name
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn arguments(&self) -> &str {
        &self.arguments
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    System { message: String },
    User { message: String },
    Agent { message: String },
    RunTools { calls: Vec<ToolCall> },
    ToolResult { result: String, call_id: String },
}

impl Message {
    pub fn is_last(&self) -> bool {
        match self {
            Message::RunTools { .. } => false,
            _ => true,
        }
    }

    pub fn system(message: &str) -> Self {
        Self::System {
            message: message.to_string(),
        }
    }

    pub fn user(message: &str) -> Self {
        Self::User {
            message: message.to_string(),
        }
    }

    pub fn run_tools(calls: Vec<ToolCall>) -> Self {
        Self::RunTools { calls }
    }

    pub fn agent(message: &str) -> Self {
        Self::Agent {
            message: message.to_string(),
        }
    }

    pub fn text(&self) -> String {
        match self {
            Message::System { message } => message.clone(),
            Message::User { message } => message.clone(),
            Message::Agent { message } => message.clone(),
            Message::RunTools { .. } => "".to_string(),
            Message::ToolResult { result, .. } => result.clone(),
        }
    }
}

impl LLM {
    pub fn run(&self, messages: &Vec<Message>, tools: &Vec<Tool>) -> Message {
        match self {
            LLM::ChatGpt(version) => ChatGpt::run(version, messages, tools),
        }
    }
}
