// src/parallel_exec.rs
use rayon::prelude::*;
use crossbeam::channel::{bounded, unbounded, Sender};
use std::thread;
use std::time::Duration;
use std::os::unix::process::ExitStatusExt;
use crate::command::Command;

pub struct ParallelExecutor {
    thread_pool: rayon::ThreadPool,
    #[allow(dead_code)]
    worker_threads: Vec<WorkerThread>,
}

impl ParallelExecutor {
    pub fn new(num_threads: usize) -> std::io::Result<Self> {
        let thread_pool = rayon::ThreadPoolBuilder::new()
            .num_threads(num_threads)
            .build()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        
        let mut worker_threads = Vec::new();
        for i in 0..num_threads {
            worker_threads.push(WorkerThread::new(format!("worker-{}", i)));
        }
        
        Ok(Self {
            thread_pool,
            worker_threads,
        })
    }
    
    pub fn execute_parallel_commands(&self, commands: Vec<Command>) -> Vec<CommandResult> {
        self.thread_pool.install(|| {
            commands
                .par_iter()
                .map(|cmd| self.execute_single(cmd))
                .collect()
        })
    }
    
    pub fn execute_parallel_pipeline(&self, pipelines: Vec<Vec<Command>>) -> Vec<PipelineResult> {
        self.thread_pool.install(|| {
            pipelines
                .par_iter()
                .map(|pipeline| self.execute_pipeline(pipeline))
                .collect()
        })
    }
    
    fn execute_single(&self, command: &Command) -> CommandResult {
        use std::process::Command as StdCommand;
        use std::time::Instant;
        
        let start = Instant::now();
        
        let output = StdCommand::new(&command.program)
            .args(&command.args)
            .output();
        
        let duration = start.elapsed();
        
        match output {
            Ok(output) => CommandResult {
                command: command.clone(),
                exit_code: output.status.code().unwrap_or(-1),
                stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                duration,
            },
            Err(e) => CommandResult {
                command: command.clone(),
                exit_code: -1,
                stdout: String::new(),
                stderr: e.to_string(),
                duration,
            },
        }
    }
    
    fn execute_pipeline(&self, commands: &[Command]) -> PipelineResult {
        use std::process::{Command as StdCommand, Stdio};
        use std::time::Instant;
        
        let start = Instant::now();
        let mut last_output = Vec::new();
        let mut results = Vec::new();
        
        for (i, command) in commands.iter().enumerate() {
            let mut cmd = StdCommand::new(&command.program);
            cmd.args(&command.args);
            
            if i > 0 {
                cmd.stdin(Stdio::piped());
            }
            
            if i < commands.len() - 1 {
                cmd.stdout(Stdio::piped());
            }
            
            let mut child = match cmd.spawn() {
                Ok(child) => child,
                Err(e) => {
                    results.push(CommandResult {
                        command: command.clone(),
                        exit_code: -1,
                        stdout: String::new(),
                        stderr: e.to_string(),
                        duration: Duration::from_secs(0),
                    });
                    continue;
                }
            };
            
            // Write previous output to stdin
            if i > 0 {
                if let Some(mut stdin) = child.stdin.take() {
                    use std::io::Write;
                    let _ = stdin.write_all(&last_output);
                }
            }
            
            let output = child.wait_with_output().unwrap_or_else(|e| {
                std::process::Output {
                    status: std::process::ExitStatus::from_raw(1),
                    stdout: Vec::new(),
                    stderr: e.to_string().into_bytes(),
                }
            });
            
            last_output = output.stdout.clone();
            
            results.push(CommandResult {
                command: command.clone(),
                exit_code: output.status.code().unwrap_or(-1),
                stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                duration: Duration::from_secs(0),
            });
        }
        
        PipelineResult {
            commands: commands.to_vec(),
            results,
            total_duration: start.elapsed(),
        }
    }
    
