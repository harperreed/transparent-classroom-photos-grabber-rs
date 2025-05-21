// ABOUTME: Core library for transparent-classroom-photos-grabber-rs
// ABOUTME: Provides functionality to fetch and download photos from Transparent Classroom

use std::sync::Once;

pub mod config;
pub mod error;

// This ensures env_logger is only initialized once
static INIT: Once = Once::new();

/// Initialize the library
///
/// Sets up logging with env_logger. This is safe to call multiple times
/// as it will only initialize the logger on the first call.
pub fn init() {
    INIT.call_once(|| {
        env_logger::init();
        log::debug!("Logger initialized");
    });
}

/// Placeholder for future functionality
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
