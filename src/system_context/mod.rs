use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

pub struct SystemContexts {
    system_prompt: String,
    workspace_root_dir: Option<PathBuf>,
    working_dir: PathBuf,
    instruction: Option<PathBuf>,
}

impl SystemContexts {
    pub fn new() -> SystemContexts {
        let system_prompt = include_str!("system.txt").to_string();
        let workspace_root_dir = Self::detect_git_root();
        let working_dir =
            env::current_dir().expect("current directory should always be accessible at startup");

        let search_root = workspace_root_dir.as_deref().unwrap_or(&working_dir);
        let instruction = Self::find_instruction_file(search_root);
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

        let workspace_root_display = match &self.workspace_root_dir {
            Some(path) => format!("{:?}", path),
            None => "Not a git-managed project".to_string(),
        };

        format!(
            "{}\n<env>\nWorking directory: {:?}\nWorkspace root folder: {}\n</env>\n{}",
            self.system_prompt, self.working_dir, workspace_root_display, instruction,
        )
    }

    pub fn update_working_dir(&mut self) -> &mut Self {
        self.working_dir =
            env::current_dir().expect("current directory should always be accessible");
        self
    }

    pub fn reload_instruction(&mut self) -> &mut Self {
        let search_root = self
            .workspace_root_dir
            .as_deref()
            .unwrap_or(&self.working_dir);
        self.instruction = Self::find_instruction_file(search_root);
        self
    }

    fn detect_git_root() -> Option<PathBuf> {
        let output = Command::new("git")
            .args(["rev-parse", "--show-toplevel"])
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let root = PathBuf::from(String::from_utf8_lossy(&output.stdout).trim());
        Some(root)
    }

    fn find_instruction_file(root: &Path) -> Option<PathBuf> {
        let candidates = ["AGENTS.md", "MICRODE.md", "CONTEXT.md"];

        candidates
            .iter()
            .map(|name| root.join(name))
            .find(|path| path.exists())
    }
}
