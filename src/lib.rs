pub mod shell;
pub mod command;
pub mod utils;
pub use command::{Command,CommandType};
pub mod config{
    pub const SHELL_NAME:&str="rshell";
    pub const SHELL_VERSION:&str="0.1.0";
    pub const DEFAULT_PROMPT:&str="$ ";
    pub const HISTORY_SIZE:usize=1000;
    pub const MAX_COMMAND_LENGTH:usize=4096;
    pub const MAX_ARGS:usize=100;
}

#[derive(Debug)]
pub enum ShellError{
    ParseError(String),
    ExecutionError(String),
    IOError(std::io::Error),
    SystemError(String),
}

impl std::fmt::Display for ShellError{
    fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result {
        match self{
            ShellError::ParseError(msg)=>write!(f,"Parse error: {}",msg),
            ShellError::ExecutionError(msg)=>write!(f,"Execution error: {}",msg),
            ShellError::IOError(err) => write!(f,"IO error: {}",err),
            ShellError::SystemError(msg) => write!(f,"System error: {}",msg),
        }
    }
}
impl std::error::Error for ShellError{}
impl From<std::io::Error> for ShellError{
    fn from(err:std::io::Error)->Self{
        ShellError::IOError(err)
    }
}
pub type ShellResult<T>=Result<T,ShellError>;