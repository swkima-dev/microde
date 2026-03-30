mod memory;
mod tool;
mod util;

use std::env;
use std::io::{self, Write};

use rig::client::CompletionClient;
use rig::completion::Completion;
use rig::message::AssistantContent;
use rig::providers::anthropic::{Client, completion::ANTHROPIC_VERSION_LATEST};
use rig::tool::ToolSet;
use tool::{bash::Bash, grep::Grep, grob::Grob, read::Read, write::FullWrite};

const SYSTEM_PROMPT: &str = "You are a helpful chatbot.";

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenvy::from_filename(".env.local").ok();
    let api_key = &std::env::var("ANTHROPIC_API_KEY")?;
    let client = Client::builder()
        .api_key(api_key)
        .anthropic_version(ANTHROPIC_VERSION_LATEST)
        .build()?;

    // Anthropic latest model has 1M token context window, so we set max_token, which
    // is threshold of compaction, 70% of it.
    let mut main_memory = memory::ConversationMemory::new(700_000);

    let mut user_buf = String::new();
    let max_tokens = main_memory.max_tokens();
    let mut current_tokens: u64;
    let mut main_tool = ToolSet::default();
    main_tool.add_tool(Bash);
    main_tool.add_tool(Read);
    main_tool.add_tool(Grep);
    main_tool.add_tool(Grob);
    main_tool.add_tool(FullWrite);

    let workspace_root_folder = env::current_dir()?;

    loop {
        print!("\nYou: ");
        io::stdout().flush()?;

        user_buf.clear();
        io::stdin()
            .read_line(&mut user_buf)
            .expect("Failed to read line");

        let input = user_buf.trim().to_string(); // inputの借用がきれいでない。後でリファクタリング
        if input == "exit" {
            break;
        }
        if input.is_empty() {
            continue;
        }
        main_memory.push_user(input.as_str());

        let working_directory = env::current_dir()?;
        let environment_prompt = format!(
            "<env>
                Working directory: {working_directory:?}
                Workspace root folder: {workspace_root_folder:?}
            </env>"
        );

        let agent = client
            .agent("claude-sonnet-4-6")
            .preamble(format!("{}\n{}", SYSTEM_PROMPT, environment_prompt).as_str())
            .tool(Bash)
            .tool(Read)
            .tool(Grep)
            .tool(Grob)
            .tool(FullWrite)
            .build();

        loop {
            let messages = main_memory.messages();
            let (prompt, history) = messages.split_last().expect("messages should not be empty");
            let prompt = prompt.clone();

            let response = agent
                .completion(prompt, history.to_vec())
                .await?
                .send()
                .await?;

            println!("\nAssistant: {}", util::extract_text(&response.choice));
            main_memory.push_assistant(&response);
            current_tokens = main_memory.current_tokens();
            println!("Token Usage: {} / {}", current_tokens, max_tokens);

            let has_tool_calls = response
                .choice
                .iter()
                .any(|c| matches!(c, AssistantContent::ToolCall(_)));

            if !has_tool_calls {
                break;
            };

            for content in response.choice.iter() {
                match content {
                    AssistantContent::ToolCall(tool_call) => {
                        let name = &tool_call.function.name;
                        let args = &tool_call.function.arguments;
                        print!("APPROVE?: Agent ask you to use {}({}). y/n ", name, args);
                        io::stdout().flush()?;

                        user_buf.clear();
                        io::stdin()
                            .read_line(&mut user_buf)
                            .expect("Failed to read line");

                        let input = user_buf.trim();
                        if input == "y" {
                            let result = main_tool.call(&name, args.to_string()).await?;
                            main_memory.push_tool_result(&tool_call.id, result);
                        } else {
                            main_memory.push_tool_result(
                                &tool_call.id,
                                format!(
                                    "Tool use was denied by user. Denied tool call: {}, {}",
                                    name,
                                    args.to_string()
                                ),
                            );
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}
