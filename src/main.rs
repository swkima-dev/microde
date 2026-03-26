mod util;

use rig::OneOrMany;
use rig::client::CompletionClient;
use rig::completion::Message;
use rig::completion::Prompt;
use rig::message::{AssistantContent, UserContent};
use rig::providers::anthropic::{Client, completion::ANTHROPIC_VERSION_LATEST};
use util::count_tokens;

pub struct ConversationMemory {
    messages: Vec<Message>,
    max_token: usize,
    token_usage: usize,
}

impl ConversationMemory {
    pub fn new(max_token: usize) -> Self {
        Self {
            messages: Vec::new(),
            max_token,
            token_usage: 0,
        }
    }

    pub fn add_user_message(&mut self, input: &str, input_tokens: usize) {
        let message = Message::User {
            content: OneOrMany::one(UserContent::text(input)),
        };

        self.messages.push(message);

        self.token_usage += input_tokens;
    }

    pub fn add_assistant_message(&mut self, input: &str, input_tokens: usize) {
        let message = Message::Assistant {
            id: None,
            content: OneOrMany::one(AssistantContent::text(input)),
        };

        self.messages.push(message);
        self.token_usage += input_tokens;
    }

    pub fn get_messages(&self) -> &[Message] {
        &self.messages
    }

    pub fn clear(&mut self) {
        self.messages.clear();
        self.token_usage = 0;
    }
}

use rig::completion::Chat;

impl ConversationMemory {
    pub async fn compact<T>(&mut self, client: &T) -> Result<(), Box<dyn std::error::Error>>
    where
        T: Chat,
    {
        if self.token_usage <= self.max_token {
            return Ok(());
        }

        let response = client
            .chat(
                "Please provide a concise summary of this conversation, \
                 capturing key points, decisions, and context.",
                self.messages.clone(),
            )
            .await?;

        self.messages.clear();
        self.add_assistant_message(&response, count_tokens(&response).await.unwrap());

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenvy::from_filename(".env.local").ok();
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
