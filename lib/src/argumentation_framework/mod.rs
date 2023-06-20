//! Solver for Dung's Argumentation Frameworks.
use std::{collections::BTreeSet, marker::PhantomData, sync::atomic::AtomicUsize};

use crate::{Error, Result};
use ::clingo::{defaults::Non, ShowType, SolveMode, ToSymbol};
use fallible_iterator::FallibleIterator;

use self::{clingo::Logger, parser::parse_apx_tgf, semantics::ArgumentationFrameworkSemantic};

use crate::{
    framework::{GenericExtension, IterGuard},
    Framework,
};

pub static ID_COUNTER: Counter = Counter::new();

pub struct Counter(AtomicUsize);

impl Counter {
    const fn new() -> Self {
        Self(AtomicUsize::new(0))
    }

    pub fn next(&self) -> usize {
        self.0.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    }
}

pub type ArgumentID = String;
type Control = ::clingo::GenericControl<clingo::Logger, Non, Non, Non>;

mod clingo;
mod parser;
pub mod semantics;
pub mod symbols;
#[cfg(test)]
mod tests;

/// Dung's Argumentation Framework
///
/// A simple graph with arguments (vertices) and attacks (edges).
///
/// # Example
/// ```
/// use fallible_iterator::FallibleIterator;
/// use lib::{semantics, argumentation_framework::ArgumentationFramework, Framework};
/// # use std::collections::BTreeSet;
///
/// let mut af = ArgumentationFramework::<semantics::Admissible>::new(
///     r#"
///         arg(1).
///         arg(2).
///         att(1,2).
///     "#,
/// )
/// .expect("Initializing AF");
///
/// let extensions = af
///     .enumerate_extensions()
///     .expect("Enumerating extensions")
///     .by_ref()
///     .collect::<BTreeSet<_>>();
/// ```
pub struct ArgumentationFramework<S: ArgumentationFrameworkSemantic> {
    clingo_ctl: Option<Control>,
    _initial_file: String,
    _semantics: PhantomData<S>,
}

/// An update to the [`ArgumentationFramework`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Patch {
    /// Add an additional argument
    EnableArgument(symbols::Argument),
    /// Delete this argument
    DisableArgument(symbols::Argument),
    /// Add an additional attack
    EnableAttack(symbols::Attack),
    /// Delete this attack
    DisableAttack(symbols::Attack),
}

impl Patch {
    /// Parse a full update line in APXM or TGFM format.
    ///
    /// # Example
    ///
    /// ```
    /// # use lib::argumentation_framework::{symbols::{Argument, Attack}, Patch};
    /// let patches = Patch::parse_line("+arg(a4):att(a4, a1):att(a2, a4).").unwrap();
    /// assert_eq!(
    ///    patches,
    ///    vec![
    ///        Patch::EnableArgument(Argument::new("a4", false)),
    ///        Patch::EnableAttack(Attack::new("a4", "a1", false)),
    ///        Patch::EnableAttack(Attack::new("a2", "a4", false)),
    ///    ]
    /// );
    ///
    /// let patches = Patch::parse_line("+att(a1, a3).").unwrap();
    /// assert_eq!(
    ///     patches,
    ///     vec![
    ///         Patch::EnableAttack(Attack::new("a1","a3", false)),
    ///     ]
    /// );
    ///
    /// let patches = Patch::parse_line("-att(a2,a1).").unwrap();
    /// assert_eq!(
    ///     patches,
    ///     vec![
    ///         Patch::DisableAttack(Attack::new("a2", "a1", false)),
    ///     ]
    /// );
    ///
    /// let patches = Patch::parse_line("-arg(a3).").unwrap();
    /// assert_eq!(
    ///     patches,
    ///     vec![
    ///         Patch::DisableArgument(Argument::new("a3", false)),
    ///     ]
    /// );
    /// ```
    pub fn parse_line(input: &str) -> Result<Vec<Self>> {
        let patches = parser::parse_apxm_tgfm_patch_line(input)?;
        Ok(patches)
    }
}

