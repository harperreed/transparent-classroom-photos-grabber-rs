use std::env;
use tempfile::TempDir;
use transparent_classroom_photos_grabber_rs::{client::Client, config::Config};

// Helper function to create a test configuration
fn create_test_config() -> Config {
    Config {
        email: "test@example.com".to_string(),
        password: "password123".to_string(),
        school_id: 12345,
        child_id: 67890,
    }
}

// Helper to isolate environment variable tests
fn with_isolated_env<F, R>(test_fn: F) -> R
where
    F: FnOnce() -> R,
{
    // Create a temporary directory for the test
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let temp_path = temp_dir.path().to_path_buf();

    // Get the list of environment variables we care about
    let env_vars = [
        "TC_EMAIL",
        "TC_PASSWORD",
        "SCHOOL",
        "CHILD",
        "DOTENV_PATH",
        "RUST_BACKTRACE",
        "RUST_LOG",
    ];

    // Store the original values
    let orig_values: Vec<(String, Option<String>)> = env_vars
        .iter()
        .map(|&name| (name.to_string(), env::var(name).ok()))
        .collect();

    // Remove variables for clean test
    for var in &env_vars {
        env::remove_var(var);
    }

    // Disable dotenv for tests
    env::set_var("DOTENV_DISABLED", "1");

    // Redirect .env location to non-existent directory to ensure it's not found
    env::set_var("DOTENV_PATH", temp_path.display().to_string());

    // Initialize the library to set up logging
    transparent_classroom_photos_grabber_rs::init();

    // Run the test
    let result = test_fn();

    // Restore original environment
    for (name, value) in orig_values {
        match value {
            Some(val) => env::set_var(name, val),
            None => env::remove_var(name),
        }
    }

    // Remove our dotenv disable flag
    env::remove_var("DOTENV_DISABLED");

    result
}

#[test]
fn test_client_creation() {
    with_isolated_env(|| {
        // Create a test configuration
        let config = create_test_config();

        // Create a client
        let client = Client::new(config).expect("Failed to create client");

        // Verify the base URL
        assert_eq!(
            client.base_url(),
            "https://www.transparentclassroom.com/schools/12345"
        );
    });
}
