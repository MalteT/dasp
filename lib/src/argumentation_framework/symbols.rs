use clingo::{Symbol, ToSymbol};

use super::ArgumentID;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Argument {
    pub id: ArgumentID,
    pub optional: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attack {
    pub from: ArgumentID,
    pub to: ArgumentID,
    pub optional: bool,
}

impl Argument {
    pub fn new<S: Into<ArgumentID>>(id: S, optional: bool) -> Self {
        Argument {
            id: id.into(),
            optional,
        }
    }
}

impl Attack {
    pub fn new<S: Into<ArgumentID>, T: Into<ArgumentID>>(from: S, to: T, optional: bool) -> Self {
        Attack {
            from: from.into(),
            to: to.into(),
            optional,
        }
    }
}

impl ToSymbol for Argument {
    fn symbol(&self) -> Result<clingo::Symbol, clingo::ClingoError> {
        Symbol::create_function("argument", &[Symbol::create_id(&self.id, true)?], true)
    }
}

impl ToSymbol for Attack {
    fn symbol(&self) -> Result<Symbol, clingo::ClingoError> {
        Symbol::create_function(
            "attack",
            &[
                Symbol::create_id(&self.from, true)?,
                Symbol::create_id(&self.to, true)?,
            ],
            true,
        )
    }
}
