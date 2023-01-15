use clingo::{ClingoError, Symbol, ToSymbol};

use super::ArgID;

#[derive(Debug, Clone, ToSymbol, PartialEq, Eq)]
pub struct Arg {
    pub id: ArgID,
}

#[derive(Debug, Clone, ToSymbol, PartialEq, Eq)]
pub struct Att {
    pub from: ArgID,
    pub to: ArgID,
}

impl From<ArgID> for Arg {
    fn from(id: ArgID) -> Self {
        Self { id }
    }
}
