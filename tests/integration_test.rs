// ABOUTME: This file contains end-to-end integration tests covering the complete photo download workflow
// ABOUTME: Tests include full login + post fetching + photo downloading + metadata creation scenarios

use mockito::Matcher;
use std::env;
use std::fs;
use tempfile::TempDir;
use transparent_classroom_photos_grabber_rs::{client::Client, config::Config, error::AppError};

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

// Helper function to create a test client with mockito server (copied from client_test.rs)
fn create_mock_client(server: &mockito::Server) -> Result<Client, AppError> {
    let config = create_test_config();
    let base_url = server.url();
    Client::new_with_base_url(config, base_url)
}

// Helper to isolate environment variable tests (copied from client_test.rs)
fn with_isolated_env<F, R>(test_fn: F) -> R
where
    F: FnOnce() -> R,
{
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let temp_path = temp_dir.path().to_path_buf();

    let env_vars = [
        "TC_EMAIL",
        "TC_PASSWORD",
        "SCHOOL",
        "CHILD",
        "DOTENV_PATH",
        "RUST_BACKTRACE",
        "RUST_LOG",
    ];

    let orig_values: Vec<(String, Option<String>)> = env_vars
        .iter()
        .map(|&name| (name.to_string(), env::var(name).ok()))
        .collect();

    for var in &env_vars {
        env::remove_var(var);
    }

    env::set_var("DOTENV_DISABLED", "1");
    env::set_var("DOTENV_PATH", temp_path.display().to_string());
    transparent_classroom_photos_grabber_rs::init();

    let result = test_fn();

    for (name, value) in orig_values {
        match value {
            Some(val) => env::set_var(name, val),
            None => env::remove_var(name),
        }
    }

    env::remove_var("DOTENV_DISABLED");
    result
}

