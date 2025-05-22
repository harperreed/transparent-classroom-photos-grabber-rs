// ABOUTME: Command-line interface definition and argument parsing
// ABOUTME: Handles CLI commands like setup, help, and configuration options

use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "tc-photos-grabber")]
#[command(about = "A Rust implementation to download photos from Transparent Classroom")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Output directory for downloaded photos
    #[arg(short, long, value_name = "DIR")]
    pub output: Option<PathBuf>,

    /// Run in dry-run mode (show what would be downloaded without downloading)
    #[arg(long)]
    pub dry_run: bool,

    /// Verbose output
    #[arg(short, long)]
    pub verbose: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Set up configuration interactively
    Setup(SetupArgs),
    /// Download photos (default command)
    Download(DownloadArgs),
    /// Show current configuration
    Config(ConfigArgs),
}

#[derive(Args)]
pub struct SetupArgs {
    /// Force setup even if config file exists
    #[arg(long)]
    pub force: bool,
}

#[derive(Args)]
pub struct DownloadArgs {
    /// Output directory for downloaded photos
    #[arg(short, long, value_name = "DIR")]
    pub output: Option<PathBuf>,

    /// Run in dry-run mode
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Args)]
pub struct ConfigArgs {
    /// Show configuration file path
    #[arg(long)]
    pub path: bool,
}
