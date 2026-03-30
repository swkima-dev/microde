use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct FullWriteArgs {
    path: String,
    content: String,
}

#[derive(Deserialize, Serialize)]
pub struct FullWrite;

impl Tool for FullWrite {
    const NAME: &'static str = "write";

    type Error = std::io::Error;
    type Args = FullWriteArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "write".to_string(),
            description: include_str!("write.txt").to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "File path to write"},
                    "content": { "type": "string", "description": "Contents of the file to be written"}
                },
                "required": ["path", "content"]
            }),
        }
    }

    async fn call(&self, args: FullWriteArgs) -> Result<Self::Output, Self::Error> {
        let path = Path::new(&args.path);

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?
        }

        let mut file = File::create(&path)?;

        file.write_all(args.content.as_bytes())?;

        Ok(format!(
            "Successfully wrote {} bytes to {}",
            args.content.len(),
            args.path
        ))
    }
}
