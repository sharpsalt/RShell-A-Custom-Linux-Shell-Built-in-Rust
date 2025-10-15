use nix::sys::signal::{self, Signal, SigHandler, SigSet, SigAction, SaFlags};
use std::sync::atomic::{AtomicBool, Ordering};

static SIGINT_RECEIVED: AtomicBool = AtomicBool::new(false);
static SIGTSTP_RECEIVED: AtomicBool = AtomicBool::new(false);

extern "C" fn handle_sigint(_: i32) {
    SIGINT_RECEIVED.store(true, Ordering::SeqCst);
    println!();
}

extern "C" fn handle_sigtstp(_: i32) {
    SIGTSTP_RECEIVED.store(true, Ordering::SeqCst);
}

extern "C" fn handle_sigchld(_: i32) {
    // Child process status changed
    // We'll check this in the main loop
}

pub fn setup_signal_handlers() -> Result<(), nix::Error> {
    // SIGINT handler (Ctrl+C)
    let sig_action = SigAction::new(
        SigHandler::Handler(handle_sigint),
        SaFlags::empty(),
        SigSet::empty(),
    );
    unsafe {
        signal::sigaction(Signal::SIGINT, &sig_action)?;
    }
    
    // SIGTSTP handler (Ctrl+Z)
    let sig_action = SigAction::new(
        SigHandler::Handler(handle_sigtstp),
        SaFlags::empty(),
        SigSet::empty(),
    );
    unsafe {
        signal::sigaction(Signal::SIGTSTP, &sig_action)?;
    }
    
    // SIGCHLD handler (child process status change)
    let sig_action = SigAction::new(
        SigHandler::Handler(handle_sigchld),
        SaFlags::SA_RESTART | SaFlags::SA_NOCLDSTOP,
        SigSet::empty(),
    );
    unsafe {
        signal::sigaction(Signal::SIGCHLD, &sig_action)?;
    }
    
    Ok(())
}

pub fn check_signals() -> (bool, bool) {
    let sigint = SIGINT_RECEIVED.swap(false, Ordering::SeqCst);
    let sigtstp = SIGTSTP_RECEIVED.swap(false, Ordering::SeqCst);
    (sigint, sigtstp)
}

pub fn ignore_signals() -> Result<(), nix::Error> {
    unsafe {
        signal::signal(Signal::SIGINT, SigHandler::SigIgn)?;
        signal::signal(Signal::SIGTSTP, SigHandler::SigIgn)?;
    }
    Ok(())
}

pub fn restore_default_signals() -> Result<(), nix::Error> {
    unsafe {
        signal::signal(Signal::SIGINT, SigHandler::SigDfl)?;
        signal::signal(Signal::SIGTSTP, SigHandler::SigDfl)?;
    }
    Ok(())
}
