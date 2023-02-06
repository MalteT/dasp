use clingo::{ClingoError, Symbol, ToSymbol};

use super::ArgumentID;

const CLINGO_SYMBOL_ARG: &str = "arg";
const CLINGO_SYMBOL_ATT: &str = "att";

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Argument(pub ArgumentID);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attack(pub ArgumentID, pub ArgumentID);

impl From<ArgumentID> for Argument {
    fn from(id: ArgumentID) -> Self {
        Self(id)
    }
}

impl ToSymbol for Argument {
    fn symbol(&self) -> Result<Symbol, ClingoError> {
        Symbol::create_function(CLINGO_SYMBOL_ARG, &[Symbol::create_string(&self.0)?], true)
    }
}

impl ToSymbol for Attack {
    fn symbol(&self) -> Result<Symbol, ClingoError> {
        Symbol::create_function(
            CLINGO_SYMBOL_ATT,
            &[
                Symbol::create_string(&self.0)?,
                Symbol::create_string(&self.1)?,
            ],
            true,
        )
    }
}
