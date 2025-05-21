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
    }
}

// Helper function to create a test configuration with mockito server
fn create_mock_client(server: &mockito::Server) -> Result<Client, AppError> {
    let config = Config {
        email: "test@example.com".to_string(),
        password: "password123".to_string(),
        school_id: 12345,
        child_id: 67890,
    };

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

#[test]
fn test_login_flow() {
    with_isolated_env(|| {
        // Create a mock server
        let mut server = mockito::Server::new();

        // 1. Mock the sign-in page GET request that returns a page with CSRF token
        let signin_html = r#"
        <!DOCTYPE html>
        <html>
        <head>
            <meta name="csrf-token" content="test_csrf_token_12345" />
        </head>
        <body>
            <form action="/souls/sign_in" method="post">
                <input type="hidden" name="authenticity_token" value="test_csrf_token_12345" />
                <input type="text" name="soul[email]" />
                <input type="password" name="soul[password]" />
                <input type="submit" name="commit" value="Sign In" />
            </form>
        </body>
        </html>
        "#;

        let signin_get_mock = server
            .mock("GET", "/souls/sign_in")
            .with_status(200)
            .with_header("content-type", "text/html")
            .with_body(signin_html)
            .create();

        // 2. Mock the sign-in POST request that would process the login
        let dashboard_html = r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Dashboard - Transparent Classroom</title>
        </head>
        <body>
            <h1>Dashboard</h1>
            <div class="welcome">Welcome, User!</div>
            <div class="main-content">
                <!-- Dashboard content would go here -->
            </div>
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
                Matcher::Regex("soul%5Bemail%5D=test%40example.com".to_string()),
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

        // Verify the login was successful
        assert!(result.is_ok(), "Login failed: {:?}", result.err());

        // Verify that our mock endpoints were called
        signin_get_mock.assert();
        signin_post_mock.assert();
    });
}

#[test]
fn test_login_failure_invalid_credentials() {
    with_isolated_env(|| {
        // Create a mock server
        let mut server = mockito::Server::new();

        // Mock the sign-in page GET request that returns a page with CSRF token
        let signin_html = r#"
        <!DOCTYPE html>
        <html>
        <head>
            <meta name="csrf-token" content="test_csrf_token_12345" />
        </head>
        <body>
            <form action="/souls/sign_in" method="post">
                <input type="hidden" name="authenticity_token" value="test_csrf_token_12345" />
                <input type="text" name="soul[email]" />
                <input type="password" name="soul[password]" />
                <input type="submit" name="commit" value="Sign In" />
            </form>
        </body>
        </html>
        "#;

        let signin_get_mock = server
            .mock("GET", "/souls/sign_in")
            .with_status(200)
            .with_header("content-type", "text/html")
            .with_body(signin_html)
            .create();

        // Mock a failed login response
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

        // Verify the login failed with the expected error
        assert!(result.is_err(), "Login should have failed but succeeded");
        if let Err(err) = result {
            assert!(err.to_string().contains("Invalid email or password"));
        }

        // Verify that our mock endpoints were called
        signin_get_mock.assert();
        signin_post_mock.assert();
    });
}

#[test]
fn test_login_failure_no_csrf_token() {
    with_isolated_env(|| {
        // Create a mock server
        let mut server = mockito::Server::new();

        // Mock the sign-in page GET request that returns a page WITHOUT a CSRF token
        let signin_html = r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Sign In - Transparent Classroom</title>
        </head>
        <body>
            <form action="/souls/sign_in" method="post">
                <!-- Missing CSRF token -->
                <input type="text" name="soul[email]" />
                <input type="password" name="soul[password]" />
                <input type="submit" name="commit" value="Sign In" />
            </form>
        </body>
        </html>
        "#;

        let signin_get_mock = server
            .mock("GET", "/souls/sign_in")
            .with_status(200)
            .with_header("content-type", "text/html")
            .with_body(signin_html)
            .create();

        // Create the client with mockito server URL
        let client = create_mock_client(&server).expect("Failed to create mock client");

        // Execute the login flow
        let result = client.login();

        // Verify the login failed with the expected error
        assert!(result.is_err(), "Login should have failed but succeeded");
        if let Err(err) = result {
            assert!(err.to_string().contains("Could not find CSRF token"));
        }

        // Verify that our mock endpoint was called
        signin_get_mock.assert();
    });
}
