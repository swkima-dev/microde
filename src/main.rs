use rig::client::CompletionClient;
use rig::completion::Prompt;
use rig::providers::anthropic::{Client, completion::ANTHROPIC_VERSION_LATEST};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let api_key = &std::env::var("ANTHROPIC_API_KEY")?;
    let client = Client::builder()
        .api_key(api_key)
        .anthropic_version(ANTHROPIC_VERSION_LATEST)
        .build()?;

    let agent = client
        .agent("claude-sonnet-4-6")
        .preamble("You are an helpful agent.")
        .build();

    let response = agent.prompt("Hello!, who are you?").await?;

    println!("{response}");

    Ok(())
}
