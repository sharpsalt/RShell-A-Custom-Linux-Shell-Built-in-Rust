#[derive(Debug,Clone,PartialEq)]
pub enum Token{
    Word(String),
    Pipe,
    Redirect(RedirectType),
    And,
    Or,
    Background,
    Semicolon,
}

#[derive(Debug,Clone,PartialEq)]
pub enum RedirectType{
    Input,           // <
    Output,          // >
    Append,          // >>
    Error,           // 2>
    ErrorAppend,     // 2>>
}

pub struct Parser{
    tokens:Vec<Token>,
    position:usize,
}

impl Parser{
        pub fn parse(&mut self)->Result<crate::command::CommandType,ParseError>{
        use crate::command::{Command,CommandType};
        if self.tokens.is_empty(){
            return Err(ParseError {
                message:"No tokens to parse".to_string(),
                position:0,
            });
        }
        if let Some(Token::Word(program))=self.tokens.get(0){
            let mut command=Command::new(program.clone());
            for token in &self.tokens[1..]{
                if let Token::Word(arg)=token{
                    command.args.push(arg.clone());
                }
            }
            Ok(CommandType::Simple(command))
        }else{
            Err(ParseError{
                message:"Expected command".to_string(),
                position:0,
            })
        }
    }
    pub fn new(input:&str)->Result<Self,ParseError>{
        let tokens=Self::tokenize(input)?;
        Ok(Self{tokens,position:0})
    }
    
    fn tokenize(input:&str)->Result<Vec<Token>,ParseError>{
        let mut tokens=Vec::new();
        let mut chars=input.chars().peekable();
        let mut current_word=String::new();
        while let Some(ch)=chars.next(){
            match ch{
                ' '|'\t'=>{
                    if !current_word.is_empty(){
                        tokens.push(Token::Word(current_word.clone()));
                        current_word.clear();
                    }
                }
                '|'=>{
                    if !current_word.is_empty(){
                        tokens.push(Token::Word(current_word.clone()));
                        current_word.clear();
                    }
                    tokens.push(Token::Pipe);
                }
                '&'=>{
                    if chars.peek()==Some(&'&'){
                        chars.next(); // consume second &
                        if !current_word.is_empty(){
                            tokens.push(Token::Word(current_word.clone()));
                            current_word.clear();
                        }
                        tokens.push(Token::And);
                    }else{
                        if !current_word.is_empty(){
                            tokens.push(Token::Word(current_word.clone()));
                            current_word.clear();
                        }
                        tokens.push(Token::Background);
                    }
                }
                // willl add more logic for redirets,quotes,etc
                _=>current_word.push(ch),
            }
        }
        if !current_word.is_empty(){
            tokens.push(Token::Word(current_word));
        }
        Ok(tokens)
    }
}

#[derive(Debug)]
pub struct ParseError {
    pub message:String,
    pub position:usize,
}