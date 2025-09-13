use std::collections::HashMap;
use std::env;
pub mod parser;
pub mod executor;
pub mod builtins;
pub use parser::Parser;
pub use executor::Executor;
pub struct Shell{
    pub environment:HashMap<String,String>,
    pub current_dir:String,
    pub last_exit_code:i32,
    pub history:Vec<String>,
    pub jobs:Vec<Job>,
}

#[derive(Debug,Clone)]
pub struct Job{
    pub id:usize,
    pub pid:i32,
    pub command:String,
    pub status:JobStatus,
}

#[derive(Debug,Clone)]
pub enum JobStatus{
    Running,
    Stopped,
    Done,
}

impl Shell{
    pub fn new()->Self{
        let mut env=HashMap::new();
        for (key,value) in env::vars(){
            env.insert(key,value);
        }
        
        Self{
            environment:env,
            current_dir:env::current_dir().unwrap_or_default().to_string_lossy().to_string(),
            last_exit_code:0,
            history:Vec::new(),
            jobs:Vec::new(),
        }
    }
    
    pub fn add_job(&mut self,pid:i32,command:String){
        let id=self.jobs.len()+1;
        self.jobs.push(Job {
            id,
            pid,
            command,
            status: JobStatus::Running,
        });
    }
}