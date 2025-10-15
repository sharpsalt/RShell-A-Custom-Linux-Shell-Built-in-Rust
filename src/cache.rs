// src/cache.rs
use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, Duration};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::fs;
use lru::LruCache;
use std::num::NonZeroUsize;

#[derive(Clone)]
pub struct CommandCache {
    // Path to executable mapping
    path_cache: Arc<RwLock<LruCache<String, Option<PathBuf>>>>,
    // Directory contents cache
    dir_cache: Arc<RwLock<HashMap<PathBuf, DirCacheEntry>>>,
    // Builtin commands set for O(1) lookup
    builtins: Arc<HashSet<String>>,
    // Command completion cache
    completion_cache: Arc<RwLock<HashMap<String, Vec<String>>>>,
    // Stats for monitoring
    stats: Arc<RwLock<CacheStats>>,
}

#[derive(Clone, Debug)]
struct DirCacheEntry {
    entries: Vec<String>,
    timestamp: SystemTime,
}

#[derive(Clone, Debug, Default)]
pub struct CacheStats {
    pub path_hits: usize,
    pub path_misses: usize,
    pub dir_hits: usize,
    pub dir_misses: usize,
    pub completion_hits: usize,
    pub completion_misses: usize,
}

impl CommandCache {
    pub fn new() -> Self {
        let mut builtins = HashSet::new();
        for cmd in &[
            "cd", "pwd", "echo", "export", "unset", "exit", 
            "history", "jobs", "help", "fg", "bg", "kill",
            "alias", "unalias", "theme", "config"
        ] {
            builtins.insert(cmd.to_string());
        }
        
        Self {
            path_cache: Arc::new(RwLock::new(LruCache::new(
                NonZeroUsize::new(1000).unwrap()
            ))),
            dir_cache: Arc::new(RwLock::new(HashMap::new())),
            builtins: Arc::new(builtins),
            completion_cache: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(CacheStats::default())),
        }
    }
    
    pub fn is_builtin(&self, cmd: &str) -> bool {
        self.builtins.contains(cmd)
    }
    
    pub fn find_command(&self, cmd: &str) -> Option<PathBuf> {
        // Check if it's a builtin first (O(1))
        if self.is_builtin(cmd) {
            return Some(PathBuf::from(cmd));
        }
        
        // Check if it's an absolute or relative path
        if cmd.contains('/') {
            let path = Path::new(cmd);
            if path.exists() && is_executable(path) {
                return Some(path.to_path_buf());
            }
            return None;
        }
        
        // Check cache first
        {
            let mut cache = self.path_cache.write().unwrap();
            if let Some(cached) = cache.get(cmd) {
                self.stats.write().unwrap().path_hits += 1;
                return cached.clone();
            }
        }
        
        self.stats.write().unwrap().path_misses += 1;
        
        // Search in PATH
        let result = self.search_in_path(cmd);
        
        // Cache the result
        {
            let mut cache = self.path_cache.write().unwrap();
            cache.put(cmd.to_string(), result.clone());
        }
        
        result
    }
    
    fn search_in_path(&self, cmd: &str) -> Option<PathBuf> {
        std::env::var("PATH").ok().and_then(|path_var| {
            path_var
                .split(':')
                .filter_map(|dir| {
                    let full_path = Path::new(dir).join(cmd);
                    if full_path.exists() && is_executable(&full_path) {
                        Some(full_path)
                    } else {
                        None
                    }
                })
                .next()
        })
    }
    
    pub fn get_dir_contents(&self, dir: &Path) -> Option<Vec<String>> {
        let now = SystemTime::now();
        
        // Check cache first
        {
            let cache = self.dir_cache.read().unwrap();
            if let Some(entry) = cache.get(dir) {
                // Cache entries expire after 60 seconds
                if now.duration_since(entry.timestamp).unwrap_or(Duration::MAX) 
                    < Duration::from_secs(60) {
                    self.stats.write().unwrap().dir_hits += 1;
                    return Some(entry.entries.clone());
                }
            }
        }
        
        self.stats.write().unwrap().dir_misses += 1;
        
        // Read directory
        let entries: Vec<String> = fs::read_dir(dir)
            .ok()?
            .filter_map(|entry| {
                entry.ok().and_then(|e| {
                    e.file_name().to_str().map(|s| s.to_string())
                })
            })
            .collect();
        
        // Cache the result
        {
            let mut cache = self.dir_cache.write().unwrap();
            cache.insert(
                dir.to_path_buf(),
                DirCacheEntry {
                    entries: entries.clone(),
                    timestamp: now,
                },
            );
        }
        
        Some(entries)
    }
    
    pub fn get_completions(&self, prefix: &str) -> Vec<String> {
        // Check cache first
        {
            let cache = self.completion_cache.read().unwrap();
            if let Some(completions) = cache.get(prefix) {
                self.stats.write().unwrap().completion_hits += 1;
                return completions.clone();
            }
        }
        
        self.stats.write().unwrap().completion_misses += 1;
        
        let mut completions = Vec::new();
        
        // Add builtins
        for builtin in self.builtins.iter() {
            if builtin.starts_with(prefix) {
                completions.push(builtin.clone());
            }
        }
        
        // Add commands from PATH
        if let Ok(path_var) = std::env::var("PATH") {
            for dir in path_var.split(':') {
                if let Some(entries) = self.get_dir_contents(Path::new(dir)) {
                    for entry in entries {
                        if entry.starts_with(prefix) && !completions.contains(&entry) {
                            completions.push(entry);
                        }
                    }
                }
            }
        }
        
        completions.sort();
        
        // Cache the result
        {
            let mut cache = self.completion_cache.write().unwrap();
            cache.insert(prefix.to_string(), completions.clone());
        }
        
        completions
    }
    
    pub fn invalidate(&self) {
        self.path_cache.write().unwrap().clear();
        self.dir_cache.write().unwrap().clear();
        self.completion_cache.write().unwrap().clear();
    }
    
    pub fn invalidate_path(&self, path: &Path) {
        let mut dir_cache = self.dir_cache.write().unwrap();
        dir_cache.remove(path);
        
        // Also invalidate parent directory
        if let Some(parent) = path.parent() {
            dir_cache.remove(parent);
        }
    }
    
    pub fn get_stats(&self) -> CacheStats {
        self.stats.read().unwrap().clone()
    }
}

fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    path.metadata()
        .map(|m| m.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

// Background cache warmer
pub struct CacheWarmer {
    cache: CommandCache,
}

impl CacheWarmer {
    pub fn new(cache: CommandCache) -> Self {
        Self { cache }
    }
    
    pub fn warm_up(&self) {
        use std::thread;
        
        let cache = self.cache.clone();
        thread::spawn(move || {
            // Pre-warm PATH directories
            if let Ok(path_var) = std::env::var("PATH") {
                for dir in path_var.split(':') {
                    let _ = cache.get_dir_contents(Path::new(dir));
                }
            }
            
            // Pre-warm common commands
            for cmd in &["ls", "git", "gcc", "cargo", "python", "node", "npm"] {
                let _ = cache.find_command(cmd);
            }
        });
    }
}