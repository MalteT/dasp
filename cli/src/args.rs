use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use lazy_static::lazy_static;

use crate::path_or_stdin::PathOrStdin;

lazy_static! {
    /// Command line arguments
    pub static ref ARGS: Args = Args::parse();
}

/// Enumeration of all possible tasks
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum CliTask {
    CeAd,
    CeAdD,
    EeAd,
    EeAdD,
    SeAd,
    SeAdD,
}

/// Modulear ASP solver FOr Dynamics
#[derive(Debug, Parser)]
#[command(version, about)]
pub struct Args {
    /// File to load.
    #[arg(short, long)]
    pub file: PathBuf,
    /// Task to execute
    #[arg(short = 'p', long, requires = "file")]
    pub task: CliTask,
    /// File to read updates from. Use '-' for stdin
    #[arg(long, short, default_value_t = PathOrStdin::Stdin)]
    pub update_file: PathOrStdin,
}
