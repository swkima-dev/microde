use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};
use std::io;
use std::time::Duration;

const DEFAULT_TIMEOUT_MS: u64 = 120_000;
const MAX_TIMEOUT_MS: u64 = 600_000;

#[derive(Deserialize)]
pub struct BashArgs {
    command: String,
    timeout: Option<u64>,
}

#[derive(Deserialize, Serialize)]
pub struct Bash;

impl Tool for Bash {
    const NAME: &'static str = "bash";

    type Error = io::Error;
    type Args = BashArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "bash".to_string(),
            description: include_str!("bash.txt").to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "required": ["command"],
                "properties": {
                    "command": {
                        "type": "string",
                        "description": "The bash command to execute"
                    },
                    "timeout": {
                        "type": "number",
                        "description": "Optional timeout in milliseconds (default: 120000, max: 600000)"
                    }
                }
            }),
        }
    }

    async fn call(&self, args: BashArgs) -> Result<Self::Output, Self::Error> {
        let timeout_ms = args
            .timeout
            .unwrap_or(DEFAULT_TIMEOUT_MS)
            .min(MAX_TIMEOUT_MS);

        let output = tokio::process::Command::new("bash")
            .arg("-c")
            .arg(&args.command)
            .output();

        let result = tokio::time::timeout(Duration::from_millis(timeout_ms), output).await;

        match result {
            Ok(Ok(output)) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                let exit_code = output.status.code().unwrap_or(-1);

                let mut result = String::new();

                if !stdout.is_empty() {
                    result.push_str(&stdout);
                }
                if !stderr.is_empty() {
                    if !result.is_empty() {
                        result.push('\n');
                    }
                    result.push_str("[stderr]\n");
                    result.push_str(&stderr);
                }

                if exit_code != 0 {
                    result.push_str(&format!("\n[exit code: {}]", exit_code));
                }

                if result.is_empty() {
                    result.push_str("(no output)");
                }

                Ok(result)
            }
            Ok(Err(e)) => Err(e),
            Err(_) => Err(io::Error::new(
                io::ErrorKind::TimedOut,
                format!(
                    "Command timed out after {}ms: {}",
                    timeout_ms, args.command
                ),
            )),
        }
    }
}
