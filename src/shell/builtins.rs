use crate::shell::Shell;
use std::env;

pub fn execute_builtin(shell:&mut Shell,command:&str,args:&[String])->Option<i32>{
    match command{
        "cd"=>builtin_cd(shell,args),
        "pwd"=>builtin_pwd(shell),
        "echo"=>builtin_echo(args),
        "export"=>builtin_export(shell,args),
        "unset"=>builtin_unset(shell,args),
        "exit"=>builtin_exit(args),
        "history"=>builtin_history(shell),
        "jobs"=>builtin_jobs(shell),
        _=>None,
    }
}

fn builtin_cd(shell: &mut Shell,args: &[String])->Option<i32>{
    let path=if args.is_empty(){
        shell.environment.get("HOME").cloned().unwrap_or_else(|| "/".to_string())
    }else{
        args[0].clone()
    };
    
    match env::set_current_dir(&path){
        Ok(_)=>{
            shell.current_dir=env::current_dir().unwrap_or_default().to_string_lossy().to_string();
            Some(0)
        }
        Err(_)=>{
            eprintln!("cd: {}: No such file or directory",path);
            Some(1)
        }
    }
}

fn builtin_pwd(shell:&Shell)->Option<i32>{
    println!("{}",shell.current_dir);
    Some(0)
}

fn builtin_echo(args:&[String])->Option<i32>{
    println!("{}",args.join(" "));
    Some(0)
}

fn builtin_export(shell:&mut Shell,args:&[String])->Option<i32>{
    for arg in args{
        if let Some(eq_pos)=arg.find('='){
            let (key,value)=arg.split_at(eq_pos);
            let value=&value[1..];// skip the '=' character
            shell.environment.insert(key.to_string(),value.to_string());
            env::set_var(key,value);
        }else{
            // Export existing variable
            if let Some(value)=shell.environment.get(arg){
                env::set_var(arg,value);
            }
        }
    }
    Some(0)
}

fn builtin_unset(shell:&mut Shell,args:&[String])->Option<i32>{
    for arg in args{
        shell.environment.remove(arg);
        std::env::remove_var(arg);
    }
    Some(0)
}

fn builtin_exit(args:&[String])->Option<i32>{
    let exit_code=if args.is_empty(){
        0
    }else{
        args[0].parse::<i32>().unwrap_or(1)
    };
    println!("exit");
    std::process::exit(exit_code);
}

fn builtin_history(shell:&Shell)->Option<i32>{
    for (i, command) in shell.history.iter().enumerate(){
        println!("{:4} {}",i+1,command);
    }
    Some(0)
}

fn builtin_jobs(shell:&Shell)->Option<i32>{
    for job in &shell.jobs{
        println!("[{}] {:?} {}",job.id,job.status,job.command);
    }
    Some(0)
}