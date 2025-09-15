/// Macro for convenient console logging in debug builds.
/// 
/// In debug builds, this forwards to `console.log()` for debugging.
/// In release builds, this becomes a no-op to minimize overhead.
/// 
/// # Usage
/// 
/// ```rust
/// console_log!("Debug message: {}", value);
/// ```
#[cfg(feature = "debug")]
#[macro_export]
macro_rules! console_log {
    ($($t:tt)*) => (unsafe { log(&format_args!($($t)*).to_string()) })
}

/// No-op console logging for release builds.
/// This ensures debug logging has zero runtime cost in production.
#[cfg(not(feature = "debug"))]
#[macro_export]
macro_rules! console_log {
    ($($t:tt)*) => {}
}