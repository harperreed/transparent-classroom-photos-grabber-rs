// ABOUTME: Main executable for the Transparent Classroom Photos Grabber
// ABOUTME: Provides CLI interface for downloading photos with setup and configuration options

use std::path::{Path, PathBuf};
use std::process;
use std::time::Duration;

use clap::Parser;
use colored::*;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use log::{debug, error, info};
use transparent_classroom_photos_grabber_rs::{
    cli::{Cli, Commands},
    client::Client,
    config::Config,
    error::AppError,
};

fn main() {
    // Initialize the library (sets up logging)
    transparent_classroom_photos_grabber_rs::init();

    // Parse command-line arguments
    let cli = Cli::parse();

    // Set log level based on verbose flag
    if cli.verbose {
        std::env::set_var("RUST_LOG", "debug");
    }

    // Run the application
    if let Err(e) = run(cli) {
        error!("Application error: {}", e);
        println!();
        println!("{} {}", "❌".red(), "Error:".bright_red().bold());
        println!("{}", e.to_string().red());
        println!();
        process::exit(1);
    }
}

fn run(cli: Cli) -> Result<(), AppError> {
    match cli.command {
        Some(Commands::Setup(setup_args)) => handle_setup(setup_args),
        Some(Commands::Download(download_args)) => handle_download(
            download_args.output.or(cli.output),
            download_args.dry_run || cli.dry_run,
        ),
        Some(Commands::Config(config_args)) => handle_config(config_args),
        None => handle_download(cli.output, cli.dry_run),
    }
}

fn handle_setup(
    args: transparent_classroom_photos_grabber_rs::cli::SetupArgs,
) -> Result<(), AppError> {
    println!();
    println!(
        "{}",
        "🛠️  Transparent Classroom Photos Grabber Setup"
            .bright_cyan()
            .bold()
    );
    println!("{}", "─".repeat(50).dimmed());

    // Check if config already exists
    if let Ok(config_path) = Config::get_config_file_path() {
        if config_path.exists() && !args.force {
            println!();
            println!(
                "{} {}",
                "ℹ️".blue(),
                "Configuration already exists!".yellow()
            );
            println!(
                "{} {} {}",
                "📄".blue(),
                "Location:".bright_blue(),
                config_path.display().to_string().bright_white()
            );
            println!();
            println!(
                "{}",
                "Use --force to overwrite existing configuration".dimmed()
            );
            return Ok(());
        }
    }

    // Run interactive setup
    let _config = Config::interactive_setup().map_err(AppError::Config)?;

    println!();
    println!("{}", "─".repeat(50).dimmed());
    println!(
        "{} {}",
        "🎉".green(),
        "Setup Complete!".bright_green().bold()
    );
    println!();
    println!("You can now run the application to download photos:");
    println!("  {}", "tc-photos-grabber".cyan());
    println!("{}", "─".repeat(50).dimmed());

    Ok(())
}

fn handle_config(
    args: transparent_classroom_photos_grabber_rs::cli::ConfigArgs,
) -> Result<(), AppError> {
    if args.path {
        match Config::get_config_file_path() {
            Ok(path) => {
                println!("{}", path.display());
                return Ok(());
            }
            Err(e) => {
                return Err(AppError::Config(e));
            }
        }
    }

    // Show current configuration
    println!();
    println!("{}", "📋 Current Configuration".bright_cyan().bold());
    println!("{}", "─".repeat(50).dimmed());

    match Config::load() {
        Ok(config) => {
            println!("{} {}", "Email:".bright_blue(), config.email.bright_white());
            println!(
                "{} {}",
                "School ID:".bright_blue(),
                config.school_id.to_string().bright_white()
            );
            println!(
                "{} {}",
                "Child ID:".bright_blue(),
                config.child_id.to_string().bright_white()
            );
            println!(
                "{} {}",
                "School Lat:".bright_blue(),
                config.school_lat.to_string().bright_white()
            );
            println!(
                "{} {}",
                "School Lng:".bright_blue(),
                config.school_lng.to_string().bright_white()
            );
            println!(
                "{} {}",
                "School Keywords:".bright_blue(),
                config.school_keywords.bright_white()
            );

            if let Ok(config_path) = Config::get_config_file_path() {
                println!();
                println!(
                    "{} {}",
                    "Config file:".bright_blue(),
                    config_path.display().to_string().bright_white()
                );
            }
        }
        Err(e) => {
            println!("{} {}", "❌".red(), "No configuration found".red());
            println!("{}", e.to_string().dimmed());
            println!();
            println!("Run setup to create configuration:");
            println!("  {}", "tc-photos-grabber setup".cyan());
        }
    }

    println!("{}", "─".repeat(50).dimmed());
    Ok(())
}

fn handle_download(output_dir: Option<PathBuf>, dry_run: bool) -> Result<(), AppError> {
    // Print header
    println!();
    if dry_run {
        println!(
            "{}",
            "🧪 Transparent Classroom Photos Grabber (Dry Run)"
                .bright_yellow()
                .bold()
        );
    } else {
        println!(
            "{}",
            "🚀 Transparent Classroom Photos Grabber"
                .bright_cyan()
                .bold()
        );
    }
    println!("{}", "─".repeat(50).dimmed());

    // Get the output directory from args or use default
    let output_dir = output_dir.unwrap_or_else(|| PathBuf::from("./photos"));

    // Load configuration from multiple sources
    let spinner = create_spinner("🔧 Loading configuration...");
    info!("Loading configuration");
    let config = Config::load()?;
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

    if dry_run {
        println!();
        println!(
            "{} {}",
            "🧪".yellow(),
            "DRY RUN MODE - No files will be downloaded"
                .bright_yellow()
                .bold()
        );
        println!();

        for (i, post) in posts.iter().enumerate() {
            println!(
                "{} {} {} {}",
                "📄".blue(),
                format!("[{}/{}]", i + 1, posts.len()).bright_white().bold(),
                truncate_title(&post.title, 50).bright_white(),
                format!("({} photos)", post.photo_urls.len()).dimmed()
            );
        }

        let total_photos: usize = posts.iter().map(|p| p.photo_urls.len()).sum();
        println!();
        println!("{} {}", "📊".blue(), "Summary:".bright_blue());
        println!(
            "  Would download {} photos from {} posts",
            total_photos,
            posts.len()
        );
        println!("  Would save to: {}", output_dir.display());
        return Ok(());
    }

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
