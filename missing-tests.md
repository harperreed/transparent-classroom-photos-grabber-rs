I'll review the codebase to identify missing test cases for the transparent-classroom-photos-grabber-rs project. Let me analyze the code and create a comprehensive list of missing tests that should be implemented.

# Missing Test Cases for transparent-classroom-photos-grabber-rs

## Issue 1: Missing Tests for Error Case in `parse_posts_json`

The `parse_posts_json` method in `src/client.rs` has a potential memory leak with `Box::leak` and lacks tests for various error conditions.

```markdown
### Description:
The `parse_posts_json` method extracts post data from JSON responses, but currently lacks tests for error conditions and edge cases.

### What to Test:
- Test with malformed JSON
- Test with empty JSON array
- Test with missing required fields
- Test with numeric IDs vs string IDs (to verify Box::leak behavior)
- Test with different JSON structures (nested posts in "posts" vs "data" fields)

### Implementation Example:
```rust
#[test]
fn test_parse_posts_json_malformed() {
    with_isolated_env(|| {
        let mut server = mockito::Server::new();
        let client = create_mock_client(&server).expect("Failed to create mock client");

        let malformed_json = r#"{"this is not valid json"#;
        let result = client.parse_posts_json(malformed_json);
        assert!(result.is_err());
        match result {
            Err(AppError::Parse(msg)) => {
                assert!(msg.contains("Failed to parse JSON response"));
            }
            _ => panic!("Expected Parse error for malformed JSON"),
        }
    });
}

#[test]
fn test_parse_posts_json_empty_array() {
    with_isolated_env(|| {
        let mut server = mockito::Server::new();
        let client = create_mock_client(&server).expect("Failed to create mock client");

        let empty_json = r#"[]"#;
        let result = client.parse_posts_json(empty_json);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    });
}

#[test]
fn test_parse_posts_json_numeric_id() {
    with_isolated_env(|| {
        let mut server = mockito::Server::new();
        let client = create_mock_client(&server).expect("Failed to create mock client");

        // Test with numeric ID to verify Box::leak behavior
        let json_with_numeric_id = r#"[
            {"id": 12345, "normalized_text": "Test Post", "author": "Test Author", "date": "2023-01-01"}
        ]"#;

        let result = client.parse_posts_json(json_with_numeric_id);
        assert!(result.is_ok());
        let posts = result.unwrap();
        assert_eq!(posts.len(), 1);
        assert_eq!(posts[0].id, "12345");
    });
}

#[test]
fn test_parse_posts_json_different_structures() {
    with_isolated_env(|| {
        let mut server = mockito::Server::new();
        let client = create_mock_client(&server).expect("Failed to create mock client");

        // Test with posts in "posts" field
        let json_with_posts_field = r#"{"posts": [
            {"id": "post1", "normalized_text": "Test Post 1", "author": "Test Author 1", "date": "2023-01-01"}
        ]}"#;

        let result = client.parse_posts_json(json_with_posts_field);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);

        // Test with posts in "data" field
        let json_with_data_field = r#"{"data": [
            {"id": "post2", "normalized_text": "Test Post 2", "author": "Test Author 2", "date": "2023-01-02"}
        ]}"#;

        let result = client.parse_posts_json(json_with_data_field);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
    });
}
```
```

## Issue 2: Missing Tests for `parse_date_to_timestamp` Method

The `parse_date_to_timestamp` method in `src/client.rs` is untested and handles different date formats.

```markdown
### Description:
The `parse_date_to_timestamp` method is responsible for parsing various date formats for proper file timestamping, but lacks dedicated tests to verify this functionality.

### What to Test:
- Test parsing RFC3339 date format
- Test parsing YYYY-MM-DD format
- Test parsing invalid date formats
- Test handling empty date strings

### Implementation Example:
```rust
#[test]
fn test_parse_date_to_timestamp() {
    with_isolated_env(|| {
        let mut server = mockito::Server::new();
        let client = create_mock_client(&server).expect("Failed to create mock client");

        // Test RFC3339 format
        let rfc3339_date = "2023-01-15T12:30:45Z";
        let timestamp = client.parse_date_to_timestamp(rfc3339_date);
        assert!(timestamp.is_some());

        // Test YYYY-MM-DD format
        let simple_date = "2023-01-15";
        let timestamp = client.parse_date_to_timestamp(simple_date);
        assert!(timestamp.is_some());

        // Test invalid format
        let invalid_date = "January 15, 2023";
        let timestamp = client.parse_date_to_timestamp(invalid_date);
        assert!(timestamp.is_none());

        // Test empty string
        let empty_date = "";
        let timestamp = client.parse_date_to_timestamp(empty_date);
        assert!(timestamp.is_none());
    });
}
```
```