#[test]
fn test_complete_workflow_login_posts_download() {
    with_isolated_env(|| {
        // Test the complete workflow: login -> get posts -> download photos
        let mut server = mockito::Server::new();

        // Mock web form authentication (using exact pattern from working test)
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

        let _signin_get_mock = server
            .mock("GET", "/souls/sign_in")
            .with_status(200)
            .with_header("content-type", "text/html")
            .with_body(signin_html)
            .create();

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

        let _signin_post_mock = server
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

        // Mock observations endpoint with sample posts
        let observations_html = r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Observations - Transparent Classroom</title>
        </head>
        <body>
            <div class="observations-container">
                <div class="observation" id="obs-123">
                    <div class="observation-text">Art Activity</div>
                    <div class="observation-author">Teacher Smith</div>
                    <div class="observation-date">Jan 15, 2023</div>
                    <a class="observation-link" href="/observations/123">View Details</a>
                    <div class="observation-photo">
                        <img src="/uploads/photos/art1.jpg" alt="Photo 1">
                    </div>
                    <div class="observation-photo">
                        <img src="/uploads/photos/art2.jpg" alt="Photo 2">
                    </div>
                </div>
                <div class="observation" id="obs-456">
                    <div class="observation-text">Outdoor Play</div>
                    <div class="observation-author">Teacher Jones</div>
                    <div class="observation-date">Jan 16, 2023</div>
                    <a class="observation-link" href="/observations/456">View Details</a>
                    <div class="observation-photo">
                        <img src="/uploads/photos/outdoor1.jpg" alt="Photo 3">
                    </div>
                </div>
            </div>
        </body>
        </html>
        "#;

        let observations_mock = server
            .mock("GET", "/observations")
            .with_status(200)
            .with_header("content-type", "text/html")
            .with_body(observations_html)
            .create();

        // Mock photo downloads
        let photo_bytes_1 = b"FAKE_ART_PHOTO_1_DATA";
        let photo_bytes_2 = b"FAKE_ART_PHOTO_2_DATA";
        let photo_bytes_3 = b"FAKE_OUTDOOR_PHOTO_DATA";

        let photo1_mock = server
            .mock("GET", "/uploads/photos/art1.jpg")
            .with_status(200)
            .with_header("content-type", "image/jpeg")
            .with_body(photo_bytes_1.as_slice())
            .create();

        let photo2_mock = server
            .mock("GET", "/uploads/photos/art2.jpg")
            .with_status(200)
            .with_header("content-type", "image/jpeg")
            .with_body(photo_bytes_2.as_slice())
            .create();

        let photo3_mock = server
            .mock("GET", "/uploads/photos/outdoor1.jpg")
            .with_status(200)
            .with_header("content-type", "image/jpeg")
            .with_body(photo_bytes_3.as_slice())
            .create();

        // Create client and temp directory
        let client = create_mock_client(&server).expect("Failed to create mock client");
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        // Execute complete workflow

        // Step 1: Login
        let login_result = client.login();
        assert!(
            login_result.is_ok(),
            "Login should succeed: {:?}",
            login_result.err()
        );

        // Step 2: Get posts
        let posts_result = client.get_posts(0);
        assert!(
            posts_result.is_ok(),
            "Get posts should succeed: {:?}",
            posts_result.err()
        );

        let posts = posts_result.unwrap();
        assert_eq!(posts.len(), 2, "Should have 2 posts");

        // Verify post data (parsed from HTML)
        assert_eq!(posts[0].id, "obs-123");
        assert_eq!(posts[0].title, "Art Activity");
        assert_eq!(posts[0].author, "Teacher Smith");
        assert_eq!(posts[0].photo_urls.len(), 2);

        assert_eq!(posts[1].id, "obs-456");
        assert_eq!(posts[1].title, "Outdoor Play");
        assert_eq!(posts[1].author, "Teacher Jones");
        assert_eq!(posts[1].photo_urls.len(), 1);

        // Step 3: Download all photos from first post
        let download_result = client.download_all_photos(&posts[0], temp_dir.path());
        assert!(
            download_result.is_ok(),
            "Download all photos should succeed: {:?}",
            download_result.err()
        );

        let downloaded_paths = download_result.unwrap();
        assert_eq!(downloaded_paths.len(), 2, "Should download 2 photos");

        // Verify photo files exist and have correct content
        let photo1_content = fs::read(&downloaded_paths[0]).expect("Should read photo 1");
        assert_eq!(
            photo1_content, photo_bytes_1,
            "Photo 1 content should match"
        );

        let photo2_content = fs::read(&downloaded_paths[1]).expect("Should read photo 2");
        assert_eq!(
            photo2_content, photo_bytes_2,
            "Photo 2 content should match"
        );

        // Verify metadata files exist
        for path in &downloaded_paths {
            let metadata_path = path.with_extension("metadata.txt");
            assert!(
                metadata_path.exists(),
                "Metadata file should exist for {:?}",
                path
            );

            let metadata_content =
                fs::read_to_string(&metadata_path).expect("Should read metadata");
            assert!(
                metadata_content.contains("Art Activity"),
                "Metadata should contain post title"
            );
            assert!(
                metadata_content.contains("Teacher Smith"),
                "Metadata should contain author"
            );
        }

        // Step 4: Download photo from second post
        let single_download_result = client.download_photo(&posts[1], 0, temp_dir.path());
        assert!(
            single_download_result.is_ok(),
            "Single photo download should succeed: {:?}",
            single_download_result.err()
        );

        let single_photo_path = single_download_result.unwrap();
        let photo3_content = fs::read(&single_photo_path).expect("Should read photo 3");
        assert_eq!(
            photo3_content, photo_bytes_3,
            "Photo 3 content should match"
        );

        // Verify all mocks were called
        observations_mock.assert();
        photo1_mock.assert();
        photo2_mock.assert();
        photo3_mock.assert();
    });
}

