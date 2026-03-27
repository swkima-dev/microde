use rig::OneOrMany;
use rig::message::AssistantContent;

/// Extract concatenated text from assistant content blocks.
pub fn extract_text(choice: &OneOrMany<AssistantContent>) -> String {
    choice
        .iter()
        .filter_map(|c| match c {
            AssistantContent::Text(t) => Some(t.text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("")
}