## Issue 3: Missing Tests for `discover_endpoints` Method

The `discover_endpoints` method in `src/client.rs` lacks tests to verify its functionality.

```markdown
### Description:
The `discover_endpoints` method attempts to discover available endpoints by examining the main school page, but lacks tests to verify this functionality works correctly.

### What to Test:
- Test with a school page containing observation/event/photo links
- Test with a school page with no relevant links
- Test with malformed HTML
- Test error handling for HTTP failures

### Implementation Example:
```rust
#[test]
fn test_discover_endpoints_with_links() {
    with_isolated_env(|| {
        let mut server = mockito::Server::new();
        let client = create_mock_client(&server).expect("Failed to create mock client");

        // Mock school page with relevant links
        let html = r#"<!DOCTYPE html><html><body>
            <a href="/observations">Observations</a>
            <a href="/events">Events</a>
            <a href="/photos">Photos</a>
            <a href="https://example.com/posts">External Posts</a>
        </body></html>"#;

        let _mock = server
            .mock("GET", "/")
            .with_status(200)
            .with_header("content-type", "text/html")
            .with_body(html)
            .create();

        let result = client.discover_endpoints();
        assert!(result.is_ok());
        let endpoints = result.unwrap();
        assert!(endpoints.len() >= 4);

        // Check that the endpoints contain our expected links
        assert!(endpoints.iter().any(|url| url.contains("/observations")));
        assert!(endpoints.iter().any(|url| url.contains("/events")));
        assert!(endpoints.iter().any(|url| url.contains("/photos")));
        assert!(endpoints.iter().any(|url| url.contains("example.com/posts")));
    });
}

#[test]
fn test_discover_endpoints_no_links() {
    with_isolated_env(|| {
        let mut server = mockito::Server::new();
        let client = create_mock_client(&server).expect("Failed to create mock client");

        // Mock school page with no relevant links
        let html = r#"<!DOCTYPE html><html><body>
            <a href="/profile">Profile</a>
            <a href="/settings">Settings</a>
        </body></html>"#;

        let _mock = server
            .mock("GET", "/")
            .with_status(200)
            .with_header("content-type", "text/html")
            .with_body(html)
            .create();

        let result = client.discover_endpoints();
        assert!(result.is_ok());
        let endpoints = result.unwrap();
        assert!(endpoints.is_empty());
    });
}

#[test]
fn test_discover_endpoints_http_error() {
    with_isolated_env(|| {
        let mut server = mockito::Server::new();
        let client = create_mock_client(&server).expect("Failed to create mock client");

        // Mock server error
        let _mock = server
            .mock("GET", "/")
            .with_status(500)
            .with_body("Server Error")
            .create();

        let result = client.discover_endpoints();
        assert!(result.is_err());
    });
}
```
```

## Issue 4: Missing Tests for `sanitize_filename` Function

The `sanitize_filename` function in `src/client.rs` has no dedicated tests.

