//! Safe logging wrappers over `host_log_debug`.

use super::imports;

/// Emit a debug log message through the host diagnostics layer.
///
/// Messages are visible in the app's developer mode and the CLI's
/// `atlas inspect` output. No-op in release builds if the host strips them.
pub fn debug(msg: &str) {
    let bytes = msg.as_bytes();
    unsafe {
        imports::host_log_debug(bytes.as_ptr(), bytes.len() as u32);
    }
}

/// Formatted debug log. Allocates a String for formatting.
#[macro_export]
macro_rules! atlas_log {
    ($($arg:tt)*) => {
        $crate::host::log::debug(&format!($($arg)*))
    };
}
