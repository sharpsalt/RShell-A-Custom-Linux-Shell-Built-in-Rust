// main.rs
mod shell;
mod command;
mod utils;
mod signal_handler;
mod job_control;
mod line_editor;
mod config;
mod alias;

use rustyline::{Editor, Config as RustylineConfig, EditMode, CompletionType};
use shell::{Shell, parser::Parser, executor::Executor};
use line_editor::ShellHelper;
use config::Config;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load or create configuration
    let config = match Config::load() {
        Ok(c) => c,
        Err(_) => {
            println!("No configuration found. Let's set one up!");
            config::init_config_interactive()?
        }
    };
    
    // Setup signal handlers
    signal_handler::setup_signal_handlers()?;
    
    // Initialize shell with config
    let mut shell = Shell::new();
    shell.config = config.clone();
    shell.aliases = config.aliases.clone();
    
    // Create line editor with custom helper
    let rustyline_config = RustylineConfig::builder()
        .history_ignore_space(true)
        .completion_type(CompletionType::List)
        .edit_mode(match config.general.edit_mode {
            config::EditMode::Vi => EditMode::Vi,
            config::EditMode::Emacs => EditMode::Emacs,
        })
        .build();
    
    let helper = ShellHelper::new(config.theme.clone(), shell.aliases.clone());
    let mut rl = Editor::with_config(rustyline_config)?;
    rl.set_helper(Some(helper));
    
    // Load history
    let history_file = config.general.history_file.replace('~', &dirs::home_dir()
        .unwrap_or_default()
        .to_string_lossy());
    let _ = rl.load_history(&history_file);
    
    // Create executor
    let mut executor = Executor::new();
    
    // Print welcome message with theme
    println!("{}Welcome to RShell v0.1.0{}", 
             shell.config.theme.prompt_color, 
             shell.config.theme.reset_color);
    println!("Theme: {} | Type 'help' for commands", shell.config.theme.name);
    println!();
    
    loop {
        // Check for completed background jobs
        executor.check_background_jobs(&mut shell);
        
        // Check for signals
        let (sigint, sigtstp) = signal_handler::check_signals();
        if sigint {
            println!("^C");
            continue;
        }
        if sigtstp {
            println!("^Z");
            continue;
        }
        
        // Update prompt
        let prompt = shell.format_prompt();
        if let Some(helper) = rl.helper_mut() {
            helper.set_prompt(&prompt);
            helper.update_aliases(shell.aliases.clone());
        }
        
        // Read line
        let readline = rl.readline(&prompt);
        
        match readline {
            Ok(line) => {
                if line.trim().is_empty() {
                    continue;
                }
                
                // Add to history
                rl.add_history_entry(&line);
                shell.history.push(line.clone());
                
                // Expand aliases
                let expanded = alias::expand_aliases(&line, &shell.aliases);
                
                // Check for auto-cd
                if shell.config.general.auto_cd {
                    let trimmed = expanded.trim();
                    if std::path::Path::new(trimmed).is_dir() {
                        if let Err(e) = std::env::set_current_dir(trimmed) {
                            eprintln!("cd: {}: {}", trimmed, e);
                        } else {
                            shell.current_dir = std::env::current_dir()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .to_string();
                        }
                        continue;
                    }
                }
                
                // Parse and execute
                match Parser::new(&expanded) {
                    Ok(mut parser) => {
                        match parser.parse() {
                            Ok(command_type) => {
                                let exit_code = executor.execute(&mut shell, command_type);
                                shell.last_exit_code = exit_code;
                            }
                            Err(e) => {
                                eprintln!("{}Parse error: {}{}", 
                                    shell.config.theme.error_color,
                                    e.message,
                                    shell.config.theme.reset_color);
                                shell.last_exit_code = 1;
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("{}Error: {}{}", 
                            shell.config.theme.error_color,
                            e.message,
                            shell.config.theme.reset_color);
                        shell.last_exit_code = 1;
                    }
                }
            }
            Err(rustyline::error::ReadlineError::Interrupted) => {
                println!("^C");
                continue;
            }
            Err(rustyline::error::ReadlineError::Eof) => {
                println!("exit");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    
    // Save history
    let _ = rl.save_history(&history_file);
    
    // Save config with updated aliases
    shell.config.aliases = shell.aliases.clone();
    let _ = shell.save_config();
    
    Ok(())
}