```markdown
### Description:
The `sanitize_filename` function sanitizes strings for use in filenames, but has no dedicated tests to verify its behavior.

### What to Test:
- Test replacement of spaces with underscores
- Test removal of problematic characters (/, \, :, etc.)
- Test truncation of long filenames
- Test edge cases (empty strings, strings with only problematic characters)
- Test Unicode character handling

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
    fn test_sanitize_filename_problematic_chars() {
        assert_eq!(sanitize_filename("file/name:with*illegal?chars"), "filenamewithlegalchars");
        assert_eq!(sanitize_filename("\"quotes'and<other>|chars\\"), "quotesandotherchars");
    }

    #[test]
    fn test_sanitize_filename_truncation() {
        let long_name = "This is a very long filename that should be truncated to fifty characters only because it is too long";
        let result = sanitize_filename(long_name);
        assert_eq!(result.len(), 50);
        assert_eq!(result, "This_is_a_very_long_filename_that_should_be_truncated");
    }

    #[test]
    fn test_sanitize_filename_edge_cases() {
        assert_eq!(sanitize_filename(""), "");
        assert_eq!(sanitize_filename("/:*?\"<>|'"), "");

        // Test with Unicode characters
        assert_eq!(sanitize_filename("résumé.pdf"), "résumé.pdf");
        assert_eq!(sanitize_filename("emoji🙂file.txt"), "emoji🙂file.txt");
    }
}
```
```

## Issue 5: Missing Tests for `embed_metadata` Method

The `embed_metadata` method in `src/client.rs` needs tests to verify metadata is correctly embedded.

```markdown
### Description:
The `embed_metadata` method creates metadata files for downloaded photos but lacks tests to verify this functionality.

### What to Test:
- Test successful metadata file creation
- Test metadata content accuracy
- Test error handling for filesystem issues
- Test with various post data formats

### Implementation Example:
```rust
#[test]
fn test_embed_metadata_success() {
    with_isolated_env(|| {
        let mut server = mockito::Server::new();
        let client = create_mock_client(&server).expect("Failed to create mock client");

        // Create a temporary directory for testing
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let photo_path = temp_dir.path().join("test_photo.jpg");

        // Create a dummy file to represent the photo
        fs::write(&photo_path, b"dummy image data").expect("Failed to create test photo");

        // Create a test post
        let post = Post {
            id: "test-123".to_string(),
            title: "Test Post Title".to_string(),
            author: "Test Author".to_string(),
            date: "2023-01-15".to_string(),
            url: "https://example.com/post/123".to_string(),
            photo_urls: vec!["https://example.com/photo.jpg".to_string()],
        };

        // Call the method
        let result = client.embed_metadata(&post, &photo_path);

        // Verify success
        assert!(result.is_ok());

        // Check that metadata file was created
        let metadata_path = photo_path.with_extension("metadata.json");
        assert!(metadata_path.exists());

        // Verify content
        let metadata_content = fs::read_to_string(metadata_path).expect("Failed to read metadata file");
        let metadata: serde_json::Value = serde_json::from_str(&metadata_content).expect("Failed to parse metadata JSON");

        assert_eq!(metadata["title"], "Test Post Title");
        assert_eq!(metadata["author"], "Test Author");
        assert_eq!(metadata["date"], "2023-01-15");
        assert_eq!(metadata["url"], "https://example.com/post/123");
        assert_eq!(metadata["post_id"], "test-123");
        assert_eq!(metadata["school_location"]["latitude"], 41.9032776);
        assert_eq!(metadata["school_location"]["longitude"], -87.6663027);
    });
}

#[test]
fn test_embed_metadata_filesystem_error() {
    with_isolated_env(|| {
        let mut server = mockito::Server::new();
        let client = create_mock_client(&server).expect("Failed to create mock client");

        // Create a read-only directory
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let photo_path = temp_dir.path().join("test_photo.jpg");

        // Create a dummy file
        fs::write(&photo_path, b"dummy image data").expect("Failed to create test photo");

        // Make the directory read-only on Unix systems
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let permissions = fs::Permissions::from_mode(0o444); // read-only
            fs::set_permissions(temp_dir.path(), permissions).expect("Failed to set permissions");
        }

        // Create a test post
        let post = Post {
            id: "test-123".to_string(),
            title: "Test Post".to_string(),
            author: "Test Author".to_string(),
            date: "2023-01-15".to_string(),
            url: "https://example.com/post/123".to_string(),
            photo_urls: vec!["https://example.com/photo.jpg".to_string()],
        };

        // This should fail on Unix systems due to permissions
        #[cfg(unix)]
        {
            let result = client.embed_metadata(&post, &photo_path);
            assert!(result.is_err());

            // Reset permissions so the temp directory can be cleaned up
            let permissions = fs::Permissions::from_mode(0o755);
            fs::set_permissions(temp_dir.path(), permissions).expect("Failed to reset permissions");
        }
    });
}
```
```

## Issue 6: Missing Tests for `set_file_timestamps` Method

The `set_file_timestamps` method in `src/client.rs` lacks dedicated tests.

```markdown
### Description:
The `set_file_timestamps` method sets file creation and modification timestamps but lacks tests to verify this behavior.

