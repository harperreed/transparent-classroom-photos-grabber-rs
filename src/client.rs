// ABOUTME: HTTP client for Transparent Classroom API
// ABOUTME: Manages authentication and requests to the API

use log::{debug, info, warn};
use reqwest::blocking::{Client as ReqwestClient, ClientBuilder};
use reqwest::cookie::Jar;
use scraper::{Html, Selector};
use std::collections::HashMap;
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
        // Construct the default base URL using the school ID from config
        let base_url = format!(
            "https://www.transparentclassroom.com/schools/{}",
            config.school_id
        );

        Self::new_with_base_url(config, base_url)
    }

    /// Create a new client with a specific base URL (useful for testing)
    pub fn new_with_base_url(config: Config, base_url: String) -> Result<Self, AppError> {
        // Create a cookie jar to store session cookies
        let cookie_jar = Arc::new(Jar::default());

        // Build a reqwest client with cookies enabled
        let http_client = ClientBuilder::new()
            .cookie_provider(Arc::clone(&cookie_jar))
            .build()
            .map_err(|e| AppError::Generic(format!("Failed to create HTTP client: {}", e)))?;

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

    /// Login to Transparent Classroom
    ///
    /// This method performs the login flow:
    /// 1. GET the sign_in page to obtain a CSRF token
    /// 2. POST the credentials with the CSRF token
    /// 3. Verify successful login
    pub fn login(&self) -> Result<(), AppError> {
        debug!("Starting login flow");

        // Step 1: GET the sign_in page
        let sign_in_url = format!("{}/souls/sign_in", self.base_url);
        debug!("Fetching sign-in page: {}", sign_in_url);

        let response = self
            .http_client
            .get(&sign_in_url)
            .send()
            .map_err(|e| AppError::Generic(format!("Failed to fetch sign-in page: {}", e)))?;

        if !response.status().is_success() {
            return Err(AppError::Generic(format!(
                "Failed to fetch sign-in page. Status: {}",
                response.status()
            )));
        }

        // Get the response body as text
        let html = response.text().map_err(|e| {
            AppError::Generic(format!("Failed to read sign-in page content: {}", e))
        })?;

        // Step 2: Parse the HTML and extract the CSRF token
        let csrf_token = self.extract_csrf_token(&html)?;
        debug!("Successfully extracted CSRF token");

        // Step 3: POST credentials with the CSRF token
        let mut form_data = HashMap::new();
        form_data.insert("utf8", "✓");
        form_data.insert("authenticity_token", &csrf_token);
        form_data.insert("soul[email]", &self.config.email);
        form_data.insert("soul[password]", &self.config.password);
        form_data.insert("soul[remember_me]", "0");
        form_data.insert("commit", "Sign In");

        debug!("Submitting login form to: {}", sign_in_url);
        let response = self
            .http_client
            .post(&sign_in_url)
            .form(&form_data)
            .send()
            .map_err(|e| AppError::Generic(format!("Failed to submit login form: {}", e)))?;

        // Step 4: Verify successful login
        if !response.status().is_success() {
            return Err(AppError::Generic(format!(
                "Login failed. Status: {}",
                response.status()
            )));
        }

        // Check if we were redirected to the dashboard, which indicates successful login
        // Or if we can see content that's only available after login
        let html = response
            .text()
            .map_err(|e| AppError::Generic(format!("Failed to read post-login page: {}", e)))?;

        if html.contains("Invalid email or password") {
            return Err(AppError::Generic(
                "Login failed: Invalid email or password".to_string(),
            ));
        }

        if !html.contains("Dashboard") && !html.contains("My Account") {
            warn!("Login may have failed - could not find expected post-login content");
        }

        info!("Login successful");
        Ok(())
    }

    /// Extract CSRF token from HTML
    fn extract_csrf_token(&self, html: &str) -> Result<String, AppError> {
        let document = Html::parse_document(html);

        // Try to find meta tag with name="csrf-token"
        let selector = Selector::parse("meta[name=\"csrf-token\"]").unwrap();

        if let Some(element) = document.select(&selector).next() {
            if let Some(token) = element.value().attr("content") {
                return Ok(token.to_string());
            }
        }

        // Alternative: check for input with name="authenticity_token"
        let selector = Selector::parse("input[name=\"authenticity_token\"]").unwrap();

        if let Some(element) = document.select(&selector).next() {
            if let Some(token) = element.value().attr("value") {
                return Ok(token.to_string());
            }
        }

        Err(AppError::Parse(
            "Could not find CSRF token in page".to_string(),
        ))
    }
}
