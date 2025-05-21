// ABOUTME: Caching implementation for Transparent Classroom API responses
// ABOUTME: Provides functionality to read and write cached data to disk as JSON

use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use crate::error::AppError;

/// Represents a cached item with timestamp
#[derive(Debug, Serialize, Deserialize)]
pub struct CacheData<T> {
    /// When the data was stored
    pub timestamp: SystemTime,

    /// The actual payload data
    pub payload: T,
}

/// Configuration for the cache behavior
#[derive(Debug, Clone, Copy)]
pub struct CacheConfig {
    /// Maximum age of cache data in seconds
    pub max_age_secs: u64,

    /// Whether to create the cache directory if it doesn't exist
    pub create_dirs: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_age_secs: 60 * 60, // 1 hour by default
            create_dirs: true,
        }
    }
}

/// Read cached data from disk
///
/// Returns:
/// - `Ok(Some(data))` if valid cache exists and isn't expired
/// - `Ok(None)` if cache doesn't exist or is expired
/// - `Err(_)` if there's an error reading or parsing the cache
pub fn read_cache<T>(path: &Path, config: CacheConfig) -> Result<Option<CacheData<T>>, AppError>
where
    T: for<'de> Deserialize<'de>,
{
    debug!("Attempting to read cache from: {}", path.display());

    // Check if the cache file exists
    if !path.exists() {
        debug!("Cache file doesn't exist at: {}", path.display());
        return Ok(None);
    }

    // Open and read the cache file
    let file = match File::open(path) {
        Ok(file) => file,
        Err(e) => {
            warn!("Failed to open cache file at {}: {}", path.display(), e);
            return Err(AppError::Io(e));
        }
    };

    let reader = BufReader::new(file);

    // Parse the JSON
    let cache_data: CacheData<T> = match serde_json::from_reader(reader) {
        Ok(data) => data,
        Err(e) => {
            warn!("Failed to parse cache data from {}: {}", path.display(), e);
            return Err(AppError::Generic(format!("Failed to parse cache: {}", e)));
        }
    };

    // Check if the cache is expired
    let now = SystemTime::now();
    let max_age = Duration::from_secs(config.max_age_secs);

    match now.duration_since(cache_data.timestamp) {
        Ok(age) if age > max_age => {
            debug!(
                "Cache at {} is expired (age: {:?}, max: {:?})",
                path.display(),
                age,
                max_age
            );
            Ok(None)
        }
        Ok(age) => {
            debug!("Using cache from {} (age: {:?})", path.display(), age);
            Ok(Some(cache_data))
        }
        Err(e) => {
            warn!(
                "Cache timestamp is in the future at {}: {}",
                path.display(),
                e
            );
            Ok(None)
        }
    }
}

/// Write data to cache on disk
///
/// Stores the data with the current timestamp in JSON format
pub fn write_cache<T>(path: &Path, data: T, config: CacheConfig) -> Result<(), AppError>
where
    T: Serialize,
{
    debug!("Writing cache to: {}", path.display());

    // Create parent directories if they don't exist and it's requested
    if config.create_dirs {
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                debug!("Creating cache directory: {}", parent.display());
                fs::create_dir_all(parent).map_err(|e| {
                    warn!(
                        "Failed to create cache directory at {}: {}",
                        parent.display(),
                        e
                    );
                    AppError::Io(e)
                })?;
            }
        }
    }

    // Create the cache data with current timestamp
    let cache_data = CacheData {
        timestamp: SystemTime::now(),
        payload: data,
    };

    // Write the JSON to file
    let file = match File::create(path) {
        Ok(file) => file,
        Err(e) => {
            warn!("Failed to create cache file at {}: {}", path.display(), e);
            return Err(AppError::Io(e));
        }
    };

    let writer = BufWriter::new(file);

    match serde_json::to_writer_pretty(writer, &cache_data) {
        Ok(()) => {
            info!("Successfully wrote cache to: {}", path.display());
            Ok(())
        }
        Err(e) => {
            warn!("Failed to write cache data to {}: {}", path.display(), e);
            Err(AppError::Generic(format!("Failed to write cache: {}", e)))
        }
    }
}

/// Get the default cache directory
pub fn default_cache_dir() -> PathBuf {
    if let Some(cache_dir) = dirs::cache_dir() {
        cache_dir.join("transparent-classroom-cache")
    } else {
        PathBuf::from("./.cache/transparent-classroom-cache")
    }
}
