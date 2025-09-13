# RShell 🐚

A modern, feature-rich shell implementation written in Rust, designed as a learning project showcasing systems programming concepts and Rust's capabilities.

![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)
![Platform](https://img.shields.io/badge/platform-linux%20%7C%20macos-lightgrey?style=for-the-badge)

## Features

### Core Functionality
- **Interactive Command Line**: Full-featured REPL with command history
- **Built-in Commands**: Essential shell builtins (`cd`, `pwd`, `echo`, `export`, `history`, etc.)
- **External Command Execution**: Run any system command with proper PATH resolution
- **Environment Variable Management**: Full support for environment variables and expansion
- **Command History**: Persistent command history with search capabilities

### Advanced Features
- **Pipes and Redirection**: Full I/O redirection support (`>`, `<`, `>>`, `|`)
- **Background Jobs**: Job control with background process management (`&`)
- **Command Chaining**: Logical operators (`&&`, `||`, `;`)
- **Signal Handling**: Proper handling of Ctrl+C, Ctrl+Z, and other signals
- **Tab Completion**: Intelligent completion for commands and filenames
- **Process Management**: Advanced process control and monitoring

### User Experience
- **Colored Output**: Syntax highlighting and themed interface
- **Smart Parsing**: Handles quotes, escapes, and complex command structures  
- **Error Handling**: Comprehensive error reporting and recovery
- **Configuration**: Customizable settings and aliases
- **Cross-platform**: Works on Linux and macOS

## Usecase
<img width="1920" height="1080" alt="Screenshot from 2025-09-13 19-26-15" src="https://github.com/user-attachments/assets/61dbeada-849e-40ff-ab15-861ca1177fd5" />
<img width="857" height="369" alt="Screenshot from 2025-09-13 19-27-33" src="https://github.com/user-attachments/assets/9ba6ed7d-9487-499a-a6bb-ca4cb2ff01ed" />



## Table of Contents

- [Installation](#installation)
- [Quick Start](#quick-start)
- [Usage Examples](#usage-examples)
- [Built-in Commands](#built-in-commands)
- [Advanced Features](#advanced-features)
- [Configuration](#configuration)
- [Development](#development)
- [Architecture](#architecture)
- [Contributing](#contributing)
- [License](#license)

## Installation

### Prerequisites

- **Rust** (1.70.0 or later) - [Install Rust](https://rustup.rs/)
- **Git** - For cloning the repository
- **Linux/macOS** - Currently supports Unix-like systems

### Build from Source

```bash
# Install Rust in your system first
sudo apt update
<img width="910" height="26" alt="Screenshot from 2025-09-13 19-38-00" src="https://github.com/user-attachments/assets/6f3bc74c-bddc-4e03-82b8-37e5500cb8b5" />
sudo apt install cargo rustc
<img width="910" height="26" alt="Screenshot from 2025-09-13 19-37-36" src="https://github.com/user-attachments/assets/137d5815-6281-4dfa-b562-7f5aca1ea95f" />

# Clone the repository
git clone https://github.com/sharpsalt/RShell-A-Custom-Linux-Shell-Built-in-Rust.git
cd rust_shell

# Build the project
cargo build --release

# Run the shell
cargo run --release
```

### Development Build

```bash
# For development with debug info
cargo build
cargo run

# Run tests
cargo test

# Check code without building
cargo check
```

## Quick Start

1. **Launch RShell**:
   ```bash
   cargo run
   ```

2. **Try basic commands**:
   ```bash
   rshell:/home/user$ pwd
   /home/user
   
   rshell:/home/user$ echo "Hello, World!"
   Hello, World!
   
   rshell:/home/user$ ls | grep .txt
   document.txt
   readme.txt
   ```

3. **Exit the shell**:
   ```bash
   rshell:/home/user$ exit
   ```

## Usage Examples

### Basic Commands
```bash
# Navigation
cd /tmp
pwd
cd ~

# File operations
ls -la
cat file.txt
touch newfile.txt
```

### Pipes and Redirection
```bash
# Pipe commands
ls | grep .rs | wc -l

# Redirect output
echo "Hello" > output.txt
cat file1.txt file2.txt > combined.txt
ls -la >> log.txt

# Redirect input
sort < unsorted.txt
```

### Background Jobs
```bash
# Run in background
sleep 100 &

# List jobs
jobs

# Job control (when implemented)
fg %1
bg %1
```

### Environment Variables
```bash
# Set variables
export MY_VAR="Hello World"
echo $MY_VAR

# Use in commands
export PATH=$PATH:/new/path
```

### Command Chaining
```bash
# Conditional execution
mkdir test_dir && cd test_dir && touch file.txt

# Alternative execution
ls non_existent || echo "Directory not found"

# Sequential execution
cd /tmp; ls; pwd
```

##  Built-in Commands

| Command | Description | Usage |
|---------|-------------|-------|
| `cd` | Change directory | `cd [directory]` |
| `pwd` | Print working directory | `pwd` |
| `echo` | Display text | `echo [text...]` |
| `export` | Set environment variable | `export VAR=value` |
| `unset` | Remove environment variable | `unset VAR` |
| `history` | Show command history | `history` |
| `jobs` | List active jobs | `jobs` |
| `exit` | Exit the shell | `exit [code]` |
| `help` | Display help information | `help` |

##  Advanced Features

### Command History
- **Persistent History**: Commands are saved across sessions
- **History Search**: Use `history` to view past commands
- **History Size**: Configurable history limit (default: 1000)

### Process Management
- **Background Jobs**: Run commands with `&` suffix
- **Job Control**: Monitor and control background processes
- **Signal Handling**: Graceful handling of interrupts

### Smart Parsing
```bash
# Quoted arguments
echo "Hello World" 'Single quotes'

# Escaped characters
echo "Path with spaces: /my\ path"

# Variable expansion
echo "Home directory: $HOME"
echo "Current user: ${USER}"
```

### Error Handling
```bash
# Command not found
rshell$ nonexistent_command
rshell: nonexistent_command: command not found

# Invalid syntax
rshell$ ls |
rshell: parse error: expected command after pipe
```

##  Configuration

RShell supports configuration through a TOML file:

**Location**: `~/.config/rshell/config.toml`

```toml
# Example configuration
[shell]
history_size = 2000
prompt_format = "{user}@{host}:{pwd}$ "
auto_cd = true

[aliases]
ll = "ls -la"
la = "ls -la"
grep = "grep --color=auto"

[environment]
EDITOR = "vim"
PAGER = "less"
```

##  Development

### Project Structure

```
rust_shell/
├── src/
│   ├── main.rs              # Main entry point
│   ├── lib.rs               # Library root
│   ├── shell/               # Core shell logic
│   │   ├── mod.rs
│   │   ├── parser.rs        # Command parsing
│   │   ├── executor.rs      # Command execution
│   │   └── builtins.rs      # Built-in commands
│   ├── command/             # Command structures
│   │   ├── mod.rs
│   │   └── command.rs
│   └── utils/               # Utility functions
│       ├── mod.rs
│       └── helpers.rs
├── tests/                   # Test files
├── docs/                    # Documentation
└── examples/                # Example scripts
```

### Key Components

1. **Parser** (`src/shell/parser.rs`):
   - Tokenizes input into commands, arguments, and operators
   - Handles quotes, escapes, and special characters
   - Builds command AST for execution

2. **Executor** (`src/shell/executor.rs`):
   - Executes parsed commands
   - Manages process creation and monitoring
   - Handles pipes, redirections, and job control

3. **Built-ins** (`src/shell/builtins.rs`):
   - Implements shell built-in commands
   - Manages shell state and environment

### Testing

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Run integration tests
cargo test --test integration_tests
```

### Debugging

```bash
# Build with debug symbols
cargo build

# Run with debug output
RUST_LOG=debug cargo run

# Use with debugger
rust-gdb target/debug/rshell
```

##  Architecture

### Design Principles

1. **Modular Design**: Clear separation between parsing, execution, and I/O
2. **Memory Safety**: Leverages Rust's ownership system for safe systems programming
3. **Error Handling**: Comprehensive error types and propagation
4. **Extensibility**: Plugin-friendly architecture for adding features

### Data Flow

```
Input → Tokenizer → Parser → AST → Executor → Output
  ↓                                    ↓
History                           Process Management
```

### Key Data Structures

- **`Command`**: Represents a single command with arguments and redirections
- **`CommandType`**: Enum for different command types (simple, pipeline, chain)
- **`Shell`**: Main shell state including environment and history
- **`Token`**: Lexical tokens from input parsing

## Roadmap

### Phase 1: Core Features ✅
- [x] Basic command execution
- [x] Built-in commands
- [x] Environment variables
- [x] Command history
- [x] Basic parsing

### Phase 2: Advanced Shell Features 🚧
- [ ] Pipes and redirection
- [ ] Background jobs
- [ ] Signal handling
- [ ] Command substitution
- [ ] Glob expansion

### Phase 3: User Experience 📋
- [ ] Tab completion
- [ ] Syntax highlighting
- [ ] Configuration system
- [ ] Aliases
- [ ] Themes

### Phase 4: Performance 📋
- [ ] Command caching
- [ ] Async I/O
- [ ] Memory optimization
- [ ] Parallel execution

##  Contributing

I welcome contributions! Here's how to get started:

### Development Setup

```bash
# Fork and clone the repo
git clone https://github.com/sharpsalt/RShell-A-Custom-Linux-Shell-Built-in-Rust.git
cd rust_shell

# Create a feature branch
git checkout -b feature/your-feature-name

# Make changes and test
cargo test
cargo fmt
cargo clippy

# Commit and push
git commit -am "Add your feature"
git push origin feature/your-feature-name
```

### Guidelines

- **Code Style**: Follow Rust conventions and run `cargo fmt`
- **Testing**: Add tests for new features
- **Documentation**: Update docs for public APIs
- **Commit Messages**: Use clear, descriptive commit messages

### Areas for Contribution

-  **Bug fixes**: Check the issues for reported bugs
-  **New features**: Implement items from the roadmap
-  **Documentation**: Improve docs and examples
-  **Testing**: Add more comprehensive tests
-  **Performance**: Optimize existing features

##  License

This project is licensed under the Apache License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- **Rust Community**: For excellent documentation and libraries
- **Unix Shell Design**: Inspiration from bash, zsh, and other shells
- **Educational Resources**: Various systems programming tutorials and books

##  Support

- **Email**: Contact to me for contributions or any related bugs at [srijanv0@gmail.com]

---

## Project Stats

- **Language**: Rust
- **Dependencies**: Minimal (rustyline,nix,dirs)
- **Platform Support**: Linux, macOS

---

<div align="center">
**⭐ Star this repo if you found it helpful!**

</div>

---

*Built with lots of Efforts, efforts matters...*
