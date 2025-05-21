use serde::{Deserialize, Serialize};
use std::fs;
use std::time::{Duration, SystemTime};
use tempfile::TempDir;

use transparent_classroom_photos_grabber_rs::cache::{
    read_cache, write_cache, CacheConfig, CacheData,
};

// Example struct to use for testing
#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Post {
    id: u32,
    title: String,
    content: String,
}

// Create a temporary directory to store test cache files
fn setup_temp_dir() -> TempDir {
    TempDir::new().expect("Failed to create temporary directory")
}

// Helper function to create a test post
fn create_test_post() -> Post {
    Post {
        id: 42,
        title: "Test Post".to_string(),
        content: "This is a test post for caching".to_string(),
    }
}

#[test]
fn test_write_and_read_cache() {
    // Create a temporary directory for testing
    let temp_dir = setup_temp_dir();
    let cache_path = temp_dir.path().join("test_cache.json");

    // Create test data
    let post = create_test_post();

    // Write data to cache
    let config = CacheConfig::default();
    write_cache(&cache_path, &post, config).expect("Failed to write cache");

    // Verify file was created
    assert!(cache_path.exists(), "Cache file was not created");

    // Read data from cache
    let cached_data = read_cache::<Post>(&cache_path, config).expect("Failed to read cache");

    // Verify the data was read correctly
    assert!(cached_data.is_some(), "Expected Some(CacheData), got None");

    let cache_data = cached_data.unwrap();
    assert_eq!(
        cache_data.payload, post,
        "Cached data doesn't match original"
    );

    // Verify the timestamp is recent
    let now = SystemTime::now();
    let age = now
        .duration_since(cache_data.timestamp)
        .expect("Cache timestamp is in the future");

    assert!(age < Duration::from_secs(5), "Cache timestamp is too old");
}

#[test]
fn test_cache_expiry() {
    // Create a temporary directory for testing
    let temp_dir = setup_temp_dir();
    let cache_path = temp_dir.path().join("expired_cache.json");

    // Create test data
    let post = create_test_post();

    // Create a manual cache with an expired timestamp
    // 2 hours in the past
    let two_hours = Duration::from_secs(2 * 60 * 60);
    let old_timestamp = SystemTime::now()
        .checked_sub(two_hours)
        .expect("Failed to create timestamp");

    let cache_data = CacheData {
        timestamp: old_timestamp,
        payload: post,
    };

    // Write the expired cache directly
    let json = serde_json::to_string_pretty(&cache_data).expect("Failed to serialize cache data");
    fs::write(&cache_path, json).expect("Failed to write cache file");

    // Try to read the cache with default config (1 hour max age)
    let config = CacheConfig::default();
    let cached_data = read_cache::<Post>(&cache_path, config).expect("Failed to read cache");

    // Verify the cache is considered expired
    assert!(cached_data.is_none(), "Cache should be expired but wasn't");

    // Try with a longer max age (3 hours)
    let config = CacheConfig {
        max_age_secs: 3 * 60 * 60,
        create_dirs: true,
    };
    let cached_data = read_cache::<Post>(&cache_path, config).expect("Failed to read cache");

    // Verify the cache is valid with the longer max age
    assert!(
        cached_data.is_some(),
        "Cache should be valid with longer max age"
    );
}

#[test]
fn test_missing_cache() {
    // Create a temporary directory for testing
    let temp_dir = setup_temp_dir();
    let nonexistent_path = temp_dir.path().join("nonexistent_cache.json");

    // Try to read from a cache that doesn't exist
    let config = CacheConfig::default();
    let cached_data = read_cache::<Post>(&nonexistent_path, config).expect("Failed to read cache");

    // Verify we get None for a missing cache file
    assert!(
        cached_data.is_none(),
        "Expected None for missing cache file"
    );
}

#[test]
fn test_create_cache_dirs() {
    // Create a temporary directory for testing
    let temp_dir = setup_temp_dir();
    let nested_path = temp_dir
        .path()
        .join("nested")
        .join("dirs")
        .join("cache.json");

    // Make sure the parent directories don't exist
    assert!(!nested_path.parent().unwrap().exists());

    // Create test data
    let post = create_test_post();

    // Write cache with create_dirs enabled
    let config = CacheConfig {
        max_age_secs: 60 * 60,
        create_dirs: true,
    };

    write_cache(&nested_path, &post, config).expect("Failed to write cache");

    // Verify the directories were created
    assert!(nested_path.parent().unwrap().exists());
    assert!(nested_path.exists(), "Cache file was not created");

    // Read the cache to verify it works
    let cached_data = read_cache::<Post>(&nested_path, config).expect("Failed to read cache");
    assert!(cached_data.is_some());
}

#[test]
fn test_cache_invalid_json() {
    // Create a temporary directory for testing
    let temp_dir = setup_temp_dir();
    let invalid_path = temp_dir.path().join("invalid_cache.json");

    // Write invalid JSON to the cache file
    fs::write(&invalid_path, "{ invalid json: this is not valid }").expect("Failed to write file");

    // Try to read the invalid cache
    let config = CacheConfig::default();
    let result = read_cache::<Post>(&invalid_path, config);

    // Verify we get an error
    assert!(result.is_err(), "Expected error for invalid JSON");
}
