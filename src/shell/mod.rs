// src/shell/mod.rs
use std::collections::HashMap;
use std::env;
use crate::config::Config;

pub mod parser;
pub mod executor;
pub mod builtins;

// Only export what's actually used

pub struct Shell {
    pub environment: HashMap<String, String>,
    pub current_dir: String,
    pub last_exit_code: i32,
    pub history: Vec<String>,
    pub jobs: Vec<Job>,
    pub aliases: HashMap<String, String>,
    pub config: Config,
    pub username: String,
    pub hostname: String,
}

#[derive(Debug, Clone)]
pub struct Job {
    pub id: usize,
    pub pid: i32,
    pub command: String,
    pub status: JobStatus,
}

#[derive(Debug, Clone)]
pub enum JobStatus {
    Running,
    Stopped,
    Done,
}

impl Shell {
    pub fn new() -> Self {
        let mut env = HashMap::new();
        for (key, value) in env::vars() {
            env.insert(key, value);
        }
        
        let config = Config::load().unwrap_or_default();
        let aliases = config.aliases.clone();
        
        let username = env::var("USER").unwrap_or_else(|_| "user".to_string());
        let hostname = gethostname::gethostname()
            .to_string_lossy()
            .to_string();
        
        Self {
            environment: env,
            current_dir: env::current_dir()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            last_exit_code: 0,
            history: Vec::new(),
            jobs: Vec::new(),
            aliases,
            config,
            username,
            hostname,
        }
    }
    
    pub fn add_job(&mut self, pid: i32, command: String) {
        let id = self.jobs.len() + 1;
        self.jobs.push(Job {
            id,
            pid,
            command,
            status: JobStatus::Running,
        });
    }
    
    pub fn format_prompt(&self) -> String {
        let mut prompt = self.config.general.prompt_format.clone();
        
        prompt = prompt.replace("{user}", &self.username);
        prompt = prompt.replace("{host}", &self.hostname);
        prompt = prompt.replace("{cwd}", &self.get_cwd_display());
        
        let symbol = if self.last_exit_code == 0 {
            &self.config.theme.prompt_symbol
        } else {
            &self.config.theme.prompt_symbol_error
        };
        prompt = prompt.replace("{symbol}", symbol);
        
        prompt
    }
    
    fn get_cwd_display(&self) -> String {
        if let Ok(home) = env::var("HOME") {
            if self.current_dir.starts_with(&home) {
                return self.current_dir.replacen(&home, "~", 1);
            }
        }
        self.current_dir.clone()
    }
    
    pub fn save_config(&self) -> std::io::Result<()> {
        self.config.save()
    }
}