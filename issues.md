# Code Review Issues for transparent-classroom-photos-grabber-rs

## Issue 1: Inconsistent Config Fields Between Implementation and Tests

The `Config` struct in `src/config.rs` has required fields for `school_lat`, `school_lng`, and `school_keywords`, but some tests in `tests/client_test.rs` create Config instances without these fields. This leads to a mismatch between implementation and tests.

```rust
// In src/config.rs the struct has these fields:
pub struct Config {
    pub email: String,
    pub password: String,
    pub school_id: u32,
    pub child_id: u32,
    pub school_lat: f64,
    pub school_lng: f64,
    pub school_keywords: String,
}

// But in tests/client_test.rs, Config instances are created without the required fields:
fn create_test_config() -> Config {
    Config {
        email: "test@example.com".to_string(),
        password: "password123".to_string(),
        school_id: 12345,
        child_id: 67890,
    }
}
```

## Issue 2: Unsafe Type Conversion in `parse_posts_json`

The `parse_posts_json` method in `client.rs` uses `Box::leak` which deliberately leaks memory:

```rust
let id = post_obj.get("id")
    .and_then(|v| v.as_str().or_else(|| v.as_u64().map(|n| Box::leak(n.to_string().into_boxed_str()) as &str)))
    .unwrap_or(&format!("post_{}", i))
    .to_string();
```

This creates a memory leak for each post that has a numeric ID. A safer approach would be to directly convert to String without leaking memory.

## Issue 3: Insufficient Error Handling in `download_all_photos`

The `download_all_photos` method in `client.rs` suppresses individual photo download errors:

```rust
match self.download_photo(post, i, output_dir) {
    Ok(path) => downloaded_paths.push(path),
    Err(e) => {
        warn!("Failed to download photo {} for post {}: {}", i, post.id, e);
        // Continue with other photos even if one fails
    }
}
```

This silently continues when photos fail to download, making debugging difficult. It should at least provide a way to know which photos failed and why.

## Issue 4: Redundant Mutability in `sanitize_filename`

In `sanitize_filename`, the input string is repeatedly reassigned to the same variable:

```rust
fn sanitize_filename(input: &str) -> String {
    let mut result = input.trim().to_owned();
    // Replace spaces with underscores
    result = result.replace(' ', "_");
    // Remove characters that are problematic in filenames
    result = result.replace(
        &['/', '\\', ':', '*', '?', '"', '<', '>', '|', '\''][..],
        "",
    );
    // Truncate if too long
    if result.len() > 50 {
        result.truncate(50);
    }
    result
}
```

This is inefficient and could be rewritten to use method chaining for better readability and performance.

## Issue 5: Dead Code in `extract_csrf_token`

The `extract_csrf_token` method in `client.rs` has redundant selectors and tries multiple approaches to find the CSRF token:

```rust
let form_selector = Selector::parse("form").unwrap();
let input_selector = Selector::parse("input[name=\"soul[login]\"]").unwrap();
let auth_token_selector = Selector::parse("input[name=\"authenticity_token\"]").unwrap();
// ... more code ...
let meta_selector = Selector::parse("meta[name=\"csrf-token\"]").unwrap();
```

The code tries multiple approaches, which is good for robustness, but lacks clear prioritization or fallback patterns. It would be better to refactor this to use a clear fallback strategy with comments about which approaches are preferred.

## Issue 6: Use of Unwrap in HTML Selectors

Multiple places in the code use `unwrap()` when parsing HTML selectors, which could panic if the selector syntax is invalid:

```rust
let post_selector = Selector::parse(".observation").unwrap();
let title_selector = Selector::parse(".observation-text").unwrap();
let author_selector = Selector::parse(".observation-author").unwrap();
```

These should be converted to proper error handling since they represent potential runtime failures.

## Issue 7: Hardcoded URLs and Paths

The code contains numerous hardcoded URLs and paths which makes the code brittle:

```rust
let base_url = format!(
    "https://www.transparentclassroom.com/schools/{}",
    config.school_id
);
```

```rust
let primary_url = if page <= 1 {
    format!("https://www.transparentclassroom.com/s/{}/children/{}/posts.json?locale=en", self.config.school_id, self.config.child_id)
} else {
    format!("https://www.transparentclassroom.com/s/{}/children/{}/posts.json?locale=en&page={}", self.config.school_id, self.config.child_id, page)
};
```

These should be moved to configuration or constants to make the code more maintainable.

## Issue 8: Too Many Ignored Tests

There are multiple tests marked with `#[ignore]` in `tests/config_test.rs`:

```rust
#[test]
#[ignore]
fn test_valid_config() {
    // ...
}

#[test]
#[ignore]
fn test_missing_email() {
    // ...
}
```

These tests are being skipped, which means important functionality is not being verified during test runs. They should be fixed and enabled.

## Issue 9: Inconsistent Error Handling Patterns

The codebase mixes several error handling approaches:

1. `unwrap()` or `expect()` in some places
2. Custom errors with `AppError` in others
3. Pattern matching on errors in some test cases

This inconsistency makes the code harder to maintain and reason about. A uniform approach to error handling should be adopted.

## Issue 10: Debug Logs Containing Credentials

The login code logs form data which could include sensitive information:

```rust
let mut debug_form = form_data.clone();
if debug_form.contains_key("soul[password]") {
    debug_form.insert("soul[password]", "********");
}
debug!("Submitting form data: {:?}", debug_form);
```

While the password is masked, other fields like email might still contain sensitive data. All credential-related logging should be handled more carefully.
