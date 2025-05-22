// ABOUTME: Robust configuration tests with proper environment isolation
// ABOUTME: Tests all config fields including new GPS coordinates and keywords

use std::collections::HashMap;
use std::env;
use std::sync::Mutex;
use transparent_classroom_photos_grabber_rs::{
    config::{Config, ConfigError},
    error::AppError,
};

// Global mutex to ensure config tests run sequentially to avoid env var conflicts
static ENV_MUTEX: Mutex<()> = Mutex::new(());

/// Helper to run config tests with complete environment isolation
fn with_clean_env<F, R>(test_fn: F) -> R
where
    F: FnOnce() -> R,
{
    let _guard = ENV_MUTEX.lock().unwrap();

    // Store original environment
    let original_env: HashMap<String, String> = [
        "TC_EMAIL",
        "TC_PASSWORD",
        "SCHOOL",
        "CHILD",
        "SCHOOL_LAT",
        "SCHOOL_LNG",
        "SCHOOL_KEYWORDS",
        "DOTENV_DISABLED",
    ]
    .iter()
    .filter_map(|&key| env::var(key).ok().map(|val| (key.to_string(), val)))
    .collect();

    // Clear all config-related env vars
    for &key in &[
        "TC_EMAIL",
        "TC_PASSWORD",
        "SCHOOL",
        "CHILD",
        "SCHOOL_LAT",
        "SCHOOL_LNG",
        "SCHOOL_KEYWORDS",
    ] {
        env::remove_var(key);
    }

    // Disable dotenv to prevent .env file interference
    env::set_var("DOTENV_DISABLED", "1");

    // Run the test
    let result = test_fn();

    // Restore original environment
    for &key in &[
        "TC_EMAIL",
        "TC_PASSWORD",
        "SCHOOL",
        "CHILD",
        "SCHOOL_LAT",
        "SCHOOL_LNG",
        "SCHOOL_KEYWORDS",
        "DOTENV_DISABLED",
    ] {
        env::remove_var(key);
    }
    for (key, value) in original_env {
        env::set_var(key, value);
    }

    result
}

#[test]
fn test_valid_config_complete() {
    with_clean_env(|| {
        // Set all required environment variables
        env::set_var("TC_EMAIL", "test@example.com");
        env::set_var("TC_PASSWORD", "password123");
        env::set_var("SCHOOL", "12345");
        env::set_var("CHILD", "67890");
        env::set_var("SCHOOL_LAT", "41.9032776");
        env::set_var("SCHOOL_LNG", "-87.6663027");
        env::set_var("SCHOOL_KEYWORDS", "school, montessori, chicago");

        let config = Config::from_env_with_dotenv(false).expect("Should load valid config");

        assert_eq!(config.email, "test@example.com");
        assert_eq!(config.password, "password123");
        assert_eq!(config.school_id, 12345);
        assert_eq!(config.child_id, 67890);
        assert_eq!(config.school_lat, 41.9032776);
        assert_eq!(config.school_lng, -87.6663027);
        assert_eq!(config.school_keywords, "school, montessori, chicago");
    });
}

#[test]
fn test_missing_email() {
    with_clean_env(|| {
        env::set_var("TC_PASSWORD", "password123");
        env::set_var("SCHOOL", "12345");
        env::set_var("CHILD", "67890");
        env::set_var("SCHOOL_LAT", "41.9");
        env::set_var("SCHOOL_LNG", "-87.6");
        env::set_var("SCHOOL_KEYWORDS", "test");

        let result = Config::from_env_with_dotenv(false);

        match result {
            Err(ConfigError::MissingEnv(var)) => assert_eq!(var, "TC_EMAIL"),
            other => panic!("Expected MissingEnv(TC_EMAIL), got {:?}", other),
        }
    });
}

#[test]
fn test_missing_password() {
    with_clean_env(|| {
        env::set_var("TC_EMAIL", "test@example.com");
        env::set_var("SCHOOL", "12345");
        env::set_var("CHILD", "67890");
        env::set_var("SCHOOL_LAT", "41.9");
        env::set_var("SCHOOL_LNG", "-87.6");
        env::set_var("SCHOOL_KEYWORDS", "test");

        let result = Config::from_env_with_dotenv(false);

        match result {
            Err(ConfigError::MissingEnv(var)) => assert_eq!(var, "TC_PASSWORD"),
            other => panic!("Expected MissingEnv(TC_PASSWORD), got {:?}", other),
        }
    });
}

