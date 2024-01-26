# NerdNet - Intelligence, Connected

NerdNet is an innovative platform designed to revolutionize the way Artificial Intelligence (AI) agents interact and collaborate. At its core, NerdNet is a sophisticated ecosystem where each AI agent possesses unique knowledge and a specific task, creating a dynamic network of specialized intelligence. These agents are not confined to solitary tasks; instead, they leverage an array of tools ranging from bash and MySQL for executing queries, to advanced search engines for gathering information. This flexibility allows NerdNet to adapt and respond to complex problems with precision and ingenuity.

The real power of NerdNet lies in its collaborative framework. Agents can enlist the help of others, forming ad-hoc teams to tackle challenges that are beyond the scope of a single entity. This synergy not only enhances problem-solving capabilities but also fosters an environment of continuous learning and improvement.

Whether it's automating intricate workflows, conducting in-depth analyses, or navigating the vast seas of data, NerdNet's agents work in concert to achieve goals that were once deemed impossible for individual AI systems. With a focus on innovation and adaptability, NerdNet is paving the way for a future where AI collaboration is not just a possibility, but a standard.

## Examples
### List all agents 
`Which tools do you know?`
```
❯ cargo run
   Compiling nerdnet v0.0.1 (/Users/edelprino/Projects/Personali/nerdnet)
    Finished dev [unoptimized + debuginfo] target(s) in 2.49s
     Running `target/debug/nerdnet`

Loading agent "./agents/scheduler.yml"
Loading agent "./agents/bash.yml"

Give me a command:
Which tools do you know?

Agent stuart: I'm doing `Which tools do you know?`
-------------------
I have access to two specific tools within the `functions` namespace that I can use to assist you:

1. **scheduler-agent**: This tool can schedule commands to be executed in the future. You would provide a prompt detailing what you want to schedule, and the tool will handle the timing.

2. **bash-expert**: This tool can execute any command in a bash-like environment. You provide a prompt explaining what command you need to be executed, and the tool will perform that action.

Additionally, I can use the `multi_tool_use.parallel` function from the `multi_tool_use` namespace to run multiple tools simultaneously, as long as they operate in the `functions` namespace and can work in parallel.

If you need assistance with something that might require these tools, please let me know, and I'll be happy to help!
```
### Execute a command on bash 
`Give me the list of files in my home directory`
```
❯ cargo run
   Compiling nerdnet v0.0.1 (/Users/edelprino/Projects/Personali/nerdnet)
    Finished dev [unoptimized + debuginfo] target(s) in 2.49s
     Running `target/debug/nerdnet`

Loading agent "./agents/scheduler.yml"
Loading agent "./agents/bash.yml"

Give me a command:
Give me the list of files in my home directory
Agent stuart: I'm doing `Give me the list of files in my home directory`
Agent bash-expert: I'm doing `List all the files in the home directory.`
Tool bash: I'm executing `ls ~`
-------------------
Here are the files and directories in your home directory:

- Applications
- Desktop
- Documents
- Downloads
- Google Drive
- Library
- Movies
- Music
- Projects
- Public
```
