use ::clingo::{FactBase, Part};
use clingo::Symbol;

use super::{
    semantics::ArgumentationFrameworkSemantic,
    symbols::{self, RevisionedSymbol},
    Control,
};

use crate::Result;

pub struct Logger;

impl ::clingo::Logger for Logger {
    fn log(&mut self, code: clingo::Warning, message: &str) {
        log::info!(target: "clingo", "[{code:?}] {message}")
    }
}

/// Initialize the clingo backend
///
/// Loads the given args and attacks
pub fn initialize_backend<S: ArgumentationFrameworkSemantic>(
    args: &[symbols::Argument],
    attacks: &[symbols::Attack],
) -> Result<Control> {
    let clingo_params = assemble_clingo_parameters();
    let mut ctl = ::clingo::control_with_logger(clingo_params, Logger, u32::MAX)?;
    // Add the base program
    ctl.add("base", &[], S::BASE)?;
    // Add the configured theory
    ctl.add("theory", &["revision"], S::THEORY)?;
    // Add the initial arguments and attacks as facts
    // TODO: What does it mean, is it adding the to the base?
    ctl.add_facts(&create_factbase_from_args_and_attacks(args, attacks))?;
    ground(&mut ctl, 0)?;
    Ok(ctl)
}

fn ground(ctl: &mut Control, revision: u32) -> Result {
    let parts = match revision {
        0 => {
            log::trace!("Grounding programs: base(), theory({revision})");
            vec![
                Part::new("base", vec![])?,
                Part::new("theory", vec![Symbol::create_number(revision as i32)])?,
            ]
        }
        1.. => {
            log::trace!("Grounding programs: update_{revision}(), theory({revision})");
            vec![
                Part::new(&format!("update_{revision}"), vec![])?,
                Part::new("theory", vec![Symbol::create_number(revision as i32)])?,
            ]
        }
    };
    ctl.ground(&parts)?;
    Ok(())
}

#[deprecated = "don't use factbase, create a base program instead for consistency with the rest of the code"]
fn create_factbase_from_args_and_attacks(
    args: &[symbols::Argument],
    attacks: &[symbols::Attack],
) -> FactBase {
    let mut fb = FactBase::new();
    args.iter()
        .for_each(|arg| fb.insert(&arg.symbol(0).unwrap()));
    attacks
        .iter()
        .for_each(|attack| fb.insert(&attack.symbol(0).unwrap()));
    fb
}

fn assemble_clingo_parameters() -> Vec<String> {
    // Assemble clingo parameters
    // FIXME: Make core count flexible
    vec![
        "--warn=all",
        // Use multiple cores [--parallel-mode 12]
        "--parallel-mode",
        "12",
        // Always prepare to compute all models [0]
        "0",
    ]
    .into_iter()
    .map(String::from)
    .collect()
}

fn add_program(ctl: &mut Control, name: &str, content: &str, revision: u32) -> Result {
    // Add argument to update_{revision} program
    log::trace!("Adding program {name:?} to clingo: {content:?}");
    ctl.add(&name, &[], &content)?;
    // Re-Ground
    ground(ctl, revision)?;
    Ok(())
}

pub fn add_argument<S: ArgumentationFrameworkSemantic>(
    ctl: &mut Control,
    argument: &symbols::Argument,
    revision: u32,
) -> Result {
    let name = format!("update_{revision}");
    let content = format!(r"{}. {}", argument.symbol(revision)?.to_string(), S::UPDATE);
    add_program(ctl, &name, &content, revision)
}

pub fn add_attack<S: ArgumentationFrameworkSemantic>(
    ctl: &mut Control,
    attack: &symbols::Attack,
    revision: u32,
) -> Result {
    let name = format!("update_{revision}");
    let content = format!(r"{}. {}", attack.symbol(revision)?.to_string(), S::UPDATE);
    add_program(ctl, &name, &content, revision)
}

pub fn remove_argument<S: ArgumentationFrameworkSemantic>(
    ctl: &mut Control,
    argument: &symbols::Argument,
    revision: u32,
) -> Result {
    let name = format!("update_{revision}");
    let content = format!(
        "{}. {}",
        symbols::Delete(argument).symbol(revision)?.to_string(),
        S::UPDATE
    );
    add_program(ctl, &name, &content, revision)
}

pub fn remove_attack<S: ArgumentationFrameworkSemantic>(
    ctl: &mut Control,
    attack: &symbols::Attack,
    revision: u32,
) -> Result {
    let name = format!("update_{revision}");
    let content = format!(
        "{}. {}",
        symbols::Delete(attack).symbol(revision)?.to_string(),
        S::UPDATE
    );
    add_program(ctl, &name, &content, revision)
}
