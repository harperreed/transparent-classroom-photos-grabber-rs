// ABOUTME: Main executable for the Transparent Classroom Photos Grabber
// ABOUTME: Provides CLI interface for downloading photos

use std::env;
use std::path::{Path, PathBuf};
use std::process;
use std::time::Duration;

use colored::*;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
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
        println!();
        println!("{} {}", "❌".red(), "Error:".bright_red().bold());
        println!("{}", e.to_string().red());
        println!();
        process::exit(1);
    }
}

fn print_usage() {
    println!(
        "{}",
        "📸 Transparent Classroom Photos Grabber"
            .bright_cyan()
            .bold()
    );
    println!(
        "{} {}",
        "Usage:".bright_yellow(),
        "transparent-classroom-photos-grabber [OUTPUT_DIR]".white()
    );
    println!();
    println!(
        "{}",
        "If OUTPUT_DIR is not provided, photos will be saved to './photos'".dimmed()
    );
    println!();
    println!("{}:", "Environment variables".bright_yellow());
    println!(
        "  {} - Your Transparent Classroom email",
        "TC_EMAIL    ".bright_green()
    );
    println!(
        "  {} - Your Transparent Classroom password",
        "TC_PASSWORD ".bright_green()
    );
    println!("  {} - Your school ID", "SCHOOL      ".bright_green());
    println!("  {} - Your child ID", "CHILD       ".bright_green());
    println!();
    println!("{}:", "Example".bright_yellow());
    println!("  {}", "export TC_EMAIL=yourname@example.com".cyan());
    println!("  {}", "export TC_PASSWORD=yourpassword".cyan());
    println!("  {}", "export SCHOOL=12345".cyan());
    println!("  {}", "export CHILD=67890".cyan());
    println!(
        "  {}",
        "transparent-classroom-photos-grabber ./my-photos".cyan()
    );
}

