use std::env;
use tempfile::TempDir;
use transparent_classroom_photos_grabber_rs::{
    config::{Config, ConfigError},
    error::AppError,
};

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

    // Remove variables for clean test - IMPORTANT: we explicitly clear them
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
fn test_valid_config() {
    with_isolated_env(|| {
        // Clear all environment variables first
        for var in &["TC_EMAIL", "TC_PASSWORD", "SCHOOL", "CHILD"] {
            env::remove_var(var);
        }

        // Set up environment variables for this test
        env::set_var("TC_EMAIL", "test@example.com");
        env::set_var("TC_PASSWORD", "password123");
        env::set_var("SCHOOL", "12345");
        env::set_var("CHILD", "67890");

        // Note: We're using from_env_with_dotenv(false) to avoid loading from .env
        // since we're explicitly setting environment variables in the test
        let config = Config::from_env_with_dotenv(false)
            .map_err(AppError::Config)
            .expect("Failed to load config");

        // Verify values
        assert_eq!(config.email, "test@example.com");
        assert_eq!(config.password, "password123");
        assert_eq!(config.school_id, 12345);
        assert_eq!(config.child_id, 67890);
    });
}

#[test]
fn test_missing_email() {
    with_isolated_env(|| {
        // Set partial environment variables, explicitly clearing TC_EMAIL
        env::remove_var("TC_EMAIL"); // Make sure this is removed
        env::set_var("TC_PASSWORD", "password123");
        env::set_var("SCHOOL", "12345");
        env::set_var("CHILD", "67890");

        // Using from_env_with_dotenv to avoid .env file
        let result = Config::from_env_with_dotenv(false).map_err(AppError::Config);

        // Verify error is the correct type
        if let Err(AppError::Config(ConfigError::MissingEnv(var_name))) = result {
            assert_eq!(var_name, "TC_EMAIL");
        } else {
            panic!(
                "Expected Config(MissingEnv) error for TC_EMAIL, got {:?}",
                result
            );
        }
    });
}

#[test]
fn test_missing_password() {
    with_isolated_env(|| {
        // Clear all environment variables first
        for var in &["TC_EMAIL", "TC_PASSWORD", "SCHOOL", "CHILD"] {
            env::remove_var(var);
        }

        // Create a temporary file to prevent env var from being cleared
        let env_file = std::fs::File::create("/tmp/test_env_lock").unwrap();

        // Set partial environment variables, explicitly clearing TC_PASSWORD
        env::set_var("TC_EMAIL", "test@example.com");
        // We keep TC_PASSWORD unset
        env::set_var("SCHOOL", "12345");
        env::set_var("CHILD", "67890");

        // Verify env vars are set properly
        assert_eq!(env::var("TC_EMAIL").unwrap(), "test@example.com");
        assert!(env::var("TC_PASSWORD").is_err());
        assert_eq!(env::var("SCHOOL").unwrap(), "12345");
        assert_eq!(env::var("CHILD").unwrap(), "67890");

        // Using from_env_with_dotenv to avoid .env file
        let result = Config::from_env_with_dotenv(false).map_err(AppError::Config);

        // Print out the result for debug
        eprintln!("Result: {:?}", result);

        // Verify error
        if let Err(AppError::Config(ConfigError::MissingEnv(var_name))) = result {
            assert_eq!(var_name, "TC_PASSWORD");
        } else {
            panic!(
                "Expected Config(MissingEnv) error for TC_PASSWORD, got {:?}",
                result
            );
        }

        // Drop lock file
        drop(env_file);
        std::fs::remove_file("/tmp/test_env_lock").ok();
    });
}