/// Iterator over extensions.
///
/// Using a [`::clingo::GenericSolveHandle`] internally. This always needs to be returned,
/// to recycle the handle and turn it back into the [`::clingo::GenericControl`]
pub struct ExtensionIter {
    handle: ::clingo::GenericSolveHandle<Logger, Non, Non, Non, Non>,
}

/// An extension of an [`ArgumentationFramework`].
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Extension {
    /// Just a list of included arguments
    atoms: BTreeSet<symbols::Argument>,
}

impl Extension {
    /// The empty extension
    pub const EMPTY: Extension = crate::macros::ext!();
    pub fn from_model(model: &::clingo::Model) -> Result<Self> {
        log::trace!("Converting clingo model to extension");
        let atoms = fallible_iterator::convert(
            model
                .symbols(ShowType::SHOWN)?
                .iter()
                .inspect(|symbol| log::trace!("Raw symbol in model: {:?}", symbol.to_string()))
                .map(Result::<_, Error>::Ok),
        )
        .map(|symbol| Ok(symbol.to_string()))
        .map(|symbol| Ok(symbol.trim_matches('"').to_owned()))
        .map(|name| {
            Ok(symbols::Argument {
                id: name,
                optional: false,
            })
        })
        .collect()?;
        Ok(Extension { atoms })
    }
}

impl<S: ArgumentationFrameworkSemantic> ArgumentationFramework<S> {
    pub fn apply_patch(&mut self, patch: &Patch) -> Result {
        log::trace!("Applying patch {patch:?}");
        match patch {
            Patch::EnableArgument(argument) => self.enable_argument(argument),
            Patch::DisableArgument(argument) => self.disable_argument(argument),
            Patch::EnableAttack(attack) => self.enable_attack(attack),
            Patch::DisableAttack(attack) => self.disable_attack(attack),
        }
    }
    pub fn enable_argument(&mut self, argument: &symbols::Argument) -> Result {
        let symbol_needle = argument.symbol()?;
        let target = self
            .assume_control()?
            .symbolic_atoms()?
            .iter()?
            .try_find(|x| Result::<_, ::clingo::ClingoError>::Ok(x.symbol()? == symbol_needle))?
            .ok_or(Error::Logic(format!(
                "The argument {symbol_needle} was not defined as optional and cannot be enabled now"
            )))?;
        clingo::enable_argument(self.assume_control()?, target.literal()?)?;
        Ok(())
    }
    pub fn disable_argument(&mut self, argument: &symbols::Argument) -> Result {
        let symbol_needle = argument.symbol()?;
        let target = self
            .assume_control()?
            .symbolic_atoms()?
            .iter()?
            .try_find(|x| Result::<_, ::clingo::ClingoError>::Ok(x.symbol()? == symbol_needle))?
            .ok_or(Error::Logic(format!(
                "The argument {symbol_needle} was not defined as optional and cannot be disabled now"
            )))?;
        clingo::disable_argument(self.assume_control()?, target.literal()?)?;
        Ok(())
    }
    pub fn enable_attack(&mut self, attack: &symbols::Attack) -> Result {
        let symbol_needle = attack.symbol()?;
        let target = self
            .assume_control()?
            .symbolic_atoms()?
            .iter()?
            .try_find(|x| Result::<_, ::clingo::ClingoError>::Ok(x.symbol()? == symbol_needle))?
            .ok_or(Error::Logic(format!(
                "The attack {symbol_needle} was not defined as optional and cannot be enabled now"
            )))?;
        clingo::enable_attack(self.assume_control()?, target.literal()?)?;
        Ok(())
    }
    pub fn disable_attack(&mut self, attack: &symbols::Attack) -> Result {
        let symbol_needle = attack.symbol()?;
        let target = self
            .assume_control()?
            .symbolic_atoms()?
            .iter()?
            .try_find(|x| Result::<_, ::clingo::ClingoError>::Ok(x.symbol()? == symbol_needle))?
            .ok_or(Error::Logic(format!(
                "The attack {symbol_needle} was not defined as optional and cannot be disabled now"
            )))?;
        clingo::disable_attack(self.assume_control()?, target.literal()?)?;
        Ok(())
    }
    fn assume_control(&mut self) -> Result<&mut Control> {
        self.clingo_ctl.as_mut().ok_or(Error::ClingoNotInitialized)
    }
}

