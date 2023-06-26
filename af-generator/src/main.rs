//! Tool to generate random argumentation frameworks
use std::{fmt::Write, fs::File, io::BufWriter, io::Write as IoWrite};

use clap::ValueEnum;
use rand::{rngs::SmallRng, seq::SliceRandom, Rng, SeedableRng};
use types::{Argument, ArgumentWithState, Attack, AttackWithState, State};

mod args;
mod types;

use args::ARGS;

/// Possible output formats
#[derive(Debug, Clone, Copy, Default, ValueEnum)]
pub enum Format {
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
    EnableArgument(Argument, Vec<Attack>),
    DisableArgument(Argument),
    EnableAttack(Attack),
    DisableAttack(Attack),
}

impl UpdateLine {
    /// Generate a single new update line.
    pub fn generate(
        rng: &mut impl Rng,
        args: &[ArgumentWithState],
        attacks: &[AttackWithState],
    ) -> Option<Self> {
        enum Options {
            EnableArgument,
            DisableArgument,
            EnableAttack,
            DisableAttack,
        }
        let mut valid_options: Vec<Options> = vec![];
        let dead_args = args
            .iter()
            .filter(|(_, state)| *state == State::Dead)
            .collect::<Vec<_>>();
        let dead_attacks = attacks
            .iter()
            .filter(|(_, state)| *state == State::Dead)
            .collect::<Vec<_>>();
        let alive_but_optional_args = args
            .iter()
            .filter(|(arg, state)| arg.optional && *state == State::Alive)
            .collect::<Vec<_>>();
        let alive_but_optional_attacks = attacks
            .iter()
            .filter(|(attack, state)| attack.optional && *state == State::Alive)
            .collect::<Vec<_>>();
        if !dead_args.is_empty() {
            valid_options.push(Options::EnableArgument);
        }
        if !dead_attacks.is_empty() {
            valid_options.push(Options::EnableAttack);
        }
        if !alive_but_optional_args.is_empty() {
            valid_options.push(Options::DisableArgument)
        }
        if !alive_but_optional_attacks.is_empty() {
            valid_options.push(Options::DisableAttack)
        }
        // There may not be a valid option to apply..
        let selected_option = valid_options.choose(rng)?;
        match selected_option {
            Options::EnableArgument => {
                // We know that there are dead_arguments by the above logic
                let (arg, _) = dead_args.choose(rng).unwrap();
                let attacks: Vec<_> = dead_attacks
                    .into_iter()
                    .filter(|(attack, _)| attack.contains(arg))
                    .filter(|_| rng.gen_bool(ARGS.edge_prop_when_adding_arg))
                    .map(|(attack, _)| *attack)
                    .collect();
                Some(UpdateLine::EnableArgument(*arg, attacks))
            }
            Options::EnableAttack => {
                // We know that there are dead_attacks by the above logic
                let (attack, _) = dead_attacks.choose(rng).unwrap();
                Some(UpdateLine::EnableAttack(*attack))
            }
            Options::DisableArgument => {
                // We know that there are alive_but_optional_args by the above logic
                let (arg, _) = alive_but_optional_args.choose(rng).unwrap();
                Some(UpdateLine::DisableArgument(*arg))
            }
            Options::DisableAttack => {
                // We know that there are alive_but_optional_attacks by the above logic
                let (attack, _) = alive_but_optional_attacks.choose(rng).unwrap();
                Some(UpdateLine::DisableAttack(*attack))
            }
        }
    }

    /// Format this update line respecting the requested output format.
    fn format(&self) -> String {
        match ARGS.format {
            Format::Apx => match self {
                Self::EnableArgument(arg, atts) => {
                    let mut formatted = format!("+arg({})", arg.name());
                    for attack in atts {
                        write!(formatted, ":att({}, {})", attack.from(), attack.to()).unwrap();
                    }
                    write!(formatted, ".").unwrap();
                    formatted
                }
                Self::DisableArgument(arg) => format!("-arg({}).", arg.name()),
                Self::EnableAttack(attack) => format!("+att({}, {}).", attack.from(), attack.to()),
                Self::DisableAttack(attack) => format!("-att({}, {}).", attack.from(), attack.to()),
            },
            Format::Tgf => match self {
                Self::EnableArgument(arg, atts) => {
                    let mut formatted = format!("+{}", arg.name());
                    for attack in atts {
                        write!(formatted, ":{} {}", attack.from(), attack.to()).unwrap();
                    }
                    write!(formatted, ".").unwrap();
                    formatted
                }
                Self::DisableArgument(arg) => format!("-{}", arg.name()),
                Self::EnableAttack(attack) => format!("+{} {}", attack.from(), attack.to()),
                Self::DisableAttack(attack) => format!("-{} {}", attack.from(), attack.to()),
            },
        }
    }
}

/// Argumentation Framework.
#[derive(Debug, Clone)]
struct AF {
    /// Arguments
    args: Vec<ArgumentWithState>,
    /// Attacks
    atts: Vec<AttackWithState>,
}

