// src/performance.rs
use std::time::{Duration, Instant};
use std::sync::Arc;
use parking_lot::RwLock;
use std::collections::VecDeque;

#[derive(Clone, Debug)]
pub struct PerformanceMetrics {
    pub command_execution_times: VecDeque<Duration>,
    pub parsing_times: VecDeque<Duration>,
    pub cache_hit_rate: f64,
    pub memory_usage: usize,
    pub active_threads: usize,
    pub total_commands: usize,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            command_execution_times: VecDeque::with_capacity(100),
            parsing_times: VecDeque::with_capacity(100),
            cache_hit_rate: 0.0,
            memory_usage: 0,
            active_threads: 0,
            total_commands: 0,
        }
    }
}

pub struct PerformanceMonitor {
    metrics: Arc<RwLock<PerformanceMetrics>>,
    start_time: Instant,
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(PerformanceMetrics::default())),
            start_time: Instant::now(),
        }
    }
    
    pub fn record_command_execution(&self, duration: Duration) {
        let mut metrics = self.metrics.write();
        if metrics.command_execution_times.len() >= 100 {
            metrics.command_execution_times.pop_front();
        }
        metrics.command_execution_times.push_back(duration);
        metrics.total_commands += 1;
    }
    
    pub fn record_parsing(&self, duration: Duration) {
        let mut metrics = self.metrics.write();
        if metrics.parsing_times.len() >= 100 {
            metrics.parsing_times.pop_front();
        }
        metrics.parsing_times.push_back(duration);
    }
    
    pub fn update_cache_hit_rate(&self, hits: usize, total: usize) {
        if total > 0 {
            let mut metrics = self.metrics.write();
            metrics.cache_hit_rate = (hits as f64) / (total as f64);
        }
    }
    
    pub fn update_memory_usage(&self, bytes: usize) {
        let mut metrics = self.metrics.write();
        metrics.memory_usage = bytes;
    }
    
    pub fn get_metrics(&self) -> PerformanceMetrics {
        self.metrics.read().clone()
    }
    
    pub fn get_average_execution_time(&self) -> Option<Duration> {
        let metrics = self.metrics.read();
        if metrics.command_execution_times.is_empty() {
            None
        } else {
            let total: Duration = metrics.command_execution_times.iter().sum();
            Some(total / metrics.command_execution_times.len() as u32)
        }
    }
    
    pub fn get_uptime(&self) -> Duration {
        self.start_time.elapsed()
    }
    
    pub fn print_report(&self) {
        let metrics = self.get_metrics();
        let uptime = self.get_uptime();
        
        println!("\n=== Performance Report ===");
        println!("Uptime: {:?}", uptime);
        println!("Total commands executed: {}", metrics.total_commands);
        
        if let Some(avg_exec) = self.get_average_execution_time() {
            println!("Average execution time: {:?}", avg_exec);
        }
        
        if !metrics.parsing_times.is_empty() {
            let avg_parse: Duration = metrics.parsing_times.iter().sum::<Duration>() 
                / metrics.parsing_times.len() as u32;
            println!("Average parsing time: {:?}", avg_parse);
        }
        
        println!("Cache hit rate: {:.2}%", metrics.cache_hit_rate * 100.0);
        println!("Memory usage: {} KB", metrics.memory_usage / 1024);
        println!("Active threads: {}", metrics.active_threads);
    }
}

// Profiling utilities
pub struct Profiler {
    name: String,
    start: Instant,
}

impl Profiler {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            start: Instant::now(),
        }
    }
    
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }
}

impl Drop for Profiler {
    fn drop(&mut self) {
        let elapsed = self.start.elapsed();
        if elapsed > Duration::from_millis(10) {
            eprintln!("[PROFILE] {} took {:?}", self.name, elapsed);
        }
    }
}

#[macro_export]
macro_rules! profile {
    ($name:expr) => {
        let _profiler = $crate::performance::Profiler::new($name);
    };
}