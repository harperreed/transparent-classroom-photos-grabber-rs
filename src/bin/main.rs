// ABOUTME: Main executable for the Transparent Classroom Photos Grabber
// ABOUTME: Provides CLI interface for downloading photos

use std::env;
use std::path::PathBuf;
use std::process;

use log::{debug, error, info};
use transparent_classroom_photos_grabber_rs::{client::Client, config::Config, error::AppError};

fn main() {
    // Initialize the library (sets up logging)
    transparent_classroom_photos_grabber_rs::init();

    // Parse command-line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 && (args[1] == "-h" || args[1] == "--help") {
        print_usage();
        return;
    }

    // Get the output directory from args or use default
    let output_dir = if args.len() > 1 {
        PathBuf::from(&args[1])
    } else {
        PathBuf::from("./photos")
    };

    // Run the application
    if let Err(e) = run(output_dir) {
        error!("Application error: {}", e);
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn print_usage() {
    println!("Transparent Classroom Photos Grabber");
    println!("Usage: transparent-classroom-photos-grabber [OUTPUT_DIR]");
    println!();
    println!("If OUTPUT_DIR is not provided, photos will be saved to './photos'");
    println!();
    println!("Environment variables:");
    println!("  TC_EMAIL     - Your Transparent Classroom email");
    println!("  TC_PASSWORD  - Your Transparent Classroom password");
    println!("  SCHOOL       - Your school ID");
    println!("  CHILD        - Your child ID");
    println!();
    println!("Example:");
    println!("  export TC_EMAIL=yourname@example.com");
    println!("  export TC_PASSWORD=yourpassword");
    println!("  export SCHOOL=12345");
    println!("  export CHILD=67890");
    println!("  transparent-classroom-photos-grabber ./my-photos");
}

fn run(output_dir: PathBuf) -> Result<(), AppError> {
    // Load configuration from environment
    info!("Loading configuration from environment");
    let config = Config::from_env()?;
    debug!("Loaded configuration with school ID: {}", config.school_id);

    // Create the client
    info!("Creating client and logging in");
    let client = Client::new(config)?;

    // Login to Transparent Classroom
    client.login()?;
    info!("Successfully logged in");

    // Crawl all posts from all pages
    info!("Crawling all posts from all pages");
    let posts = client.crawl_all_posts()?;

    if posts.is_empty() {
        info!("No posts found");
        println!("No posts found");
        return Ok(());
    }

    info!("Found {} posts", posts.len());
    println!("Found {} posts", posts.len());

    // Create the output directory if it doesn't exist
    if !output_dir.exists() {
        info!("Creating output directory: {}", output_dir.display());
        std::fs::create_dir_all(&output_dir)?;
    }

    // Download photos for each post
    let mut total_photos = 0;
    info!("Downloading photos to {}", output_dir.display());

    for (i, post) in posts.iter().enumerate() {
        println!(
            "Processing post {}/{}: '{}'",
            i + 1,
            posts.len(),
            post.title
        );

        if post.photo_urls.is_empty() {
            debug!("Post '{}' has no photos, skipping", post.title);
            continue;
        }

        debug!(
            "Downloading {} photos from post '{}'",
            post.photo_urls.len(),
            post.title
        );

        // Download all photos for this post directly to output directory
        match client.download_all_photos(post, &output_dir) {
            Ok(paths) => {
                let count = paths.len();
                total_photos += count;
                println!("  Downloaded {} photos", count);
            }
            Err(e) => {
                error!("Failed to download photos for post '{}': {}", post.title, e);
                println!("  Failed to download photos: {}", e);
                // Continue with other posts
            }
        }
    }

    info!(
        "Download complete. Downloaded {} photos from {} posts",
        total_photos,
        posts.len()
    );
    println!(
        "\nDownload complete! Downloaded {} photos from {} posts",
        total_photos,
        posts.len()
    );
    println!("Photos saved to: {}", output_dir.display());

    Ok(())
}

// Helper function to sanitize directory names
fn sanitize_dirname(input: &str) -> String {
    let mut result = input.trim().to_owned();

    // Replace spaces with underscores
    result = result.replace(' ', "_");

    // Remove characters that are problematic in directory names
    result = result.replace(
        &['/', '\\', ':', '*', '?', '"', '<', '>', '|', '\''][..],
        "",
    );

    // Truncate if too long
    if result.len() > 30 {
        result.truncate(30);
    }

    result
}
