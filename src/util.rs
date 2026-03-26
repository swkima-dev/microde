use reqwest::Client as ReqwestClient;
use serde_json::json;

pub async fn count_tokens(input: &str) -> Result<usize, Box<dyn std::error::Error>> {
    dotenvy::from_filename(".env.local").ok();
    let api_key = &std::env::var("ANTHROPIC_API_KEY")?;

    let client = ReqwestClient::new();
    let response = client
        .post("https://api.anthropic.com/v1/messages/count_tokens")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&json!({
            "model": "claude-sonnet-4-6",
            "messages": [{"role": "user", "content": input}]
        }))
        .send()
        .await?;

    let body: serde_json::Value = response.json().await?;
    Ok(body["input_tokens"]
        .as_u64()
        .ok_or("missing input_tokens")? as usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn claude_tokenizer_test() {
        let result = count_tokens("Hello, Claude").await.unwrap();

        assert_eq!(result, 10)
    }
}
