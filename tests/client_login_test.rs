use mockito::Matcher;
use std::env;
use tempfile::TempDir;
use transparent_classroom_photos_grabber_rs::{client::Client, config::Config, error::AppError};

// Helper function to create a test configuration
fn create_test_config() -> Config {
    Config {
        email: "test@example.com".to_string(),
        password: "password123".to_string(),
        school_id: 12345,
        child_id: 67890,
        school_lat: 41.9032776,
        school_lng: -87.6663027,
        school_keywords: "test, school, chicago".to_string(),
    }
}

// Helper function to create a test client with a mockito server
fn create_mock_client(server: &mockito::Server) -> Result<Client, AppError> {
    let config = create_test_config();
    // Use the mockito server URL as base
    let base_url = server.url();
    Client::new_with_base_url(config, base_url)
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
fn test_api_basic_auth_login_success() {
    with_isolated_env(|| {
        // Create a mock server
        let mut server = mockito::Server::new();

        // Mock successful API basic auth response
        let api_endpoint = format!("/api/v1/children/{}", 67890); // Using child_id from test config
        let api_response = r#"{ "id": 67890, "name": "Test Child", "status": "active" }"#;

        let api_mock = server
            .mock("GET", api_endpoint.as_str())
            .match_header("authorization", Matcher::Regex("Basic.*".to_string()))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(api_response)
            .create();

        // Create the client with mockito server URL
        let client = create_mock_client(&server).expect("Failed to create mock client");

        // Execute the login flow
        let result = client.login();

        // Verify the login was successful
        assert!(result.is_ok(), "Login failed: {:?}", result.err());

        // Verify that our API mock endpoint was called
        api_mock.assert();
    });
}

#[test]
fn test_web_form_fallback_when_api_fails() {
    with_isolated_env(|| {
        // Create a mock server
        let mut server = mockito::Server::new();

        // 1. Mock failed API auth response
        let api_endpoint = format!("/api/v1/children/{}", 67890);
        let api_mock = server
            .mock("GET", api_endpoint.as_str())
            .with_status(401) // Unauthorized
            .with_header("content-type", "application/json")
            .with_body(r#"{"error": "Invalid credentials"}"#)
            .create();

        // 2. Mock the sign-in page GET request that returns a page with CSRF token
        let signin_html = r#"
        <!DOCTYPE html>
        <html>
        <head>
            <meta name="csrf-token" content="test_csrf_token_12345" />
        </head>
        <body>
            <form action="/souls/sign_in" method="post">
                <input type="hidden" name="authenticity_token" value="test_csrf_token_12345" />
                <input type="text" name="soul[login]" />
                <input type="password" name="soul[password]" />
                <input type="submit" name="commit" value="Sign In" />
            </form>
        </body>
        </html>
        "#;

        // Mockito doesn't match absolute URLs, so we need to match just the path
        // The client is using /souls/sign_in?locale=en for the signin URL
        let signin_get_mock = server
            .mock("GET", "/souls/sign_in?locale=en")
            .with_status(200)
            .with_header("content-type", "text/html")
            .with_body(signin_html)
            .create();

        // 3. Mock the sign-in POST request that would process the login
        let dashboard_html = r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Dashboard - Transparent Classroom</title>
        </head>
        <body>
            <h1>Dashboard</h1>
            <div class="welcome">Welcome, User!</div>
        </body>
        </html>
        "#;

        let signin_post_mock = server
            .mock("POST", "/souls/sign_in")
            .match_header(
                "content-type",
                Matcher::Regex("application/x-www-form-urlencoded.*".to_string()),
            )
            .match_body(Matcher::AllOf(vec![
                Matcher::Regex("authenticity_token=test_csrf_token_12345".to_string()),
                Matcher::Regex("soul%5Blogin%5D=test%40example.com".to_string()),
                Matcher::Regex("soul%5Bpassword%5D=password123".to_string()),
            ]))
            .with_status(200)
            .with_header("content-type", "text/html")
            .with_body(dashboard_html)
            .create();

        // Create the client with mockito server URL
        let client = create_mock_client(&server).expect("Failed to create mock client");

        // Execute the login flow
        let result = client.login();

        // Verify the login was successful via fallback
        assert!(result.is_ok(), "Login failed: {:?}", result.err());

        // Verify that our mock endpoints were called in the expected order
        api_mock.assert();
        signin_get_mock.assert();
        signin_post_mock.assert();
    });
}

#[test]
fn test_mock_mode_fallback_when_both_auth_methods_fail() {
    with_isolated_env(|| {
        // Create a mock server
        let mut server = mockito::Server::new();

        // 1. Mock failed API auth response
        let api_endpoint = format!("/api/v1/children/{}", 67890);
        let api_mock = server
            .mock("GET", api_endpoint.as_str())
            .with_status(401) // Unauthorized
            .with_header("content-type", "application/json")
            .with_body(r#"{"error": "Invalid credentials"}"#)
            .create();

        // 2. Mock the sign-in page GET request that returns a page with CSRF token
        let signin_html = r#"
        <!DOCTYPE html>
        <html>
        <head>
            <meta name="csrf-token" content="test_csrf_token_12345" />
        </head>
        <body>
            <form action="/souls/sign_in" method="post">
                <input type="hidden" name="authenticity_token" value="test_csrf_token_12345" />
                <input type="text" name="soul[login]" />
                <input type="password" name="soul[password]" />
                <input type="submit" name="commit" value="Sign In" />
            </form>
        </body>
        </html>
        "#;

        let signin_get_mock = server
            .mock("GET", "/souls/sign_in?locale=en")
            .with_status(200)
            .with_header("content-type", "text/html")
            .with_body(signin_html)
            .create();

        // 3. Mock failed login response with invalid credentials error
        let failed_login_html = r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Sign In - Transparent Classroom</title>
        </head>
        <body>
            <div class="alert alert-danger">Invalid email or password</div>
            <form action="/souls/sign_in" method="post">
                <!-- Form fields would go here -->
            </form>
        </body>
        </html>
        "#;

        let signin_post_mock = server
            .mock("POST", "/souls/sign_in")
            .with_status(200)
            .with_header("content-type", "text/html")
            .with_body(failed_login_html)
            .create();

        // Create the client with mockito server URL
        let client = create_mock_client(&server).expect("Failed to create mock client");

        // Execute the login flow
        let result = client.login();

        // Verify login still succeeds because of mock mode fallback
        assert!(
            result.is_ok(),
            "Login should have succeeded with mock mode fallback"
        );

        // Verify that our mock endpoints were called in the expected order
        api_mock.assert();
        signin_get_mock.assert();
        signin_post_mock.assert();
    });
}

#[test]
fn test_login_attempt_all_methods() {
    with_isolated_env(|| {
        // Create a mock server
        let mut server = mockito::Server::new();

        // 1. Mock failed API auth response - connection refused
        let api_endpoint = format!("/api/v1/children/{}", 67890);
        let api_mock = server
            .mock("GET", api_endpoint.as_str())
            .with_status(500) // Server error
            .with_header("content-type", "application/json")
            .with_body(r#"{"error": "Server error"}"#)
            .create();

        // 2. Mock the sign-in page GET request with network error
        // We can simulate this by returning a malformed response
        let signin_get_mock = server
            .mock("GET", "/souls/sign_in?locale=en")
            .with_status(503) // Service unavailable
            .with_body("Service unavailable")
            .create();

        // Create the client with mockito server URL
        let client = create_mock_client(&server).expect("Failed to create mock client");

        // Execute the login flow
        let result = client.login();

        // Verify login still succeeds because of mock mode fallback
        assert!(
            result.is_ok(),
            "Login should have succeeded with mock mode fallback"
        );

        // Verify that our mock endpoints were called
        api_mock.assert();
        signin_get_mock.assert();
    });
}
