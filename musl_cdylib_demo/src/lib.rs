#[macro_export]
macro_rules! dbgprintln {
    ($($arg:tt)*) => {{
        #[cfg(debug_assertions)]
        std::println!($($arg)*);
        // Release: emit nothing, but still reference the args so variables that exist only to be
        // logged don't trip `unused_variables`.
        #[cfg(not(debug_assertions))]
        let _ = core::format_args!($($arg)*);
    }};
}

#[unsafe(no_mangle)]
pub extern "C" fn run() {
    println!("cdylib loaded, calling into musl...");

    match std::fs::read_to_string("/etc/hostname") {
        Ok(s) => dbgprintln!("read from /etc/hostname: {}", s.trim()),
        Err(e) => dbgprintln!("read failed: {e}"),
    }
}
