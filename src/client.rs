// ABOUTME: HTTP client for Transparent Classroom API
// ABOUTME: Manages authentication and requests to the API

use log::{debug, info};
use reqwest::blocking::{Client as ReqwestClient, ClientBuilder};
use reqwest::cookie::Jar;
use std::sync::Arc;

use crate::config::Config;
use crate::error::AppError;

/// API client for Transparent Classroom
#[derive(Debug)]
pub struct Client {
    /// The underlying reqwest client for making HTTP requests
    #[allow(dead_code)]
    http_client: ReqwestClient,

    /// Application configuration
    #[allow(dead_code)]
    config: Config,

    /// Base URL for the Transparent Classroom API
    base_url: String,
}

impl Client {
    /// Create a new client with the given configuration
    pub fn new(config: Config) -> Result<Self, AppError> {
        // Create a cookie jar to store session cookies
        let cookie_jar = Arc::new(Jar::default());

        // Build a reqwest client with cookies enabled
        let http_client = ClientBuilder::new()
            .cookie_provider(Arc::clone(&cookie_jar))
            .build()
            .map_err(|e| AppError::Generic(format!("Failed to create HTTP client: {}", e)))?;

        // Construct the base URL using the school ID from config
        let base_url = format!(
            "https://www.transparentclassroom.com/schools/{}",
            config.school_id
        );

        debug!("Created client for school ID: {}", config.school_id);
        info!("Client initialized with base URL: {}", base_url);

        Ok(Client {
            http_client,
            config,
            base_url,
        })
    }

    /// Get the base URL for Transparent Classroom API
    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}
