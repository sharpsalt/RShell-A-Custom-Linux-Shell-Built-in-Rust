// src/alias.rs
use std::collections::HashMap;
use crate::shell::Shell;

pub fn builtin_alias(shell: &mut Shell, args: &[String]) -> i32 {
    if args.is_empty() {
        // Display all aliases
        for (name, value) in &shell.aliases {
            println!("alias {}='{}'", name, value);
        }
        return 0;
    }
    
    for arg in args {
        if let Some(eq_pos) = arg.find('=') {
            let name = arg[..eq_pos].to_string();
            let value = arg[eq_pos + 1..].to_string();
            
            // Remove quotes if present
            let value = value.trim_matches(|c| c == '\'' || c == '"').to_string();
            
            shell.aliases.insert(name, value);
        } else {
            // Show specific alias
            if let Some(value) = shell.aliases.get(arg) {
                println!("alias {}='{}'", arg, value);
            } else {
                eprintln!("alias: {}: not found", arg);
                return 1;
            }
        }
    }
    
    0
}

pub fn builtin_unalias(shell: &mut Shell, args: &[String]) -> i32 {
    if args.is_empty() {
        eprintln!("unalias: usage: unalias name [name ...]");
        return 1;
    }
    
    for name in args {
        if name == "-a" {
            shell.aliases.clear();
            return 0;
        }
        
        if shell.aliases.remove(name).is_none() {
            eprintln!("unalias: {}: not found", name);
            return 1;
        }
    }
    
    0
}

pub fn expand_aliases(input: &str, aliases: &HashMap<String, String>) -> String {
    let mut result = String::new();
    let mut words = input.split_whitespace();
    let mut is_first = true;
    let mut in_pipeline = false;
    
    while let Some(word) = words.next() {
        if !is_first && !in_pipeline {
            result.push(' ');
        }
        
        // Check if this word should be expanded as an alias
        if (is_first || in_pipeline) && aliases.contains_key(word) {
            result.push_str(&aliases[word]);
        } else {
            result.push_str(word);
        }
        
        // Check for pipeline or command separator
        if word == "|" || word == "||" || word == "&&" || word == ";" {
            in_pipeline = true;
        } else {
            in_pipeline = false;
        }
        
        is_first = false;
    }
    
    result
}