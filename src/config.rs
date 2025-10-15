// src/config.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::io::{self, Write};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub general: GeneralConfig,
    pub aliases: HashMap<String, String>,
    pub theme: Theme,
    pub keybindings: HashMap<String, String>,
    pub env_vars: HashMap<String, String>,
    pub plugins: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub history_size: usize,
    pub history_file: String,
    pub prompt_format: String,
    pub enable_colors: bool,
    pub enable_hints: bool,
    pub enable_completion: bool,
    pub auto_cd: bool,
    pub bell_style: BellStyle,
    pub edit_mode: EditMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BellStyle {
    None,
    Visible,
    Audible,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EditMode {
    Emacs,
    Vi,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub prompt_color: String,
    pub command_color: String,
    pub builtin_color: String,
    pub alias_color: String,
    pub string_color: String,
    pub variable_color: String,
    pub operator_color: String,
    pub flag_color: String,
    pub path_color: String,
    pub error_color: String,
    pub hint_color: String,
    pub reset_color: String,
    pub prompt_symbol: String,
    pub prompt_symbol_error: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            aliases: default_aliases(),
            theme: Theme::default(),
            keybindings: HashMap::new(),
            env_vars: HashMap::new(),
            plugins: Vec::new(),
        }
    }
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            history_size: 10000,
            history_file: "~/.rshell_history".to_string(),
            prompt_format: "{user}@{host}:{cwd}{symbol} ".to_string(),
            enable_colors: true,
            enable_hints: true,
            enable_completion: true,
            auto_cd: false,
            bell_style: BellStyle::None,
            edit_mode: EditMode::Emacs,
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            prompt_color: "\x1b[1;32m".to_string(),     // Bold green
            command_color: "\x1b[1;36m".to_string(),    // Bold cyan
            builtin_color: "\x1b[1;33m".to_string(),    // Bold yellow
            alias_color: "\x1b[1;35m".to_string(),      // Bold magenta
            string_color: "\x1b[0;32m".to_string(),     // Green
            variable_color: "\x1b[0;33m".to_string(),   // Yellow
            operator_color: "\x1b[1;31m".to_string(),   // Bold red
            flag_color: "\x1b[0;36m".to_string(),       // Cyan
            path_color: "\x1b[0;34m".to_string(),       // Blue
            error_color: "\x1b[1;31m".to_string(),      // Bold red
            hint_color: "\x1b[0;90m".to_string(),       // Dark gray
            reset_color: "\x1b[0m".to_string(),         // Reset
            prompt_symbol: "$".to_string(),
            prompt_symbol_error: "$".to_string(),
        }
    }
}

impl Theme {
    pub fn colorize_prompt(&self, prompt: &str) -> String {
        format!("{}{}{}", self.prompt_color, prompt, self.reset_color)
    }
    
    pub fn ocean() -> Self {
        Self {
            name: "ocean".to_string(),
            prompt_color: "\x1b[38;5;39m".to_string(),   // Light blue
            command_color: "\x1b[38;5;51m".to_string(),  // Cyan
            builtin_color: "\x1b[38;5;45m".to_string(),  // Turquoise
            alias_color: "\x1b[38;5;141m".to_string(),   // Purple
            string_color: "\x1b[38;5;48m".to_string(),   // Green
            variable_color: "\x1b[38;5;220m".to_string(), // Gold
            operator_color: "\x1b[38;5;197m".to_string(), // Pink
            flag_color: "\x1b[38;5;87m".to_string(),     // Light cyan
            path_color: "\x1b[38;5;33m".to_string(),     // Blue
            error_color: "\x1b[38;5;196m".to_string(),   // Red
            hint_color: "\x1b[38;5;242m".to_string(),    // Gray
            reset_color: "\x1b[0m".to_string(),
            prompt_symbol: "🌊".to_string(),
            prompt_symbol_error: "💀".to_string(),
        }
    }
    
    pub fn forest() -> Self {
        Self {
            name: "forest".to_string(),
            prompt_color: "\x1b[38;5;34m".to_string(),   // Green
            command_color: "\x1b[38;5;76m".to_string(),  // Light green
            builtin_color: "\x1b[38;5;142m".to_string(), // Brown
            alias_color: "\x1b[38;5;178m".to_string(),   // Gold
            string_color: "\x1b[38;5;70m".to_string(),   // Dark green
            variable_color: "\x1b[38;5;220m".to_string(), // Yellow
            operator_color: "\x1b[38;5;166m".to_string(), // Orange
            flag_color: "\x1b[38;5;115m".to_string(),    // Mint
            path_color: "\x1b[38;5;29m".to_string(),     // Forest green
            error_color: "\x1b[38;5;124m".to_string(),   // Dark red
            hint_color: "\x1b[38;5;242m".to_string(),    // Gray
            reset_color: "\x1b[0m".to_string(),
            prompt_symbol: "🌲".to_string(),
            prompt_symbol_error: "🔥".to_string(),
        }
    }
    
