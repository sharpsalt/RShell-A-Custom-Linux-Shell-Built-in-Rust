// shell/executor.rs
use crate::command::{Command, CommandType};
use crate::shell::{Shell, builtins, JobStatus};
use std::fs::{File, OpenOptions};
use std::os::unix::io::AsRawFd;
use nix::sys::wait::{waitpid, WaitStatus, WaitPidFlag};
use nix::unistd::{fork, ForkResult, dup2, close, execvp, Pid};
use std::ffi::CString;

// Simplified OptimizedExecutor without complex dependencies
pub struct OptimizedExecutor {
    // Placeholder for future optimizations
}


pub struct Executor {
    pub background_jobs: Vec<i32>,
}

impl OptimizedExecutor {
    pub fn new() -> std::io::Result<Self> {
        Ok(Self {})
    }
    
    pub fn execute_optimized(&mut self, shell: &mut Shell, cmd_type: CommandType) -> i32 {
        // Simplified implementation - delegate to standard executor
        let mut executor = Executor::new();
        executor.execute(shell, cmd_type)
    }
    
    pub fn get_performance_report(&self) -> String {
        "Performance monitoring not available in simplified mode".to_string()
    }
}

impl Executor {
    pub fn new() -> Self {
        Self {
            background_jobs: Vec::new(),
        }
    }
    
    pub fn execute(&mut self, shell: &mut Shell, cmd_type: CommandType) -> i32 {
        match cmd_type {
            CommandType::Simple(command) => self.execute_simple(shell, command),
            CommandType::Pipeline(commands) => self.execute_pipeline(shell, commands),
            CommandType::And(left, right) => {
                let left_result = self.execute(shell, *left);
                if left_result == 0 {
                    self.execute(shell, *right)
                } else {
                    left_result
                }
            }
            CommandType::Or(left, right) => {
                let left_result = self.execute(shell, *left);
                if left_result != 0 {
                    self.execute(shell, *right)
                } else {
                    left_result
                }
            }
        }
    }
    
    fn execute_simple(&mut self, shell: &mut Shell, command: Command) -> i32 {
        // Check if it's a builtin command
        if let Some(exit_code) = builtins::execute_builtin(
            shell, &command.program, &command.args
        ) {
            return exit_code;
        }
        
        // Fork and execute external command
        self.execute_external(shell, command)
    }
    
    fn execute_external(&mut self, shell: &mut Shell, command: Command) -> i32 {
        unsafe {
            match fork() {
                Ok(ForkResult::Parent { child }) => {
                    let pid = child.as_raw();
                    
                    if command.background {
                        // Add to background jobs
                        shell.add_job(pid, format!("{} {}", command.program, command.args.join(" ")));
                        self.background_jobs.push(pid);
                        println!("[{}] {}", shell.jobs.len(), pid);
                        0
                    } else {
                        // Wait for foreground process
                        match waitpid(child, None) {
                            Ok(WaitStatus::Exited(_, code)) => code as i32,
                            Ok(WaitStatus::Signaled(_, sig, _)) => 128 + sig as i32,
                            _ => 1,
                        }
                    }
                }
                Ok(ForkResult::Child) => {
                    // Setup redirections
                    if let Some(stdin_file) = command.stdin_redirect {
                        let file = File::open(&stdin_file).expect("Failed to open input file");
                        dup2(file.as_raw_fd(), 0).expect("Failed to redirect stdin");
                    }
                    
                    if let Some(stdout_file) = command.stdout_redirect {
                        let file = if command.append_stdout {
                            OpenOptions::new()
                                .create(true)
                                .append(true)
                                .open(&stdout_file)
                                .expect("Failed to open output file")
                        } else {
                            File::create(&stdout_file).expect("Failed to create output file")
                        };
                        dup2(file.as_raw_fd(), 1).expect("Failed to redirect stdout");
                    }
                    
                    if let Some(stderr_file) = command.stderr_redirect {
                        let file = File::create(&stderr_file).expect("Failed to create error file");
                        dup2(file.as_raw_fd(), 2).expect("Failed to redirect stderr");
                    }
                    
                    // Prepare arguments for execvp
                    let prog = CString::new(command.program.clone()).unwrap();
                    let mut args: Vec<CString> = vec![prog.clone()];
                    for arg in command.args {
                        args.push(CString::new(arg).unwrap());
                    }
                    
                    // Execute the command
                    execvp(&prog, &args).expect("Failed to execute command");
                    std::process::exit(127); // Command not found
                }
                Err(e) => {
                    eprintln!("Fork failed: {}", e);
                    return 1;
                }
            }
        }
    }
    