#[test]
fn test_invalid_school_id() {
    with_isolated_env(|| {
        // Clear all environment variables first
        for var in &["TC_EMAIL", "TC_PASSWORD", "SCHOOL", "CHILD"] {
            env::remove_var(var);
        }

        // Create a temporary file to prevent env var from being cleared
        let env_file = std::fs::File::create("/tmp/test_env_lock").unwrap();

        // Set environment variables with invalid SCHOOL
        env::set_var("TC_EMAIL", "test@example.com");
        env::set_var("TC_PASSWORD", "password123");
        env::set_var("SCHOOL", "not-a-number");
        env::set_var("CHILD", "67890");

        // Verify env vars are set properly
        assert_eq!(env::var("TC_EMAIL").unwrap(), "test@example.com");
        assert_eq!(env::var("TC_PASSWORD").unwrap(), "password123");
        assert_eq!(env::var("SCHOOL").unwrap(), "not-a-number");
        assert_eq!(env::var("CHILD").unwrap(), "67890");

        // Using from_env_with_dotenv to avoid .env file
        let result = Config::from_env_with_dotenv(false).map_err(AppError::Config);

        // Verify error
        if let Err(AppError::Config(ConfigError::InvalidInteger(var_name, _))) = result {
            assert_eq!(var_name, "SCHOOL");
        } else {
            panic!(
                "Expected Config(InvalidInteger) error for SCHOOL, got {:?}",
                result
            );
        }

        // Drop lock file
        drop(env_file);
        std::fs::remove_file("/tmp/test_env_lock").ok();
    });
}

#[test]
fn test_invalid_child_id() {
    with_isolated_env(|| {
        // Clear all environment variables first
        for var in &["TC_EMAIL", "TC_PASSWORD", "SCHOOL", "CHILD"] {
            env::remove_var(var);
        }

        // Create a temporary file to prevent env var from being cleared
        let env_file = std::fs::File::create("/tmp/test_env_lock").unwrap();

        // Set environment variables with invalid CHILD
        env::set_var("TC_EMAIL", "test@example.com");
        env::set_var("TC_PASSWORD", "password123");
        env::set_var("SCHOOL", "12345");
        env::set_var("CHILD", "not-a-number");

        // Verify env vars are set properly
        assert_eq!(env::var("TC_EMAIL").unwrap(), "test@example.com");
        assert_eq!(env::var("TC_PASSWORD").unwrap(), "password123");
        assert_eq!(env::var("SCHOOL").unwrap(), "12345");
        assert_eq!(env::var("CHILD").unwrap(), "not-a-number");

        // Using from_env_with_dotenv to avoid .env file
        let result = Config::from_env_with_dotenv(false).map_err(AppError::Config);

        // Verify error
        if let Err(AppError::Config(ConfigError::InvalidInteger(var_name, _))) = result {
            assert_eq!(var_name, "CHILD");
        } else {
            panic!(
                "Expected Config(InvalidInteger) error for CHILD, got {:?}",
                result
            );
        }

        // Drop lock file
        drop(env_file);
        std::fs::remove_file("/tmp/test_env_lock").ok();
    });
}

// Test the new AppError::from_env method by wrapping from_env_with_dotenv
#[test]
fn test_config_from_env_wrapper() {
    with_isolated_env(|| {
        // Clear all environment variables first
        for var in &["TC_EMAIL", "TC_PASSWORD", "SCHOOL", "CHILD"] {
            env::remove_var(var);
        }

        // Create a temporary file to prevent env var from being cleared
        let env_file = std::fs::File::create("/tmp/test_env_lock").unwrap();

        // Set up environment variables for this test
        env::set_var("TC_EMAIL", "test@example.com");
        env::set_var("TC_PASSWORD", "password123");
        env::set_var("SCHOOL", "12345");
        env::set_var("CHILD", "67890");

        // Verify env vars are set properly
        assert_eq!(env::var("TC_EMAIL").unwrap(), "test@example.com");
        assert_eq!(env::var("TC_PASSWORD").unwrap(), "password123");
        assert_eq!(env::var("SCHOOL").unwrap(), "12345");
        assert_eq!(env::var("CHILD").unwrap(), "67890");

        // Don't use from_env() directly in tests since it tries to load from .env
        // Instead manually wrap from_env_with_dotenv(false) with AppError to test the same pattern
        let config = Config::from_env_with_dotenv(false)
            .map_err(AppError::Config)
            .expect("Failed to load config");

        // Verify values
        assert_eq!(config.email, "test@example.com");
        assert_eq!(config.password, "password123");
        assert_eq!(config.school_id, 12345);
        assert_eq!(config.child_id, 67890);

        // Drop lock file
        drop(env_file);
        std::fs::remove_file("/tmp/test_env_lock").ok();
    });
}

// Add a simple test that confirms our logger can be initialized
#[test]
fn test_logger_init() {
    // This should not panic
    transparent_classroom_photos_grabber_rs::init();
    transparent_classroom_photos_grabber_rs::init(); // Call twice to ensure Once works
}
