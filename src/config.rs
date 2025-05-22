// ABOUTME: Configuration management for Transparent Classroom API
// ABOUTME: Loads settings from environment variables or config file

use dotenv::dotenv;
use log::{debug, warn};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

use crate::error::AppError;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Config file error: {0}")]
    ConfigFileError(#[from] serde_yaml::Error),

    #[error("Config file not found")]
    ConfigFileNotFound,
}

impl Config {
    /// Load configuration from multiple sources in order of priority:
    /// 1. Environment variables
    /// 2. Config file
    /// 3. .env file
    pub fn load() -> Result<Self, AppError> {
        // Try environment variables first
        if let Ok(config) = Self::from_env_with_dotenv(false) {
            debug!("Loaded configuration from environment variables");
            return Ok(config);
        }

        // Try config file
        if let Ok(config) = Self::from_file() {
            debug!("Loaded configuration from config file");
            return Ok(config);
        }

        // Fallback to .env file
        Self::from_env_with_dotenv(true).map_err(AppError::Config)
    }

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

    /// Get the standard config file path for the current platform
    pub fn get_config_file_path() -> Result<PathBuf, ConfigError> {
        let config_dir = dirs::config_dir().ok_or_else(|| {
            ConfigError::IoError(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Could not find config directory",
            ))
        })?;

