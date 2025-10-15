// src/scripting.rs
use crate::shell::Shell;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct ScriptEngine {
    variables: std::collections::HashMap<String, String>,
}

impl ScriptEngine {
    pub fn new() -> Self {
        Self {
            variables: std::collections::HashMap::new(),
        }
    }
    
    pub fn execute_script(&mut self, shell: &mut Shell, script_path: &Path) -> Result<i32, String> {
        let file = File::open(script_path)
            .map_err(|e| format!("Failed to open script: {}", e))?;
        
        let reader = BufReader::new(file);
        let mut last_exit_code = 0;
        
        for (line_num, line) in reader.lines().enumerate() {
            let line = line.map_err(|e| format!("Error reading line {}: {}", line_num + 1, e))?;
            let line = line.trim();
            
            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            
            // Process variable assignments
            if let Some((var, value)) = line.split_once('=') {
                let var = var.trim();
                let value = self.expand_variables(value.trim());
                self.variables.insert(var.to_string(), value);
                continue;
            }
            
           
            let expanded_line = self.expand_variables(line);
            
            match crate::shell::parser::Parser::new(&expanded_line) {
                Ok(mut parser) => {
                    match parser.parse() {
                        Ok(command_type) => {
                            let mut executor = crate::shell::executor::Executor::new();
                            last_exit_code = executor.execute(shell, command_type);
                        }
                        Err(e) => {
                            eprintln!("Script error at line {}: {:?}", line_num + 1, e);
                            return Err(format!("Parse error at line {}: {:?}", line_num + 1, e));
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Script error at line {}: {:?}", line_num + 1, e);
                    return Err(format!("Parse error at line {}: {:?}", line_num + 1, e));
                }
            }
            
            if last_exit_code != 0 && self.variables.get("errexit").is_some() {
                return Ok(last_exit_code);
            }
        }
        
        Ok(last_exit_code)
    }
    
    pub fn set_variable(&mut self, name: String, value: String) {
        self.variables.insert(name, value);
    }
    
    pub fn get_variable(&self, name: &str) -> Option<&String> {
        self.variables.get(name)
    }
    
    fn expand_variables(&self, input: &str) -> String {
        let mut result = input.to_string();
        
        // Simple variable expansion: $VAR or ${VAR}
        for (var, value) in &self.variables {
            let patterns = [format!("${}", var), format!("${{{}}}", var)];
            for pattern in &patterns {
                result = result.replace(pattern, value);
            }
        }
        
        result
    }
}

impl Default for ScriptEngine {
    fn default() -> Self {
        Self::new()
    }
}
