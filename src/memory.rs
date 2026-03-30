use rig::OneOrMany;
use rig::completion::CompletionResponse;
use rig::completion::Message;
use rig::message::{ToolResult, ToolResultContent, UserContent};

pub struct ConversationMemory {
    messages: Vec<Message>,
    max_tokens: u64,
    /// Approximate token count, derived from the latest API response's total_tokens.
    current_tokens: u64,
}

#[allow(dead_code)]
impl ConversationMemory {
    pub fn new(max_tokens: u64) -> Self {
        Self {
            messages: Vec::new(),
            max_tokens,
            current_tokens: 0,
        }
    }

    pub fn push_user(&mut self, input: &str) {
        self.messages.push(Message::user(input));
    }

    pub fn push_assistant<T>(&mut self, response: &CompletionResponse<T>) {
        self.current_tokens = response.usage.total_tokens;
        self.messages.push(Message::Assistant {
            id: None,
            content: response.choice.clone(),
        });
    }

    pub fn push_system(&mut self, input: &str) {
        self.messages.push(Message::system(input));
    }

    pub fn push_tool_result(&mut self, tool_call_id: &str, result: String) {
        self.messages.push(Message::User {
            content: OneOrMany::one(UserContent::ToolResult(ToolResult {
                id: tool_call_id.to_string(),
                call_id: None,
                content: OneOrMany::one(ToolResultContent::text(result)),
            })),
        });
    }

    pub fn push(&mut self, message: Message) {
        self.messages.push(message);
    }
    pub fn messages(&self) -> &[Message] {
        &self.messages
    }

    pub fn max_tokens(&self) -> u64 {
        self.max_tokens
    }

    pub fn current_tokens(&self) -> u64 {
        self.current_tokens
    }

    pub fn clear(&mut self) {
        self.messages.clear();
        self.current_tokens = 0;
    }

    pub fn should_compact(&self) -> bool {
        self.current_tokens > self.max_tokens
    }
}
