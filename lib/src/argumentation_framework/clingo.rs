use clingo::{ClingoError, Control, FactBase, Part, Symbol, SymbolicAtom, TruthValue};
use fallible_iterator::FallibleIterator;
use log::warn;

use super::{semantics::ArgumentationFrameworkSemantic, symbols};

use crate::Result;

const CLINGO_SYMBOL_ARG_EXTERNAL: &str = "arg_";
const CLINGO_SYMBOL_ATT_EXTERNAL: &str = "att_";

/// Initialize the clingo backend
///
/// Loads the given args and attacks
pub fn initialize_backend<S: ArgumentationFrameworkSemantic>(
    args: &[symbols::Argument],
    attacks: &[symbols::Attack],
) -> Result<::clingo::Control> {
    let clingo_params = assemble_clingo_parameters();
    let mut ctl = ::clingo::control(clingo_params)?;
    let facts = create_factbase_from_args_and_attacks(args, attacks);
    ctl.add("theory", &[], S::PROGRAM)?;
    ctl.add("show", &[], "#show. #show X : in(X).")?;
    ctl.add_facts(&facts)?;
    ground_all_parts(&mut ctl)?;
    // Set external literals
    // We start with all arguments and attacks enabled
    enable_all_external_args_and_attacks(&mut ctl)?;
    Ok(ctl)
}

fn ground_all_parts(ctl: &mut Control) -> Result {
    let base = Part::new("base", vec![])?;
    let theory = Part::new("theory", vec![])?;
    let show = Part::new("show", vec![])?;
    ctl.ground(&[base, theory, show])?;
    Ok(())
}

fn enable_all_external_args_and_attacks(ctl: &mut Control) -> Result {
    let symbolic_atoms = fallible_iterator::convert(
        ctl.symbolic_atoms()?
            .iter()?
            .map(Result::<_, ClingoError>::Ok),
    );
    symbolic_atoms
        // Make sure we're talking about external atoms
        .filter(SymbolicAtom::is_external)
        // We only care about the symbol and the literal
        .map(|atom| Ok((atom.symbol()?, atom.literal()?)))
        .filter(|(symbol, _literal)| {
            let name = symbol.name()?;
            Ok(name == CLINGO_SYMBOL_ARG_EXTERNAL || name == CLINGO_SYMBOL_ATT_EXTERNAL)
        })
        .for_each(|(_, literal)| ctl.assign_external(literal, TruthValue::True))?;
    Ok(())
}

fn create_factbase_from_args_and_attacks(
    args: &[symbols::Argument],
    attacks: &[symbols::Attack],
) -> FactBase {
    let mut fb = FactBase::new();
    args.iter().for_each(|arg| fb.insert(arg));
    attacks.iter().for_each(|attack| fb.insert(attack));
    fb
}

fn assemble_clingo_parameters() -> Vec<String> {
    // Assemble clingo parameters
    // FIXME: Make core count flexible
    vec![
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

pub fn add_argument_to_facts_and_enable_external_symbol(
    ctl: &mut Control,
    argument: &symbols::Argument,
) -> Result {
    // Add to facts
    let mut facts = FactBase::new();
    facts.insert(&argument);
    ctl.add_facts(&facts)?;
    // Re-Ground
    ground_all_parts(ctl)?;
    // Enable the external symbol
    let mut symbolic_atoms = fallible_iterator::convert(
        ctl.symbolic_atoms()?
            .iter()?
            .map(Result::<_, ClingoError>::Ok),
    );
    let requested_symbol = Symbol::create_function(
        CLINGO_SYMBOL_ARG_EXTERNAL,
        &[Symbol::create_string(&argument.0)?],
        true,
    )?;
    // We've just added the symbol during grounding, this should never panic
    let atom = symbolic_atoms
        .find(|atom| Ok(atom.symbol()? == requested_symbol))?
        .expect("BUG: The requested symbol was just added. This should not panic!");
    ctl.assign_external(atom.literal()?, TruthValue::True)?;
    Ok(())
}

pub fn disable_external_argument_symbol(ctl: &mut Control, argument: &symbols::Argument) -> Result {
    let mut symbolic_atoms = fallible_iterator::convert(
        ctl.symbolic_atoms()?
            .iter()?
            .map(Result::<_, ClingoError>::Ok),
    );
    let requested_symbol = Symbol::create_function(
        CLINGO_SYMBOL_ARG_EXTERNAL,
        &[Symbol::create_string(&argument.0)?],
        true,
    )?;
    let atom = symbolic_atoms.find(|atom| Ok(atom.symbol()? == requested_symbol))?;
    if let Some(atom) = atom {
        ctl.assign_external(atom.literal()?, TruthValue::True)?;
    } else {
        warn!("Requested disabling argument, but argument does not exist");
    }
    Ok(())
}

pub fn disable_external_attack_symbol(ctl: &mut Control, attack: &symbols::Attack) -> Result {
    let mut symbolic_atoms = fallible_iterator::convert(
        ctl.symbolic_atoms()?
            .iter()?
            .map(Result::<_, ClingoError>::Ok),
    );
    let requested_symbol = Symbol::create_function(
        CLINGO_SYMBOL_ATT_EXTERNAL,
        &[
            Symbol::create_string(&attack.0)?,
            Symbol::create_string(&attack.1)?,
        ],
        true,
    )?;
    let atom = symbolic_atoms.find(|atom| Ok(atom.symbol()? == requested_symbol))?;
    if let Some(atom) = atom {
        ctl.assign_external(atom.literal()?, TruthValue::True)?;
    } else {
        warn!("Requested disabling attack, but attack does not exist");
    }
    Ok(())
}

pub fn add_attack_to_facts_and_enable_external_symbol(
    ctl: &mut Control,
    attack: &symbols::Attack,
) -> Result {
    // Add to facts
    let mut facts = FactBase::new();
    facts.insert(&attack);
    ctl.add_facts(&facts)?;
    // Re-Ground
    ground_all_parts(ctl)?;
    // Enable the external symbol
    let mut symbolic_atoms = fallible_iterator::convert(
        ctl.symbolic_atoms()?
            .iter()?
            .map(Result::<_, ClingoError>::Ok),
    );
    let requested_symbol = Symbol::create_function(
        CLINGO_SYMBOL_ATT_EXTERNAL,
        &[
            Symbol::create_string(&attack.0)?,
            Symbol::create_string(&attack.1)?,
        ],
        true,
    )?;
    // We've just added the symbol during grounding, this should never panic
    let atom = symbolic_atoms
        .find(|atom| Ok(atom.symbol()? == requested_symbol))?
        .expect("BUG: The requested symbol was just added. This should not panic!");
    ctl.assign_external(atom.literal()?, TruthValue::True)?;
    Ok(())
}