#[test]
fn test_missing_school() {
    with_clean_env(|| {
        env::set_var("TC_EMAIL", "test@example.com");
        env::set_var("TC_PASSWORD", "password123");
        env::set_var("CHILD", "67890");
        env::set_var("SCHOOL_LAT", "41.9");
        env::set_var("SCHOOL_LNG", "-87.6");
        env::set_var("SCHOOL_KEYWORDS", "test");

        let result = Config::from_env_with_dotenv(false);

        match result {
            Err(ConfigError::MissingEnv(var)) => assert_eq!(var, "SCHOOL"),
            other => panic!("Expected MissingEnv(SCHOOL), got {:?}", other),
        }
    });
}

#[test]
fn test_missing_child() {
    with_clean_env(|| {
        env::set_var("TC_EMAIL", "test@example.com");
        env::set_var("TC_PASSWORD", "password123");
        env::set_var("SCHOOL", "12345");
        env::set_var("SCHOOL_LAT", "41.9");
        env::set_var("SCHOOL_LNG", "-87.6");
        env::set_var("SCHOOL_KEYWORDS", "test");

        let result = Config::from_env_with_dotenv(false);

        match result {
            Err(ConfigError::MissingEnv(var)) => assert_eq!(var, "CHILD"),
            other => panic!("Expected MissingEnv(CHILD), got {:?}", other),
        }
    });
}

#[test]
fn test_missing_school_lat() {
    with_clean_env(|| {
        env::set_var("TC_EMAIL", "test@example.com");
        env::set_var("TC_PASSWORD", "password123");
        env::set_var("SCHOOL", "12345");
        env::set_var("CHILD", "67890");
        env::set_var("SCHOOL_LNG", "-87.6");
        env::set_var("SCHOOL_KEYWORDS", "test");

        let result = Config::from_env_with_dotenv(false);

        match result {
            Err(ConfigError::MissingEnv(var)) => assert_eq!(var, "SCHOOL_LAT"),
            other => panic!("Expected MissingEnv(SCHOOL_LAT), got {:?}", other),
        }
    });
}

#[test]
fn test_missing_school_lng() {
    with_clean_env(|| {
        env::set_var("TC_EMAIL", "test@example.com");
        env::set_var("TC_PASSWORD", "password123");
        env::set_var("SCHOOL", "12345");
        env::set_var("CHILD", "67890");
        env::set_var("SCHOOL_LAT", "41.9");
        env::set_var("SCHOOL_KEYWORDS", "test");

        let result = Config::from_env_with_dotenv(false);

        match result {
            Err(ConfigError::MissingEnv(var)) => assert_eq!(var, "SCHOOL_LNG"),
            other => panic!("Expected MissingEnv(SCHOOL_LNG), got {:?}", other),
        }
    });
}

#[test]
fn test_missing_school_keywords() {
    with_clean_env(|| {
        env::set_var("TC_EMAIL", "test@example.com");
        env::set_var("TC_PASSWORD", "password123");
        env::set_var("SCHOOL", "12345");
        env::set_var("CHILD", "67890");
        env::set_var("SCHOOL_LAT", "41.9");
        env::set_var("SCHOOL_LNG", "-87.6");

        let result = Config::from_env_with_dotenv(false);

        match result {
            Err(ConfigError::MissingEnv(var)) => assert_eq!(var, "SCHOOL_KEYWORDS"),
            other => panic!("Expected MissingEnv(SCHOOL_KEYWORDS), got {:?}", other),
        }
    });
}

#[test]
fn test_invalid_school_id() {
    with_clean_env(|| {
        env::set_var("TC_EMAIL", "test@example.com");
        env::set_var("TC_PASSWORD", "password123");
        env::set_var("SCHOOL", "not-a-number");
        env::set_var("CHILD", "67890");
        env::set_var("SCHOOL_LAT", "41.9");
        env::set_var("SCHOOL_LNG", "-87.6");
        env::set_var("SCHOOL_KEYWORDS", "test");

        let result = Config::from_env_with_dotenv(false);

        match result {
            Err(ConfigError::InvalidInteger(var, _)) => assert_eq!(var, "SCHOOL"),
            other => panic!("Expected InvalidInteger(SCHOOL), got {:?}", other),
        }
    });
}

#[test]
fn test_invalid_child_id() {
    with_clean_env(|| {
        env::set_var("TC_EMAIL", "test@example.com");
        env::set_var("TC_PASSWORD", "password123");
        env::set_var("SCHOOL", "12345");
        env::set_var("CHILD", "not-a-number");
        env::set_var("SCHOOL_LAT", "41.9");
        env::set_var("SCHOOL_LNG", "-87.6");
        env::set_var("SCHOOL_KEYWORDS", "test");

        let result = Config::from_env_with_dotenv(false);

        match result {
            Err(ConfigError::InvalidInteger(var, _)) => assert_eq!(var, "CHILD"),
            other => panic!("Expected InvalidInteger(CHILD), got {:?}", other),
        }
    });
}