#[test]
fn test_workflow_resilience_to_partial_failures() {
    with_isolated_env(|| {
        // Test that workflow continues gracefully when some photos fail to download
        let mut server = mockito::Server::new();

        // Mock successful authentication
        let signin_html = r#"
        <!DOCTYPE html>
        <html>
        <head>
            <meta name="csrf-token" content="test_csrf_token_12345" />
        </head>
        <body>
            <form action="/souls/sign_in" method="post">
                <input type="hidden" name="authenticity_token" value="test_csrf_token_12345" />
            </form>
        </body>
        </html>
        "#;

        let _signin_get_mock = server
            .mock("GET", "/souls/sign_in")
            .with_status(200)
            .with_header("content-type", "text/html")
            .with_body(signin_html)
            .create();

        let _signin_post_mock = server
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
            .with_body("<!DOCTYPE html><html><body><h1>Dashboard</h1></body></html>")
            .create();

        // Mock posts with multiple photos
        let observations_html = r#"
        <!DOCTYPE html>
        <html>
        <body>
            <div class="observations-container">
                <div class="observation" id="mixed-success-post">
                    <div class="observation-text">Mixed Success Test</div>
                    <div class="observation-author">Teacher Test</div>
                    <div class="observation-date">Jan 20, 2023</div>
                    <a class="observation-link" href="/observations/mixed">View Details</a>
                    <div class="observation-photo">
                        <img src="/uploads/photos/success.jpg" alt="Success Photo">
                    </div>
                    <div class="observation-photo">
                        <img src="/uploads/photos/failure.jpg" alt="Failure Photo">
                    </div>
                    <div class="observation-photo">
                        <img src="/uploads/photos/success2.jpg" alt="Success Photo 2">
                    </div>
                </div>
            </div>
        </body>
        </html>
        "#;

        let _observations_mock = server
            .mock("GET", "/observations")
            .with_status(200)
            .with_header("content-type", "text/html")
            .with_body(observations_html)
            .create();

        // Mock photo downloads: some succeed, some fail
        let success_bytes = b"SUCCESS_PHOTO_DATA";

        let _photo_success1 = server
            .mock("GET", "/uploads/photos/success.jpg")
            .with_status(200)
            .with_header("content-type", "image/jpeg")
            .with_body(success_bytes.as_slice())
            .create();

        let _photo_failure = server
            .mock("GET", "/uploads/photos/failure.jpg")
            .with_status(404)
            .with_header("content-type", "text/html")
            .with_body("Not Found")
            .create();

        let _photo_success2 = server
            .mock("GET", "/uploads/photos/success2.jpg")
            .with_status(200)
            .with_header("content-type", "image/jpeg")
            .with_body(success_bytes.as_slice())
            .create();

        // Create client and execute workflow
        let client = create_mock_client(&server).expect("Failed to create mock client");
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        client.login().expect("Login should succeed");
        let posts = client.get_posts(0).expect("Get posts should succeed");
        assert_eq!(posts.len(), 1);

        // Test individual photo downloads (some should succeed, some fail)
        let result1 = client.download_photo(&posts[0], 0, temp_dir.path());
        assert!(result1.is_ok(), "First photo should succeed");

        let result2 = client.download_photo(&posts[0], 1, temp_dir.path());
        assert!(result2.is_err(), "Second photo should fail");

        let result3 = client.download_photo(&posts[0], 2, temp_dir.path());
        assert!(result3.is_ok(), "Third photo should succeed");

        // Verify successful downloads exist
        if let Ok(path1) = result1 {
            assert!(path1.exists(), "First photo file should exist");
            let content = fs::read(path1).expect("Should read first photo");
            assert_eq!(content, success_bytes);
        }

        if let Ok(path3) = result3 {
            assert!(path3.exists(), "Third photo file should exist");
            let content = fs::read(path3).expect("Should read third photo");
            assert_eq!(content, success_bytes);
        }
    });
}
