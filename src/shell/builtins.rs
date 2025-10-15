// src/shell/builtins.rs
use crate::shell::Shell;
use crate::config::Config;
use std::env;

pub fn builtin_theme(shell: &mut Shell, args: &[String]) -> i32 {
    if args.is_empty() {
        println!("Current theme: {}", shell.config.theme.name);
        println!("\nAvailable themes:");
        println!("  default - Classic terminal colors");
        println!("  ocean   - Blue and cyan theme");
        println!("  forest  - Green nature theme");
        println!("  dracula - Dark purple theme");
        return 0;
    }
    
    match args[0].as_str() {
        "list" => {
            println!("Available themes:");
            println!("  default - Classic terminal colors");
            println!("  ocean   - Blue and cyan theme 🌊");
            println!("  forest  - Green nature theme 🌲");
            println!("  dracula - Dark purple theme 🦇");
        }
        "set" => {
            if args.len() < 2 {
                eprintln!("Usage: theme set <theme_name>");
                return 1;
            }
            
            shell.config.theme = shell.config.get_theme_by_name(&args[1]);
            shell.config.theme.name = args[1].clone();
            
            if let Err(e) = shell.save_config() {
                eprintln!("Failed to save theme: {}", e);
                return 1;
            }
            
            println!("Theme changed to: {}", args[1]);
        }
        "preview" => {
            if args.len() < 2 {
                eprintln!("Usage: theme preview <theme_name>");
                return 1;
            }
            
            let theme = shell.config.get_theme_by_name(&args[1]);
            println!("Preview of '{}' theme:", args[1]);
            println!("{}Prompt color{}", theme.prompt_color, theme.reset_color);
            println!("{}Command color{}", theme.command_color, theme.reset_color);
            println!("{}Builtin color{}", theme.builtin_color, theme.reset_color);
            println!("{}String color{}", theme.string_color, theme.reset_color);
            println!("{}Variable color{}", theme.variable_color, theme.reset_color);
            println!("{}Path color{}", theme.path_color, theme.reset_color);
            println!("{}Error color{}", theme.error_color, theme.reset_color);
        }
        _ => {
            eprintln!("Unknown theme command: {}", args[0]);
            return 1;
        }
    }
    
    0
}

pub fn builtin_config(shell: &mut Shell, args: &[String]) -> i32 {
    if args.is_empty() {
        println!("RShell Configuration");
        println!("====================");
        println!("Theme: {}", shell.config.theme.name);
        println!("History size: {}", shell.config.general.history_size);
        println!("Enable hints: {}", shell.config.general.enable_hints);
        println!("Enable completion: {}", shell.config.general.enable_completion);
        println!("Auto-cd: {}", shell.config.general.auto_cd);
        println!("\nConfig file: ~/.config/rshell/config.toml");
        return 0;
    }
    
    match args[0].as_str() {
        "reload" => {
            match Config::load() {
                Ok(config) => {
                    shell.config = config;
                    shell.aliases = shell.config.aliases.clone();
                    println!("Configuration reloaded");
                    0
                }
                Err(e) => {
                    eprintln!("Failed to reload config: {}", e);
                    1
                }
            }
        }
        "edit" => {
            let editor = env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());
            let config_path = dirs::home_dir()
                .map(|h| h.join(".config/rshell/config.toml"))
                .unwrap();
            
            std::process::Command::new(editor)
                .arg(config_path)
                .status()
                .map(|s| s.code().unwrap_or(1))
                .unwrap_or(1)
        }
        "init" => {
            match crate::config::init_config_interactive() {
                Ok(config) => {
                    shell.config = config;
                    shell.aliases = shell.config.aliases.clone();
                    0
                }
                Err(e) => {
                    eprintln!("Failed to initialize config: {}", e);
                    1
                }
            }
        }
        _ => {
            eprintln!("Unknown config command: {}", args[0]);
            1
        }
    }
}

