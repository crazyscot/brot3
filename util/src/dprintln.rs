//! A switchable debug println macro
// (c) 2026 Ross Younger

/// A switchable debug version of println!()
// Trap: this definition must appear BEFORE all the modules that use it, otherwise they won't see
// it. That doesn't apply when macro_export is used.
#[macro_export]
#[clippy::format_args]
macro_rules! dprintln {
    ($flag:expr, $($arg:tt)*) => {
        if $flag {
            eprintln!($($arg)*);
        }
    };
}
