// ABOUTME: This file contains comprehensive error handling tests for network failures, file I/O errors, and edge cases
// ABOUTME: Tests cover authentication failures, download errors, filesystem issues, and recovery scenarios

use std::fs;
use std::path::Path;
use tempfile::TempDir;
use transparent_classroom_photos_grabber_rs::{
    client::{Client, Post},
    config::Config,
};

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

#[test]
fn test_network_timeout_during_login() {
    let mut server = mockito::Server::new();

    // Mock API endpoint that times out
    let api_endpoint = format!("/api/v1/children/{}", 67890);
    let _api_mock = server
        .mock("GET", api_endpoint.as_str())
        .with_status(408) // Request timeout
        .with_header("content-type", "application/json")
        .with_body(r#"{"error": "Request timeout"}"#)
        .create();

    // Mock web signin to also timeout
    let _signin_mock = server
        .mock("GET", "/souls/sign_in?locale=en")
        .with_status(408)
        .with_body("Request timeout")
        .create();

    let config = create_test_config();
    let client = Client::new_with_base_url(config, server.url()).unwrap();

    // Login should fail gracefully when both auth methods timeout
    let result = client.login();
    assert!(
        result.is_err(),
        "Login should fail when both auth methods timeout"
    );

    // Verify we get a meaningful error message
    if let Err(error) = result {
        let error_msg = format!("{:?}", error);
        assert!(
            error_msg.contains("Failed to authenticate"),
            "Error should mention authentication failure"
        );
    }
}

#[test]
fn test_server_error_during_photo_download() {
    let mut server = mockito::Server::new();

    // Mock successful login
    let signin_html = r#"<!DOCTYPE html><html><head><meta name="csrf-token" content="token" /></head><body></body></html>"#;
    let _signin_get_mock = server
        .mock("GET", "/souls/sign_in?locale=en")
        .with_status(200)
        .with_header("content-type", "text/html")
        .with_body(signin_html)
        .create();

    let _signin_post_mock = server
        .mock("POST", "/souls/sign_in")
        .with_status(200)
        .with_header("content-type", "text/html")
        .with_body("<!DOCTYPE html><html><body><h1>Dashboard</h1></body></html>")
        .create();

    // Mock photo URL that returns server error
    let photo_path = "/uploads/photos/123.jpg";
    let _photo_mock = server
        .mock("GET", photo_path)
        .with_status(500) // Internal server error
        .with_header("content-type", "text/html")
        .with_body("Internal Server Error")
        .create();

    let config = create_test_config();
    let client = Client::new_with_base_url(config, server.url()).unwrap();
    client.login().expect("Login should succeed");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let post = Post {
        id: "error-test".to_string(),
        title: "Error Test Post".to_string(),
        author: "Test Author".to_string(),
        date: "Jan 20, 2023".to_string(),
        url: format!("{}/posts/123", server.url()),
        photo_urls: vec![format!("{}{}", server.url(), photo_path)],
    };

    // Photo download should fail
    let result = client.download_photo(&post, 0, temp_dir.path());
    assert!(
        result.is_err(),
        "Photo download should fail with server error"
    );
}

#[test]
fn test_invalid_photo_url_format() {
    let mut server = mockito::Server::new();

    // Mock successful login
    let signin_html = r#"<!DOCTYPE html><html><head><meta name="csrf-token" content="token" /></head><body></body></html>"#;
    let _signin_get_mock = server
        .mock("GET", "/souls/sign_in?locale=en")
        .with_status(200)
        .with_header("content-type", "text/html")
        .with_body(signin_html)
        .create();

    let _signin_post_mock = server
        .mock("POST", "/souls/sign_in")
        .with_status(200)
        .with_header("content-type", "text/html")
        .with_body("<!DOCTYPE html><html><body><h1>Dashboard</h1></body></html>")
        .create();

    let config = create_test_config();
    let client = Client::new_with_base_url(config, server.url()).unwrap();
    client.login().expect("Login should succeed");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let post = Post {
        id: "invalid-url-test".to_string(),
        title: "Invalid URL Test".to_string(),
        author: "Test Author".to_string(),
        date: "Jan 20, 2023".to_string(),
        url: format!("{}/posts/123", server.url()),
        photo_urls: vec!["not-a-valid-url".to_string()], // Invalid URL
    };

    // Photo download should handle invalid URL gracefully
    let result = client.download_photo(&post, 0, temp_dir.path());
    assert!(
        result.is_err(),
        "Photo download should fail with invalid URL"
    );
}

#[test]
fn test_filesystem_write_permission_error() {
    let mut server = mockito::Server::new();

    // Mock successful login and photo response
    let signin_html = r#"<!DOCTYPE html><html><head><meta name="csrf-token" content="token" /></head><body></body></html>"#;
    let _signin_get_mock = server
        .mock("GET", "/souls/sign_in?locale=en")
        .with_status(200)
        .with_header("content-type", "text/html")
        .with_body(signin_html)
        .create();

    let _signin_post_mock = server
        .mock("POST", "/souls/sign_in")
        .with_status(200)
        .with_header("content-type", "text/html")
        .with_body("<!DOCTYPE html><html><body><h1>Dashboard</h1></body></html>")
        .create();

    let photo_path = "/uploads/photos/123.jpg";
    let photo_bytes = b"FAKE_IMAGE_DATA_FOR_TESTING";
    let _photo_mock = server
        .mock("GET", photo_path)
        .with_status(200)
        .with_header("content-type", "image/jpeg")
        .with_body(photo_bytes.as_slice())
        .create();

    let config = create_test_config();
    let client = Client::new_with_base_url(config, server.url()).unwrap();
    client.login().expect("Login should succeed");

    // Use /dev/null (read-only) as output directory to simulate permission error
    let readonly_path = Path::new("/dev/null");
    let post = Post {
        id: "perm-test".to_string(),
        title: "Permission Test".to_string(),
        author: "Test Author".to_string(),
        date: "Jan 20, 2023".to_string(),
        url: format!("{}/posts/123", server.url()),
        photo_urls: vec![format!("{}{}", server.url(), photo_path)],
    };

    // Photo download should fail due to filesystem permission error
    let result = client.download_photo(&post, 0, readonly_path);
    assert!(
        result.is_err(),
        "Photo download should fail with permission error"
    );
}

#[test]
fn test_malformed_html_response() {
    let mut server = mockito::Server::new();

    // Mock API failure
    let api_endpoint = format!("/api/v1/children/{}", 67890);
    let _api_mock = server
        .mock("GET", api_endpoint.as_str())
        .with_status(401)
        .with_header("content-type", "application/json")
        .with_body(r#"{"error": "Unauthorized"}"#)
        .create();

    // Mock malformed HTML response (missing CSRF token, malformed structure)
    let malformed_html = r#"
    <html><head><title>Broken Page</title></head>
    <body>
    <!-- No CSRF token -->
    <!-- No proper form structure -->
    <div>Broken content</div>
    </body></html>
    "#;

    let _signin_get_mock = server
        .mock("GET", "/souls/sign_in?locale=en")
        .with_status(200)
        .with_header("content-type", "text/html")
        .with_body(malformed_html)
        .create();

    let config = create_test_config();
    let client = Client::new_with_base_url(config, server.url()).unwrap();

    // Login should fail gracefully when HTML is malformed and API auth fails
    let result = client.login();
    assert!(
        result.is_err(),
        "Login should fail with malformed HTML response"
    );

    // Verify we get a meaningful error message
    if let Err(error) = result {
        let error_msg = format!("{:?}", error);
        assert!(
            error_msg.contains("Failed to authenticate"),
            "Error should mention authentication failure"
        );
    }
}

#[test]
fn test_posts_authentication_error_handling() {
    let mut server = mockito::Server::new();

    // Mock successful API auth for login
    let api_endpoint = format!("/api/v1/children/{}", 67890);
    let api_response = r#"{ "id": 67890, "name": "Test Child", "status": "active" }"#;
    let _api_mock = server
        .mock("GET", api_endpoint.as_str())
        .match_header(
            "authorization",
            mockito::Matcher::Regex("Basic.*".to_string()),
        )
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(api_response)
        .create();

    // Mock posts endpoint to return 401 (authentication error)
    let posts_endpoint = format!("/s/{}/children/{}/posts.json?locale=en", 12345, 67890);
    let _posts_mock = server
        .mock("GET", posts_endpoint.as_str())
        .with_status(401)
        .with_header("content-type", "application/json")
        .with_body(r#"{"error": "Unauthorized"}"#)
        .create();

    // Also mock fallback endpoints to fail
    let _observations_mock = server
        .mock("GET", "/observations")
        .with_status(401)
        .with_header("content-type", "text/html")
        .with_body("Unauthorized")
        .create();

    let config = create_test_config();
    let client = Client::new_with_base_url(config, server.url()).unwrap();
    client.login().expect("Login should succeed");

    // Get posts should fail gracefully with meaningful error
    let result = client.get_posts(0);
    assert!(
        result.is_err(),
        "Getting posts should fail with authentication error"
    );

    // Verify we get a meaningful error message about authentication
    if let Err(error) = result {
        let error_msg = format!("{:?}", error);
        assert!(
            error_msg.contains("authentication") || error_msg.contains("Unauthorized"),
            "Error should mention authentication issue: {}",
            error_msg
        );
    }
}

#[test]
fn test_corrupt_photo_data() {
    let mut server = mockito::Server::new();

    // Mock successful login
    let signin_html = r#"<!DOCTYPE html><html><head><meta name="csrf-token" content="token" /></head><body></body></html>"#;
    let _signin_get_mock = server
        .mock("GET", "/souls/sign_in?locale=en")
        .with_status(200)
        .with_header("content-type", "text/html")
        .with_body(signin_html)
        .create();

    let _signin_post_mock = server
        .mock("POST", "/souls/sign_in")
        .with_status(200)
        .with_header("content-type", "text/html")
        .with_body("<!DOCTYPE html><html><body><h1>Dashboard</h1></body></html>")
        .create();

    // Mock photo URL that returns corrupt/incomplete data
    let photo_path = "/uploads/photos/123.jpg";
    let corrupt_data = b"CORRUPT_DATA_NOT_VALID_IMAGE"; // Not a valid image format
    let _photo_mock = server
        .mock("GET", photo_path)
        .with_status(200)
        .with_header("content-type", "image/jpeg") // Claims to be JPEG but isn't
        .with_body(corrupt_data.as_slice())
        .create();

    let config = create_test_config();
    let client = Client::new_with_base_url(config, server.url()).unwrap();
    client.login().expect("Login should succeed");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let post = Post {
        id: "corrupt-test".to_string(),
        title: "Corrupt Data Test".to_string(),
        author: "Test Author".to_string(),
        date: "Jan 20, 2023".to_string(),
        url: format!("{}/posts/123", server.url()),
        photo_urls: vec![format!("{}{}", server.url(), photo_path)],
    };

    // Photo download should still succeed (we don't validate image format, just download bytes)
    let result = client.download_photo(&post, 0, temp_dir.path());
    assert!(
        result.is_ok(),
        "Photo download should succeed even with corrupt data: {:?}",
        result.err()
    );

    // Verify the corrupt data was written to file
    let downloaded_path = result.unwrap();
    let content = fs::read(&downloaded_path).expect("Should be able to read downloaded file");
    assert_eq!(content, corrupt_data, "Should write corrupt data as-is");
}

#[test]
fn test_large_number_of_photos_in_single_post() {
    let mut server = mockito::Server::new();

    // Mock successful login
    let signin_html = r#"<!DOCTYPE html><html><head><meta name="csrf-token" content="token" /></head><body></body></html>"#;
    let _signin_get_mock = server
        .mock("GET", "/souls/sign_in?locale=en")
        .with_status(200)
        .with_header("content-type", "text/html")
        .with_body(signin_html)
        .create();

    let _signin_post_mock = server
        .mock("POST", "/souls/sign_in")
        .with_status(200)
        .with_header("content-type", "text/html")
        .with_body("<!DOCTYPE html><html><body><h1>Dashboard</h1></body></html>")
        .create();

    // Create many photo URLs to test handling of large numbers
    let num_photos = 20; // Reduced from 50 to speed up test
    let photo_bytes = b"FAKE_IMAGE_DATA";

    // Mock all photo endpoints
    for i in 0..num_photos {
        let photo_path = format!("/uploads/photos/{}.jpg", i);
        let _photo_mock = server
            .mock("GET", photo_path.as_str())
            .with_status(200)
            .with_header("content-type", "image/jpeg")
            .with_body(photo_bytes.as_slice())
            .create();
    }

    let config = create_test_config();
    let client = Client::new_with_base_url(config, server.url()).unwrap();
    client.login().expect("Login should succeed");

    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let photo_urls: Vec<String> = (0..num_photos)
        .map(|i| format!("{}/uploads/photos/{}.jpg", server.url(), i))
        .collect();

    let post = Post {
        id: "many-photos-test".to_string(),
        title: "Many Photos Test".to_string(),
        author: "Test Author".to_string(),
        date: "Jan 20, 2023".to_string(),
        url: format!("{}/posts/123", server.url()),
        photo_urls,
    };

    // Download all photos should handle large numbers
    let result = client.download_all_photos(&post, temp_dir.path());
    assert!(
        result.is_ok(),
        "Should handle downloading many photos: {:?}",
        result.err()
    );

    let downloaded_paths = result.unwrap();
    assert_eq!(
        downloaded_paths.len(),
        num_photos,
        "Should download all photos"
    );

    // Verify all files exist
    for path in downloaded_paths {
        assert!(path.exists(), "All photos should be downloaded");
    }
}
