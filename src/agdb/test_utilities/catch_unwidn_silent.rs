use std::panic::UnwindSafe;
use std::thread::Result as ThreadResult;

pub fn catch_unwind_silent<F: FnOnce() -> R + UnwindSafe, R>(f: F) -> ThreadResult<R> {
    let prev_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let result = std::panic::catch_unwind(f);
    std::panic::set_hook(prev_hook);
    result
}
