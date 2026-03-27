use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};
use std::io;

#[derive(Deserialize)]
pub struct GrobArgs {
    pattern: String,
}

#[derive(Deserialize, Serialize)]
pub struct Grob;

impl Tool for Grob {
    const NAME: &'static str = "grob";

    type Error = std::io::Error;
    type Args = GrobArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "grob".to_string(),
            description: include_str!("grob.txt").to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "required": ["pattern"],
                "properties": {
                    "pattern": {
                        "type": "string",
                        "description": "Glob pattern to match files against (e.g. \"src/**/*.rs\")"
                    }
                }
            }),
        }
    }

    async fn call(&self, args: GrobArgs) -> Result<Self::Output, Self::Error> {
        let mut paths: Vec<String> = glob::glob(&args.pattern)
            .map_err(io::Error::other)?
            .filter_map(|r| r.ok())
            .map(|p| p.display().to_string())
            .collect();

        paths.sort();

        Ok(paths.join("\n"))
    }
}