#[test]
fn test_invalid_school_lat() {
    with_clean_env(|| {
        env::set_var("TC_EMAIL", "test@example.com");
        env::set_var("TC_PASSWORD", "password123");
        env::set_var("SCHOOL", "12345");
        env::set_var("CHILD", "67890");
        env::set_var("SCHOOL_LAT", "not-a-float");
        env::set_var("SCHOOL_LNG", "-87.6");
        env::set_var("SCHOOL_KEYWORDS", "test");

        let result = Config::from_env_with_dotenv(false);

        match result {
            Err(ConfigError::InvalidFloat(var, _)) => assert_eq!(var, "SCHOOL_LAT"),
            other => panic!("Expected InvalidFloat(SCHOOL_LAT), got {:?}", other),
        }
    });
}

#[test]
fn test_invalid_school_lng() {
    with_clean_env(|| {
        env::set_var("TC_EMAIL", "test@example.com");
        env::set_var("TC_PASSWORD", "password123");
        env::set_var("SCHOOL", "12345");
        env::set_var("CHILD", "67890");
        env::set_var("SCHOOL_LAT", "41.9");
        env::set_var("SCHOOL_LNG", "not-a-float");
        env::set_var("SCHOOL_KEYWORDS", "test");

        let result = Config::from_env_with_dotenv(false);

        match result {
            Err(ConfigError::InvalidFloat(var, _)) => assert_eq!(var, "SCHOOL_LNG"),
            other => panic!("Expected InvalidFloat(SCHOOL_LNG), got {:?}", other),
        }
    });
}

#[test]
fn test_edge_case_coordinates() {
    with_clean_env(|| {
        env::set_var("TC_EMAIL", "test@example.com");
        env::set_var("TC_PASSWORD", "password123");
        env::set_var("SCHOOL", "12345");
        env::set_var("CHILD", "67890");
        env::set_var("SCHOOL_LAT", "0.0"); // Equator
        env::set_var("SCHOOL_LNG", "0.0"); // Prime Meridian
        env::set_var("SCHOOL_KEYWORDS", ""); // Empty keywords

        let config =
            Config::from_env_with_dotenv(false).expect("Should handle edge case coordinates");

        assert_eq!(config.school_lat, 0.0);
        assert_eq!(config.school_lng, 0.0);
        assert_eq!(config.school_keywords, "");
    });
}

#[test]
fn test_extreme_coordinates() {
    with_clean_env(|| {
        env::set_var("TC_EMAIL", "test@example.com");
        env::set_var("TC_PASSWORD", "password123");
        env::set_var("SCHOOL", "12345");
        env::set_var("CHILD", "67890");
        env::set_var("SCHOOL_LAT", "-90.0"); // South Pole
        env::set_var("SCHOOL_LNG", "180.0"); // Date Line
        env::set_var("SCHOOL_KEYWORDS", "extreme,location,test");

        let config =
            Config::from_env_with_dotenv(false).expect("Should handle extreme coordinates");

        assert_eq!(config.school_lat, -90.0);
        assert_eq!(config.school_lng, 180.0);
        assert_eq!(config.school_keywords, "extreme,location,test");
    });
}

#[test]
fn test_app_error_wrapper() {
    with_clean_env(|| {
        // Test that AppError properly wraps ConfigError
        let result: Result<Config, AppError> =
            Config::from_env_with_dotenv(false).map_err(AppError::Config);

        match result {
            Err(AppError::Config(ConfigError::MissingEnv(var))) => {
                assert_eq!(var, "TC_EMAIL"); // First missing var
            }
            other => panic!("Expected AppError::Config(MissingEnv), got {:?}", other),
        }
    });
}

#[test]
fn test_precision_coordinates() {
    with_clean_env(|| {
        env::set_var("TC_EMAIL", "test@example.com");
        env::set_var("TC_PASSWORD", "password123");
        env::set_var("SCHOOL", "12345");
        env::set_var("CHILD", "67890");
        env::set_var("SCHOOL_LAT", "41.9032776123456"); // High precision
        env::set_var("SCHOOL_LNG", "-87.6663027654321"); // High precision
        env::set_var("SCHOOL_KEYWORDS", "precision,test,location");

        let config =
            Config::from_env_with_dotenv(false).expect("Should handle high precision coordinates");

        // Check that precision is maintained
        assert!((config.school_lat - 41.9032776123456).abs() < f64::EPSILON);
        assert!((config.school_lng - (-87.6663027654321)).abs() < f64::EPSILON);
    });
}
