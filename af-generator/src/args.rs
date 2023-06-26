use std::{
    ffi::{OsStr, OsString},
    fmt::Write,
    path::PathBuf,
};

use clap::Parser;
use lazy_static::lazy_static;

use crate::Format;

lazy_static! {
    /// Global command line arguments
    pub static ref ARGS: Args = Args::parse();
}

/// Generate AFs and optional updates for the dynamic context.
#[derive(Debug, clap::Parser)]
pub struct Args {
    /// Size of the initial AF.
    #[arg(
        short = 'n',
        long = "size",
        default_value_t = 1_000,
        value_name = "NUM"
    )]
    pub arg_count: usize,
    /// Number of updates to generate.
    #[arg(short = 'u', long = "updates", default_value_t = 0, value_name = "NUM")]
    pub nr_of_updates: usize,
    /// Output path to write to.
    /// The main file will be written to PATH-initial.EXT.
    /// The update file will be written to PATH-updates.EXTm.
    #[arg(short, long, value_name = "PATH")]
    output: PathBuf,
    /// Format for written files.
    #[arg(short, long, value_name = "EXT")]
    pub format: Format,
    /// Edge propability
    #[arg(
        short = 'p',
        long = "edge",
        value_name = "FLOAT",
        default_value_t = 0.05
    )]
    pub edge_prop: f64,
    /// Probability by which attacks from and to every other argument
    /// should be selected when an argument-add update is created.
    /// If the argument `3` is added, consider every possible (optional) attack and add it with this probability.
    #[arg(long = "update-edge", value_name = "FLOAT", default_value_t = 0.25)]
    pub edge_prop_when_adding_arg: f64,
    /// Probability by which an argument is marked as optional. Updates will only change optional arguments and attacks.
    #[arg(long, value_name = "FLOAT", default_value_t = 0.05)]
    pub arg_optional_prop: f64,
    /// Probability by which an attack is marked as optional. Updates will only change optional arguments and attacks.
    #[arg(long, value_name = "FLOAT", default_value_t = 0.05)]
    pub attack_optional_prop: f64,
    /// Whether to write the intermediate frameworks to PATH-intermediate-NUMBER.EXT. These intermediates will be
    /// generated after every generated update and reflect the framework after this update.
    #[arg(long, default_value_t = false)]
    pub output_intermediates: bool,
}

impl Args {
    pub fn get_initial_output_path(&self) -> PathBuf {
        let mut file_name = self.output_file_name();
        write!(
            file_name,
            "-initial.{}",
            self.format.as_initial_file_ending()
        )
        .expect("Creating initial file path");
        self.output.with_file_name(file_name)
    }
    pub fn get_update_output_path(&self) -> PathBuf {
        let mut file_name = self.output_file_name();
        write!(
            file_name,
            "-updates.{}",
            self.format.as_update_file_ending()
        )
        .expect("Creating update file path");
        self.output.with_file_name(file_name)
    }
    pub fn get_intermediate_output_path(&self, nr: usize) -> PathBuf {
        let mut file_name = self.output_file_name();
        write!(
            file_name,
            "-intermediate-{}.{}",
            nr,
            self.format.as_initial_file_ending()
        )
        .expect("Creating intermediate file path");
        self.output.with_file_name(file_name)
    }

    fn output_file_name(&self) -> OsString {
        self.output
            .file_name()
            .map(OsStr::to_os_string)
            .unwrap_or_else(|| OsString::from("af"))
    }
}