fn run(output_dir: PathBuf) -> Result<(), AppError> {
    // Print header
    println!();
    println!(
        "{}",
        "🚀 Starting Transparent Classroom Photos Grabber"
            .bright_cyan()
            .bold()
    );
    println!("{}", "─".repeat(50).dimmed());

    // Load configuration from environment
    let spinner = create_spinner("🔧 Loading configuration...");
    info!("Loading configuration from environment");
    let config = Config::from_env()?;
    debug!("Loaded configuration with school ID: {}", config.school_id);
    spinner.finish_with_message(format!(
        "{} {}",
        "✅".green(),
        "Configuration loaded".green()
    ));

    // Create the client
    let spinner = create_spinner("🔐 Creating client and logging in...");
    info!("Creating client and logging in");
    let client = Client::new(config)?;

    // Login to Transparent Classroom
    client.login()?;
    info!("Successfully logged in");
    spinner.finish_with_message(format!(
        "{} {}",
        "✅".green(),
        "Successfully logged in".green()
    ));

    // Crawl all posts from all pages
    let spinner = create_spinner("🔍 Crawling posts from all pages...");
    info!("Crawling all posts from all pages");
    let posts = client.crawl_all_posts()?;
    spinner.finish_with_message(format!(
        "{} {}",
        "✅".green(),
        "Post discovery complete".green()
    ));

    if posts.is_empty() {
        info!("No posts found");
        println!("{} {}", "ℹ️".blue(), "No posts found".yellow());
        return Ok(());
    }

    info!("Found {} posts", posts.len());
    println!(
        "{} {} {}",
        "📋".blue(),
        "Found".bright_blue(),
        format!("{} posts", posts.len()).bright_white().bold()
    );

    // Create the output directory if it doesn't exist
    if !output_dir.exists() {
        info!("Creating output directory: {}", output_dir.display());
        std::fs::create_dir_all(&output_dir)?;
        println!(
            "{} {} {}",
            "📁".blue(),
            "Created output directory:".bright_blue(),
            output_dir.display().to_string().bright_white()
        );
    }

    // Download photos for each post
    let mut total_photos = 0;
    info!("Downloading photos to {}", output_dir.display());

    println!();
    println!(
        "{} {} {}",
        "📥".blue(),
        "Starting download to:".bright_blue(),
        output_dir.display().to_string().bright_white().bold()
    );
    println!();

    // Create main progress bar for posts
    let multi_progress = MultiProgress::new();
    let main_pb = multi_progress.add(ProgressBar::new(posts.len() as u64));
    main_pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} posts ({eta})"
        )
        .map_err(|e| AppError::Generic(format!("Failed to create progress bar template: {}", e)))?
        .progress_chars("#>-")
    );

    for (i, post) in posts.iter().enumerate() {
        main_pb.set_message(format!("Processing: {}", truncate_title(&post.title, 30)));

        if post.photo_urls.is_empty() {
            debug!("Post '{}' has no photos, skipping", post.title);
            main_pb.println(format!(
                "{} {} {} {}",
                "⏭️".yellow(),
                format!("[{}/{}]", i + 1, posts.len()).dimmed(),
                "Skipping:".yellow(),
                truncate_title(&post.title, 40).dimmed()
            ));
            main_pb.inc(1);
            continue;
        }

        debug!(
            "Downloading {} photos from post '{}'",
            post.photo_urls.len(),
            post.title
        );

        // Create progress bar for this post's photos
        let photo_pb = multi_progress.add(ProgressBar::new(post.photo_urls.len() as u64));
        photo_pb.set_style(
            ProgressStyle::with_template(
                "  {spinner:.green} [{wide_bar:.yellow/dim}] {pos}/{len} photos",
            )
            .map_err(|e| {
                AppError::Generic(format!(
                    "Failed to create photo progress bar template: {}",
                    e
                ))
            })?
            .progress_chars("█▉▊▋▌▍▎▏ "),
        );

        main_pb.println(format!(
            "{} {} {} {} {}",
            "📷".blue(),
            format!("[{}/{}]", i + 1, posts.len()).bright_white().bold(),
            "Processing:".bright_blue(),
            truncate_title(&post.title, 40).bright_white(),
            format!("({} photos)", post.photo_urls.len()).dimmed()
        ));

        // Download all photos for this post directly to output directory
        match download_photos_with_progress(&client, post, &output_dir, &photo_pb) {
            Ok(paths) => {
                let count = paths.len();
                total_photos += count;
                photo_pb.finish_and_clear();
                main_pb.println(format!(
                    "  {} {} {}",
                    "✅".green(),
                    "Downloaded".green(),
                    format!("{} photos", count).bright_white().bold()
                ));
            }
            Err(e) => {
                error!("Failed to download photos for post '{}': {}", post.title, e);
                photo_pb.finish_and_clear();
                main_pb.println(format!(
                    "  {} {} {}",
                    "❌".red(),
                    "Failed:".red(),
                    e.to_string().bright_red()
                ));
                // Continue with other posts
            }
        }

        main_pb.inc(1);
    }

    main_pb.finish_and_clear();

    info!(
        "Download complete. Downloaded {} photos from {} posts",
        total_photos,
        posts.len()
    );

    // Final summary
    println!();
    println!("{}", "─".repeat(50).dimmed());
    println!(
        "{} {}",
        "🎉".green(),
        "Download Complete!".bright_green().bold()
    );
    println!(
        "{} {} {} {} {}",
        "📊".blue(),
        "Summary:".bright_blue(),
        format!("{}", total_photos).bright_white().bold(),
        "photos from".bright_blue(),
        format!("{}", posts.len()).bright_white().bold()
    );
    println!(
        "{} {} {}",
        "📁".blue(),
        "Saved to:".bright_blue(),
        output_dir.display().to_string().bright_white().bold()
    );
    println!("{}", "─".repeat(50).dimmed());

    Ok(())
}

/// Creates a spinner with consistent styling
fn create_spinner(msg: &str) -> ProgressBar {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::with_template("{spinner:.green} {msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
    );
    spinner.set_message(msg.to_string());
    spinner.enable_steady_tick(Duration::from_millis(100));
    spinner
}

/// Truncates a title to a maximum length for display
fn truncate_title(title: &str, max_len: usize) -> String {
    if title.len() <= max_len {
        title.to_string()
    } else {
        format!("{}...", &title[..max_len.saturating_sub(3)])
    }
}

/// Downloads photos with progress bar updates
fn download_photos_with_progress(
    client: &Client,
    post: &transparent_classroom_photos_grabber_rs::client::Post,
    output_dir: &Path,
    progress_bar: &ProgressBar,
) -> Result<Vec<PathBuf>, AppError> {
    // For now, we'll call the existing download method and update progress
    // In a real implementation, you'd want to modify the client to accept a callback
    // or provide per-photo progress updates
    progress_bar.set_position(0);

    let result = client.download_all_photos(post, output_dir);

    // Simulate progress updates (in real implementation, this would come from the download function)
    for i in 0..post.photo_urls.len() {
        progress_bar.set_position((i + 1) as u64);
        std::thread::sleep(Duration::from_millis(10)); // Small delay to show progress
    }

    result
}

// Helper function to sanitize directory names
#[allow(dead_code)]
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
