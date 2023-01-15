use std::{
    ffi::{OsStr, OsString},
    fmt::Write,
    fs::File,
    io::BufWriter,
    io::Write as IoWrite,
    path::PathBuf,
};

use clap::{Parser, ValueEnum};
use lazy_static::lazy_static;
use rand::{rngs::SmallRng, seq::SliceRandom, Rng, SeedableRng};

/// Arguments are simple numbers
type Arg = usize;
/// Attacks are given by their origin and target
type Att = (usize, usize);

lazy_static! {
    /// Global command line arguments
    static ref ARGS: Args = Args::parse();
}

/// Possible output formats
#[derive(Debug, Clone, Copy, Default, ValueEnum)]
enum Format {
    Apx,
    #[default]
    Tgf,
}

impl Format {
    const fn as_initial_file_ending(&self) -> &'static str {
        match self {
            Format::Apx => "apx",
            Format::Tgf => "tgf",
        }
    }
    const fn as_update_file_ending(&self) -> &'static str {
        match self {
            Format::Apx => "apxm",
            Format::Tgf => "tgfm",
        }
    }
}

/// Possible update lines
enum UpdateLine {
    AddArg(Arg, Vec<Att>),
    DelArg(Arg),
    AddAtt(Att),
    DelAtt(Att),
}

impl UpdateLine {
    /// Generate a single new update line.
    pub fn generate(rng: &mut impl Rng, args: &[Arg], atts: &[Att]) -> Self {
        enum Options {
            AddArg,
            DelArg,
            AddAtt,
            DelAtt,
        }
        // We can always add arguments!
        let mut valid_options = vec![Options::AddArg];
        if !args.is_empty() {
            // Can only delete arguments or add attacks if any arguments exist
            valid_options.push(Options::DelArg);
            valid_options.push(Options::AddAtt);
        }
        if !atts.is_empty() {
            // Can only delete attacks if there are any
            valid_options.push(Options::DelAtt);
        }
        // The following `unwrap`s are all infallible
        let selected_option = valid_options.choose(rng).unwrap();
        match selected_option {
            Options::AddArg => {
                let max = args.iter().max().copied().unwrap_or_default();
                let new = max + 1;
                let mut atts: Vec<Att> = args
                    .iter()
                    .flat_map(|&other| [(new, other), (other, new)])
                    .filter(|_| rng.gen::<f32>() < ARGS.edge_prop_when_adding_arg)
                    .collect();
                if rng.gen::<f32>() < ARGS.edge_prop_when_adding_arg {
                    // Yes, attack yourself!
                    atts.push((new, new));
                }
                UpdateLine::AddArg(new, atts)
            }
            Options::AddAtt => {
                let from = args.choose(rng).unwrap();
                let to = args.choose(rng).unwrap();
                UpdateLine::AddAtt((*from, *to))
            }
            Options::DelArg => {
                let arg = args.choose(rng).unwrap();
                UpdateLine::DelArg(*arg)
            }
            Options::DelAtt => {
                let att = atts.choose(rng).unwrap();
                UpdateLine::DelAtt(*att)
            }
        }
    }

    /// Format this update line respecting the requested output format.
    fn format(&self) -> String {
        match ARGS.format {
            Format::Apx => match self {
                Self::AddArg(arg, atts) => {
                    let mut formatted = format!("+arg({arg})");
                    for (from, to) in atts {
                        write!(formatted, ":att({from},{to})").unwrap();
                    }
                    write!(formatted, ".").unwrap();
                    formatted
                }
                Self::DelArg(arg) => format!("-arg({arg})."),
                Self::AddAtt((from, to)) => format!("+att({from},{to})."),
                Self::DelAtt((from, to)) => format!("-att({from},{to})."),
            },
            Format::Tgf => match self {
                Self::AddArg(arg, atts) => {
                    let mut formatted = format!("+{arg}");
                    for (from, to) in atts {
                        write!(formatted, ":{from} {to}").unwrap();
                    }
                    write!(formatted, ".").unwrap();
                    formatted
                }
                Self::DelArg(arg) => format!("-{arg}"),
                Self::AddAtt((from, to)) => format!("+{from} {to}"),
                Self::DelAtt((from, to)) => format!("-{from} {to}"),
            },
        }
    }
}

/// Argumentation Framework.
#[derive(Debug, Clone)]
struct AF {
    /// Arguments
    args: Vec<Arg>,
    /// Attacks
    atts: Vec<Att>,
}

