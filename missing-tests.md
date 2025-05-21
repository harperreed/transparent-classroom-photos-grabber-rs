I'll carefully analyze the code to identify missing test cases for the transparent-classroom-photos-grabber-rs project. Looking at the existing code and tests, I'll identify gaps in test coverage that should be addressed.

## Missing Test Cases

### Issue 1: Missing Tests for `sanitize_dirname` in bin/main.rs

```markdown
## Missing Tests for `sanitize_dirname` in bin/main.rs

The `sanitize_dirname` function in `src/bin/main.rs` is critical for creating safe directory names for downloaded photos, but it lacks dedicated tests.

### What to Test:
- Test that spaces are correctly replaced with underscores
- Test that illegal characters (/, \, :, *, ?, ", <, >, |, ') are stripped
- Test that long names are properly truncated to 30 characters
- Test with empty strings and strings containing only illegal characters
- Test with edge cases like Unicode characters and emojis

### Implementation Example:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_dirname_spaces() {
        assert_eq!(sanitize_dirname("test name"), "test_name");
        assert_eq!(sanitize_dirname("  leading trailing  "), "leading_trailing");
    }

    #[test]
    fn test_sanitize_dirname_illegal_chars() {
        assert_eq!(sanitize_dirname("file/name:with*illegal?chars"), "filenamewithlegalchars");
        assert_eq!(sanitize_dirname("\"quotes'and<other>|chars\\"), "quotesandotherchars");
    }

    #[test]
    fn test_sanitize_dirname_length() {
        let long_name = "This is a very long directory name that should be truncated to thirty characters only";
        let result = sanitize_dirname(long_name);
        assert_eq!(result.len(), 30);
        assert_eq!(result, "This_is_a_very_long_directory_");
    }

    #[test]
    fn test_sanitize_dirname_edge_cases() {
        assert_eq!(sanitize_dirname(""), "");
        assert_eq!(sanitize_dirname("/:*?\"<>|'"), "");
        // Test with Unicode
        assert_eq!(sanitize_dirname("café"), "café");
    }
}
```
```

### Issue 2: Missing Tests for `run` Function in main.rs

```markdown
## Missing Tests for `run` Function in bin/main.rs

The `run` function in `src/bin/main.rs` is the main workflow coordinator but lacks integration tests.

### What to Test:
- Test successful execution with valid configuration and mocked API responses
- Test handling of empty posts (no photos to download)
- Test handling of API errors during post fetching
- Test error handling during photo downloads
- Test directory creation for output