pub fn execute_builtin(shell: &mut Shell, program: &str, args: &[String]) -> Option<i32> {
    match program {
        "cd" => Some(builtin_cd(shell, args)),
        "pwd" => Some(builtin_pwd()),
        "echo" => Some(builtin_echo(args)),
        "export" => Some(builtin_export(shell, args)),
        "unset" => Some(builtin_unset(shell, args)),
        "exit" => Some(builtin_exit(args)),
        "history" => Some(builtin_history(shell)),
        "jobs" => Some(builtin_jobs(shell)),
        "help" => Some(builtin_help()),
        "fg" => Some(crate::job_control::fg_command(shell, args)),
        "bg" => Some(crate::job_control::bg_command(shell, args)),
        "kill" => Some(crate::job_control::kill_job(shell, args)),
        "alias" => Some(crate::alias::builtin_alias(shell, args)),
        "unalias" => Some(crate::alias::builtin_unalias(shell, args)),
        "theme" => Some(builtin_theme(shell, args)),
        "config" => Some(builtin_config(shell, args)),
        _ => None,
    }
}

fn builtin_cd(shell: &mut Shell, args: &[String]) -> i32 {
    let new_dir = if args.is_empty() {
        env::var("HOME").unwrap_or_else(|_| "/".to_string())
    } else {
        args[0].clone()
    };
    
    let path = if new_dir.starts_with('~') {
        new_dir.replacen('~', &env::var("HOME").unwrap_or_else(|_| "/".to_string()), 1)
    } else {
        new_dir
    };
    
    match env::set_current_dir(&path) {
        Ok(_) => {
            shell.current_dir = env::current_dir()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            0
        }
        Err(e) => {
            eprintln!("cd: {}: {}", path, e);
            1
        }
    }
}

fn builtin_pwd() -> i32 {
    match env::current_dir() {
        Ok(path) => {
            println!("{}", path.display());
            0
        }
        Err(e) => {
            eprintln!("pwd: {}", e);
            1
        }
    }
}

fn builtin_echo(args: &[String]) -> i32 {
    let output = args.join(" ");
    println!("{}", output);
    0
}

fn builtin_export(shell: &mut Shell, args: &[String]) -> i32 {
    if args.is_empty() {
        for (key, value) in &shell.environment {
            println!("{}={}", key, value);
        }
        return 0;
    }
    
    for arg in args {
        if let Some(eq_pos) = arg.find('=') {
            let key = arg[..eq_pos].to_string();
            let value = arg[eq_pos + 1..].to_string();
            shell.environment.insert(key.clone(), value.clone());
            env::set_var(key, value);
        } else {
            eprintln!("export: '{}': not a valid identifier", arg);
            return 1;
        }
    }
    0
}

fn builtin_unset(shell: &mut Shell, args: &[String]) -> i32 {
    for var in args {
        shell.environment.remove(var);
        env::remove_var(var);
    }
    0
}

fn builtin_exit(args: &[String]) -> i32 {
    let exit_code = if args.is_empty() {
        0
    } else {
        args[0].parse().unwrap_or(0)
    };
    std::process::exit(exit_code);
}

fn builtin_history(shell: &mut Shell) -> i32 {
    for (i, cmd) in shell.history.iter().enumerate() {
        println!("{:5} {}", i + 1, cmd);
    }
    0
}

fn builtin_jobs(shell: &mut Shell) -> i32 {
    if shell.jobs.is_empty() {
        println!("No active jobs");
    } else {
        for job in &shell.jobs {
            println!("[{}] {:?} {}", job.id, job.status, job.command);
        }
    }
    0
}

fn builtin_help() -> i32 {
    println!("RShell Built-in Commands:");
    println!("  cd [dir]         - Change directory");
    println!("  pwd              - Print working directory");
    println!("  echo [text]      - Display text");
    println!("  export VAR=val   - Set environment variable");
    println!("  unset VAR        - Unset environment variable");
    println!("  history          - Show command history");
    println!("  jobs             - List active jobs");
    println!("  fg [job]         - Bring job to foreground");
    println!("  bg [job]         - Resume job in background");
    println!("  kill <pid|%job>  - Kill a process or job");
    println!("  alias            - Set command alias");
    println!("  unalias          - Remove alias");
    println!("  theme            - Manage shell themes");
    println!("  config           - Manage configuration");
    println!("  help             - Show this help message");
    println!("  exit [code]      - Exit the shell");
    0
}

pub fn is_builtin(program: &str) -> bool {
    matches!(
        program,
        "cd" | "pwd" | "echo" | "export" | "unset" | "exit" | 
        "history" | "jobs" | "help" | "fg" | "bg" | "kill" |
        "alias" | "unalias" | "theme" | "config"
    )
}