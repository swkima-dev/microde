use rig::client::Client;
use rig::client::CompletionClient;
use rig::completion::Completion;
use rig::completion::Message;
use rig::providers::anthropic::client::AnthropicExt;

use crate::util;

pub async fn compaction(
    client: &Client<AnthropicExt>,
    messages: &[Message],
) -> anyhow::Result<String> {
    let system_prompt = include_str!("compaction.txt").to_string();

    let agent = client
        .agent("claude-sonnet-4-6")
        .preamble(&system_prompt)
        .build();
    let response = agent
        .completion("Please summarize the message history.", messages.to_vec())
        .await?
        .send()
        .await?;

    Ok(util::extract_text(&response.choice))
}
