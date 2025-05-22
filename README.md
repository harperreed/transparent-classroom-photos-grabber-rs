# 📸 Transparent Classroom Photos Grabber

> A Rust implementation to download photos from Transparent Classroom

[![MIT License](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)

## 🌟 Summary

Transparent Classroom Photos Grabber is a command-line tool that lets you easily download and organize photos from your child's Transparent Classroom account. It automatically handles authentication, crawls through all available posts, and downloads photos with proper metadata including location and timestamps.

Built in Rust for speed and reliability, this tool ensures you never miss a classroom moment while maintaining a well-organized photo collection.

## 🚀 How to Use

### Prerequisites

- Rust and Cargo installed ([install instructions](https://www.rust-lang.org/tools/install))
- Transparent Classroom account credentials
- School ID and Child ID from Transparent Classroom

### Installation

```bash
# Clone the repository
git clone https://github.com/harperreed/transparent-classroom-photos-grabber-rs.git
cd transparent-classroom-photos-grabber-rs

# Build the project
cargo build --release
```

### Configuration

You need to set the following environment variables:

```bash
export TC_EMAIL="your.email@example.com"
export TC_PASSWORD="your_password"
export SCHOOL=12345        # Your school ID
export CHILD=67890         # Your child ID
export SCHOOL_LAT=41.9032  # School latitude
export SCHOOL_LNG=-87.6663 # School longitude
export SCHOOL_KEYWORDS="school, montessori, chicago"  # Keywords for metadata
```

Alternatively, create a `.env` file in the project directory with these values (see `.env.example`).

### Running

```bash
# Download photos to default ./photos directory
cargo run --release

# Or specify a custom output directory
cargo run --release -- /path/to/output/directory
```

### Command-line Options

```
Usage: transparent-classroom-photos-grabber [OUTPUT_DIR]

If OUTPUT_DIR is not provided, photos will be saved to './photos'
```

## 🔧 Technical Information

### Architecture

The project is structured around these core components:

- **Client**: Handles authentication and API interactions with Transparent Classroom
- **Config**: Manages user configuration from environment variables
- **Cache**: Provides efficient caching of API responses to reduce network requests
- **Error**: Centralized error handling for robust operation

### Features

- ✅ Secure authentication to Transparent Classroom
- ✅ Automatic crawling of all posts with photos
- ✅ Smart caching to avoid unnecessary downloads
- ✅ Embedded metadata including GPS coordinates and timestamps
- ✅ Progress indicators for download tracking
- ✅ Fallback mechanisms for handling different API responses
- ✅ Skip already downloaded photos for incremental updates

### Dependencies

Key libraries used:

- `reqwest`: HTTP client for API requests
- `scraper`: HTML parsing for extracting photo information
- `chrono`: Date and time handling
- `serde`: Serialization/deserialization for JSON data
- `indicatif`: Progress bars for CLI experience
- `dotenv`: Environment variable management

### Development

For development and testing:

```bash
# Run tests
cargo test

# Run with debug logging
RUST_LOG=debug cargo run

# Format code
cargo fmt

# Check for issues
cargo clippy
```

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgements

- Built by [Harper Reed](https://github.com/harperreed)
- Inspired by the original Python implementation
