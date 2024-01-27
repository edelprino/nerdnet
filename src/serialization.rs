use std::collections::HashMap;

use crate::llm::LLM;
use crate::Agent as DomainAgent;
use crate::Argument as DomainArgument;
use crate::Executable;
use crate::Tool as DomainTool;
use serde::{Deserialize, Serialize};

// pub fn tools_from_yaml(yaml: &str) -> Vec<DomainTool> {
//     let tools: Vec<Tool> = serde_yaml::from_str(yaml).unwrap();
//     tools.into_iter().map(|tool| tool.into()).collect()
// }
pub fn tools_from_yaml(yaml: &str) -> HashMap<String, DomainTool> {
    let tools: Vec<Tool> = serde_yaml::from_str(yaml).unwrap();
    tools
        .into_iter()
        .map(|tool| (tool.name.clone(), tool.into()))
        .collect()
}

pub fn agents_from_yaml(yaml: &str, tools: &HashMap<String, DomainTool>) -> DomainAgent {
    let agents: Agent = serde_yaml::from_str(yaml).unwrap();
    agents.into(tools)
}

#[derive(Debug)]
pub enum ArgumentValue {
    String(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Tool {
    name: String,
    description: String,
    #[serde(default)]
    arguments: Vec<Argument>,
    executable: String,
    code: String,
}

impl Into<DomainTool> for Tool {
    fn into(self) -> DomainTool {
        let executable = match self.executable.as_str() {
            "bash" => Executable::Bash(),
            "python" => Executable::Python(),
            _ => panic!("Unknown executable"),
        };

        DomainTool::new(
            &self.name,
            &self.description,
            executable,
            &self.code,
            self.arguments
                .into_iter()
                .map(|argument| argument.into())
                .collect(),
        )
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Argument {
    name: String,
    description: String,
    kind: String,
}

impl Into<DomainArgument> for Argument {
    fn into(self) -> DomainArgument {
        match self.kind.as_str() {
            "string" => DomainArgument::string(&self.name, &self.description),
            "env" => DomainArgument::string(&self.name, &self.description),
            _ => panic!("Unknown argument kind {}", self.kind),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Agent {
    name: String,
    description: String,
    purpose: String,
    llm: String,
    tools: Vec<String>,
}

impl Agent {
    fn into(self, toools: &HashMap<String, DomainTool>) -> DomainAgent {
        let llm = match self.llm.as_str() {
            "gpt-4-1106-preview" => LLM::ChatGpt("gpt-4-1106-preview".to_string()),
            _ => panic!("Unknown LLM {}", self.llm),
        };
        DomainAgent::new(
            &self.name,
            &self.description,
            &self.purpose,
            &llm,
            self.tools
                .into_iter()
                .map(|skill| toools.get(&skill).unwrap().clone())
                .collect(),
        )
    }
}
