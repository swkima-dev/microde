use regex::Regex;
use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

#[derive(Deserialize)]
pub struct GrepArgs {
    pattern: String,
    path: String,
}

#[derive(Deserialize, Serialize)]
pub struct Grep;

impl Tool for Grep {
    const NAME: &'static str = "grep";

    type Error = io::Error;
    type Args = GrepArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "grep".to_string(),
            description: include_str!("grep.txt").to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "required": ["pattern", "path"],
                "properties": {
                    "pattern": {
                        "type": "string",
                        "description": "Regex pattern to search for"
                    },
                    "path": {
                        "type": "string",
                        "description": "Absolute path to a file or directory to search in"
                    }
                }
            }),
        }
    }

    async fn call(&self, args: GrepArgs) -> Result<Self::Output, Self::Error> {
        let regex = Regex::new(&args.pattern).map_err(io::Error::other)?;
        let path = Path::new(&args.path);
        let mut results = Vec::new();

        if path.is_file() {
            search_file(&regex, path, &mut results)?;
        } else if path.is_dir() {
            search_dir(&regex, path, &mut results)?;
        }

        Ok(results.join("\n"))
    }
}

fn search_file(regex: &Regex, path: &Path, results: &mut Vec<String>) -> io::Result<()> {
    let file = fs::File::open(path)?;
    let reader = BufReader::new(file);

    for (line_num, line_result) in reader.lines().enumerate() {
        let line = line_result?;
        if regex.is_match(&line) {
            results.push(format!("{}:{}: {}", path.display(), line_num + 1, line));
        }
    }

    Ok(())
}

fn search_dir(regex: &Regex, dir: &Path, results: &mut Vec<String>) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let _ = search_file(regex, &path, results);
        } else if path.is_dir() {
            search_dir(regex, &path, results)?;
        }
    }

    Ok(())
}
