//! Main interface for communication between this library and clingo
//!

use ::clingo::Part;
use clingo::SolverLiteral;

use super::{semantics::ArgumentationFrameworkSemantic, symbols, Control};

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
    // Add the facts
    let facts = args.iter().fold(String::new(), |acc, argument| {
        if argument.optional {
            acc + &format!(r#"#external argument({}). "#, argument.id)
        } else {
            acc + &format!(r#"argument({}). "#, argument.id)
        }
    });
    let facts = attacks.iter().fold(facts, |acc, attack| {
        if attack.optional {
            acc + &format!(r#"#external attack({}, {}). "#, attack.from, attack.to)
        } else {
            acc + &format!(r#"attack({}, {}). "#, attack.from, attack.to)
        }
    });
    ctl.add("facts", &[], &facts)?;
    // Add the base program
    ctl.add("base", &[], S::BASE)?;
    ctl.add(
        "show",
        &[],
        r#"
            #show.
            #show X: in(X).
        "#,
    )?;
    ground(&mut ctl)?;
    Ok(ctl)
}

fn ground(ctl: &mut Control) -> Result {
    log::trace!("Grounding programs: base(), show(), and facts()");
    let parts = vec![
        Part::new("base", vec![])?,
        Part::new("show", vec![])?,
        Part::new("facts", vec![])?,
    ];
    ctl.ground(&parts)?;
    Ok(())
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

pub fn enable_argument(ctl: &mut Control, argument: SolverLiteral) -> Result {
    ctl.assign_external(argument, clingo::TruthValue::True)?;
    Ok(())
}

pub fn disable_argument(ctl: &mut Control, argument: SolverLiteral) -> Result {
    ctl.assign_external(argument, clingo::TruthValue::False)?;
    Ok(())
}

pub fn enable_attack(ctl: &mut Control, attack: SolverLiteral) -> Result {
    ctl.assign_external(attack, clingo::TruthValue::True)?;
    Ok(())
}

pub fn disable_attack(ctl: &mut Control, attack: SolverLiteral) -> Result {
    ctl.assign_external(attack, clingo::TruthValue::False)?;
    Ok(())
}
