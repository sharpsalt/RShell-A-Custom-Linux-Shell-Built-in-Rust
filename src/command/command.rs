#[derive(Debug, Clone, PartialEq)]
pub enum RedirectType {
    Input,           // 
    Output,          // >
    Append,          // >>
    Error,           // 2>
    ErrorAppend,     // 2>>
}

#[derive(Debug,Clone)]
pub struct Command{
    pub program:String,
    pub args:Vec<String>,
    pub stdin_redirect:Option<String>,
    pub stdout_redirect:Option<String>,
    pub stderr_redirect:Option<String>,
    pub append_stdout:bool,
    pub background:bool,
}

impl Command{
    pub fn new(program:String)->Self{
        Self{
            program,
            args:Vec::new(),
            stdin_redirect:None,
            stdout_redirect:None,
            stderr_redirect:None,
            append_stdout:false,
            background:false,
        }
    }
}

#[derive(Debug,Clone)]
pub enum CommandType{
    Simple(Command),
    Pipeline(Vec<Command>),
    And(Box<CommandType>,Box<CommandType>),
    Or(Box<CommandType>,Box<CommandType>),
}