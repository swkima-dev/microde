use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};
use std::cmp::min;
use std::fs;
use std::{
    fs::File,
    io::{BufRead, BufReader},
};

#[derive(Deserialize)]
pub enum ReadArgs {
    Directory {
        path: String,
    },
    File {
        path: String,
        offset: usize,
        limit: usize,
    },
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
                    "path": { "type": "object", "description": "File or directory path to search"},
                    "offset": { "type": "number", "description": "Offset when reading a file"},
                    "limit": { "type": "number", "description": "Number of lines to read from the file"}
                },
                "required": ["path"]
            }),
        }
    }

    async fn call(&self, args: ReadArgs) -> Result<Self::Output, Self::Error> {
        match args {
            ReadArgs::Directory { path } => {
                let mut joined_string = String::new();

                for entry_result in fs::read_dir(path)? {
                    let entry = entry_result?;
                    joined_string.push_str(format!("{:?}", entry.path()).as_str());
                }

                Ok(joined_string)
            }
            ReadArgs::File {
                path,
                offset,
                limit,
            } => {
                const DEFAULT_LIMIT: usize = 2000;
                let file = File::open(path)?;
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
}
