//! Solver for Dung's Argumentation Frameworks.
use std::{collections::BTreeSet, marker::PhantomData};

use crate::{Error, Result};
use ::clingo::{defaults::Non, ShowType, SolveMode};
use fallible_iterator::FallibleIterator;

use self::{clingo::Logger, parser::parse_apx_tgf, semantics::ArgumentationFrameworkSemantic};

use crate::{
    framework::{GenericExtension, IterGuard},
    Framework,
};

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
    pub args: Vec<symbols::Argument>,
    pub attacks: Vec<symbols::Attack>,
    clingo_ctl: Option<Control>,
    // The update revision we'll use for the next change
    next_revision: u32,
    _initial_file: String,
    _semantics: PhantomData<S>,
}

/// An update to the [`ArgumentationFramework`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Patch {
    /// Add an additional argument
    AddArgument(symbols::Argument),
    /// Delete this argument
    RemoveArgument(symbols::Argument),
    /// Add an additional attack
    AddAttack(symbols::Attack),
    /// Delete this attack
    RemoveAttack(symbols::Attack),
}

impl Patch {
    /// Parse a full update line in APXM or TGFM format.
    ///
    /// # Example
    ///
    /// ```
    /// # use lib::argumentation_framework::{symbols::{Argument, Attack, RevisionedSymbol}, Patch};
    /// let patches = Patch::parse_line("+arg(a4):att(a4, a1):att(a2, a4).").unwrap();
    /// assert_eq!(
    ///    patches,
    ///    vec![
    ///        Patch::AddArgument(Argument(String::from("a4"))),
    ///        Patch::AddAttack(Attack(String::from("a4"), String::from("a1"))),
    ///        Patch::AddAttack(Attack(String::from("a2"), String::from("a4"))),
    ///    ]
    /// );
    ///
    /// let patches = Patch::parse_line("+att(a1, a3).").unwrap();
    /// assert_eq!(
    ///     patches,
    ///     vec![
    ///         Patch::AddAttack(Attack(String::from("a1"), String::from("a3"))),
    ///     ]
    /// );
    ///
    /// let patches = Patch::parse_line("-att(a2,a1).").unwrap();
    /// assert_eq!(
    ///     patches,
    ///     vec![
    ///         Patch::RemoveAttack(Attack(String::from("a2"), String::from("a1"))),
    ///     ]
    /// );
    ///
    /// let patches = Patch::parse_line("-arg(a3).").unwrap();
    /// assert_eq!(
    ///     patches,
    ///     vec![
    ///         Patch::RemoveArgument(Argument(String::from("a3"))),
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
    revision: u32,
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
    pub fn from_model(model: &::clingo::Model, revision: u32) -> Result<Self> {
        let atoms = fallible_iterator::convert(
            model
                .symbols(ShowType::SHOWN)?
                .iter()
                .map(Result::<_, Error>::Ok),
        )
        .inspect(|symbol| Ok(log::info!("Found shown symbol: {}", symbol.to_string())))
        .filter(|symbol| Ok(symbol.name()? == "in"))
        .map(|symbol| {
            // We just want the insides!
            let args = symbol.arguments()?;
            let rev = args[1].number()?;
            let symbol = args[0].string()?;
            Ok((symbol, rev))
        })
        .filter(|(_, rev)| Ok(*rev == revision as i32))
        .map(|(symbol, _)| Ok(symbol.trim_matches('"').to_owned()))
        .map(|name| Ok(symbols::Argument(name)))
        .collect()?;
        Ok(Extension { atoms })
    }
}

impl<S: ArgumentationFrameworkSemantic> ArgumentationFramework<S> {
    pub fn add_argument(&mut self, argument: symbols::Argument) -> Result {
        // We need to make sure clingo stays uptodate, but only if it's initialized
        let revision = self.revision();
        if let Some(ctl) = self.clingo_ctl.as_mut() {
            clingo::add_argument(ctl, &argument, revision)?;
        }
        // Push the argument to our list of arguments
        self.args.push(argument);
        Ok(())
    }
    pub fn remove_argument(
        &mut self,
        argument: &symbols::Argument,
    ) -> Result<Option<symbols::Argument>> {
        if let Some(idx) = self.args.iter().position(|a| a == argument) {
            let revision = self.revision();
            if let Some(ctl) = self.clingo_ctl.as_mut() {
                // Disable the external symbol for this argument
                clingo::remove_argument(ctl, &argument, revision)?;
            }
            Ok(Some(self.args.swap_remove(idx)))
        } else {
            Ok(None)
        }
    }
    pub fn add_attack(&mut self, attack: symbols::Attack) -> Result {
        // Make sure to keep clingo uptodate
        let revision = self.revision();
        log::trace!("Adding attack {attack:?} in revision {revision}");
        if let Some(ctl) = self.clingo_ctl.as_mut() {
            clingo::add_attack(ctl, &attack, revision)?;
        }
        self.attacks.push(attack);
        Ok(())
    }
    pub fn remove_attack(&mut self, attack: &symbols::Attack) -> Result<Option<symbols::Attack>> {
        if let Some(idx) = self.attacks.iter().position(|a| a == attack) {
            let revision = self.revision();
            if let Some(ctl) = self.clingo_ctl.as_mut() {
                // Disable the external symbol for this attack
                clingo::remove_attack(ctl, &attack, revision)?;
            }
            Ok(Some(self.attacks.swap_remove(idx)))
        } else {
            Ok(None)
        }
    }
    /// Apply the given patch to the argumentation framework.
    pub fn apply_patch(&mut self, patch: Patch) -> Result {
        log::trace!("Applying patch {patch:?}");
        match patch {
            Patch::AddArgument(arg) => self.add_argument(arg),
            Patch::RemoveArgument(arg) => self.remove_argument(&arg).map(|_| ()),

            Patch::AddAttack(att) => self.add_attack(att),
            Patch::RemoveAttack(att) => self.remove_attack(&att).map(|_| ()),
        }
    }
    fn revision(&mut self) -> u32 {
        let rev = self.next_revision;
        self.next_revision += 1;
        rev
    }
}

impl<S: ArgumentationFrameworkSemantic> Framework for ArgumentationFramework<S> {
    type Extension = Extension;
    type ExtensionIter = ExtensionIter;

    fn enumerate_extensions(&mut self) -> Result<IterGuard<'_, Self>> {
        let ctl = self.clingo_ctl.take().expect("Clingo control initialized");
        let handle = ctl.solve(SolveMode::YIELD, &[])?;
        Ok(IterGuard::new(
            self,
            ExtensionIter {
                revision: self.next_revision - 1,
                handle,
            },
        ))
    }

    fn new(input: &str) -> Result<Self> {
        let (args, attacks) = parse_apx_tgf(input)?;
        let clingo_ctl = clingo::initialize_backend::<S>(&args, &attacks)?;
        Ok(ArgumentationFramework {
            args,
            attacks,
            next_revision: 1,
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
        .for_each(|patch| self.apply_patch(patch))
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
                .map(|atom| atom.0.clone())
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
            Ok(Some(model)) => Some(Extension::from_model(model, self.revision)).transpose(),
            Ok(None) => Ok(None),
            Err(why) => Err(why),
        }
    }
}

impl FromIterator<ArgumentID> for Extension {
    fn from_iter<T: IntoIterator<Item = ArgumentID>>(iter: T) -> Self {
        Self {
            atoms: iter.into_iter().map(symbols::Argument).collect(),
        }
    }
}
