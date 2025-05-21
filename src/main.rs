// ABOUTME: Main entry point for transparent-classroom-photos-grabber-rs
// ABOUTME: Provides CLI interface to download photos from Transparent Classroom

use transparent_classroom_photos_grabber_rs::{config::Config, init};

/// Main entry point for the application
fn main() {
    // Initialize the library (sets up logging)
    init();

    // Attempt to load configuration
    match Config::from_env() {
        Ok(config) => {
            println!("Successfully loaded configuration:");
            println!("  Email: {}", config.email);
            println!("  School ID: {}", config.school_id);
            println!("  Child ID: {}", config.child_id);
        }
        Err(err) => {
            eprintln!("Failed to load configuration: {}", err);
            std::process::exit(1);
        }
    }
}