### What to Test:
- Test that timestamps are correctly set
- Test with various timestamp values
- Test error handling for invalid files
- Test platform-specific behavior (if possible)

### Implementation Example:
```rust
#[test]
fn test_set_file_timestamps() {
    with_isolated_env(|| {
        let mut server = mockito::Server::new();
        let client = create_mock_client(&server).expect("Failed to create mock client");

        // Create a temporary directory for testing
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let test_file = temp_dir.path().join("test_file.txt");

        // Create a dummy file
        fs::write(&test_file, b"test content").expect("Failed to create test file");

        // Create a timestamp 1 day in the past
        let now = std::time::SystemTime::now();
        let one_day = std::time::Duration::from_secs(24 * 60 * 60);
        let past_time = now.checked_sub(one_day).expect("Time calculation error");

        // Set the timestamp
        let result = client.set_file_timestamps(&test_file, past_time);
        assert!(result.is_ok());

        // Verify the modification time was changed
        let metadata = fs::metadata(&test_file).expect("Failed to get file metadata");
        let mtime = metadata.modified().expect("Failed to get modification time");

        // The timestamps should be close (within 1 second tolerance for filesystem differences)
        let diff = match mtime.duration_since(past_time) {
            Ok(d) => d,
            Err(e) => e.duration(),
        };

        assert!(diff.as_secs() < 2, "Timestamp difference too large: {:?}", diff);
    });
}

#[test]
fn test_set_file_timestamps_nonexistent_file() {
    with_isolated_env(|| {
        let mut server = mockito::Server::new();
        let client = create_mock_client(&server).expect("Failed to create mock client");

        // Create a path to a nonexistent file
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let nonexistent_file = temp_dir.path().join("nonexistent.txt");

        // Try to set the timestamp
        let now = std::time::SystemTime::now();
        let result = client.set_file_timestamps(&nonexistent_file, now);

        // Should fail with an IO error
        assert!(result.is_err());
        match result {
            Err(AppError::Io(_)) => {} // expected
            err => panic!("Expected IO error, got {:?}", err),
        }
    });
}
```
```

## Issue 7: Missing Tests for `photo_already_exists` Method

The `photo_already_exists` method in `src/client.rs` lacks dedicated tests.

```markdown
### Description:
The `photo_already_exists` method checks if a photo already exists to avoid duplicate downloads, but lacks tests to verify this functionality.

### What to Test:
- Test when a matching photo exists
- Test when no matching photo exists
- Test with various filename patterns
- Test with empty or nonexistent directories