        Ok(config_dir
            .join("transparent-classroom-photos-grabber")
            .join("config.yaml"))
    }

    /// Load configuration from config file
    pub fn from_file() -> Result<Self, ConfigError> {
        let config_path = Self::get_config_file_path()?;
        Self::from_file_path(&config_path)
    }

    /// Load configuration from a specific file path
    pub fn from_file_path(path: &Path) -> Result<Self, ConfigError> {
        if !path.exists() {
            return Err(ConfigError::ConfigFileNotFound);
        }

        let content = fs::read_to_string(path)?;
        let config: Config = serde_yaml::from_str(&content)?;
        debug!("Configuration loaded from file: {}", path.display());
        Ok(config)
    }

    /// Save configuration to config file
    pub fn save_to_file(&self) -> Result<(), ConfigError> {
        let config_path = Self::get_config_file_path()?;
        self.save_to_file_path(&config_path)
    }

    /// Save configuration to a specific file path
    pub fn save_to_file_path(&self, path: &Path) -> Result<(), ConfigError> {
        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_yaml::to_string(self)?;
        fs::write(path, content)?;
        debug!("Configuration saved to file: {}", path.display());
        Ok(())
    }

    /// Attempt to derive school location from the Transparent Classroom portal
    pub fn derive_school_location(
        school_id: u32,
        email: &str,
        password: &str,
    ) -> Result<(f64, f64), ConfigError> {
        use reqwest::blocking::Client as ReqwestClient;
        use scraper::{Html, Selector};

        // Create a temporary HTTP client
        let client = ReqwestClient::new();

        // Try to get school information from the API or web interface
        let base_url = format!("https://www.transparentclassroom.com/schools/{}", school_id);

        // First try the API endpoint for school information
        let api_url = format!(
            "{}/api/v1/schools/{}",
            "https://www.transparentclassroom.com", school_id
        );

        if let Ok(response) = client
            .get(&api_url)
            .basic_auth(email, Some(password))
            .send()
        {
            if response.status().is_success() {
                if let Ok(text) = response.text() {
                    // Try to parse JSON response for school info with location
                    if let Ok(school_info) = serde_json::from_str::<serde_json::Value>(&text) {
                        if let (Some(lat), Some(lng)) = (
                            school_info.get("latitude").and_then(|v| v.as_f64()),
                            school_info.get("longitude").and_then(|v| v.as_f64()),
                        ) {
                            return Ok((lat, lng));
                        }
                    }
                }
            }
        }

        // If API doesn't work, try to scrape the school page for location info
        if let Ok(response) = client.get(&base_url).send() {
            if response.status().is_success() {
                if let Ok(html) = response.text() {
                    let document = Html::parse_document(&html);

                    // Look for common patterns that might contain coordinates
                    // Check for Google Maps embed or similar
                    let script_selector = Selector::parse("script").unwrap();
                    for script in document.select(&script_selector) {
                        let script_text = script.inner_html();

                        // Look for lat/lng patterns in JavaScript
                        if let Some((lat, lng)) = Self::extract_coordinates_from_text(&script_text)
                        {
                            return Ok((lat, lng));
                        }
                    }

                    // Look for meta tags that might contain location
                    let meta_selector = Selector::parse("meta").unwrap();
                    for meta in document.select(&meta_selector) {
                        if let Some(content) = meta.value().attr("content") {
                            if let Some((lat, lng)) = Self::extract_coordinates_from_text(content) {
                                return Ok((lat, lng));
                            }
                        }
                    }
                }
            }
        }

        Err(ConfigError::IoError(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Could not derive school location from portal",
        )))
    }

    /// Extract latitude and longitude coordinates from text
    fn extract_coordinates_from_text(text: &str) -> Option<(f64, f64)> {
        use regex::Regex;

        // Look for various coordinate patterns
        let patterns = [
            // lat: 40.7128, lng: -74.0060 or latitude: 40.7128, longitude: -74.0060
            r"lat(?:itude)?\s*:\s*([+-]?\d+\.?\d*),?\s*lng|lon(?:gitude)?\s*:\s*([+-]?\d+\.?\d*)",
            // "40.7128,-74.0060" or "40.7128, -74.0060"
            r"([+-]?\d+\.?\d+)\s*,\s*([+-]?\d+\.?\d+)",
            // Google Maps style: new google.maps.LatLng(40.7128, -74.0060)
            r"LatLng\s*\(\s*([+-]?\d+\.?\d*)\s*,\s*([+-]?\d+\.?\d*)\s*\)",
        ];

        for pattern in &patterns {
            if let Ok(re) = Regex::new(pattern) {
                if let Some(captures) = re.captures(text) {
                    if let (Some(lat_str), Some(lng_str)) = (captures.get(1), captures.get(2)) {
                        if let (Ok(lat), Ok(lng)) = (
                            lat_str.as_str().parse::<f64>(),
                            lng_str.as_str().parse::<f64>(),
                        ) {
                            // Basic validation: reasonable coordinate ranges
                            if (-90.0..=90.0).contains(&lat) && (-180.0..=180.0).contains(&lng) {
                                return Some((lat, lng));
                            }
                        }
                    }
                }
            }
        }

        None
    }

    /// Interactive setup of configuration
    pub fn interactive_setup() -> Result<Self, ConfigError> {
        use dialoguer::{Input, Password};

        println!("📋 Setting up Transparent Classroom Photos Grabber configuration");
        println!();

        let email: String = Input::new()
            .with_prompt("Email")
            .interact_text()
            .map_err(|e| ConfigError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        let password: String = Password::new()
            .with_prompt("Password")
            .interact()
            .map_err(|e| ConfigError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        let school_id: u32 = Input::new()
            .with_prompt("School ID")
            .interact_text()
            .map_err(|e| ConfigError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        let child_id: u32 = Input::new()
            .with_prompt("Child ID")
            .interact_text()
            .map_err(|e| ConfigError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        // For lat/lng, offer to derive from school location or enter manually
        use dialoguer::Confirm;

        let derive_location = Confirm::new()
            .with_prompt("Would you like to try to derive the school's location automatically?")
            .default(true)
            .interact()
            .map_err(|e| ConfigError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        let (school_lat, school_lng) = if derive_location {
            println!("Attempting to derive school location...");
            match Self::derive_school_location(school_id, &email, &password) {
                Ok((lat, lng)) => {
                    println!("✅ Successfully derived school location: {}, {}", lat, lng);
                    (lat, lng)
                }
                Err(e) => {
                    println!("⚠️  Could not derive location automatically: {}", e);
                    println!("You can enter coordinates manually or leave as 0.0, 0.0");

                    let lat: f64 = Input::new()
                        .with_prompt("School Latitude")
                        .with_initial_text("0.0")
                        .interact_text()
                        .map_err(|e| {
                            ConfigError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e))
                        })?;

                    let lng: f64 = Input::new()
                        .with_prompt("School Longitude")
                        .with_initial_text("0.0")
                        .interact_text()
                        .map_err(|e| {
                            ConfigError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e))
                        })?;

                    (lat, lng)
                }
            }
        } else {
            let lat: f64 = Input::new()
                .with_prompt("School Latitude")
                .with_initial_text("0.0")
                .interact_text()
                .map_err(|e| {
                    ConfigError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e))
                })?;

            let lng: f64 = Input::new()
                .with_prompt("School Longitude")
                .with_initial_text("0.0")
                .interact_text()
                .map_err(|e| {
                    ConfigError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e))
                })?;

            (lat, lng)
        };

        let school_keywords: String = Input::new()
            .with_prompt("School Keywords")
            .with_initial_text("")
            .interact_text()
            .map_err(|e| ConfigError::IoError(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        let config = Config {
            email,
            password,
            school_id,
            child_id,
            school_lat,
            school_lng,
            school_keywords,
        };

        // Save the configuration
        config.save_to_file()?;
        println!("✅ Configuration saved successfully!");

        Ok(config)
    }
}
