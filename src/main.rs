use shogi_engine::usi::run_usi_loop;
use std::{
    any::Any,
    backtrace::Backtrace,
    panic::{self, UnwindSafe},
    process,
};

fn install_panic_hook() {
    panic::set_hook(Box::new(|info| {
        eprintln!("[engine panic] {}\n{}", info, Backtrace::force_capture());
    }));
}

fn run_with_panic_logging<F>(f: F)
where
    F: FnOnce() + UnwindSafe,
{
    if let Err(payload) = panic::catch_unwind(f) {
        let msg = format_panic_payload(&payload);
        eprintln!("[engine panic] unhandled panic payload: {}", msg);
        // Ensure non-zero exit so the caller knows the engine died.
        process::exit(101);
    }
}

fn format_panic_payload(payload: &Box<dyn Any + Send>) -> String {
    if let Some(s) = payload.downcast_ref::<&str>() {
        (*s).to_string()
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else {
        "unknown payload type".to_string()
    }
}

unsafe extern "C" fn signal_handler(
    signal: libc::c_int,
    _info: *mut libc::siginfo_t,
    _context: *mut libc::c_void,
) {
    let backtrace = Backtrace::force_capture();
    eprintln!(
        "[engine signal] received {}, capturing backtrace\n{}",
        signal, backtrace
    );
    libc::_exit(128 + signal);
}

fn install_signal_handlers() {
    unsafe {
        let mut action: libc::sigaction = std::mem::zeroed();
        action.sa_sigaction = signal_handler as usize;
        action.sa_flags = libc::SA_SIGINFO | libc::SA_ONSTACK | libc::SA_RESETHAND;
        libc::sigemptyset(&mut action.sa_mask);
        libc::sigaction(libc::SIGSEGV, &action, std::ptr::null_mut());
        libc::sigaction(libc::SIGABRT, &action, std::ptr::null_mut());
    }
}

fn main() {
    install_panic_hook();
    install_signal_handlers();
    run_with_panic_logging(|| run_usi_loop());
}
