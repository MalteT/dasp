use clap::{Parser, ValueEnum};
use lazy_static::lazy_static;

use std::path::PathBuf;

use crate::{Error, Result};

use self::path_or_stdin::PathOrStdin;

mod path_or_stdin;

lazy_static! {
    /// Command line arguments
    static ref ARGS: Args = Args::parse();
}

/// Enumeration of all possible tasks
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum CliTask {
    CeAd,
    CeAdD,
    CeCo,
    CeCoD,
    CeSt,
    CeStD,
    EeAd,
    EeAdD,
    EeCo,
    EeCoD,
    EeSt,
    EeStD,
    SeAd,
    SeAdD,
    SeCo,
    SeCoD,
    SeSt,
    SeStD,
}

/// Possible file formats
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum FileFormat {
    Tgf,
    Apx,
}

/// Modulear ASP solver FOr Dynamics
#[derive(Debug, Parser)]
#[command(version, about)]
pub struct Args {
    /// File to load
    #[arg(short, long, requires = "file_format")]
    file: Option<PathBuf>,
    /// Task to execute
    #[arg(short = 'p', long, requires = "file")]
    task: Option<CliTask>,
    /// Show supprted formats
    #[arg(long)]
    formats: bool,
    /// Show supported problems
    #[arg(long)]
    problems: bool,
    /// File format for `--file`
    #[arg(long = "fo")]
    file_format: Option<FileFormat>,
    /// Additional parameter for the problem
    #[arg(long, short)]
    additional_parameter: Option<String>,
    /// File to read updates from. Use '-' for stdin
    #[arg(long, short, default_value_t = PathOrStdin::Stdin)]
    update_file: PathOrStdin,
}

impl Args {
    pub fn file_content(&self) -> Result<Option<String>> {
        self.file
            .as_ref()
            .map(::std::fs::read_to_string)
            .transpose()
            .map_err(Error::from)
    }

    pub fn should_show_problems(&self) -> bool {
        self.problems
    }

    pub fn should_show_formats(&self) -> bool {
        self.formats
    }

    pub fn task(&self) -> Option<CliTask> {
        self.task
    }

    pub fn update_file(&self) -> &PathOrStdin {
        &self.update_file
    }
}