impl AF {
    /// Generate a new argumentation framework
    fn generate(rng: &mut impl Rng) -> Self {
        // Generate af arguments and attacks
        let args = generate_arguments(rng)
            .map(|arg| {
                (
                    arg,
                    if arg.optional {
                        State::Dead
                    } else {
                        State::Alive
                    },
                )
            })
            .collect();
        let atts = generate_attacks(rng)
            .map(|attack| {
                (
                    attack,
                    if attack.optional {
                        State::Dead
                    } else {
                        State::Alive
                    },
                )
            })
            .collect();
        Self { args, atts }
    }
    fn write_framework_to_file(
        &self,
        output: &mut BufWriter<File>,
        alive_only: bool,
    ) -> ::std::io::Result<()> {
        match ARGS.format {
            Format::Apx => {
                self.args
                    .iter()
                    .filter(|(_, state)| !alive_only || *state == State::Alive)
                    .map(|(arg, _)| {
                        let arg_string = format!("arg({})", arg.name());
                        if !alive_only && arg.optional {
                            format!("{arg_string}. opt({arg_string}).")
                        } else {
                            format!("{arg_string}.")
                        }
                    })
                    .try_for_each(|line| writeln!(output, "{line}"))?;
                self.atts
                    .iter()
                    .filter(|(_, state)| !alive_only || *state == State::Alive)
                    .map(|(attack, _)| {
                        let attack_string = format!("att({}, {})", attack.from(), attack.to());
                        if !alive_only && attack.optional {
                            format!("{attack_string}. opt({attack_string}).")
                        } else {
                            format!("{attack_string}.")
                        }
                    })
                    .try_for_each(|line| writeln!(output, "{line}"))?;
            }
            Format::Tgf => {
                self.args
                    .iter()
                    .filter(|(_, state)| !alive_only || *state == State::Alive)
                    .map(|(arg, _)| {
                        format!(
                            "{}{}",
                            arg.name(),
                            if !alive_only && arg.optional { "?" } else { "" }
                        )
                    })
                    .try_for_each(|line| writeln!(output, "{line}"))?;
                writeln!(output, "#")?;
                self.atts
                    .iter()
                    .filter(|(_, state)| !alive_only || *state == State::Alive)
                    .map(|(attack, _)| {
                        format!(
                            "{} {}{}",
                            attack.from(),
                            attack.to(),
                            if !alive_only && attack.optional {
                                "?"
                            } else {
                                ""
                            }
                        )
                    })
                    .try_for_each(|line| writeln!(output, "{line}"))?;
            }
        }
        Ok(())
    }
    /// Write the initial file
    fn write_initial_file(&self) -> ::std::io::Result<()> {
        let initial_file_path = ARGS.get_initial_output_path();
        let mut output = BufWriter::new(File::create(initial_file_path)?);
        self.write_framework_to_file(&mut output, false)
    }
    fn write_intermediate_file(&self, nr: usize) -> ::std::io::Result<()> {
        let initial_file_path = ARGS.get_intermediate_output_path(nr);
        let mut output = BufWriter::new(File::create(initial_file_path)?);
        self.write_framework_to_file(&mut output, true)
    }
    /// Generate and apply updates
    fn generate_apply_updates(&mut self, rng: &mut impl Rng) -> Vec<UpdateLine> {
        let mut updates = vec![];
        // Initial intermediate, without `opt`s
        if ARGS.output_intermediates {
            if let Err(why) = self.write_intermediate_file(0) {
                log::warn!("Failed to write intermediate number 0: {why}");
            }
        }
        for update_nr in 1..=ARGS.nr_of_updates {
            let update = UpdateLine::generate(rng, &self.args, &self.atts);
            match update {
                Some(update) => {
                    self.apply_update(&update);
                    if ARGS.output_intermediates {
                        if let Err(why) = self.write_intermediate_file(update_nr) {
                            log::warn!("Failed to write intermediate number {update_nr}: {why}");
                        }
                    }
                    updates.push(update);
                }
                None => {
                    log::error!("Could not find any update to generate. No optional arguments or attacks exist")
                }
            }
        }
        updates
    }
    /// Apply a single update line
    fn apply_update(&mut self, update: &UpdateLine) {
        match update {
            UpdateLine::EnableArgument(arg, attacks) => {
                self.args
                    .iter_mut()
                    .find(|(argument, _)| argument == arg)
                    .expect("BUG: Could not find argument to enable")
                    .1 = State::Alive;
                self.atts
                    .iter_mut()
                    .filter(|(attack, _)| attacks.contains(attack))
                    .for_each(|(_, state)| *state = State::Alive);
            }
            UpdateLine::DisableArgument(arg) => {
                self.args
                    .iter_mut()
                    .find(|(argument, _)| argument == arg)
                    .expect("BUG: Could not find argument to disable")
                    .1 = State::Dead
            }
            UpdateLine::EnableAttack(att) => {
                self.atts
                    .iter_mut()
                    .find(|(attack, _)| att == attack)
                    .expect("BUG: Could not find attack to enable")
                    .1 = State::Alive
            }
            UpdateLine::DisableAttack(att) => {
                self.atts
                    .iter_mut()
                    .find(|(attack, _)| att == attack)
                    .expect("BUG: Could not find attack to disable")
                    .1 = State::Dead
            }
        }
    }
}

fn generate_arguments<R: Rng>(rng: &'_ mut R) -> impl Iterator<Item = Argument> + '_ {
    (0..ARGS.arg_count).map(|id| {
        let optional = rng.gen_bool(ARGS.arg_optional_prop);
        Argument::new(id, optional)
    })
}

fn generate_attacks<R: Rng>(rng: &'_ mut R) -> impl Iterator<Item = Attack> + '_ {
    (0..ARGS.arg_count)
        .flat_map(|from| (0..ARGS.arg_count).map(move |to| (from, to)))
        .filter_map(|(from, to)| {
            if rng.gen_bool(ARGS.edge_prop) {
                let optional = rng.gen_bool(ARGS.attack_optional_prop);
                Some(Attack::from_raw(from, to, optional))
            } else {
                None
            }
        })
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
