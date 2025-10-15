// src/job_control.rs
use crate::shell::{Shell, JobStatus};
use nix::sys::signal::{kill, Signal};
use nix::sys::wait::{waitpid, WaitStatus};
use nix::unistd::Pid;

pub fn fg_command(shell: &mut Shell, args: &[String]) -> i32 {
    let job_id = if args.is_empty() {
        // Get the most recent job
        shell.jobs.iter().rposition(|j| matches!(j.status, JobStatus::Running | JobStatus::Stopped))
    } else {
        // Parse job ID
        args[0].parse::<usize>().ok().and_then(|id| {
            if id > 0 && id <= shell.jobs.len() {
                Some(id - 1)
            } else {
                None
            }
        })
    };
    
    match job_id {
        Some(idx) => {
            let job = &mut shell.jobs[idx];
            let pid = job.pid;
            
            println!("{}", job.command);
            
            // Send SIGCONT if job was stopped
            if matches!(job.status, JobStatus::Stopped) {
                kill(Pid::from_raw(pid), Signal::SIGCONT).ok();
            }
            
            // Wait for the job
            match waitpid(Pid::from_raw(pid), None) {
                Ok(WaitStatus::Exited(_, code)) => {
                    job.status = JobStatus::Done;
                    code as i32
                }
                Ok(WaitStatus::Signaled(_, sig, _)) => {
                    job.status = JobStatus::Done;
                    128 + sig as i32
                }
                Ok(WaitStatus::Stopped(_, _)) => {
                    job.status = JobStatus::Stopped;
                    148  // 128 + SIGTSTP (20)
                }
                _ => 1,
            }
        }
        None => {
            eprintln!("fg: no such job");
            1
        }
    }
}

pub fn bg_command(shell: &mut Shell, args: &[String]) -> i32 {
    let job_id = if args.is_empty() {
        shell.jobs.iter().rposition(|j| matches!(j.status, JobStatus::Stopped))
    } else {
        args[0].parse::<usize>().ok().and_then(|id| {
            if id > 0 && id <= shell.jobs.len() {
                Some(id - 1)
            } else {
                None
            }
        })
    };
    
    match job_id {
        Some(idx) => {
            let job = &mut shell.jobs[idx];
            if matches!(job.status, JobStatus::Stopped) {
                kill(Pid::from_raw(job.pid), Signal::SIGCONT).ok();
                job.status = JobStatus::Running;
                println!("[{}] {} &", idx + 1, job.command);
                0
            } else {
                eprintln!("bg: job already running");
                1
            }
        }
        None => {
            eprintln!("bg: no such job");
            1
        }
    }
}

pub fn kill_job(shell: &mut Shell, args: &[String]) -> i32 {
    if args.is_empty() {
        eprintln!("kill: usage: kill [job_id|pid]");
        return 1;
    }
    
    let target = &args[0];
    
    if target.starts_with('%') {
        // Job ID
        if let Ok(job_id) = target[1..].parse::<usize>() {
            if job_id > 0 && job_id <= shell.jobs.len() {
                let pid = shell.jobs[job_id - 1].pid;
                match kill(Pid::from_raw(pid), Signal::SIGTERM) {
                    Ok(_) => {
                        println!("Sent SIGTERM to job {}", job_id);
                        0
                    }
                    Err(e) => {
                        eprintln!("kill: failed to kill job {}: {}", job_id, e);
                        1
                    }
                }
            } else {
                eprintln!("kill: %{}: no such job", job_id);
                1
            }
        } else {
            eprintln!("kill: invalid job id");
            1
        }
    } else {
        // PID
        if let Ok(pid) = target.parse::<i32>() {
            match kill(Pid::from_raw(pid), Signal::SIGTERM) {
                Ok(_) => {
                    println!("Sent SIGTERM to process {}", pid);
                    0
                }
                Err(e) => {
                    eprintln!("kill: failed to kill process {}: {}", pid, e);
                    1
                }
            }
        } else {
            eprintln!("kill: invalid process id");
            1
        }
    }
}