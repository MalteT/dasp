use thiserror::Error;

pub type Result<T, E = Error> = ::std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("IO Error: {0}")]
    Io(#[from] ::std::io::Error),
    #[error("Clingo Error: {0}")]
    Clingo(#[from] ::clingo::ClingoError),
    #[error("parser error")]
    Parser(#[from] crate::framework::ParserError),
}
