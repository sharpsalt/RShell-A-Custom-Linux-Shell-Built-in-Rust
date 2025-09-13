mod shell;
mod command;
mod utils;
use rustyline::{Editor,Result as RustylineResult};
use shell::{Shell,parser::Parser,executor::Executor};
fn main()->RustylineResult<()>{
    let mut shell=Shell::new();
    let mut executor=Executor::new();
    let mut rl=Editor::<()>::new()?;
    println!("Welcome to RShell v0.1.0");
    println!("Type 'exit' to quit.");
    loop{
        let prompt=format!("rshell:{}$ ",shell.current_dir);
        let readline=rl.readline(&prompt);
        match readline{
            Ok(line)=>{
                if line.trim().is_empty(){
                    continue;
                }
                rl.add_history_entry(&line);
                shell.history.push(line.clone());
                match Parser::new(&line){
                    Ok(mut parser)=>{
                        match parser.parse(){
                            Ok(command_type)=>{
                                let exit_code=executor.execute(&mut shell,command_type);
                                shell.last_exit_code=exit_code;
                            }
                            Err(e)=>{
                                eprintln!("Parse error: {:?}",e);
                                shell.last_exit_code = 1;
                            }
                        }
                    }
                    Err(e)=>{
                        eprintln!("Tokenize error: {:?}",e);
                        shell.last_exit_code=1;
                    }
                }
            }
            Err(rustyline::error::ReadlineError::Interrupted)=>{
                println!("^C");
                continue;
            }
            Err(rustyline::error::ReadlineError::Eof)=>{
                println!("exit");
                break;
            }
            Err(err)=>{
                println!("Error: {:?}",err);
                break;
            }
        }
    }
    Ok(())
}