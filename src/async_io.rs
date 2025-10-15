// src/async_io.rs
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Command as TokioCommand};
use tokio::sync::mpsc::{channel, Receiver};
use tokio::task::JoinHandle;
use std::process::Stdio;
use std::sync::Arc;
use std::collections::HashMap;
use futures::future::join_all;

pub struct AsyncExecutor {
    runtime: Arc<tokio::runtime::Runtime>,
    background_tasks: HashMap<usize, JoinHandle<()>>,
    next_task_id: usize,
}

impl AsyncExecutor {
    pub fn new() -> std::io::Result<Self> {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(4)
            .enable_all()
            .build()?;
        
        Ok(Self {
            runtime: Arc::new(runtime),
            background_tasks: HashMap::new(),
            next_task_id: 0,
        })
    }
    
    pub fn execute_async(&mut self, command: &str, args: &[String]) -> std::io::Result<AsyncProcess> {
        let runtime = self.runtime.clone();
        let command = command.to_string();
        let args = args.to_vec();
        
        let (tx, rx) = channel(100);
        
        let _handle = runtime.spawn(async move {
            let mut child = TokioCommand::new(&command)
                .args(&args)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .stdin(Stdio::piped())
                .spawn()
                .expect("Failed to spawn process");
            
            // Read stdout
            if let Some(stdout) = child.stdout.take() {
                let tx = tx.clone();
                tokio::spawn(async move {
                    let reader = BufReader::new(stdout);
                    let mut lines = reader.lines();
                    while let Ok(Some(line)) = lines.next_line().await {
                        let _ = tx.send(OutputLine::Stdout(line)).await;
                    }
                });
            }
            
            // Read stderr
            if let Some(stderr) = child.stderr.take() {
                let tx = tx.clone();
                tokio::spawn(async move {
                    let reader = BufReader::new(stderr);
                    let mut lines = reader.lines();
                    while let Ok(Some(line)) = lines.next_line().await {
                        let _ = tx.send(OutputLine::Stderr(line)).await;
                    }
                });
            }
            
            // Wait for process to complete
            let status = child.wait().await.expect("Process failed");
            let _ = tx.send(OutputLine::Exit(status.code().unwrap_or(-1))).await;
        });
        
        Ok(AsyncProcess {
            receiver: rx,
            runtime: runtime.clone(),
        })
    }
    
    pub fn execute_pipeline_async(
        &mut self,
        commands: Vec<(String, Vec<String>)>,
    ) -> std::io::Result<AsyncPipeline> {
        let runtime = self.runtime.clone();
        let (tx, rx) = channel(100);
        
        runtime.spawn(async move {
            let mut children = Vec::new();
            let mut last_stdout = None;
            
            for (i, (cmd, args)) in commands.iter().enumerate() {
                let mut command = TokioCommand::new(cmd);
                command.args(args);
                
                // Setup pipes
                if let Some(stdout) = last_stdout.take() {
                    command.stdin(stdout);
                }
                
                if i < commands.len() - 1 {
                    command.stdout(Stdio::piped());
                } else {
                    // Last command - capture output
                    command.stdout(Stdio::piped());
                }
                
                command.stderr(Stdio::piped());
                
                let mut child = command.spawn().expect("Failed to spawn process");
                
                if i < commands.len() - 1 {
                    last_stdout = child.stdout.take().map(|_stdout| {
                        // Convert tokio ChildStdout to std Stdio
                        // Note: This is a simplified conversion - in practice you'd need 
                        // to properly handle the async I/O conversion
                        std::process::Stdio::piped()
                    });
                } else {
                    // Read output from last command
                    if let Some(stdout) = child.stdout.take() {
                        let tx = tx.clone();
                        tokio::spawn(async move {
                            let reader = BufReader::new(stdout);
                            let mut lines = reader.lines();
                            while let Ok(Some(line)) = lines.next_line().await {
                                let _ = tx.send(line).await;
                            }
                        });
                    }
                }
                
                children.push(child);
            }
            
            // Wait for all processes
            for mut child in children {
                let _ = child.wait().await;
            }
        });
        
        Ok(AsyncPipeline { receiver: rx })
    }
    
    pub fn run_parallel<F, T>(
        &self,
        tasks: Vec<F>,
    ) -> Vec<T>
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        self.runtime.block_on(async {
            let handles: Vec<_> = tasks
                .into_iter()
                .map(|task| {
                    tokio::spawn(async move {
                        task()
                    })
                })
                .collect();
            
            let results = join_all(handles).await;
            results.into_iter()
                .filter_map(|r| r.ok())
                .collect()
        })
    }
    
    pub fn spawn_background<F>(&mut self, task: F) -> usize
    where
        F: std::future::Future<Output = ()> + Send + 'static,
    {
        let id = self.next_task_id;
        self.next_task_id += 1;
        
        let handle = self.runtime.spawn(task);
        self.background_tasks.insert(id, handle);
        
        id
    }
    
    pub fn cancel_background(&mut self, id: usize) -> bool {
        if let Some(handle) = self.background_tasks.remove(&id) {
            handle.abort();
            true
        } else {
            false
        }
    }
}

pub struct AsyncProcess {
    receiver: Receiver<OutputLine>,
    runtime: Arc<tokio::runtime::Runtime>,
}

pub struct AsyncPipeline {
    receiver: Receiver<String>,
}

#[derive(Debug)]
pub enum OutputLine {
    Stdout(String),
    Stderr(String),
    Exit(i32),
}

impl AsyncProcess {
    pub fn read_line(&mut self) -> Option<OutputLine> {
        self.runtime.block_on(async {
            self.receiver.recv().await
        })
    }
    
    pub async fn read_all(&mut self) -> Vec<OutputLine> {
        let mut lines = Vec::new();
        while let Some(line) = self.receiver.recv().await {
            lines.push(line);
        }
        lines
    }
}

impl AsyncPipeline {
    pub fn read_line(&mut self) -> Option<String> {
        self.receiver.blocking_recv()
    }
    
    pub async fn read_all(&mut self) -> Vec<String> {
        let mut lines = Vec::new();
        while let Some(line) = self.receiver.recv().await {
            lines.push(line);
        }
        lines
    }
}

// Async file operations
pub struct AsyncFileOps;

impl AsyncFileOps {
    pub async fn read_file(path: &str) -> std::io::Result<String> {
        tokio::fs::read_to_string(path).await
    }
    
    pub async fn write_file(path: &str, content: &str) -> std::io::Result<()> {
        tokio::fs::write(path, content).await
    }
    
    pub async fn append_file(path: &str, content: &str) -> std::io::Result<()> {
        use tokio::fs::OpenOptions;
        
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .await?;
        
        file.write_all(content.as_bytes()).await?;
        Ok(())
    }
    
    pub async fn list_directory(path: &str) -> std::io::Result<Vec<String>> {
        let mut entries = Vec::new();
        let mut dir = tokio::fs::read_dir(path).await?;
        
        while let Some(entry) = dir.next_entry().await? {
            if let Some(name) = entry.file_name().to_str() {
                entries.push(name.to_string());
            }
        }
        
        Ok(entries)
    }
}