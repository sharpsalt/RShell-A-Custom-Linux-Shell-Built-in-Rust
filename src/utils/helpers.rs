use std::collections::HashMap;
use std::fs::File;
use std::io::{self,Write,BufRead,BufReader};
use std::path::Path;

pub fn print_error(message:&str){
    eprintln!("rshell: {}",message);
}

pub fn print_warning(message:&str){
    eprintln!("rshell: warning: {}",message);
}

pub fn print_help() {
    println!("RShell - A custom shell implementation in Rust");
    println!();
    println!("BUILT-IN COMMANDS:");
    println!("  cd [dir]          Change directory");
    println!("  pwd               Print working directory");
    println!("  echo [args...]    Echo arguments");
    println!("  export VAR=value  Set environment variable");
    println!("  unset VAR         Unset environment variable");
    println!("  history           Show command history");
    println!("  jobs              Show active jobs");
    println!("  help              Show this help message");
    println!("  exit [code]       Exit the shell");
    println!();
    println!("FEATURES:");
    println!("  - Command execution");
    println!("  - I/O redirection (>, <, >>)");
    println!("  - Pipes (|)");
    println!("  - Background processes (&)");
    println!("  - Command chaining (&&, ||)");
    println!("  - Environment variables");
    println!("  - Command history");
    println!();
    println!("EXAMPLES:");
    println!("  ls -la");
    println!("  ls | grep .txt");
    println!("  echo \"Hello World\" > output.txt");
    println!("  cat file.txt | grep pattern | wc -l");
    println!("  export PATH=$PATH:/new/path");
    println!("  cd /tmp && ls -la");
}

pub fn load_config(config_path:&Path)->io::Result<HashMap<String,String>>{
    let mut config = HashMap::new();
    if !config_path.exists(){
        return Ok(config);
    }
    let file=File::open(config_path)?;
    let reader=BufReader::new(file);
    for line in reader.lines(){
        let line=line?;
        let lin =line.trim();
        if line.is_empty() || line.starts_with('#'){
            continue;
        }
        
        if let Some(eq_pos)=line.find('='){
            let key=line[..eq_pos].trim().to_string();
            let value=line[eq_pos + 1..].trim().to_string();
            config.insert(key, value);
        }
    }
    Ok(config)
}

pub fn save_config(config:&HashMap<String,String>,config_path:&Path)->io::Result<()>{
    let mut file=File::create(config_path)?;
    writeln!(file,"# RShell Configuration File")?;
    writeln!(file,"# Generated automatically")?;
    writeln!(file)?;
    for(key,value) in config{
        writeln!(file,"{}={}",key,value)?;
    }
    Ok(())
}

pub fn format_duration(duration:std::time::Duration)->String{
    let total_seconds=duration.as_secs();
    let hours=total_seconds/3600;
    let minutes=(total_seconds%3600)/60;
    let seconds=total_seconds%60;
    if hours>0{
        format!("{}h {}m {}s", hours,minutes,seconds)
    }else if minutes>0{
        format!("{}m {}s",minutes,seconds)
    }else{
        format!("{}s",seconds)
    }
}

pub fn is_valid_command_name(name: &str)->bool{
    !name.is_empty() && !name.contains('\0') && !name.contains('/') && name.chars().all(|c|c.is_alphanumeric() || c=='_' || c=='-')
}

pub fn sanitize_string(input:&str)->String{
    input.chars().filter(|&c|c!='\0'&&c.is_ascii()).collect()
}

pub fn signal_to_exit_code(signal:i32)->i32{
    128+signal
}

/// Parse an exit code from a string
pub fn parse_exit_code(s:&str)->Result<i32, String>{
    s.parse::<i32>()
        .map_err(|_|format!("Invalid exit code: {}",s))
        .and_then(|code|{
            if(0..=255).contains(&code){
                Ok(code)
            }else{
                Err(format!("Exit code out of range (0-255): {}",code))
            }
        })
}

pub fn get_default_prompt(current_dir:&str,username:&str,hostname:&str)->String{
    format!("{}@{}:{}$ ",username,hostname,current_dir)
}

// Truncate a string to a maximum length, adding "..." if truncated
pub fn truncate_string(s:&str,max_len:usize)->String{
    if s.len()<=max_len{
        s.to_string()
    }else{
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

pub fn is_interactive()->bool{
    unsafe { libc::isatty(libc::STDIN_FILENO) == 1 }
    //will return 1 if the shell is running in interactivaemode
}

pub fn get_system_info()->HashMap<String,String>{
    let mut info=HashMap::new();
    if let Ok(username)=std::env::var("USER"){
        info.insert("username".to_string(),username);
    }
    if let Ok(hostname)=std::env::var("HOSTNAME"){
        info.insert("hostname".to_string(),hostname);
    }
    info.insert("shell".to_string(),"rshell".to_string());
    info.insert("version".to_string(),env!("CARGO_PKG_VERSION").to_string());
    info
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_is_valid_command_name() {
        assert!(is_valid_command_name("ls"));
        assert!(is_valid_command_name("grep"));
        assert!(is_valid_command_name("my_command"));
        assert!(is_valid_command_name("test-cmd"));
        assert!(!is_valid_command_name(""));
        assert!(!is_valid_command_name("cmd/with/slash"));
        assert!(!is_valid_command_name("cmd\0with\0null"));
    }
    
    #[test]
    fn test_parse_exit_code() {
        assert_eq!(parse_exit_code("0"), Ok(0));
        assert_eq!(parse_exit_code("255"), Ok(255));
        assert!(parse_exit_code("256").is_err());
        assert!(parse_exit_code("-1").is_err());
        assert!(parse_exit_code("abc").is_err());
    }
    
    #[test]
    fn test_truncate_string(){
        assert_eq!(truncate_string("hello", 10),"hello");
        assert_eq!(truncate_string("hello world",8),"hello...");
        assert_eq!(truncate_string("hi",5),"hi");
    }
}