    pub fn execute_pipeline(&mut self, shell: &mut Shell, commands: Vec<Command>) -> i32 {
        if commands.is_empty() {
            return 0;
        }
        
        if commands.len() == 1 {
            return self.execute_simple(shell, commands.into_iter().next().unwrap());
        }
        
        let mut pids = Vec::new();
        let mut pipes = Vec::new();
        
        // Create pipes for all but the last command
        for _ in 0..commands.len() - 1 {
            let (read_fd, write_fd) = nix::unistd::pipe().expect("Failed to create pipe");
            pipes.push((read_fd, write_fd));
        }
        
        for (i, command) in commands.iter().enumerate() {
            unsafe {
                match fork() {
                    Ok(ForkResult::Parent { child }) => {
                        pids.push(child);
                        
                        // Close pipe ends we don't need in parent
                        if i > 0 {
                            close(pipes[i - 1].0).ok();
                        }
                        if i < pipes.len() {
                            close(pipes[i].1).ok();
                        }
                    }
                    Ok(ForkResult::Child) => {
                        // Setup pipes
                        if i > 0 {
                            // Not first command, redirect stdin from previous pipe
                            dup2(pipes[i - 1].0, 0).expect("Failed to redirect stdin");
                            close(pipes[i - 1].0).ok();
                            close(pipes[i - 1].1).ok();
                        }
                        
                        if i < pipes.len() {
                            // Not last command, redirect stdout to next pipe
                            dup2(pipes[i].1, 1).expect("Failed to redirect stdout");
                            close(pipes[i].0).ok();
                            close(pipes[i].1).ok();
                        }
                        
                        // Close all other pipes
                        for (read_fd, write_fd) in &pipes {
                            close(*read_fd).ok();
                            close(*write_fd).ok();
                        }
                        
                        // Check if it's a builtin
                        if builtins::is_builtin(&command.program) {
                            // For builtins in pipeline, we need to handle them specially
                            // For now, just exit with error
                            std::process::exit(1);
                        }
                        
                        // Execute the command
                        let prog = CString::new(command.program.clone()).unwrap();
                        let mut args: Vec<CString> = vec![prog.clone()];
                        for arg in &command.args {
                            args.push(CString::new(arg.clone()).unwrap());
                        }
                        
                        execvp(&prog, &args).expect("Failed to execute command");
                        std::process::exit(127);
                    }
                    Err(e) => {
                        eprintln!("Fork failed: {}", e);
                        return 1;
                    }
                }
            }
        }
        
        // Close all pipes in parent
        for (read_fd, write_fd) in pipes {
            close(read_fd).ok();
            close(write_fd).ok();
        }
        
        // Wait for all children
        let mut last_status = 0;
        for pid in pids {
            match waitpid(pid, None) {
                Ok(WaitStatus::Exited(_, code)) => last_status = code as i32,
                Ok(WaitStatus::Signaled(_, sig, _)) => last_status = 128 + sig as i32,
                _ => last_status = 1,
            }
        }
        
        last_status
    }
    
    pub fn check_background_jobs(&mut self, shell: &mut Shell) {
        let mut completed = Vec::new();
        
        for &pid in &self.background_jobs {
            match waitpid(Pid::from_raw(pid), Some(WaitPidFlag::WNOHANG)) {
                Ok(WaitStatus::Exited(_, code)) => {
                    println!("[Done] {} exited with code {}", pid, code);
                    completed.push(pid);
                    
                    // Update job status
                    if let Some(job) = shell.jobs.iter_mut().find(|j| j.pid == pid) {
                        job.status = JobStatus::Done;
                    }
                }
                Ok(WaitStatus::Signaled(_, sig, _)) => {
                    println!("[Done] {} terminated by signal {}", pid, sig);
                    completed.push(pid);
                    
                    if let Some(job) = shell.jobs.iter_mut().find(|j| j.pid == pid) {
                        job.status = JobStatus::Done;
                    }
                }
                _ => {}
            }
        }
        
        // Remove completed jobs
        self.background_jobs.retain(|pid| !completed.contains(pid));
    }
}