impl<S: ArgumentationFrameworkSemantic> Framework for ArgumentationFramework<S> {
    type Extension = Extension;
    type ExtensionIter = ExtensionIter;

    fn enumerate_extensions(&mut self) -> Result<IterGuard<'_, Self>> {
        log::trace!("Solving.. enumerating extensions");
        let ctl = self.clingo_ctl.take().expect("Clingo control initialized");
        let handle = ctl.solve(SolveMode::YIELD, &[])?;
        Ok(IterGuard::new(self, ExtensionIter { handle }))
    }

    fn new(input: &str) -> Result<Self> {
        let (args, attacks) = parse_apx_tgf(input)?;
        let clingo_ctl = clingo::initialize_backend::<S>(&args, &attacks)?;
        Ok(ArgumentationFramework {
            _semantics: PhantomData,
            _initial_file: input.to_owned(),
            clingo_ctl: Some(clingo_ctl),
        })
    }

    fn update(&mut self, update_line: &str) -> Result<()> {
        fallible_iterator::convert(
            parser::parse_apxm_tgfm_patch_line(update_line)?
                .into_iter()
                .map(Ok),
        )
        .for_each(|patch| self.apply_patch(&patch))
    }

    fn drop_extension_iter(&mut self, iter: Self::ExtensionIter) -> Result<()> {
        self.clingo_ctl = Some(iter.handle.close()?);
        Ok(())
    }
}

impl GenericExtension for Extension {
    type Arg = symbols::Argument;

    fn contains(&self, arg: &Self::Arg) -> bool {
        self.atoms.contains(arg)
    }

    fn format(&self) -> String {
        String::from("[")
            + &self
                .atoms
                .iter()
                .map(|atom| atom.id.clone())
                .reduce(|acc, atom| format!("{acc},{atom}"))
                .unwrap_or_default()
            + "]"
    }
}

fn print_model(model: &::clingo::Model) {
    // get model type
    let model_type = model.model_type().unwrap();

    let type_string = match model_type {
        ::clingo::ModelType::StableModel => "Stable model",
        ::clingo::ModelType::BraveConsequences => "Brave consequences",
        ::clingo::ModelType::CautiousConsequences => "Cautious consequences",
    };

    // get running number of model
    let number = model.number().unwrap();

    log::trace!("== {}: {}", type_string, number);

    fn print(model: &::clingo::Model, label: &str, show: ShowType) {
        // retrieve the symbols in the model
        let atoms = model
            .symbols(show)
            .expect("Failed to retrieve symbols in the model.")
            .iter()
            .fold(String::new(), |line, atom| line + " " + &atom.to_string());
        log::trace!("{label}: {atoms}");
    }

    print(model, "--  shown", ShowType::SHOWN);
    print(model, "--  atoms", ShowType::ATOMS);
    print(model, "--  terms", ShowType::TERMS);
    print(model, "-- ~atoms", ShowType::COMPLEMENT | ShowType::ATOMS);
}

impl FallibleIterator for ExtensionIter {
    type Item = Extension;
    type Error = Error;

    fn next(&mut self) -> Result<Option<Self::Item>> {
        log::trace!("Fetching next extension from iterator");
        if let Err(why) = self.handle.resume() {
            log::warn!("Error while resuming solving");
            return Err(why.into());
        }
        match self.handle.model().map_err(crate::Error::from) {
            Ok(Some(model)) => {
                print_model(model);
                Some(Extension::from_model(model)).transpose()
            }
            Ok(None) => Ok(None),
            Err(why) => Err(why),
        }
    }
}

impl FromIterator<ArgumentID> for Extension {
    fn from_iter<T: IntoIterator<Item = ArgumentID>>(iter: T) -> Self {
        Self {
            atoms: iter
                .into_iter()
                .map(|id| symbols::Argument {
                    id,
                    optional: false,
                })
                .collect(),
        }
    }
}
