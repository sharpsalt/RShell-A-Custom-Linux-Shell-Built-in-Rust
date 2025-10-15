// shell/parser.rs
use crate::command::{Command, CommandType, RedirectType};
use glob::glob;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Word(String),
    Pipe,
    Redirect(RedirectType),
    And,
    Or,
    Background,
    Semicolon,
    LeftParen,
    RightParen,
    Newline,
}

pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    pub fn new(input: &str) -> Result<Self, ParseError> {
        let expanded = expand_command_substitution(input)?;
        let tokens = Self::tokenize(&expanded)?;
        Ok(Self { tokens, position: 0 })
    }
    
    pub fn parse(&mut self) -> Result<CommandType, ParseError> {
        self.parse_logical_or()
    }
    
    fn parse_logical_or(&mut self) -> Result<CommandType, ParseError> {
        let mut left = self.parse_logical_and()?;
        
        while self.position < self.tokens.len() {
            if let Some(Token::Or) = self.peek() {
                self.consume()?;
                let right = self.parse_logical_and()?;
                left = CommandType::Or(Box::new(left), Box::new(right));
            } else {
                break;
            }
        }
        Ok(left)
    }
    
    fn parse_logical_and(&mut self) -> Result<CommandType, ParseError> {
        let mut left = self.parse_pipeline()?;
        
        while self.position < self.tokens.len() {
            if let Some(Token::And) = self.peek() {
                self.consume()?;
                let right = self.parse_pipeline()?;
                left = CommandType::And(Box::new(left), Box::new(right));
            } else {
                break;
            }
        }
        Ok(left)
    }
    
    fn parse_pipeline(&mut self) -> Result<CommandType, ParseError> {
        let mut commands = vec![self.parse_simple_command()?];
        
        while self.position < self.tokens.len() {
            if let Some(Token::Pipe) = self.peek() {
                self.consume()?;
                commands.push(self.parse_simple_command()?);
            } else {
                break;
            }
        }
        
        if commands.len() == 1 {
            Ok(CommandType::Simple(commands.into_iter().next().unwrap()))
        } else {
            Ok(CommandType::Pipeline(commands))
        }
    }
    
    fn parse_simple_command(&mut self) -> Result<Command, ParseError> {
        // Skip whitespace
        while let Some(Token::Newline) = self.peek() {
            self.consume()?;
        }
        
        let program = match self.consume()? {
            Token::Word(s) => s,
            _ => return Err(ParseError::new("Expected command".to_string(), self.position)),
        };
        
        let mut command = Command::new(program);
        
        while self.position < self.tokens.len() {
            match self.peek() {
                Some(Token::Word(s)) => {
                    let word = s.clone();
                    self.consume()?;
                    
                    // Apply glob expansion
                    let expanded = expand_glob(&word);
                    if expanded.is_empty() {
                        command.args.push(word);
                    } else {
                        command.args.extend(expanded);
                    }
                }
                Some(Token::Redirect(redirect_type)) => {
                    let redirect = redirect_type.clone();
                    self.consume()?;
                    
                    let target = match self.consume()? {
                        Token::Word(s) => s,
                        _ => return Err(ParseError::new("Expected redirect target".to_string(), self.position)),
                    };
                    
                    match redirect {
                        RedirectType::Input => command.stdin_redirect = Some(target),
                        RedirectType::Output => {
                            command.stdout_redirect = Some(target);
                            command.append_stdout = false;
                        }
                        RedirectType::Append => {
                            command.stdout_redirect = Some(target);
                            command.append_stdout = true;
                        }
                        RedirectType::Error => command.stderr_redirect = Some(target),
                        RedirectType::ErrorAppend => command.stderr_redirect = Some(target),
                    }
                }
                Some(Token::Background) => {
                    command.background = true;
                    self.consume()?;
                    break;
                }
                Some(Token::Pipe) | Some(Token::And) | Some(Token::Or) | Some(Token::Semicolon) => break,
                _ => break,
            }
        }
        
        Ok(command)
    }
    
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }
    
    fn consume(&mut self) -> Result<Token, ParseError> {
        if self.position >= self.tokens.len() {
            return Err(ParseError::new("Unexpected end of input".to_string(), self.position));
        }
        let token = self.tokens[self.position].clone();
        self.position += 1;
        Ok(token)
    }
    
    fn tokenize(input: &str) -> Result<Vec<Token>, ParseError> {
        let mut tokens = Vec::new();
        let mut chars = input.chars().peekable();
        let mut current_word = String::new();
        let mut in_single_quote = false;
        let mut in_double_quote = false;
        
        while let Some(ch) = chars.next() {
            // Handle quotes
            if ch == '\'' && !in_double_quote {
                in_single_quote = !in_single_quote;
                continue;
            }
            if ch == '"' && !in_single_quote {
                in_double_quote = !in_double_quote;
                continue;
            }
            
            if in_single_quote || in_double_quote {
                current_word.push(ch);
                continue;
            }
            
            match ch {
                ' ' | '\t' => {
                    if !current_word.is_empty() {
                        tokens.push(Token::Word(current_word.clone()));
                        current_word.clear();
                    }
                }
                '\n' => {
                    if !current_word.is_empty() {
                        tokens.push(Token::Word(current_word.clone()));
                        current_word.clear();
                    }
                    tokens.push(Token::Newline);
                }
                '|' => {
                    if chars.peek() == Some(&'|') {
                        chars.next();
                        if !current_word.is_empty() {
                            tokens.push(Token::Word(current_word.clone()));
                            current_word.clear();
                        }
                        tokens.push(Token::Or);
                    } else {
                        if !current_word.is_empty() {
                            tokens.push(Token::Word(current_word.clone()));
                            current_word.clear();
                        }
                        tokens.push(Token::Pipe);
                    }
                }
                '&' => {
                    if chars.peek() == Some(&'&') {
                        chars.next();
                        if !current_word.is_empty() {
                            tokens.push(Token::Word(current_word.clone()));
                            current_word.clear();
                        }
                        tokens.push(Token::And);
                    } else {
                        if !current_word.is_empty() {
                            tokens.push(Token::Word(current_word.clone()));
                            current_word.clear();
                        }
                        tokens.push(Token::Background);
                    }
                }
                '<' => {
                    if !current_word.is_empty() {
                        tokens.push(Token::Word(current_word.clone()));
                        current_word.clear();
                    }
                    tokens.push(Token::Redirect(RedirectType::Input));
                }
                '>' => {
                    if !current_word.is_empty() {
                        tokens.push(Token::Word(current_word.clone()));
                        current_word.clear();
                    }
                    if chars.peek() == Some(&'>') {
                        chars.next();
                        tokens.push(Token::Redirect(RedirectType::Append));
                    } else {
                        tokens.push(Token::Redirect(RedirectType::Output));
                    }
                }
                '2' if chars.peek() == Some(&'>') => {
                    chars.next(); // consume '>'
                    if !current_word.is_empty() {
                        tokens.push(Token::Word(current_word.clone()));
                        current_word.clear();
                    }
                    if chars.peek() == Some(&'>') {
                        chars.next();
                        tokens.push(Token::Redirect(RedirectType::ErrorAppend));
                    } else {
                        tokens.push(Token::Redirect(RedirectType::Error));
                    }
                }
                ';' => {
                    if !current_word.is_empty() {
                        tokens.push(Token::Word(current_word.clone()));
                        current_word.clear();
                    }
                    tokens.push(Token::Semicolon);
                }
                '(' => {
                    if !current_word.is_empty() {
                        tokens.push(Token::Word(current_word.clone()));
                        current_word.clear();
                    }
                    tokens.push(Token::LeftParen);
                }
                ')' => {
                    if !current_word.is_empty() {
                        tokens.push(Token::Word(current_word.clone()));
                        current_word.clear();
                    }
                    tokens.push(Token::RightParen);
                }
                '\\' if chars.peek().is_some() => {
                    // Handle escape sequences
                    if let Some(next_ch) = chars.next() {
                        current_word.push(next_ch);
                    }
                }
                _ => current_word.push(ch),
            }
        }
        
        if !current_word.is_empty() {
            tokens.push(Token::Word(current_word));
        }
        
        Ok(tokens)
    }
}

