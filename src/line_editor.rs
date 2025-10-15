// src/line_editor.rs
use rustyline::{
    completion::{Completer, FilenameCompleter, Pair},
    highlight::{Highlighter, MatchingBracketHighlighter},
    hint::{Hinter, HistoryHinter},
    validate::{Validator, ValidationResult, ValidationContext},
    Helper, Context, Result,
};
use std::borrow::Cow::{self, Borrowed, Owned};
use std::collections::HashSet;
use std::path::Path;
use std::env;
use crate::config::Theme;

pub struct ShellHelper {
    pub completer: FilenameCompleter,
    pub highlighter: MatchingBracketHighlighter,
    pub hinter: HistoryHinter,
    pub colored_prompt: String,
    pub theme: Theme,
    pub builtins: HashSet<String>,
    pub aliases: std::collections::HashMap<String, String>,
}

impl ShellHelper {
    pub fn new(theme: Theme, aliases: std::collections::HashMap<String, String>) -> Self {
        let mut builtins = HashSet::new();
        builtins.insert("cd".to_string());
        builtins.insert("pwd".to_string());
        builtins.insert("echo".to_string());
        builtins.insert("export".to_string());
        builtins.insert("unset".to_string());
        builtins.insert("exit".to_string());
        builtins.insert("history".to_string());
        builtins.insert("jobs".to_string());
        builtins.insert("help".to_string());
        builtins.insert("fg".to_string());
        builtins.insert("bg".to_string());
        builtins.insert("kill".to_string());
        builtins.insert("alias".to_string());
        builtins.insert("unalias".to_string());
        builtins.insert("theme".to_string());
        builtins.insert("config".to_string());
        
        Self {
            completer: FilenameCompleter::new(),
            highlighter: MatchingBracketHighlighter::new(),
            hinter: HistoryHinter {},
            colored_prompt: String::new(),
            theme,
            builtins,
            aliases,
        }
    }
    
    pub fn set_prompt(&mut self, prompt: &str) {
        self.colored_prompt = self.theme.colorize_prompt(prompt);
    }
    
    pub fn update_aliases(&mut self, aliases: std::collections::HashMap<String, String>) {
        self.aliases = aliases;
    }
}