impl AF {
    /// Generate a new argumentation framework
    fn generate(rng: &mut impl Rng) -> Self {
        // Generate af arguments and attacks
        let args: Vec<usize> = generate_arguments().collect();
        let atts: Vec<(usize, usize)> = generate_attacks(rng).collect();
        Self { args, atts }
    }
    /// Write the initial file
    fn write_initial_file(&self) -> ::std::io::Result<()> {
        let initial_file_path = ARGS.get_initial_output_path();
        let mut output = BufWriter::new(File::create(initial_file_path)?);

        match ARGS.format {
            Format::Apx => {
                self.args
                    .iter()
                    .map(|arg| format!("arg({arg})."))
                    .try_for_each(|line| writeln!(output, "{line}"))?;
                self.atts
                    .iter()
                    .map(|(from, to)| format!("att({from},{to})."))
                    .try_for_each(|line| writeln!(output, "{line}"))?;
            }
            Format::Tgf => {
                self.args
                    .iter()
                    .map(|arg| format!("{arg}"))
                    .try_for_each(|line| writeln!(output, "{line}"))?;
                writeln!(output, "#")?;
                self.atts
                    .iter()
                    .map(|(from, to)| format!("{from} {to}"))
                    .try_for_each(|line| writeln!(output, "{line}"))?;
            }
        }
        Ok(())
    }
    /// Generate and apply updates
    fn generate_apply_updates(&mut self, rng: &mut impl Rng) -> Vec<UpdateLine> {
        let mut updates = vec![];
        for _ in 0..ARGS.nr_of_updates {
            let update = UpdateLine::generate(rng, &self.args, &self.atts);
            self.apply_update(&update);
            updates.push(update);
        }
        updates
    }
    /// Apply a single update line
    fn apply_update(&mut self, update: &UpdateLine) {
        match update {
            UpdateLine::AddArg(arg, atts) => {
                self.args.push(*arg);
                self.atts.extend(atts)
            }
            UpdateLine::DelArg(arg) => self.args.retain(|a| a != arg),
            UpdateLine::AddAtt(att) => self.atts.push(*att),
            UpdateLine::DelAtt(att) => self.atts.retain(|a| a != att),
        }
    }
}

/// Generate AFs and optional updates for the dynamic context.
#[derive(Debug, clap::Parser)]
struct Args {
    /// Size of the initial AF.
    #[arg(
        short = 'n',
        long = "size",
        default_value_t = 1_000,
        value_name = "NUM"
    )]
    arg_count: usize,
    /// Number of updates to generate.
    #[arg(short = 'u', long = "updates", default_value_t = 0, value_name = "NUM")]
    nr_of_updates: usize,
    /// Output path to write to.
    /// The main file will be written to PATH-initial.EXT.
    /// The update file will be written to PATH-updates.EXTm.
    #[arg(short, long, value_name = "PATH")]
    output: PathBuf,
    /// Format for written files.
    #[arg(short, long, value_name = "EXT")]
    format: Format,
    /// Edge propability
    #[arg(
        short = 'p',
        long = "edge",
        value_name = "FLOAT",
        default_value_t = 0.05
    )]
    edge_prop: f32,
    /// Probability by which attacks from and to every other argument
    /// should be selected when an argument-add update is created.
    /// If the argument `3` is added, consider every possible new attack and add it with this probability.
    #[arg(long = "update-edge", value_name = "FLOAT", default_value_t = 0.0025)]
    edge_prop_when_adding_arg: f32,
}

impl Args {
    fn get_initial_output_path(&self) -> PathBuf {
        let mut file_name = self
            .output
            .file_name()
            .map(OsStr::to_os_string)
            .unwrap_or_else(|| OsString::from("af"));
        write!(
            file_name,
            "-initial.{}",
            self.format.as_initial_file_ending()
        )
        .expect("Creating initial file path");
        self.output.with_file_name(file_name)
    }
    fn get_update_output_path(&self) -> PathBuf {
        let mut file_name = self
            .output
            .file_name()
            .map(OsStr::to_os_string)
            .unwrap_or_else(|| OsString::from("af"));
        write!(
            file_name,
            "-updates.{}",
            self.format.as_update_file_ending()
        )
        .expect("Creating initial file path");
        self.output.with_file_name(file_name)
    }
}

fn generate_arguments() -> impl Iterator<Item = usize> {
    0..ARGS.arg_count
}

fn generate_attacks<R: Rng>(rng: &mut R) -> impl Iterator<Item = (usize, usize)> + '_ {
    (0..ARGS.arg_count)
        .flat_map(|from| (0..ARGS.arg_count).map(move |to| (from, to)))
        .filter(|_| rng.gen::<f32>() < ARGS.edge_prop)
}

fn write_update_file(updates: &[UpdateLine]) -> ::std::io::Result<()> {
    let update_file_path = ARGS.get_update_output_path();
    let mut output = BufWriter::new(File::create(update_file_path)?);
    updates
        .iter()
        .map(|update| update.format())
        .try_for_each(|line| writeln!(output, "{line}"))
}

fn main() {
    // Initialize the PRNG
    let mut rng = SmallRng::from_rng(rand::thread_rng()).expect("Initializing RNG");
    // Generate AF
    let mut af = AF::generate(&mut rng);
    // Write the initial file
    af.write_initial_file().expect("Writing intial file");
    // Write update file
    let updates = af.generate_apply_updates(&mut rng);
    if !updates.is_empty() {
        // Only write the file if we actually have updates to write
        write_update_file(&updates).expect("Writing update file");
    }
}