    pub fn map_reduce<T, M, R, FR>(
        &self,
        items: Vec<T>,
        map_fn: M,
        reduce_fn: FR,
    ) -> Option<R>
    where
        T: Send + Sync,
        M: Fn(&T) -> R + Send + Sync,
        R: Send,
        FR: Fn(R, R) -> R + Send + Sync,
    {
        self.thread_pool.install(|| {
            items
                .par_iter()
                .map(map_fn)
                .reduce_with(reduce_fn)
        })
    }
    
    pub fn execute_with_timeout(
        &self,
        command: Command,
        timeout: Duration,
    ) -> CommandResult {
        let (tx, rx) = bounded(1);
        let command_clone = command.clone();
        
        self.thread_pool.spawn(move || {
            let result = Self::execute_single_static(&command_clone);
            let _ = tx.send(result);
        });
        
        match rx.recv_timeout(timeout) {
            Ok(result) => result,
            Err(_) => CommandResult {
                command,
                exit_code: -1,
                stdout: String::new(),
                stderr: "Command timed out".to_string(),
                duration: timeout,
            },
        }
    }
    
    fn execute_single_static(command: &Command) -> CommandResult {
        use std::process::Command as StdCommand;
        use std::time::Instant;
        
        let start = Instant::now();
        
        let output = StdCommand::new(&command.program)
            .args(&command.args)
            .output();
        
        let duration = start.elapsed();
        
        match output {
            Ok(output) => CommandResult {
                command: command.clone(),
                exit_code: output.status.code().unwrap_or(-1),
                stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                duration,
            },
            Err(e) => CommandResult {
                command: command.clone(),
                exit_code: -1,
                stdout: String::new(),
                stderr: e.to_string(),
                duration,
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct CommandResult {
    pub command: Command,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub duration: Duration,
}

#[derive(Debug, Clone)]
pub struct PipelineResult {
    pub commands: Vec<Command>,
    pub results: Vec<CommandResult>,
    pub total_duration: Duration,
}

struct WorkerThread {
    sender: Sender<WorkItem>,
    handle: Option<thread::JoinHandle<()>>,
}

enum WorkItem {
    Command(Command, Sender<CommandResult>),
    Shutdown,
}

impl WorkerThread {
    fn new(name: String) -> Self {
        let (tx, rx) = unbounded();
        
        let handle = thread::Builder::new()
            .name(name)
            .spawn(move || {
                while let Ok(item) = rx.recv() {
                    match item {
                        WorkItem::Command(cmd, result_tx) => {
                            let result = Self::execute_command(&cmd);
                            let _ = result_tx.send(result);
                        }
                        WorkItem::Shutdown => break,
                    }
                }
            })
            .unwrap();
        
        Self {
            sender: tx,
            handle: Some(handle),
        }
    }
    
    fn execute_command(command: &Command) -> CommandResult {
        use std::process::Command as StdCommand;
        use std::time::Instant;
        
        let start = Instant::now();
        
        let output = StdCommand::new(&command.program)
            .args(&command.args)
            .output();
        
        let duration = start.elapsed();
        
        match output {
            Ok(output) => CommandResult {
                command: command.clone(),
                exit_code: output.status.code().unwrap_or(-1),
                stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                duration,
            },
            Err(e) => CommandResult {
                command: command.clone(),
                exit_code: -1,
                stdout: String::new(),
                stderr: e.to_string(),
                duration,
            },
        }
    }
}

impl Drop for WorkerThread {
    fn drop(&mut self) {
        let _ = self.sender.send(WorkItem::Shutdown);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

// Batch command executor for efficient bulk operations
pub struct BatchExecutor {
    parallel_exec: ParallelExecutor,
    batch_size: usize,
}

impl BatchExecutor {
    pub fn new(num_threads: usize, batch_size: usize) -> std::io::Result<Self> {
        Ok(Self {
            parallel_exec: ParallelExecutor::new(num_threads)?,
            batch_size,
        })
    }
    
    pub fn execute_batch(&self, commands: Vec<Command>) -> Vec<CommandResult> {
        commands
            .chunks(self.batch_size)
            .flat_map(|batch| {
                self.parallel_exec.execute_parallel_commands(batch.to_vec())
            })
            .collect()
    }
}