// src/history.rs
use std::collections::VecDeque;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

pub struct History {
    commands: VecDeque<String>,
    max_size: usize,
    file_path: Option<PathBuf>,
}

impl History {
    pub fn new(max_size: usize) -> Self {
        Self {
            commands: VecDeque::new(),
            max_size,
            file_path: None,
        }
    }
    
    pub fn with_file(max_size: usize, file_path: PathBuf) -> Self {
        let mut history = Self {
            commands: VecDeque::new(),
            max_size,
            file_path: Some(file_path),
        };
        history.load_from_file();
        history
    }
    
    pub fn add(&mut self, command: String) {
        if command.trim().is_empty() {
            return;
        }
        
        // Don't add duplicate consecutive commands
        if let Some(last) = self.commands.back() {
            if last == &command {
                return;
            }
        }
        
        self.commands.push_back(command);
        
        // Maintain max size
        while self.commands.len() > self.max_size {
            self.commands.pop_front();
        }
        
        self.save_to_file();
    }
    
    pub fn get(&self, index: usize) -> Option<&String> {
        self.commands.get(index)
    }
    
    pub fn get_all(&self) -> Vec<&String> {
        self.commands.iter().collect()
    }
    
    pub fn search(&self, pattern: &str) -> Vec<&String> {
        self.commands
            .iter()
            .filter(|cmd| cmd.contains(pattern))
            .collect()
    }
    
    pub fn clear(&mut self) {
        self.commands.clear();
        self.save_to_file();
    }
    
    fn load_from_file(&mut self) {
        if let Some(ref path) = self.file_path {
            if let Ok(file) = File::open(path) {
                let reader = BufReader::new(file);
                for line in reader.lines() {
                    if let Ok(command) = line {
                        self.commands.push_back(command);
                    }
                }
                
                // Maintain max size
                while self.commands.len() > self.max_size {
                    self.commands.pop_front();
                }
            }
        }
    }
    
    fn save_to_file(&self) {
        if let Some(ref path) = self.file_path {
            if let Ok(mut file) = OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(path)
            {
                for command in &self.commands {
                    writeln!(file, "{}", command).unwrap_or(());
                }
            }
        }
    }
}

impl Default for History {
    fn default() -> Self {
        Self::new(1000)
    }
}
