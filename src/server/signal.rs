use std::sync::atomic::{AtomicBool, Ordering};

pub static SHOULD_STOP: AtomicBool = AtomicBool::new(false);
const SIGINT: i32 = 2;

unsafe extern "C" {
    fn signal(sig: i32, handler: extern "C" fn(i32)) -> usize;
}

extern "C" fn handle_sigint(_signal: i32) {
    SHOULD_STOP.store(true, Ordering::SeqCst);
}

pub fn install_sigint_handler() {
    unsafe {
        let _ = signal(SIGINT, handle_sigint);
    }
}
