# 📸 Transparent Classroom Photos Grabber

> A Rust implementation to download photos from Transparent Classroom

[![MIT License](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)

## 🌟 Summary

Transparent Classroom Photos Grabber is a robust command-line tool that allows you to easily download and organize photos from your child's Transparent Classroom account. The application automatically handles authentication, crawls through all available posts, and downloads photos with proper metadata including GPS coordinates and timestamps.

Built in Rust for performance and reliability, this tool ensures you never miss precious classroom moments while maintaining a well-organized photo collection. It provides seamless handling of authentication, caching to avoid unnecessary downloads, and smart fallback mechanisms to ensure the best possible experience.

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

You can configure the application in several ways:

#### Option 1: Environment Variables

Set the following environment variables:

```bash
export TC_EMAIL="your.email@example.com"
export TC_PASSWORD="your_password"
export SCHOOL=12345        # Your school ID
export CHILD=67890         # Your child ID
export SCHOOL_LAT=41.9032  # School latitude
export SCHOOL_LNG=-87.6663 # School longitude
export SCHOOL_KEYWORDS="school, montessori, chicago"  # Keywords for metadata
```

#### Option 2: Create a .env File

Create a `.env` file in the project directory with these values (see `.env.example`).

#### Option 3: Interactive Setup

Run the setup command to configure the application interactively:

```bash
cargo run --release -- setup
```

### Running

```bash
# Basic usage (downloads to default ./photos directory)
cargo run --release

# Specify a custom output directory
cargo run --release -- --output /path/to/output/directory

# Run in dry-run mode (shows what would be downloaded without downloading)
cargo run --release -- --dry-run

# Show verbose output for debugging
cargo run --release -- --verbose
```

### Command-line Options

```
Usage: tc-photos-grabber [OPTIONS] [COMMAND]

Commands:
  setup     Set up configuration interactively
  download  Download photos (default command)
  config    Show current configuration
  help      Print this message or the help of the given subcommand(s)

Options:
  -o, --output <DIR>  Output directory for downloaded photos
      --dry-run       Run in dry-run mode (show what would be downloaded without downloading)
  -v, --verbose       Verbose output
  -h, --help          Print help
  -V, --version       Print version
```

## 🔧 Technical Information

### Architecture

The project is structured around these core components:

- **Client**: Handles authentication and API interactions with Transparent Classroom
- **Config**: Manages user configuration from environment variables or config files
- **Cache**: Provides efficient caching of API responses to reduce network requests
- **Error**: Centralized error handling for robust operation
- **CLI**: Command-line interface with intuitive options and clear output

### Features

- ✅ Secure authentication to Transparent Classroom with fallback mechanisms
- ✅ Automatic crawling of all posts with photos
- ✅ Smart caching to avoid unnecessary downloads
- ✅ Embedded metadata including GPS coordinates and timestamps
- ✅ Progress indicators for download tracking
- ✅ Dry-run mode to preview what would be downloaded
- ✅ Skip already downloaded photos for incremental updates
- ✅ Support for multiple authentication methods
- ✅ Detailed logging for troubleshooting
- ✅ Automatic directory creation and organization

### Dependencies

Key libraries used:

- `reqwest`: HTTP client for API requests
- `scraper`: HTML parsing for extracting photo information
- `chrono`: Date and time handling
- `serde`: Serialization/deserialization for JSON data
- `indicatif`: Progress bars for CLI experience
- `dotenv`: Environment variable management
- `clap`: Command-line argument parsing
- `colored`: Terminal output styling

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

# Build documentation
cargo doc --open
```

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgements

- Built by [Harper Reed](https://github.com/harperreed)
- Inspired by the original Python implementation
