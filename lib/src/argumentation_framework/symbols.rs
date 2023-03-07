use clingo::Symbol;

use crate::Result;

use super::ArgumentID;

const CLINGO_SYMBOL_ARG: &str = "argument";
const CLINGO_SYMBOL_ATT: &str = "attack";
const CLINGO_SYMBOL_DELETE: &str = "delete";

pub trait RevisionedSymbol {
    fn symbol(&self, revision: u32) -> Result<Symbol>;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Argument(pub ArgumentID);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attack(pub ArgumentID, pub ArgumentID);

pub struct Delete<'s, S: RevisionedSymbol>(pub &'s S);

impl Argument {
    pub fn new<S: Into<ArgumentID>>(id: S) -> Self {
        Argument(id.into())
    }
}

impl Attack {
    pub fn new<S: Into<ArgumentID>, T: Into<ArgumentID>>(from: S, to: T) -> Self {
        Attack(from.into(), to.into())
    }
}

impl From<ArgumentID> for Argument {
    fn from(id: ArgumentID) -> Self {
        Self(id)
    }
}

impl RevisionedSymbol for Argument {
    fn symbol(&self, revision: u32) -> Result<Symbol> {
        debug_assert!(revision <= i32::MAX as u32);
        let symb = Symbol::create_function(
            CLINGO_SYMBOL_ARG,
            &[
                Symbol::create_string(&self.0)?,
                Symbol::create_number(revision as i32),
            ],
            true,
        )?;
        Ok(symb)
    }
}

impl RevisionedSymbol for Attack {
    /// Get the symbol for this Attack for the given revision.
    /// ```
    /// # use lib::argumentation_framework::symbols::{RevisionedSymbol, Attack};
    /// let att = Attack::new("from", "to");
    /// let symb_string = att.symbol(42).unwrap().to_string();
    /// assert_eq!(symb_string, r#"attack("from","to",42)"#);
    /// ```
    fn symbol(&self, revision: u32) -> Result<Symbol> {
        debug_assert!(revision <= i32::MAX as u32);
        let symb = Symbol::create_function(
            CLINGO_SYMBOL_ATT,
            &[
                Symbol::create_string(&self.0)?,
                Symbol::create_string(&self.1)?,
                Symbol::create_number(revision as i32),
            ],
            true,
        )?;
        Ok(symb)
    }
}

impl<'s, S: RevisionedSymbol> RevisionedSymbol for Delete<'s, S> {
    fn symbol(&self, revision: u32) -> Result<Symbol> {
        debug_assert!(revision <= i32::MAX as u32);
        let symb = Symbol::create_function(
            CLINGO_SYMBOL_DELETE,
            &[
                self.0.symbol(revision)?,
                Symbol::create_number(revision as i32),
            ],
            true,
        )?;
        Ok(symb)
    }
}
