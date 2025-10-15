// src/memory_pool.rs
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;

pub struct StringPool {
    pool: Arc<Mutex<VecDeque<String>>>,
    max_size: usize,
    string_capacity: usize,
}

impl StringPool {
    pub fn new(max_size: usize, string_capacity: usize) -> Self {
        let mut pool = VecDeque::with_capacity(max_size);
        
        // Pre-allocate strings
        for _ in 0..max_size / 2 {
            pool.push_back(String::with_capacity(string_capacity));
        }
        
        Self {
            pool: Arc::new(Mutex::new(pool)),
            max_size,
            string_capacity,
        }
    }
    
    pub fn acquire(&self) -> String {
        let mut pool = self.pool.lock().unwrap();
        pool.pop_front().unwrap_or_else(|| String::with_capacity(self.string_capacity))
    }
    
    pub fn release(&self, mut s: String) {
        s.clear();
        
        let mut pool = self.pool.lock().unwrap();
        if pool.len() < self.max_size && s.capacity() <= self.string_capacity * 2 {
            pool.push_back(s);
        }
    }
    
    pub fn with_string<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut String) -> R,
    {
        let mut s = self.acquire();
        let result = f(&mut s);
        self.release(s);
        result
    }
}

pub struct VecPool<T> {
    pool: Arc<Mutex<VecDeque<Vec<T>>>>,
    max_size: usize,
    vec_capacity: usize,
}

impl<T> VecPool<T> {
    pub fn new(max_size: usize, vec_capacity: usize) -> Self {
        let mut pool = VecDeque::with_capacity(max_size);
        
        // Pre-allocate vectors
        for _ in 0..max_size / 2 {
            pool.push_back(Vec::with_capacity(vec_capacity));
        }
        
        Self {
            pool: Arc::new(Mutex::new(pool)),
            max_size,
            vec_capacity,
        }
    }
    
    pub fn acquire(&self) -> Vec<T> {
        let mut pool = self.pool.lock().unwrap();
        pool.pop_front().unwrap_or_else(|| Vec::with_capacity(self.vec_capacity))
    }
    
    pub fn release(&self, mut v: Vec<T>) {
        v.clear();
        
        let mut pool = self.pool.lock().unwrap();
        if pool.len() < self.max_size && v.capacity() <= self.vec_capacity * 2 {
            pool.push_back(v);
        }
    }
}

// Memory-efficient command buffer
pub struct CommandBuffer {
    buffer: Vec<u8>,
    position: usize,
    capacity: usize,
}

impl CommandBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(capacity),
            position: 0,
            capacity,
        }
    }
    
    pub fn write(&mut self, data: &[u8]) -> Result<(), &'static str> {
        let required = self.position + data.len();
        if required > self.capacity {
            return Err("Buffer overflow");
        }
        
        if required > self.buffer.len() {
            self.buffer.resize(required, 0);
        }
        
        self.buffer[self.position..required].copy_from_slice(data);
        self.position = required;
        Ok(())
    }
    
    pub fn read(&self) -> &[u8] {
        &self.buffer[..self.position]
    }
    
    pub fn clear(&mut self) {
        self.position = 0;
    }
    
    pub fn as_str(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(self.read())
    }
}

// Optimized string interning for frequently used strings
pub struct StringInterner {
    interned: Arc<Mutex<HashMap<String, Arc<str>>>>,
}

impl StringInterner {
    pub fn new() -> Self {
        Self {
            interned: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    pub fn intern(&self, s: &str) -> Arc<str> {
        let mut interned = self.interned.lock().unwrap();
        
        if let Some(existing) = interned.get(s) {
            existing.clone()
        } else {
            let arc: Arc<str> = Arc::from(s);
            interned.insert(s.to_string(), arc.clone());
            arc
        }
    }
    
    pub fn clear(&self) {
        self.interned.lock().unwrap().clear();
    }
}

// Zero-copy parsing utilities
pub struct ZeroCopyParser<'a> {
    input: &'a str,
    position: usize,
}

impl<'a> ZeroCopyParser<'a> {
    pub fn new(input: &'a str) -> Self {
        Self { input, position: 0 }
    }
    
    pub fn peek(&self) -> Option<char> {
        self.input[self.position..].chars().next()
    }
    
    pub fn advance(&mut self, n: usize) {
        self.position = (self.position + n).min(self.input.len());
    }
    
    pub fn consume_while<F>(&mut self, predicate: F) -> &'a str
    where
        F: Fn(char) -> bool,
    {
        let start = self.position;
        while let Some(ch) = self.peek() {
            if predicate(ch) {
                self.advance(ch.len_utf8());
            } else {
                break;
            }
        }
        &self.input[start..self.position]
    }
    
    pub fn remaining(&self) -> &'a str {
        &self.input[self.position..]
    }
}

use std::collections::HashMap;

// Memory usage tracker
pub struct MemoryTracker {
    allocations: Arc<Mutex<HashMap<String, usize>>>,
}

impl MemoryTracker {
    pub fn new() -> Self {
        Self {
            allocations: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    pub fn track(&self, name: &str, size: usize) {
        let mut allocations = self.allocations.lock().unwrap();
        *allocations.entry(name.to_string()).or_insert(0) += size;
    }
    
    pub fn untrack(&self, name: &str, size: usize) {
        let mut allocations = self.allocations.lock().unwrap();
        if let Some(current) = allocations.get_mut(name) {
            *current = current.saturating_sub(size);
        }
    }
    
    pub fn report(&self) -> Vec<(String, usize)> {
        let allocations = self.allocations.lock().unwrap();
        let mut report: Vec<_> = allocations.iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        report.sort_by_key(|(_, size)| std::cmp::Reverse(*size));
        report
    }
    
    pub fn total_allocated(&self) -> usize {
        self.allocations.lock().unwrap().values().sum()
    }
}