use std::time::{SystemTime, UNIX_EPOCH};

/// Get the current epoch time in seconds
pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Expected now to be greater than epoch")
        .as_secs()
}