    pub fn dracula() -> Self {
        Self {
            name: "dracula".to_string(),
            prompt_color: "\x1b[38;5;141m".to_string(),  // Purple
            command_color: "\x1b[38;5;117m".to_string(), // Light blue
            builtin_color: "\x1b[38;5;215m".to_string(), // Orange
            alias_color: "\x1b[38;5;212m".to_string(),   // Pink
            string_color: "\x1b[38;5;228m".to_string(),  // Yellow
            variable_color: "\x1b[38;5;117m".to_string(), // Cyan
            operator_color: "\x1b[38;5;197m".to_string(), // Red
            flag_color: "\x1b[38;5;159m".to_string(),    // Light purple
            path_color: "\x1b[38;5;111m".to_string(),    // Blue
            error_color: "\x1b[38;5;196m".to_string(),   // Bright red
            hint_color: "\x1b[38;5;59m".to_string(),     // Dark gray
            reset_color: "\x1b[0m".to_string(),
            prompt_symbol: "🦇".to_string(),
            prompt_symbol_error: "💉".to_string(),
        }
    }
}

fn default_aliases() -> HashMap<String, String> {
    let mut aliases = HashMap::new();
    aliases.insert("ll".to_string(), "ls -l".to_string());
    aliases.insert("la".to_string(), "ls -la".to_string());
    aliases.insert("..".to_string(), "cd ..".to_string());
    aliases.insert("...".to_string(), "cd ../..".to_string());
    aliases.insert("gs".to_string(), "git status".to_string());
    aliases.insert("gc".to_string(), "git commit".to_string());
    aliases.insert("gp".to_string(), "git push".to_string());
    aliases
}

impl Config {
    pub fn load() -> io::Result<Self> {
        let config_path = Self::config_path()?;
        
        if config_path.exists() {
            let contents = fs::read_to_string(&config_path)?;
            toml::from_str(&contents)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
        } else {
            let config = Self::default();
            config.save()?;
            Ok(config)
        }
    }
    
    pub fn save(&self) -> io::Result<()> {
        let config_path = Self::config_path()?;
        
        // Create config directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        let contents = toml::to_string_pretty(self)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        fs::write(&config_path, contents)?;
        
        Ok(())
    }
    
    fn config_path() -> io::Result<PathBuf> {
        let home = dirs::home_dir()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Home directory not found"))?;
        Ok(home.join(".config").join("rshell").join("config.toml"))
    }
    
    pub fn get_theme_by_name(&self, name: &str) -> Theme {
        match name {
            "default" => Theme::default(),
            "ocean" => Theme::ocean(),
            "forest" => Theme::forest(),
            "dracula" => Theme::dracula(),
            _ => self.theme.clone(),
        }
    }
}

pub fn init_config_interactive() -> io::Result<Config> {
    println!("Welcome to RShell Configuration Setup!");
    println!("======================================");
    
    let mut config = Config::default();
    
    // Choose theme
    println!("\nAvailable themes:");
    println!("  1. default - Classic terminal colors");
    println!("  2. ocean   - Blue and cyan theme");
    println!("  3. forest  - Green nature theme");
    println!("  4. dracula - Dark purple theme");
    
    print!("Choose a theme (1-4) [1]: ");
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    
    config.theme = match input.trim() {
        "2" => Theme::ocean(),
        "3" => Theme::forest(),
        "4" => Theme::dracula(),
        _ => Theme::default(),
    };
    
    // Configure general settings
    print!("\nEnable command hints? (y/n) [y]: ");
    io::stdout().flush()?;
    input.clear();
    io::stdin().read_line(&mut input)?;
    config.general.enable_hints = !matches!(input.trim().to_lowercase().as_str(), "n" | "no");
    
    print!("Enable auto-completion? (y/n) [y]: ");
    io::stdout().flush()?;
    input.clear();
    io::stdin().read_line(&mut input)?;
    config.general.enable_completion = !matches!(input.trim().to_lowercase().as_str(), "n" | "no");
    
    print!("Enable auto-cd (type directory name to cd)? (y/n) [n]: ");
    io::stdout().flush()?;
    input.clear();
    io::stdin().read_line(&mut input)?;
    config.general.auto_cd = matches!(input.trim().to_lowercase().as_str(), "y" | "yes");
    
    // Save configuration
    config.save()?;
    
    println!("\nConfiguration saved to ~/.config/rshell/config.toml");
    println!("You can edit this file manually to further customize your shell.");
    
    Ok(config)
}