### Implementation Example:
```rust
#[test]
fn test_photo_already_exists_match() {
    with_isolated_env(|| {
        let mut server = mockito::Server::new();
        let client = create_mock_client(&server).expect("Failed to create mock client");

        // Create a temporary directory for testing
        let temp_dir = TempDir::new().expect("Failed to create temp directory");

        // Create a test post
        let post = Post {
            id: "test-123".to_string(),
            title: "Test Post".to_string(),
            author: "Test Author".to_string(),
            date: "2023-01-15".to_string(),
            url: "https://example.com/post/123".to_string(),
            photo_urls: vec!["https://example.com/photo.jpg".to_string()],
        };

        // Create a file that matches the post ID pattern
        let existing_file = temp_dir.path().join("test-123_max.jpg");
        fs::write(&existing_file, b"test image data").expect("Failed to create test file");

        // Check if the photo exists
        let result = client.photo_already_exists(&post, temp_dir.path());

        // Should find the existing file
        assert!(result.is_some());
        assert_eq!(result.unwrap(), existing_file);
    });
}

#[test]
fn test_photo_already_exists_no_match() {
    with_isolated_env(|| {
        let mut server = mockito::Server::new();
        let client = create_mock_client(&server).expect("Failed to create mock client");

        // Create a temporary directory for testing
        let temp_dir = TempDir::new().expect("Failed to create temp directory");

        // Create a test post
        let post = Post {
            id: "test-123".to_string(),
            title: "Test Post".to_string(),
            author: "Test Author".to_string(),
            date: "2023-01-15".to_string(),
            url: "https://example.com/post/123".to_string(),
            photo_urls: vec!["https://example.com/photo.jpg".to_string()],
        };

        // Create a file with a different ID
        let different_file = temp_dir.path().join("different-456_max.jpg");
        fs::write(&different_file, b"test image data").expect("Failed to create test file");

        // Check if the photo exists
        let result = client.photo_already_exists(&post, temp_dir.path());

        // Should not find a match
        assert!(result.is_none());
    });
}

#[test]
fn test_photo_already_exists_empty_dir() {
    with_isolated_env(|| {
        let mut server = mockito::Server::new();
        let client = create_mock_client(&server).expect("Failed to create mock client");

        // Create a temporary directory for testing
        let temp_dir = TempDir::new().expect("Failed to create temp directory");

        // Create a test post
        let post = Post {
            id: "test-123".to_string(),
            title: "Test Post".to_string(),
            author: "Test Author".to_string(),
            date: "2023-01-15".to_string(),
            url: "https://example.com/post/123".to_string(),
            photo_urls: vec!["https://example.com/photo.jpg".to_string()],
        };

        // Check if the photo exists in an empty directory
        let result = client.photo_already_exists(&post, temp_dir.path());

        // Should not find anything
        assert!(result.is_none());
    });
}

#[test]
fn test_photo_already_exists_nonexistent_dir() {
    with_isolated_env(|| {
        let mut server = mockito::Server::new();
        let client = create_mock_client(&server).expect("Failed to create mock client");

        // Create a path to a nonexistent directory
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let nonexistent_dir = temp_dir.path().join("nonexistent");

        // Create a test post
        let post = Post {
            id: "test-123".to_string(),
            title: "Test Post".to_string(),
            author: "Test Author".to_string(),
            date: "2023-01-15".to_string(),
            url: "https://example.com/post/123".to_string(),
            photo_urls: vec!["https://example.com/photo.jpg".to_string()],
        };

        // Check if the photo exists in a nonexistent directory
        let result = client.photo_already_exists(&post, &nonexistent_dir);

        // Should handle this gracefully and return None
        assert!(result.is_none());
    });
}
```
```

## Issue 8: Missing Tests for `extract_attribute` Method

The `extract_attribute` helper method in `src/client.rs` lacks dedicated tests.

```markdown
### Description:
The `extract_attribute` helper method extracts attributes from HTML elements but lacks dedicated tests.

### What to Test:
- Test extracting existing attributes
- Test extracting non-existent attributes
- Test with various HTML element types

### Implementation Example:
```rust
#[test]
fn test_extract_attribute() {
    with_isolated_env(|| {
        let mut server = mockito::Server::new();
        let client = create_mock_client(&server).expect("Failed to create mock client");

        // Create HTML elements for testing
        let html = r#"<div id="test-div" class="test-class" data-custom="custom-value"></div>"#;
        let document = scraper::Html::parse_document(html);
        let selector = scraper::Selector::parse("div").unwrap();
        let element = document.select(&selector).next().unwrap();

        // Test extracting existing attributes
        assert_eq!(client.extract_attribute(&element, "id"), Some("test-div".to_string()));
        assert_eq!(client.extract_attribute(&element, "class"), Some("test-class".to_string()));
        assert_eq!(client.extract_attribute(&element, "data-custom"), Some("custom-value".to_string()));

        // Test extracting non-existent attribute
        assert_eq!(client.extract_attribute(&element, "nonexistent"), None);
    });
}
```
```

## Issue 9: Missing Tests for Rate Limiting and Retry Logic

The code likely has rate limiting behavior that should be tested.

```markdown
### Description:
There appears to be rate limiting behavior and retry logic in the client, but it lacks dedicated tests to verify this functionality.

### What to Test:
- Test that rate limiting properly spaces requests
- Test that retries happen on certain failures
- Test maximum retry behavior