### Implementation Example:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Matcher;
    use std::path::PathBuf;
    use tempfile::TempDir;
    use transparent_classroom_photos_grabber_rs::{client::Post, config::Config};

    // Helper to set up environment for testing
    fn setup_test_env() -> (mockito::Server, PathBuf) {
        let server = mockito::Server::new();
        let temp_dir = TempDir::new().unwrap();

        // Set up env vars to point to mock server
        std::env::set_var("TC_EMAIL", "test@example.com");
        std::env::set_var("TC_PASSWORD", "password123");
        std::env::set_var("SCHOOL", "12345");
        std::env::set_var("CHILD", "67890");

        (server, temp_dir.path().to_path_buf())
    }

    #[test]
    fn test_run_successful_flow() {
        let (mut server, output_dir) = setup_test_env();

        // Mock login endpoint
        let _signin_mock = server.mock("GET", "/souls/sign_in")
            .with_status(200)
            .with_body(r#"<html><head><meta name="csrf-token" content="token" /></head></html>"#)
            .create();

        let _signin_post_mock = server.mock("POST", "/souls/sign_in")
            .with_status(200)
            .with_body(r#"<html><body><h1>Dashboard</h1></body></html>"#)
            .create();

        // Mock posts endpoint
        let _posts_mock = server.mock("GET", "/observations")
            .with_status(200)
            .with_body(r#"<html><body>
                <div class="observation" id="obs-123">
                    <div class="observation-text">Test Post</div>
                    <div class="observation-author">Author</div>
                    <div class="observation-date">2023-01-01</div>
                    <a class="observation-link" href="/observations/123">View</a>
                    <div class="observation-photo">
                        <img src="/uploads/photo1.jpg">
                    </div>
                </div>
            </body></html>"#)
            .create();

        // Mock photo download
        let _photo_mock = server.mock("GET", "/uploads/photo1.jpg")
            .with_status(200)
            .with_body("test photo data")
            .create();

        // Test the run function
        let result = run(output_dir);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_no_posts() {
        let (mut server, output_dir) = setup_test_env();

        // Mock login endpoints
        let _signin_mock = server.mock("GET", "/souls/sign_in")
            .with_status(200)
            .with_body(r#"<html><head><meta name="csrf-token" content="token" /></head></html>"#)
            .create();

        let _signin_post_mock = server.mock("POST", "/souls/sign_in")
            .with_status(200)
            .with_body(r#"<html><body><h1>Dashboard</h1></body></html>"#)
            .create();

        // Mock empty posts response
        let _posts_mock = server.mock("GET", "/observations")
            .with_status(200)
            .with_body(r#"<html><body><div class="observations-container"></div></body></html>"#)
            .create();

        // Test the run function
        let result = run(output_dir);
        assert!(result.is_ok()); // Should succeed but with no posts
    }

    #[test]
    fn test_run_api_error() {
        let (mut server, output_dir) = setup_test_env();

        // Mock login endpoints
        let _signin_mock = server.mock("GET", "/souls/sign_in")
            .with_status(200)
            .with_body(r#"<html><head><meta name="csrf-token" content="token" /></head></html>"#)
            .create();

        let _signin_post_mock = server.mock("POST", "/souls/sign_in")
            .with_status(200)
            .with_body(r#"<html><body><h1>Dashboard</h1></body></html>"#)
            .create();

        // Mock failed posts response
        let _posts_mock = server.mock("GET", "/observations")
            .with_status(500)
            .with_body("Server error")
            .create();

        // Test the run function
        let result = run(output_dir);
        assert!(result.is_err()); // Should fail due to API error
    }
}
```
```

### Issue 3: Missing Tests for `sanitize_filename` in client.rs

```markdown
## Missing Tests for `sanitize_filename` in client.rs

The `sanitize_filename` function in `src/client.rs` has no corresponding tests.

### What to Test:
- Test that spaces are correctly replaced with underscores
- Test that illegal characters (/, \, :, *, ?, ", <, >, |, ') are removed
- Test that long filenames are properly truncated to 50 characters
- Test with empty strings and strings with only illegal characters
- Test with edge cases like Unicode characters

### Implementation Example:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_filename_spaces() {
        assert_eq!(sanitize_filename("test file name"), "test_file_name");
        assert_eq!(sanitize_filename("  leading trailing  "), "leading_trailing");
    }

    #[test]
    fn test_sanitize_filename_illegal_chars() {
        assert_eq!(sanitize_filename("file/name:with*illegal?chars"), "filenamewithlegalchars");
        assert_eq!(sanitize_filename("\"quotes'and<other>|chars\\"), "quotesandotherchars");
    }

    #[test]
    fn test_sanitize_filename_length() {
        let long_name = "This is a very long filename that should be truncated to fifty characters only because it is too long";
        let result = sanitize_filename(long_name);
        assert_eq!(result.len(), 50);
        assert_eq!(result, "This_is_a_very_long_filename_that_should_be_truncated");
    }

    #[test]
    fn test_sanitize_filename_edge_cases() {
        assert_eq!(sanitize_filename(""), "");
        assert_eq!(sanitize_filename("/:*?\"<>|'"), "");
        // Test with Unicode
        assert_eq!(sanitize_filename("résumé.pdf"), "résumé.pdf");
    }
}
```
```

### Issue 4: Missing Tests for Error Handling in `extract_csrf_token`

```markdown
## Missing Tests for Error Handling in `extract_csrf_token`

The `extract_csrf_token` method in `src/client.rs` needs additional tests for error conditions.

### What to Test:
- Test when HTML doesn't contain a CSRF token
- Test different HTML structures for token extraction (both meta tag and input field)
- Test with malformed HTML

### Implementation Example:
```rust
#[test]
fn test_extract_csrf_token_missing() {
    with_isolated_env(|| {
        let mut server = mockito::Server::new();
        let client = create_mock_client(&server).expect("Failed to create mock client");

        // HTML without any CSRF token
        let html = r#"<!DOCTYPE html><html><head></head><body><form></form></body></html>"#;

        let result = client.extract_csrf_token(html);
        assert!(result.is_err());
        if let Err(err) = result {
            assert!(err.to_string().contains("Could not find CSRF token"));
        }
    });
}

#[test]
fn test_extract_csrf_token_meta_tag() {
    with_isolated_env(|| {
        let mut server = mockito::Server::new();
        let client = create_mock_client(&server).expect("Failed to create mock client");

        // HTML with CSRF token in meta tag
        let html = r#"<!DOCTYPE html><html><head>
            <meta name="csrf-token" content="meta-token-value">
        </head><body></body></html>"#;

        let result = client.extract_csrf_token(html);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "meta-token-value");
    });
}

#[test]
fn test_extract_csrf_token_input_field() {
    with_isolated_env(|| {
        let mut server = mockito::Server::new();
        let client = create_mock_client(&server).expect("Failed to create mock client");

        // HTML with CSRF token in input field
        let html = r#"<!DOCTYPE html><html><head></head><body>
            <form>
                <input name="authenticity_token" value="input-token-value">
            </form>
        </body></html>"#;

        let result = client.extract_csrf_token(html);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "input-token-value");
    });
}

#[test]
fn test_extract_csrf_token_malformed_html() {
    with_isolated_env(|| {
        let mut server = mockito::Server::new();
        let client = create_mock_client(&server).expect("Failed to create mock client");

        // Malformed HTML
        let html = r#"<not-valid-html><meta name="csrf-token" content="token">"#;

        // The HTML parser is quite forgiving, but we should still test with malformed HTML
        let result = client.extract_csrf_token(html);
        // Even with malformed HTML, scraper might still find the token
        if result.is_ok() {
            assert_eq!(result.unwrap(), "token");
        } else {
            assert!(result.unwrap_err().to_string().contains("Could not find CSRF token"));
        }
    });
}
```
```

### Issue 5: Missing Tests for Error Cases in `download_photo` and `download_all_photos`

```markdown
## Missing Tests for Error Cases in Photo Download Methods

The `download_photo` and `download_all_photos` methods in `src/client.rs` need more tests for error conditions.

### What to Test:
- Test with empty photo URL list
- Test with non-existent photo index
- Test with failed HTTP responses (403, 404, 500, etc.)
- Test with server timeouts
- Test with invalid/corrupt image data
- Test with filesystem permission issues

### Implementation Example:
```rust
#[test]
fn test_download_photo_empty_urls() {
    with_isolated_env(|| {
        let mut server = mockito::Server::new();
        let client = create_mock_client(&server).expect("Failed to create mock client");
        client.login().expect("Login failed");

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let output_dir = temp_dir.path();

        // Create a post with no photo URLs
        let post = Post {
            id: "no-photos".to_string(),
            title: "Post Without Photos".to_string(),
            author: "Test Author".to_string(),
            date: "2023-01-01".to_string(),
            url: "https://example.com/post".to_string(),
            photo_urls: vec![],
        };

        // Try to download a photo
        let result = client.download_photo(&post, 0, output_dir);
        assert!(result.is_err());
        if let Err(err) = result {
            assert!(err.to_string().contains("has no photos"));
        }
    });
}

#[test]
fn test_download_photo_invalid_index() {
    with_isolated_env(|| {
        let mut server = mockito::Server::new();
        let client = create_mock_client(&server).expect("Failed to create mock client");
        client.login().expect("Login failed");

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let output_dir = temp_dir.path();

        // Create a post with one photo URL
        let post = Post {
            id: "one-photo".to_string(),
            title: "Post With One Photo".to_string(),
            author: "Test Author".to_string(),
            date: "2023-01-01".to_string(),
            url: "https://example.com/post".to_string(),
            photo_urls: vec!["https://example.com/photo.jpg".to_string()],
        };

        // Try to download with invalid index
        let result = client.download_photo(&post, 1, output_dir);
        assert!(result.is_err());
        if let Err(err) = result {
            assert!(err.to_string().contains("out of range"));
        }
    });
}

#[test]
fn test_download_photo_http_error() {
    with_isolated_env(|| {
        let mut server = mockito::Server::new();
        let client = create_mock_client(&server).expect("Failed to create mock client");
        client.login().expect("Login failed");

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let output_dir = temp_dir.path();

        // Mock a 404 response for the photo URL
        let photo_path = "/uploads/not-found.jpg";
        let _photo_mock = server
            .mock("GET", photo_path)
            .with_status(404)
            .with_body("Not Found")
            .create();

        // Create a post with the photo URL
        let post = Post {
            id: "error-photo".to_string(),
            title: "Post With Error Photo".to_string(),
            author: "Test Author".to_string(),
            date: "2023-01-01".to_string(),
            url: "https://example.com/post".to_string(),
            photo_urls: vec![format!("{}{}", server.url(), photo_path)],
        };

        // Try to download - should fail due to 404
        let result = client.download_photo(&post, 0, output_dir);
        assert!(result.is_err());
        if let Err(err) = result {
            assert!(err.to_string().contains("Failed to download photo"));
        }
    });
}

#[test]
fn test_download_all_photos_mixed_success() {
    with_isolated_env(|| {
        let mut server = mockito::Server::new();
        let client = create_mock_client(&server).expect("Failed to create mock client");
        client.login().expect("Login failed");

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let output_dir = temp_dir.path();

        // Mock responses - one successful, one failing
        let photo1_path = "/uploads/photo1.jpg";
        let photo2_path = "/uploads/photo2.jpg";

        let _photo1_mock = server
            .mock("GET", photo1_path)
            .with_status(200)
            .with_body("test photo data")
            .create();

        let _photo2_mock = server
            .mock("GET", photo2_path)
            .with_status(500)
            .with_body("Server Error")
            .create();

        // Create a post with two photo URLs
        let post = Post {
            id: "mixed-results".to_string(),
            title: "Post With Mixed Results".to_string(),
            author: "Test Author".to_string(),
            date: "2023-01-01".to_string(),
            url: "https://example.com/post".to_string(),
            photo_urls: vec![
                format!("{}{}", server.url(), photo1_path),
                format!("{}{}", server.url(), photo2_path),
            ],
        };

        // Should continue even if one photo fails
        let result = client.download_all_photos(&post, output_dir);
        assert!(result.is_ok());

        // Should have downloaded only one photo successfully
        let paths = result.unwrap();
        assert_eq!(paths.len(), 1);
    });
}
```
```

### Issue 6: Missing Tests for `embed_metadata` Method

```markdown
## Missing Tests for `embed_metadata` Method

The `embed_metadata` method in `src/client.rs` lacks direct testing.

### What to Test:
- Test successful metadata embedding
- Test with different post data
- Test with filesystem errors (permission denied, etc.)
- Verify the content of the metadata file

### Implementation Example:
```rust
#[test]
fn test_embed_metadata_success() {
    with_isolated_env(|| {
        let mut server = mockito::Server::new();
        let client = create_mock_client(&server).expect("Failed to create mock client");

        // Create a temp directory and file
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let photo_path = temp_dir.path().join("test_photo.jpg");

        // Create an empty file
        std::fs::write(&photo_path, b"test image data").expect("Failed to write test file");

        // Create a test post
        let post = Post {
            id: "test-123".to_string(),
            title: "Test Post Title".to_string(),
            author: "Test Author".to_string(),
            date: "2023-01-01".to_string(),
            url: "https://example.com/post/123".to_string(),
            photo_urls: vec!["https://example.com/photo.jpg".to_string()],
        };

        // Embed metadata
        let result = client.embed_metadata(&post, &photo_path);
        assert!(result.is_ok());

        // Verify metadata file exists
        let metadata_path = photo_path.with_extension("metadata.txt");
        assert!(metadata_path.exists());

        // Check metadata content
        let content = std::fs::read_to_string(&metadata_path).expect("Failed to read metadata");
        assert!(content.contains(&post.title));
        assert!(content.contains(&post.author));
        assert!(content.contains(&post.date));
        assert!(content.contains(&post.url));
        assert!(content.contains(&post.id));
    });
}

#[test]
fn test_embed_metadata_filesystem_error() {
    with_isolated_env(|| {
        let mut server = mockito::Server::new();
        let client = create_mock_client(&server).expect("Failed to create mock client");

        // Create a temp directory but make a path to a directory (not a file)
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let photo_dir = temp_dir.path().join("photo_dir");
        std::fs::create_dir(&photo_dir).expect("Failed to create directory");

        // Create a test post
        let post = Post {
            id: "test-123".to_string(),
            title: "Test Post Title".to_string(),
            author: "Test Author".to_string(),
            date: "2023-01-01".to_string(),
            url: "https://example.com/post/123".to_string(),
            photo_urls: vec!["https://example.com/photo.jpg".to_string()],
        };

        // Trying to write metadata to a directory should fail
        let result = client.embed_metadata(&post, &photo_dir);
        assert!(result.is_err());
    });
}
```
```

### Issue 7: Missing Tests for `parse_posts` Method

```markdown
## Missing Tests for `parse_posts` Method

The `parse_posts` method in `src/client.rs` should have dedicated tests for different HTML structures.

### What to Test:
- Test with various HTML structures
- Test with empty response
- Test with malformed HTML
- Test with missing elements (no title, author, etc.)
- Test with relative and absolute URLs for photos

### Implementation Example:
```rust
#[test]
fn test_parse_posts_empty_response() {
    with_isolated_env(|| {
        let mut server = mockito::Server::new();
        let client = create_mock_client(&server).expect("Failed to create mock client");

        // HTML with no posts
        let html = r#"<!DOCTYPE html><html><body><div class="observations-container"></div></body></html>"#;

        let result = client.parse_posts(html, &server.url());
        assert!(result.is_ok());

        let posts = result.unwrap();
        assert!(posts.is_empty(), "Expected empty posts vector");
    });
}

#[test]
fn test_parse_posts_missing_elements() {
    with_isolated_env(|| {
        let mut server = mockito::Server::new();
        let client = create_mock_client(&server).expect("Failed to create mock client");

        // HTML with incomplete post data
        let html = r#"<!DOCTYPE html><html><body>
            <div class="observation" id="obs-123">
                <!-- Missing title, author, date elements -->
                <div class="observation-photo">
                    <img src="/uploads/photo1.jpg">
                </div>
            </div>
        </body></html>"#;

        let result = client.parse_posts(html, &server.url());
        assert!(result.is_ok());

        let posts = result.unwrap();
        assert_eq!(posts.len(), 1, "Expected one post");

        // Check default values for missing elements
        assert_eq!(posts[0].title, "Untitled Post");
        assert_eq!(posts[0].author, "Unknown Author");
        assert_eq!(posts[0].date, "Unknown Date");
        assert!(posts[0].photo_urls.len() == 1);
    });
}

#[test]
fn test_parse_posts_absolute_urls() {
    with_isolated_env(|| {
        let mut server = mockito::Server::new();
        let client = create_mock_client(&server).expect("Failed to create mock client");

        // HTML with absolute URLs
        let html = format!(r#"<!DOCTYPE html><html><body>
            <div class="observation" id="obs-123">
                <div class="observation-text">Test Post</div>
                <div class="observation-author">Test Author</div>
                <div class="observation-date">2023-01-01</div>
                <a class="observation-link" href="https://example.com/observations/123">View</a>
                <div class="observation-photo">
                    <img src="https://example.com/uploads/photo1.jpg">
                </div>
            </div>
        </body></html>"#);

        let result = client.parse_posts(&html, &server.url());
        assert!(result.is_ok());

        let posts = result.unwrap();
        assert_eq!(posts.len(), 1, "Expected one post");

        // Check URLs are preserved as absolute
        assert_eq!(posts[0].url, "https://example.com/observations/123");
        assert_eq!(posts[0].photo_urls[0], "https://example.com/uploads/photo1.jpg");
    });
}

#[test]
fn test_parse_posts_relative_urls() {
    with_isolated_env(|| {
        let mut server = mockito::Server::new();
        let client = create_mock_client(&server).expect("Failed to create mock client");

        // HTML with relative URLs
        let html = r#"<!DOCTYPE html><html><body>
            <div class="observation" id="obs-123">
                <div class="observation-text">Test Post</div>
                <div class="observation-author">Test Author</div>
                <div class="observation-date">2023-01-01</div>
                <a class="observation-link" href="/observations/123">View</a>
                <div class="observation-photo">
                    <img src="/uploads/photo1.jpg">
                </div>
            </div>
        </body></html>"#;

        let result = client.parse_posts(html, &server.url());
        assert!(result.is_ok());

        let posts = result.unwrap();
        assert_eq!(posts.len(), 1, "Expected one post");

        // Check URLs are converted to absolute
        let base_domain = server.url().split("/schools").next().unwrap_or("");
        assert_eq!(posts[0].url, format!("{}/observations/123", base_domain));
        assert_eq!(posts[0].photo_urls[0], format!("{}/uploads/photo1.jpg", base_domain));
    });
}
```
```

### Issue 8: Missing Tests for Default Cache Directory

```markdown
## Missing Tests for Default Cache Directory

The `default_cache_dir` function in `src/cache.rs` lacks tests.

### What to Test:
- Test that the function returns a valid directory path
- Test that the path contains the expected subfolders
- Test the fallback behavior when `dirs::cache_dir()` returns None

### Implementation Example:
```rust
use std::path::Path;
use mockall::predicate::*;
use mockall::*;

// Create a mock for testing the fallback behavior
#[automock]
pub trait CacheDirProvider {
    fn cache_dir(&self) -> Option<PathBuf>;
}

struct DefaultCacheDirProvider;

impl CacheDirProvider for DefaultCacheDirProvider {
    fn cache_dir(&self) -> Option<PathBuf> {
        dirs::cache_dir()
    }
}

// Modified function that uses the provider for testing
pub fn default_cache_dir_with_provider<P: CacheDirProvider>(provider: &P) -> PathBuf {
    if let Some(cache_dir) = provider.cache_dir() {
        cache_dir.join("transparent-classroom-cache")
    } else {
        PathBuf::from("./.cache/transparent-classroom-cache")
    }
}

#[test]
fn test_default_cache_dir_normal_path() {
    let result = default_cache_dir();

    // Check that the path includes the expected subfolder
    assert!(result.to_string_lossy().contains("transparent-classroom-cache"));

    // Should be an absolute path in normal case
    if dirs::cache_dir().is_some() {
        assert!(result.is_absolute());
    }
}

#[test]
fn test_default_cache_dir_fallback() {
    // Create a mock provider that returns None
    let mut mock_provider = MockCacheDirProvider::new();
    mock_provider.expect_cache_dir().return_const(None);

    let result = default_cache_dir_with_provider(&mock_provider);

    // Should use the fallback relative path
    assert_eq!(result, PathBuf::from("./.cache/transparent-classroom-cache"));
}
```
```
