#![allow(dead_code)]
use llm::{Message, ToolCall, LLM};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::io::prelude::*;
use std::process::Command;

mod llm;
mod serialization;

type Purpose = String;

#[derive(Debug, Serialize, Deserialize, Clone)]
enum ArgumentKind {
    String(),
}
#[derive(Debug, Serialize, Deserialize, Clone)]
struct Argument {
    name: String,
    description: String,
    kind: ArgumentKind,
}

impl Argument {
    fn string(name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            kind: ArgumentKind::String(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
enum Executable {
    Bash(),
    Agent(Agent),
    Python(),
}

impl Executable {
    fn run(&self, code: &str) -> String {
        match self {
            Executable::Bash() => {
                let output = Command::new("bash")
                    .arg("-c")
                    .arg(code)
                    .output()
                    .expect("failed to execute process");
                let stdout = String::from_utf8(output.stdout).unwrap();
                println!("output: {:?}", stdout);
                stdout
            }
            Executable::Python() => {
                let filename = format!("/tmp/script.py");
                fs::write(&filename, code).expect("Unable to write file");
                let output = Command::new("python3")
                    .arg(&filename)
                    .output()
                    .expect("failed to execute process");
                if !output.status.success() {
                    println!("error: {:?}", output.stderr);
                }
                fs::remove_file(&filename).expect("Unable to delete file");
                let stdout = String::from_utf8(output.stdout).unwrap();
                println!("output: {:?}", stdout);
                stdout
            }
            Executable::Agent(agent) => {
                let output = agent.start(code.to_string());
                output.text()
            }
        }
    }
}

#[derive(Debug)]
pub enum ArgumentValue {
    String(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Tool {
    name: String,
    description: Purpose,
    arguments: Vec<Argument>,
    executable: Executable,
    code: String,
}

impl Tool {
    fn new(
        name: &str,
        description: &str,
        executable: Executable,
        code: &str,
        arguments: Vec<Argument>,
    ) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            arguments,
            code: code.to_string(),
            executable,
        }
    }

    fn bash(name: &str, description: &str) -> Self {
        Self::new(
            name,
            description,
            Executable::Bash(),
            "{{command}}",
            vec![Argument::string("command", "Command to execute")],
        )
    }

    fn command(name: &str, description: &str, command: &str) -> Self {
        Self::new(name, description, Executable::Bash(), command, vec![])
    }

    fn execute(&self, call: ToolCall) -> Message {
        match &self.executable {
            Executable::Agent(_) => {}
            _ => {
                println!("Tool {}: I'm executing", self.name);
            }
        }
        let arguments: Value = serde_json::from_str(call.arguments()).unwrap();
        let mut command = self.code.clone();
        for argument in self.arguments.iter() {
            match &argument.kind {
                ArgumentKind::String() => {
                    let value = arguments[&argument.name].as_str().unwrap();
                    command = command.replace(&format!("{{{{{}}}}}", argument.name), value);
                }
            }
        }

        let output = self.executable.run(&command);
        Message::ToolResult {
            result: output,
            call_id: call.id().to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Agent {
    name: String,
    description: String,
    purpose: Purpose,
    llm: LLM,
    skills: Vec<Tool>,
}

impl Agent {
    fn new(name: &str, description: &str, purpose: &str, llm: &LLM, skills: Vec<Tool>) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            purpose: purpose.to_string(),
            skills,
            llm: llm.clone(),
        }
    }

    pub fn start(&self, request: String) -> Message {
        println!("Agent {}: I'm doing `{}`", self.name, request);
        self.run(&vec![
            Message::system(&self.purpose),
            Message::user(&request),
        ])
    }

    pub fn run(&self, messages: &Vec<Message>) -> Message {
        let responde = self.llm.run(&messages, &self.skills);
        let mut results = messages.to_vec();
        results.push(responde.clone());
        match responde {
            Message::RunTools { calls } => {
                for call in calls {
                    let tool = self
                        .skills
                        .iter()
                        .find(|t| call.name_is_equal(&t.name))
                        .unwrap();
                    results.push(tool.execute(call));
                }
                self.run(&results)
            }
            _ => responde,
        }
    }

    pub fn as_tool(self) -> Tool {
        Tool::new(
            &self.name.clone(),
            &self.description.clone(),
            Executable::Agent(self),
            "{{prompt}}",
            vec![Argument::string(
                "prompt",
                "Prompt to tell the tool what to do",
            )],
        )
    }
}

fn main() {
    let chatgpt4 = LLM::chatgpt("gpt-4-1106-preview");

    println!("Loading Tools");
    let mut tools: HashMap<String, Tool> = HashMap::new();
    let tool_files = fs::read_dir("./tools").unwrap();
    for tool_file in tool_files {
        let path = tool_file.unwrap().path();
        if path.extension().and_then(|s| s.to_str()) == Some("yml") {
            println!(" - {}", path.to_str().unwrap());
            let mut contents = String::new();
            fs::File::open(&path)
                .unwrap()
                .read_to_string(&mut contents)
                .unwrap();
            let tool = serialization::tools_from_yaml(&contents);
            for (name, tool) in tool {
                tools.insert(name, tool);
            }
        }
    }

    println!("Loading Agents");
    let mut agents = vec![];
    let paths = fs::read_dir("./agents").unwrap();
    for path in paths {
        let path = path.unwrap().path();
        if path.extension().and_then(|s| s.to_str()) == Some("yml") {
            println!(" - {}", path.to_str().unwrap());
            let mut contents = String::new();
            fs::File::open(&path)
                .unwrap()
                .read_to_string(&mut contents)
                .unwrap();
            let agent: Agent = serialization::agents_from_yaml(&contents, &tools);
            agents.push(agent);
        }
    }

    let stuart = Agent::new(
        "stuart",
        "You are my assistant named Stuart",
        "You are my assistant, you help me to do stuff. You have a series of tool that you can choose if you dont know how to do a thing.",
        &chatgpt4,
        agents.iter().map(|a| a.clone().as_tool()).collect(),
    );

    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        let res = stuart.start(args[1].clone());
        println!("{}", res.text());
        return;
    }

    loop {
        println!("Give me a command:");
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        input.pop();
        if input.is_empty() {
            continue;
        }
        let res = stuart.start(input);
        println!("-------------------");
        println!("{}", res.text());
    }
}