### Implementation Example:
```rust
#[test]
fn test_rate_limiting() {
    with_isolated_env(|| {
        let mut server = mockito::Server::new();
        let client = create_mock_client(&server).expect("Failed to create mock client");

        // Set up mock endpoints
        let mock1 = server
            .mock("GET", "/endpoint1")
            .with_status(200)
            .with_body("Response 1")
            .create();

        let mock2 = server
            .mock("GET", "/endpoint2")
            .with_status(200)
            .with_body("Response 2")
            .create();

        // Make two requests and measure the time between them
        let start_time = std::time::Instant::now();

        // This is a simplified example - you'd need to adapt this to call your actual API methods
        let _resp1 = client.http_client.get(&format!("{}/endpoint1", server.url()))
            .send()
            .expect("Request 1 failed");

        let _resp2 = client.http_client.get(&format!("{}/endpoint2", server.url()))
            .send()
            .expect("Request 2 failed");

        let elapsed = start_time.elapsed();

        // If there's rate limiting, there should be some minimum delay between requests
        // Adjust this threshold based on your actual rate limiting implementation
        assert!(elapsed.as_millis() >= 500, "Requests happened too quickly: {:?}", elapsed);

        // Verify both requests were made
        mock1.assert();
        mock2.assert();
    });
}

#[test]
fn test_retry_on_failure() {
    with_isolated_env(|| {
        let mut server = mockito::Server::new();
        let client = create_mock_client(&server).expect("Failed to create mock client");

        // Set up a mock that fails the first time and succeeds the second time
        let retry_mock = server
            .mock("GET", "/retry-endpoint")
            .with_status(429) // Too Many Requests
            .with_body("Rate limited")
            .expect(1)
            .create();

        let success_mock = server
            .mock("GET", "/retry-endpoint")
            .with_status(200)
            .with_body("Success after retry")
            .expect(1)
            .create();

        // If your client has a method that retries, call it here
        // Otherwise this is just a conceptual test

        // This is a simplified example assuming you have a method that retries on 429
        // let result = client.get_with_retry(&format!("{}/retry-endpoint", server.url()));
        // assert!(result.is_ok());

        // Verify both mocks were called
        retry_mock.assert();
        success_mock.assert();
    });
}
```
```

## Issue 10: Missing Tests for Different API Response Formats

The code tries multiple URLs and handles different response formats, but lacks comprehensive tests.

```markdown
### Description:
The client attempts to fetch posts from multiple URLs with different response formats (JSON vs HTML), but lacks comprehensive tests for this behavior.

### What to Test:
- Test fallback behavior when primary URL fails
- Test JSON vs HTML response handling
- Test endpoint discovery and fallback chain
- Test with various HTTP status codes

### Implementation Example:
```rust
#[test]
fn test_get_posts_primary_url_success() {
    with_isolated_env(|| {
        let mut server = mockito::Server::new();
        let client = create_mock_client(&server).expect("Failed to create mock client");

        // Mock successful JSON response from primary URL
        let json_response = r#"[
            {"id": "post1", "normalized_text": "Test Post 1", "author": "Author 1", "date": "2023-01-01"}
        ]"#;

        let primary_mock = server
            .mock("GET", "/s/12345/children/67890/posts.json?locale=en")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(json_response)
            .expect(1)
            .create();

        // Call the method
        let result = client.get_posts(1);

        // Verify success
        assert!(result.is_ok());
        let posts = result.unwrap();
        assert_eq!(posts.len(), 1);
        assert_eq!(posts[0].id, "post1");

        // Verify only primary URL was tried
        primary_mock.assert();
    });
}

#[test]
fn test_get_posts_fallback_to_html() {
    with_isolated_env(|| {
        let mut server = mockito::Server::new();
        let client = create_mock_client(&server).expect("Failed to create mock client");

        // Mock failed JSON response from primary URL
        let primary_mock = server
            .mock("GET", "/s/12345/children/67890/posts.json?locale=en")
            .with_status(404)
            .expect(1)
            .create();

        // Mock successful HTML response from fallback URL
        let html_response = r#"<!DOCTYPE html><html><body>
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

        let fallback_mock = server
            .mock("GET", "/observations")
            .with_status(200)
            .with_header("content-type", "text/html")
            .with_body(html_response)
            .expect(1)
            .create();

        // Call the method
        let
