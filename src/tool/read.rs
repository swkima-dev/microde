use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};
use std::cmp::min;
use std::fs;
use std::{
    fs::File,
    io::{BufRead, BufReader},
};

#[derive(Deserialize)]
pub struct ReadArgs {
    path: String,
    offset: Option<usize>,
    limit: Option<usize>,
}

#[derive(Deserialize, Serialize)]
pub struct Read;

impl Tool for Read {
    const NAME: &'static str = "read";

    type Error = std::io::Error;
    type Args = ReadArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "read".to_string(),
            description: include_str!("read.txt").to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "File or directory path to search"},
                    "offset": { "type": "number", "description": "Offset when reading a file"},
                    "limit": { "type": "number", "description": "Number of lines to read from the file"}
                },
                "required": ["path"]
            }),
        }
    }

    async fn call(&self, args: ReadArgs) -> Result<Self::Output, Self::Error> {
        let metadata = fs::metadata(&args.path)?;

        if metadata.is_dir() {
            let mut joined_string = String::new();

            for entry_result in fs::read_dir(&args.path)? {
                let entry = entry_result?;
                joined_string.push_str(format!("{:?}", entry.path()).as_str());
            }

            Ok(joined_string)
        } else {
            const DEFAULT_LIMIT: usize = 2000;
            let offset = args.offset.unwrap_or(0);
            let limit = args.limit.unwrap_or(DEFAULT_LIMIT);

            let file = File::open(&args.path)?;
            let reader = BufReader::new(file);

            let lines = reader.lines().skip(offset).take(min(limit, DEFAULT_LIMIT));

            let mut joined_string = String::new();

            for line_result in lines {
                let line = line_result?;
                joined_string.push_str(&line);
                joined_string.push_str("\n");
            }

            Ok(joined_string)
        }
    }
}
