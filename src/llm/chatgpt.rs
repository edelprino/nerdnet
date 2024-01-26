use std::{collections::HashMap, env};

use super::{Message, ToolCall};
use crate::{Argument, ArgumentKind, Tool};
use serde::{Deserialize, Serialize};

pub struct ChatGpt {}

#[derive(Serialize, Debug)]
#[serde(tag = "role")]
enum ChatGptMessage {
    #[serde(rename = "system")]
    System { content: String },
    #[serde(rename = "user")]
    User { content: String },
    #[serde(rename = "assistant")]
    Assistant {
        content: Option<String>,
        tool_calls: Vec<ChatGptToolCall>,
    },
    #[serde(rename = "tool")]
    Tool {
        content: String,
        tool_call_id: String,
    },
}

impl ChatGptMessage {
    fn system(content: &str) -> Self {
        Self::System {
            content: content.to_string(),
        }
    }

    fn user(content: &str) -> Self {
        Self::User {
            content: content.to_string(),
        }
    }

    fn agent(content: Option<String>, tool_calls: Vec<ChatGptToolCall>) -> Self {
        Self::Assistant {
            content,
            tool_calls,
        }
    }

    fn tool(content: &str, tool_call_id: &str) -> Self {
        Self::Tool {
            content: content.to_string(),
            tool_call_id: tool_call_id.to_string(),
        }
    }
}

#[derive(Serialize, Debug)]
struct ChatGptTool {
    #[serde(rename = "type")]
    kind: String,
    function: ChatGptFunction,
}

#[derive(Serialize, Debug)]
struct ChatGptFunction {
    description: String,
    name: String,
    parameters: ChatGptObjectParameter,
}

#[derive(Serialize, Debug)]
#[serde(tag = "type")]
enum ChatGptFunctionArgument {
    #[serde(rename = "string")]
    String { description: String },
}

impl From<&Argument> for ChatGptFunctionArgument {
    fn from(argument: &Argument) -> Self {
        match &argument.kind {
            ArgumentKind::String() => Self::String {
                description: argument.description.clone(),
            },
        }
    }
}

#[derive(Serialize, Debug)]
struct ChatGptObjectParameter {
    #[serde(rename = "type")]
    kind: String,
    properties: HashMap<String, ChatGptFunctionArgument>,
}

impl ChatGptObjectParameter {
    fn new(arguments: HashMap<String, ChatGptFunctionArgument>) -> Self {
        Self {
            kind: "object".to_string(),
            properties: arguments,
        }
    }
}

impl From<&Tool> for ChatGptTool {
    fn from(tool: &Tool) -> Self {
        Self {
            kind: "function".to_string(),
            function: ChatGptFunction {
                name: tool.name.clone(),
                description: tool.description.clone(),
                parameters: ChatGptObjectParameter::new(
                    tool.arguments
                        .iter()
                        .map(|a| (a.name.clone(), a.into()))
                        .collect(),
                ),
            },
        }
    }
}

impl From<&Message> for ChatGptMessage {
    fn from(message: &Message) -> Self {
        match message {
            Message::System { message } => Self::system(message),
            Message::User { message } => Self::user(message),
            Message::Agent { message } => Self::agent(message.clone().into(), vec![]),
            Message::RunTools { calls } => {
                Self::agent(None, calls.iter().map(|c| c.into()).collect())
            }
            Message::ToolResult { result, call_id } => Self::tool(result, call_id),
        }
    }
}

#[derive(Debug, Deserialize)]
struct ChatGptResponse {
    id: String,
    object: String,
    created: i64,
    model: String,
    choices: Vec<ChatGptChoice>,
}

impl ChatGptResponse {
    fn get_first_choice(&self) -> Option<&ChatGptChoice> {
        if self.choices.len() == 0 {
            return None;
        }
        Some(&self.choices[0])
    }
}

#[derive(Debug, Deserialize)]
struct ChatGptChoice {
    index: i64,
    finish_reason: String,
    message: ChatGptMessageResponse,
}

impl ChatGptChoice {
    fn is_tool_calls(&self) -> bool {
        self.finish_reason == "tool_calls"
    }
}

#[derive(Debug, Deserialize)]
struct ChatGptMessageResponse {
    role: String,
    content: Option<String>,
    #[serde(default = "Vec::new")]
    tool_calls: Vec<ChatGptToolCall>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ChatGptToolCall {
    id: String,
    #[serde(rename = "type")]
    kind: String,
    function: ChatGptToolFunction,
}

impl From<&ToolCall> for ChatGptToolCall {
    fn from(call: &ToolCall) -> Self {
        Self {
            id: call.id.clone(),
            kind: "function".to_string(),
            function: ChatGptToolFunction {
                name: call.name.clone(),
                arguments: call.arguments.clone(),
            },
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct ChatGptToolFunction {
    name: String,
    arguments: String,
}

impl ChatGpt {
    pub fn run(version: &str, messages: &Vec<Message>, tools: &Vec<Tool>) -> Message {
        let messages: Vec<ChatGptMessage> = messages.iter().map(|m| m.into()).collect();
        let tools: Vec<ChatGptTool> = tools.iter().map(|t| t.into()).collect();
        let json = ureq::json!({
            "model": version,
            "temperature": 0.7,
            "messages": messages,
            "tools": tools,
            "tool_choice": "auto",
        });
        let apikey = env::var("OPENAI_API_KEY").expect("You've not set the OPENAI_API_KEY");
        let body: ChatGptResponse = ureq::post("https://api.openai.com/v1/chat/completions")
            .set("Content-Type", "application/json")
            .set("Authorization", &format!("Bearer {}", apikey).to_string())
            .send_json(json)
            .map_err(|e| {
                println!("{:?}", e.into_response().unwrap().into_string());
            })
            .unwrap()
            .into_json()
            .unwrap();
        let choice = body.get_first_choice().unwrap();
        if choice.is_tool_calls() {
            return Message::run_tools(
                choice
                    .message
                    .tool_calls
                    .iter()
                    .map(|c| ToolCall {
                        id: c.id.clone(),
                        name: c.function.name.clone(),
                        arguments: c.function.arguments.clone(),
                    })
                    .collect(),
            );
        }
        Message::agent(&choice.message.content.clone().unwrap_or("".to_string()))
    }
}