impl Completer for ShellHelper {
    type Candidate = Pair;
    
    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Pair>)> {
        // First, try file completion
        let (_start, mut candidates) = self.completer.complete(line, pos, ctx)?;
        
        // Find the word being completed
        let word_start = line[..pos].rfind(' ').map(|i| i + 1).unwrap_or(0);
        let word = &line[word_start..pos];
        
        // If we're at the beginning of the line or after a pipe/semicolon, add commands
        if word_start == 0 || line[..word_start].trim_end().ends_with('|') 
            || line[..word_start].trim_end().ends_with(';') {
            
            // Add builtins
            for builtin in &self.builtins {
                if builtin.starts_with(word) {
                    candidates.push(Pair {
                        display: builtin.clone(),
                        replacement: builtin.clone(),
                    });
                }
            }
            
            // Add aliases
            for (alias, _) in &self.aliases {
                if alias.starts_with(word) {
                    candidates.push(Pair {
                        display: format!("{} (alias)", alias),
                        replacement: alias.clone(),
                    });
                }
            }
            
            // Add commands from PATH
            if let Ok(path_var) = env::var("PATH") {
                for dir in path_var.split(':') {
                    if let Ok(entries) = std::fs::read_dir(dir) {
                        for entry in entries.flatten() {
                            if let Some(name) = entry.file_name().to_str() {
                                if name.starts_with(word) && is_executable(&entry.path()) {
                                    candidates.push(Pair {
                                        display: name.to_string(),
                                        replacement: name.to_string(),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Sort and deduplicate
        candidates.sort_by(|a, b| a.display.cmp(&b.display));
        candidates.dedup_by(|a, b| a.replacement == b.replacement);
        
        Ok((word_start, candidates))
    }
}

impl Hinter for ShellHelper {
    type Hint = String;
    
    fn hint(&self, line: &str, pos: usize, ctx: &Context<'_>) -> Option<String> {
        self.hinter.hint(line, pos, ctx)
    }
}

impl Highlighter for ShellHelper {
    fn highlight<'l>(&self, line: &'l str, pos: usize) -> Cow<'l, str> {
        let mut highlighted = String::new();
        let mut chars = line.chars().peekable();
        let mut in_string = false;
        let mut string_char = '"';
        let mut current_word = String::new();
        let mut is_first_word = true;
        let mut after_pipe = false;
        
        while let Some(ch) = chars.next() {
            match ch {
                '"' | '\'' if !in_string => {
                    in_string = true;
                    string_char = ch;
                    highlighted.push_str(&self.theme.string_color);
                    highlighted.push(ch);
                }
                ch if in_string && ch == string_char => {
                    highlighted.push(ch);
                    highlighted.push_str(&self.theme.reset_color);
                    in_string = false;
                }
                ' ' | '\t' if !in_string => {
                    if !current_word.is_empty() {
                        let colored = self.colorize_word(&current_word, is_first_word || after_pipe);
                        highlighted.push_str(&colored);
                        current_word.clear();
                        is_first_word = false;
                        after_pipe = false;
                    }
                    highlighted.push(ch);
                }
                '|' | '&' | ';' | '>' | '<' if !in_string => {
                    if !current_word.is_empty() {
                        let colored = self.colorize_word(&current_word, is_first_word || after_pipe);
                        highlighted.push_str(&colored);
                        current_word.clear();
                    }
                    highlighted.push_str(&self.theme.operator_color);
                    highlighted.push(ch);
                    
                    // Check for double operators
                    if (ch == '|' || ch == '&' || ch == '>') && chars.peek() == Some(&ch) {
                        highlighted.push(chars.next().unwrap());
                    }
                    highlighted.push_str(&self.theme.reset_color);
                    
                    if ch == '|' {
                        after_pipe = true;
                    }
                    is_first_word = false;
                }
                '$' if !in_string => {
                    if chars.peek() == Some(&'(') || chars.peek() == Some(&'{') {
                        highlighted.push_str(&self.theme.variable_color);
                        highlighted.push(ch);
                    } else if chars.peek().map_or(false, |c| c.is_alphabetic() || *c == '_') {
                        highlighted.push_str(&self.theme.variable_color);
                        highlighted.push(ch);
                        // Continue reading variable name
                        while let Some(&next_ch) = chars.peek() {
                            if next_ch.is_alphanumeric() || next_ch == '_' {
                                highlighted.push(chars.next().unwrap());
                            } else {
                                break;
                            }
                        }
                        highlighted.push_str(&self.theme.reset_color);
                    } else {
                        current_word.push(ch);
                    }
                }
                _ => {
                    if in_string {
                        highlighted.push(ch);
                    } else {
                        current_word.push(ch);
                    }
                }
            }
        }
        
        // Handle remaining word
        if !current_word.is_empty() {
            let colored = self.colorize_word(&current_word, is_first_word || after_pipe);
            highlighted.push_str(&colored);
        }
        
        // Apply bracket highlighting
        let highlighted = self.highlighter.highlight(&highlighted, pos);
        
        Owned(highlighted.to_string())
    }
    
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        default: bool,
    ) -> Cow<'b, str> {
        if default {
            Borrowed(prompt)
        } else {
            Borrowed(&self.colored_prompt)
        }
    }
    
    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Owned(format!("{}{}{}", self.theme.hint_color, hint, self.theme.reset_color))
    }
    
    fn highlight_char(&self, line: &str, pos: usize) -> bool {
        self.highlighter.highlight_char(line, pos)
    }
}

impl Validator for ShellHelper {
    fn validate(&self, ctx: &mut ValidationContext) -> Result<ValidationResult> {
        let input = ctx.input();
        
        // Check for unclosed quotes
        let mut in_single_quote = false;
        let mut in_double_quote = false;
        let mut escape_next = false;
        
        for ch in input.chars() {
            if escape_next {
                escape_next = false;
                continue;
            }
            
            match ch {
                '\\' => escape_next = true,
                '\'' if !in_double_quote => in_single_quote = !in_single_quote,
                '"' if !in_single_quote => in_double_quote = !in_double_quote,
                _ => {}
            }
        }
        
        if in_single_quote || in_double_quote {
            Ok(ValidationResult::Incomplete)
        } else {
            Ok(ValidationResult::Valid(None))
        }
    }
}

impl Helper for ShellHelper {}

impl ShellHelper {
    fn colorize_word(&self, word: &str, is_command: bool) -> String {
        if is_command {
            if self.builtins.contains(word) {
                format!("{}{}{}", self.theme.builtin_color, word, self.theme.reset_color)
            } else if self.aliases.contains_key(word) {
                format!("{}{}{}", self.theme.alias_color, word, self.theme.reset_color)
            } else if is_command_in_path(word) {
                format!("{}{}{}", self.theme.command_color, word, self.theme.reset_color)
            } else {
                format!("{}{}{}", self.theme.error_color, word, self.theme.reset_color)
            }
        } else if word.starts_with('-') {
            format!("{}{}{}", self.theme.flag_color, word, self.theme.reset_color)
        } else if Path::new(word).exists() {
            format!("{}{}{}", self.theme.path_color, word, self.theme.reset_color)
        } else {
            word.to_string()
        }
    }
}

fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    path.metadata()
        .map(|m| m.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

fn is_command_in_path(cmd: &str) -> bool {
    if let Ok(path_var) = env::var("PATH") {
        for dir in path_var.split(':') {
            let full_path = Path::new(dir).join(cmd);
            if full_path.exists() && is_executable(&full_path) {
                return true;
            }
        }
    }
    false
}