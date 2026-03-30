use std::{
    env, fs,
    path::{Path, PathBuf},
};

pub struct SystemContexts {
    system_prompt: String,
    workspace_root_dir: PathBuf,
    working_dir: PathBuf,
    instruction: Option<PathBuf>,
}

impl SystemContexts {
    pub fn new() -> SystemContexts {
        let system_prompt = include_str!("system.txt").to_string();
        let workspace_root_dir =
            env::current_dir().expect("current directory should always be accessible at startup");
        let working_dir = workspace_root_dir.clone();

        let instruction = Self::find_instruction_file(&workspace_root_dir);
        SystemContexts {
            system_prompt,
            workspace_root_dir,
            working_dir,
            instruction,
        }
    }

    pub fn prompt(&self) -> String {
        let instruction = self
            .instruction
            .as_ref()
            .and_then(|path| fs::read_to_string(path).ok())
            .unwrap_or_default();

        format!(
            "{}\n<env>\nWorking directory: {:?}\nWorkspace root folder: {:?}\n</env>\n{}",
            self.system_prompt, self.working_dir, self.workspace_root_dir, instruction,
        )
    }

    pub fn update_working_dir(&mut self) -> &mut Self {
        self.working_dir =
            env::current_dir().expect("current directory should always be accessible");
        self
    }

    pub fn reload_instruction(&mut self) -> &mut Self {
        self.instruction = Self::find_instruction_file(&self.workspace_root_dir);
        self
    }

    fn find_instruction_file(root: &Path) -> Option<PathBuf> {
        let candidates = ["AGENTS.md", "MICRODE.md", "CONTEXT.md"];

        candidates
            .iter()
            .map(|name| root.join(name))
            .find(|path| path.exists())
    }
}
