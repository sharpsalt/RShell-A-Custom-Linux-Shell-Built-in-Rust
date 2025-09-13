pub mod command;
pub use command::{Command, CommandType, RedirectType};

// Parsing error type for command parsing
#[derive(Debug,Clone)]
pub struct ParseError{
    pub message:String,
    // let position=-1
    pub position:usize,
}

impl std::fmt::Display for ParseError{
    fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result{
        write!(f,"Parse error at position {}:{}",self.position,self.message)
    }
}

impl std::error::Error for ParseError{}

impl ParseError{
    pub fn new(message:String,position:usize)->Self{
        Self{message,position}
    }
}

pub mod validation{
    use super::Command;
    pub fn validate_command(command:&Command)->Result<(),String>{
        if command.program.is_empty(){
            return Err("Empty command program".to_string());
        }
        if command.program.contains('\0'){
            return Err("Null character in command program".to_string());//dangerous character hua if
        }
        for arg in &command.args{
            if arg.contains('\0'){
                //validating argumnets basically...
                return Err("Null character in command argument".to_string());
            }
        }
        Ok(())
    }
    pub fn is_builtin(program: &str)->bool{//will check whether it is a builtin command or not 
        matches!(program, "cd" | "pwd" | "echo" | "export" | "unset" | "exit" | "history" | "jobs" | "help")
    }
}