pub mod command;
pub mod config;
pub mod shell;
pub mod utils;
pub mod history;
pub mod scripting;

// Add the new modules
pub mod alias;
pub mod job_control;
pub mod signal_handler;
pub mod line_editor;
pub mod cache;
pub mod async_io;
pub mod parallel_exec;
pub mod performance;
pub mod memory_pool;

pub use command::{Command, CommandType};

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