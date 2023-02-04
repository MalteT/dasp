use clingo::{ClingoError, Symbol, ToSymbol};

use super::ArgumentID;

#[derive(Debug, Clone, ToSymbol, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Arg {
    pub id: ArgumentID,
}

#[derive(Debug, Clone, ToSymbol, PartialEq, Eq)]
pub struct Att {
    pub from: ArgumentID,
    pub to: ArgumentID,
}

impl From<ArgumentID> for Arg {
    fn from(id: ArgumentID) -> Self {
        Self { id }
    }
}
