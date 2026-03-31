mod memory;
mod subagent;
mod system_context;
mod tool;
mod util;

use cliclack::{confirm, input, intro, log, outro, spinner};
use rig::client::CompletionClient;
use rig::completion::Completion;
use rig::message::AssistantContent;
use rig::providers::anthropic::{Client, completion::ANTHROPIC_VERSION_LATEST};
use rig::tool::ToolSet;
use system_context::SystemContexts;
use tool::{bash::Bash, grep::Grep, grob::Grob, read::Read, write::FullWrite};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenvy::from_filename(".env.local").ok();

    intro(include_str!("logo.txt"))?;

    let api_key = &std::env::var("ANTHROPIC_API_KEY")?;
    let client = Client::builder()
        .api_key(api_key)
        .anthropic_version(ANTHROPIC_VERSION_LATEST)
        .build()?;

    // Anthropic latest model has 1M token context window, so we set max_token, which
    // is threshold of compaction, 70% of it.
    let mut main_memory = memory::ConversationMemory::new(700_000);

    let max_tokens = main_memory.max_tokens();
    let mut current_tokens: u64;
    let mut main_tool = ToolSet::default();
    main_tool.add_tool(Bash);
    main_tool.add_tool(Read);
    main_tool.add_tool(Grep);
    main_tool.add_tool(Grob);
    main_tool.add_tool(FullWrite);

    let mut system_prompt = SystemContexts::new();

    loop {
        let user_input: String = input("User:")
            .placeholder("Type your message... (type 'exit' to quit)")
            .multiline()
            .interact()?;
        let user_input = user_input.trim().to_string();
        if user_input == "exit" {
            outro("Goodbye!")?;
            break;
        }
        if user_input.is_empty() {
            continue;
        }
        main_memory.push_user(user_input.as_str());

        let agent = client
            .agent("claude-sonnet-4-6")
            .preamble(
                &system_prompt
                    .update_working_dir()
                    .reload_instruction()
                    .prompt(),
            )
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

            let sp = spinner();
            sp.start("Thinking...");
            let response = agent
                .completion(prompt, history.to_vec())
                .await?
                .send()
                .await?;
            sp.stop("Done");

            let text = util::extract_text(&response.choice);
            if !text.is_empty() {
                log::info(format!("Assistant\n{}", text))?;
            }
            main_memory.push_assistant(&response);
            current_tokens = main_memory.current_tokens();
            log::remark(format!("Token Usage: {} / {}", current_tokens, max_tokens))?;

            let has_tool_calls = response
                .choice
                .iter()
                .any(|c| matches!(c, AssistantContent::ToolCall(_)));

            if !has_tool_calls {
                break;
            };

            for content in response.choice.iter() {
                if let AssistantContent::ToolCall(tool_call) = content {
                    let name = &tool_call.function.name;
                    let args = &tool_call.function.arguments;

                    let approved = confirm(format!("Allow tool call: {name}({args})"))
                        .initial_value(true)
                        .interact()?;

                    if approved {
                        let sp = spinner();
                        sp.start(format!("Running {name}..."));
                        let result = main_tool.call(name, args.to_string()).await?;
                        sp.stop(format!("{name} completed"));
                        main_memory.push_tool_result(&tool_call.id, result);
                    } else {
                        log::warning("Tool call denied")?;
                        main_memory.push_tool_result(
                            &tool_call.id,
                            format!(
                                "Tool use was denied by user. Denied tool call: {}, {}",
                                name, args,
                            ),
                        );
                    }
                }
            }
        }

        if main_memory.should_compact() {
            let sp = spinner();
            sp.start("Compacting conversation history...");
            match subagent::compaction::compaction(&client, main_memory.messages()).await {
                Ok(summary) => {
                    main_memory.clear();
                    main_memory.push_system(&summary);
                    sp.stop("Compaction completed");
                }
                Err(e) => {
                    sp.stop("Compaction failed");
                    log::warning(format!(
                        "Compaction failed, continuing with full history: {e}"
                    ))?;
                }
            }
        }
    }

    Ok(())
}
