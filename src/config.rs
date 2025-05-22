// ABOUTME: Configuration management for Transparent Classroom API
// ABOUTME: Loads settings from environment variables or config file

use dotenv::dotenv;
use log::{debug, warn};
use std::env;
use thiserror::Error;

use crate::error::AppError;

#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    pub email: String,
    pub password: String,
    pub school_id: u32,
    pub child_id: u32,
    pub school_lat: f64,
    pub school_lng: f64,
    pub school_keywords: String,
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Missing environment variable: {0}")]
    MissingEnv(String),

    #[error("Invalid integer value for environment variable {0}: {1}")]
    InvalidInteger(String, #[source] std::num::ParseIntError),

    #[error("Invalid float value for environment variable {0}: {1}")]
    InvalidFloat(String, #[source] std::num::ParseFloatError),

    #[error("Environment error: {0}")]
    EnvError(#[from] env::VarError),
}

impl Config {
    /// Load configuration from environment variables
    ///
    /// If environment variables are not set, attempts to load from .env file
    pub fn from_env() -> Result<Self, AppError> {
        // Load from .env file if present (in production)
        Self::from_env_with_dotenv(true).map_err(AppError::Config)
    }

    /// Load configuration from environment variables with option to skip dotenv
    ///
    /// This is useful for testing where we don't want to load from .env
    pub fn from_env_with_dotenv(use_dotenv: bool) -> Result<Self, ConfigError> {
        // Check if dotenv is disabled via environment variable - useful for tests
        let dotenv_disabled = env::var("DOTENV_DISABLED").is_ok();

        // Try to load from .env file if requested and not disabled
        if use_dotenv && !dotenv_disabled {
            match dotenv() {
                Ok(_) => debug!("Loaded configuration from .env file"),
                Err(_) => warn!("No .env file found, using environment variables only"),
            }
        } else if dotenv_disabled {
            debug!("Dotenv loading disabled by DOTENV_DISABLED environment variable");
        }

        // Extract required environment variables
        let email =
            env::var("TC_EMAIL").map_err(|_| ConfigError::MissingEnv("TC_EMAIL".to_string()))?;

        let password = env::var("TC_PASSWORD")
            .map_err(|_| ConfigError::MissingEnv("TC_PASSWORD".to_string()))?;

        let school_id_str =
            env::var("SCHOOL").map_err(|_| ConfigError::MissingEnv("SCHOOL".to_string()))?;

        let child_id_str =
            env::var("CHILD").map_err(|_| ConfigError::MissingEnv("CHILD".to_string()))?;

        let school_lat_str = env::var("SCHOOL_LAT")
            .map_err(|_| ConfigError::MissingEnv("SCHOOL_LAT".to_string()))?;

        let school_lng_str = env::var("SCHOOL_LNG")
            .map_err(|_| ConfigError::MissingEnv("SCHOOL_LNG".to_string()))?;

        let school_keywords = env::var("SCHOOL_KEYWORDS")
            .map_err(|_| ConfigError::MissingEnv("SCHOOL_KEYWORDS".to_string()))?;

        // Parse numeric values
        let school_id = school_id_str
            .parse::<u32>()
            .map_err(|e| ConfigError::InvalidInteger("SCHOOL".to_string(), e))?;

        let child_id = child_id_str
            .parse::<u32>()
            .map_err(|e| ConfigError::InvalidInteger("CHILD".to_string(), e))?;

        let school_lat = school_lat_str
            .parse::<f64>()
            .map_err(|e| ConfigError::InvalidFloat("SCHOOL_LAT".to_string(), e))?;

        let school_lng = school_lng_str
            .parse::<f64>()
            .map_err(|e| ConfigError::InvalidFloat("SCHOOL_LNG".to_string(), e))?;

        let config = Config {
            email,
            password,
            school_id,
            child_id,
            school_lat,
            school_lng,
            school_keywords,
        };

        debug!("Configuration loaded successfully");
        Ok(config)
    }
}
