//! Solver for Dung's Argumentation Frameworks.
use std::{collections::BTreeSet, marker::PhantomData};

use crate::{Error, Result};
use clingo::{FactBase, Part, ShowType, SolveMode};
use fallible_iterator::FallibleIterator;

use self::{parser::parse_apx_tgf, semantics::ArgumentationFrameworkSemantic};

use crate::{
    framework::{GenericExtension, IterGuard},
    Framework,
};

pub type ArgumentID = String;

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
    pub args: Vec<symbols::Arg>,
    pub attacks: Vec<symbols::Att>,
    clingo_ctl: Option<::clingo::Control>,
    _initial_file: String,
    _semantics: PhantomData<S>,
}

/// An update to the [`ArgumentationFramework`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Patch {
    /// Add an additional argument
    AddArgument(symbols::Arg),
    /// Delete this argument
    DelArgument(symbols::Arg),
    /// Add an additional attack
    AddAttack(symbols::Att),
    /// Delete this attack
    DelAttack(symbols::Att),
}

impl Patch {
    /// Parse a full update line in APXM or TGFM format.
    ///
    /// # Example
    ///
    /// ```
    /// # use lib::argumentation_framework::{symbols::{Arg, Att}, Patch};
    /// let patches = Patch::parse_line("+arg(a4):att(a4, a1):att(a2, a4).").unwrap();
    /// assert_eq!(
    ///    patches,
    ///    vec![
    ///        Patch::AddArgument(Arg { id: String::from("a4") }),
    ///        Patch::AddAttack(Att { from: String::from("a4"), to: String::from("a1") }),
    ///        Patch::AddAttack(Att { from: String::from("a2"), to: String::from("a4") }),
    ///    ]
    /// );
    ///
    /// let patches = Patch::parse_line("+att(a1, a3).").unwrap();
    /// assert_eq!(
    ///     patches,
    ///     vec![
    ///         Patch::AddAttack(Att { from: String::from("a1"), to: String::from("a3") }),
    ///     ]
    /// );
    ///
    /// let patches = Patch::parse_line("-att(a2,a1).").unwrap();
    /// assert_eq!(
    ///     patches,
    ///     vec![
    ///         Patch::DelAttack(Att { from: String::from("a2"), to: String::from("a1") }),
    ///     ]
    /// );
    ///
    /// let patches = Patch::parse_line("-arg(a3).").unwrap();
    /// assert_eq!(
    ///     patches,
    ///     vec![
    ///         Patch::DelArgument(Arg { id: String::from("a3") }),
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
/// Using a [`::clingo::SolveHandle`] internally. This always needs to be returned,
/// to recycle the handle and turn it back into the [`::clingo::Control`]
pub struct ExtensionIter {
    handle: ::clingo::SolveHandle,
}

/// An extension of an [`ArgumentationFramework`].
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Extension {
    /// Just a list of included arguments
    atoms: BTreeSet<symbols::Arg>,
}

impl Extension {
    /// The empty extension
    pub const EMPTY: Extension = crate::macros::ext!();
}

impl<S: ArgumentationFrameworkSemantic> ArgumentationFramework<S> {
    /// Apply the given patch to the argumentation framework.
    fn apply_patch(&mut self, patch: Patch) {
        match patch {
            Patch::AddArgument(arg) => {
                self.args.push(arg);
            }
            Patch::DelArgument(arg) => {
                if let Some(idx) = self.args.iter().position(|a| *a == arg) {
                    self.args.swap_remove(idx);
                }
            }
            Patch::AddAttack(att) => {
                self.attacks.push(att);
            }
            Patch::DelAttack(att) => {
                if let Some(idx) = self.attacks.iter().position(|a| *a == att) {
                    self.attacks.swap_remove(idx);
                }
            }
        }
    }
}

/// Initialize the clingo backend
///
/// Loads the given args and attacks
fn initialize_clingo_backend<S: ArgumentationFrameworkSemantic>(
    args: &[symbols::Arg],
    attacks: &[symbols::Att],
) -> Result<::clingo::Control> {
    // Assemble clingo parameters
    // FIXME: Make core count flexible
    let clingo_params = vec![
        // Use multiple cores [--parallel-mode 12]
        "--parallel-mode",
        "12",
        // Always prepare to compute all models [0]
        "0",
    ]
    .into_iter()
    .map(String::from)
    .collect();
    let mut fb = FactBase::new();
    args.iter().for_each(|arg| fb.insert(arg));
    attacks.iter().for_each(|attack| fb.insert(attack));
    let mut ctl = ::clingo::control(clingo_params)?;
    ctl.add("theory", &[], S::PROGRAM)?;
    ctl.add("show", &[], "#show. #show X : in(X).")?;
    ctl.add_facts(&fb)?;
    // Ground everything directly
    let base = Part::new("base", vec![])?;
    let theory = Part::new("theory", vec![])?;
    let show = Part::new("show", vec![])?;
    ctl.ground(&[base, theory, show])?;
    Ok(ctl)
}

impl<S: ArgumentationFrameworkSemantic> Framework for ArgumentationFramework<S> {
    type Extension = Extension;
    type ExtensionIter = ExtensionIter;

    fn enumerate_extensions(&mut self) -> Result<IterGuard<'_, Self>> {
        let ctl = self.clingo_ctl.take().expect("Clingo control initialized");
        let handle = ctl.solve(SolveMode::YIELD, &[])?;
        Ok(IterGuard::new(self, ExtensionIter { handle }))
    }

    fn new(input: &str) -> Result<Self> {
        let (args, attacks) = parse_apx_tgf(input)?;
        let clingo_ctl = initialize_clingo_backend::<S>(&args, &attacks)?;
        Ok(ArgumentationFramework {
            args,
            attacks,
            _semantics: PhantomData,
            _initial_file: input.to_owned(),
            clingo_ctl: Some(clingo_ctl),
        })
    }

    fn update(&mut self, update_line: &str) -> Result<()> {
        parser::parse_apxm_tgfm_patch_line(update_line)?
            .into_iter()
            .for_each(|patch| self.apply_patch(patch));
        Ok(())
    }

    fn drop_extension_iter(&mut self, iter: Self::ExtensionIter) -> Result<()> {
        self.clingo_ctl = Some(iter.handle.close()?);
        Ok(())
    }
}

impl GenericExtension for Extension {
    type Arg = symbols::Arg;

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

impl FallibleIterator for ExtensionIter {
    type Item = Extension;
    type Error = Error;

    fn next(&mut self) -> Result<Option<Self::Item>> {
        if let Err(why) = self.handle.resume() {
            return Err(why.into());
        }
        match self.handle.model().map_err(crate::Error::from) {
            Ok(Some(model)) => Some(Extension::try_from(model)).transpose(),
            Ok(None) => Ok(None),
            Err(why) => Err(why),
        }
    }
}

impl TryFrom<&'_ ::clingo::Model> for Extension {
    type Error = crate::Error;

    fn try_from(value: &::clingo::Model) -> Result<Self, Self::Error> {
        let atoms = value
            .symbols(ShowType::SHOWN)?
            .iter()
            .map(ToString::to_string)
            .map(|id| id.trim_matches('"').to_owned())
            .map(|id| symbols::Arg { id })
            .collect();
        Ok(Extension { atoms })
    }
}

impl FromIterator<ArgumentID> for Extension {
    fn from_iter<T: IntoIterator<Item = ArgumentID>>(iter: T) -> Self {
        Self {
            atoms: iter.into_iter().map(|id| symbols::Arg { id }).collect(),
        }
    }
}
