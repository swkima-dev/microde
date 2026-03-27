mod memory;
mod util;

use std::io::{self, Write};

use rig::client::CompletionClient;
use rig::completion::Completion;
use rig::providers::anthropic::{Client, completion::ANTHROPIC_VERSION_LATEST};

const SYSTEM_PROMPT: &str = "You are a helpful chatbot for casual conversation.";

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
    loop {
        print!("\nYou: ");
        io::stdout().flush()?;

        user_buf.clear();
        io::stdin()
            .read_line(&mut user_buf)
            .expect("Failed to read line");

        let input = user_buf.trim();
        if input == "exit" {
            break;
        }
        if input.is_empty() {
            continue;
        }

        main_memory.push_user(input);

        let agent = client
            .agent("claude-sonnet-4-6")
            .preamble(SYSTEM_PROMPT)
            .build();

        let response = agent
            .completion(input.to_string(), main_memory.messages().to_vec())
            .await?
            .send()
            .await?;

        println!("\nAssistant: {}", util::extract_text(&response.choice));
        main_memory.push_assistant(&response);
        current_tokens = main_memory.current_tokens();
        println!("Token Usage: {} / {}", current_tokens, max_tokens);
    }

    Ok(())
}
