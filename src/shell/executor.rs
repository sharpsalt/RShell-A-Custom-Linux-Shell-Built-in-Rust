use crate::command::{Command,CommandType};
use crate::shell::{Shell,builtins};
use std::process;

pub struct Executor{
    background_jobs:Vec<i32>, // PIDs of background processes
}

impl Executor{
    pub fn new()->Self{
        Self{
            background_jobs:Vec::new(),
        }
    }
    
    pub fn execute(&mut self,shell:&mut Shell, cmd_type:CommandType)->i32{
        match cmd_type{
            CommandType::Simple(command)=>self.execute_simple(shell, command),
            CommandType::Pipeline(commands)=>self.execute_pipeline(shell, commands),
            CommandType::And(left,right)=>{
                let left_result=self.execute(shell,*left);
                if left_result==0{
                    self.execute(shell,*right)
                }else{
                    left_result
                }
            }
            CommandType::Or(left,right)=>{
                let left_result=self.execute(shell,*left);
                if left_result!=0{
                    self.execute(shell,*right)
                }else{
                    left_result
                }
            }
        }
    }
    
    fn execute_simple(&mut self,shell:&mut Shell,command: Command)->i32{
        if let Some(exit_code)=builtins::execute_builtin(
            shell,&command.program,&command.args
        ){
            return exit_code;
        }
        
        // for now,just execute external commands using std::process::Command
        self.execute_external(shell,command)
    }
    
    fn execute_external(&mut self,_shell:&mut Shell,command:Command)->i32{
        let mut cmd=process::Command::new(&command.program);
        cmd.args(&command.args);
        match cmd.status(){
            Ok(status)=>{
                status.code().unwrap_or(1)
            }
            Err(_)=>{
                eprintln!("rshell: {}: command not found",command.program);
                127
            }
        }
    }
    
    pub fn execute_pipeline(&mut self,_shell:&mut Shell,_commands:Vec<Command>)->i32{
        println!("Pipeline execution not yet implemented");
        0
    }
}