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

/// Represents a post from Transparent Classroom
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Post {
    /// Unique identifier of the post
    pub id: String,

    /// Title or name of the post
    pub title: String,

    /// Author of the post
    pub author: String,

    /// When the post was created
    pub date: String,

    /// URL to the post content
    pub url: String,

    /// URLs to photos attached to the post, if any
    pub photo_urls: Vec<String>,
}

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

    /// Get posts from Transparent Classroom
    ///
    /// Fetches a page of posts from the API. If page is 0, fetches the most recent posts.
    ///
    /// # Arguments
    ///
    /// * `page` - Page number to fetch (0-based)
    ///
    /// # Returns
    ///
    /// A list of posts from the specified page
    pub fn get_posts(&self, page: u32) -> Result<Vec<Post>, AppError> {
        debug!("Fetching posts page {}", page);

        // Construct URL for the posts page
        let posts_url = if page == 0 {
            format!("{}/observations", self.base_url)
        } else {
            format!("{}/observations?page={}", self.base_url, page)
        };

        // Send GET request
        debug!("Sending GET request to {}", posts_url);
        let response = self
            .http_client
            .get(&posts_url)
            .send()
            .map_err(|e| AppError::Generic(format!("Failed to fetch posts: {}", e)))?;

        if !response.status().is_success() {
            return Err(AppError::Generic(format!(
                "Failed to fetch posts. Status: {}",
                response.status()
            )));
        }

        // Get the response body
        let html = response
            .text()
            .map_err(|e| AppError::Generic(format!("Failed to read posts page content: {}", e)))?;

        // Parse the HTML and extract the posts
        self.parse_posts(&html, &posts_url)
    }

    /// Parse HTML to extract posts
    fn parse_posts(&self, html: &str, _base_url: &str) -> Result<Vec<Post>, AppError> {
        let document = Html::parse_document(html);
        let mut posts = Vec::new();

        // Try to find post elements
        let post_selector = Selector::parse(".observation").unwrap();

        for post_element in document.select(&post_selector) {
            let id = self
                .extract_attribute(&post_element, "id")
                .unwrap_or_else(|| format!("post_{}", posts.len()));

            // Extract title
            let title_selector = Selector::parse(".observation-text").unwrap();
            let title = match post_element.select(&title_selector).next() {
                Some(el) => el.text().collect::<Vec<_>>().join(" ").trim().to_string(),
                None => "Untitled Post".to_string(),
            };

            // Extract author
            let author_selector = Selector::parse(".observation-author").unwrap();
            let author = match post_element.select(&author_selector).next() {
                Some(el) => el.text().collect::<Vec<_>>().join(" ").trim().to_string(),
                None => "Unknown Author".to_string(),
            };

            // Extract date
            let date_selector = Selector::parse(".observation-date").unwrap();
            let date = match post_element.select(&date_selector).next() {
                Some(el) => el.text().collect::<Vec<_>>().join(" ").trim().to_string(),
                None => "Unknown Date".to_string(),
            };

            // Extract URL to the post
            let url_selector = Selector::parse("a.observation-link").unwrap();
            let url = match post_element.select(&url_selector).next() {
                Some(el) => {
                    if let Some(href) = el.value().attr("href") {
                        if href.starts_with("http") {
                            href.to_string()
                        } else {
                            // Handle relative URLs
                            let base_domain = self.base_url.split("/schools").next().unwrap_or("");
                            format!("{}{}", base_domain, href)
                        }
                    } else {
                        String::new()
                    }
                }
                None => String::new(),
            };

            // Extract photo URLs
            let photo_selector = Selector::parse(".observation-photo img").unwrap();
            let mut photo_urls = Vec::new();

            for photo_element in post_element.select(&photo_selector) {
                if let Some(src) = photo_element.value().attr("src") {
                    let photo_url = if src.starts_with("http") {
                        src.to_string()
                    } else {
                        // Handle relative URLs
                        let base_domain = self.base_url.split("/schools").next().unwrap_or("");
                        format!("{}{}", base_domain, src)
                    };
                    photo_urls.push(photo_url);
                }
            }

            // Create the post object
            let post = Post {
                id,
                title,
                author,
                date,
                url,
                photo_urls,
            };

            posts.push(post);
        }

        if posts.is_empty() {
            debug!("No posts found on the page");
        } else {
            debug!("Found {} posts", posts.len());
        }

        Ok(posts)
    }

    /// Helper to extract an attribute from a HTML element
    fn extract_attribute(&self, element: &scraper::ElementRef, attr_name: &str) -> Option<String> {
        element.value().attr(attr_name).map(|s| s.to_string())
    }
}