#[derive(Debug)]
pub struct ParseError {
    pub message: String,
    pub position: usize,
}

impl ParseError {
    pub fn new(message: String, position: usize) -> Self {
        Self { message, position }
    }
}

// Command substitution: Handle $(...) and `...`
fn expand_command_substitution(input: &str) -> Result<String, ParseError> {
    let mut result = String::new();
    let mut chars = input.chars().peekable();
    
    while let Some(ch) = chars.next() {
        if ch == '$' && chars.peek() == Some(&'(') {
            chars.next(); // consume '('
            let mut depth = 1;
            let mut cmd = String::new();
            
            while let Some(ch) = chars.next() {
                if ch == '(' {
                    depth += 1;
                    cmd.push(ch);
                } else if ch == ')' {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                    cmd.push(ch);
                } else {
                    cmd.push(ch);
                }
            }
            
            // Execute the command and get its output
            let output = execute_command_substitution(&cmd)?;
            result.push_str(&output);
        } else if ch == '`' {
            let mut cmd = String::new();
            while let Some(ch) = chars.next() {
                if ch == '`' {
                    break;
                }
                cmd.push(ch);
            }
            let output = execute_command_substitution(&cmd)?;
            result.push_str(&output);
        } else {
            result.push(ch);
        }
    }
    
    Ok(result)
}

fn execute_command_substitution(cmd: &str) -> Result<String, ParseError> {
    use std::process::Command;
    
    match Command::new("sh").arg("-c").arg(cmd).output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            // Remove trailing newline if present
            Ok(stdout.trim_end().to_string())
        }
        Err(e) => Err(ParseError::new(format!("Command substitution failed: {}", e), 0)),
    }
}

// Glob expansion
fn expand_glob(pattern: &str) -> Vec<String> {
    // Don't expand if pattern doesn't contain glob characters
    if !pattern.contains('*') && !pattern.contains('?') && !pattern.contains('[') {
        return vec![];
    }
    
    match glob(pattern) {
        Ok(paths) => {
            paths
                .filter_map(Result::ok)
                .map(|path| path.to_string_lossy().to_string())
                .collect()
        }
        Err(_) => vec![],
    }
}