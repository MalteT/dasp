//! Everything around the generalized framework
use thiserror::Error;

use crate::{Error, Result};

mod iter_guard;

use fallible_iterator::FallibleIterator;
pub use iter_guard::IterGuard;

/// Generic ParserError
#[derive(Debug, Error)]
pub enum ParserError {
    #[error(
        "Error while parsing file: Expected {expected:?}, but found {found:?}: ({position:?}: {text})"
    )]
    UnexpectedToken {
        found: Box<dyn ::std::fmt::Debug>,
        expected: Vec<Box<dyn ::std::fmt::Debug>>,
        position: std::ops::Range<usize>,
        text: String,
    },
    #[error("Unexpected end of input while parsing: Expected {expected:?}")]
    UnexpectedEndOfInput {
        expected: Vec<Box<dyn ::std::fmt::Debug>>,
    },
}

/// A generic extension.
pub trait GenericExtension {
    /// Argument type used by the extension.
    type Arg;
    /// Check whether the given argument is contained in this extension.
    fn contains(&self, arg: &Self::Arg) -> bool;
    /// Format the extension.
    /// The return-value should comply the ICCMA specification for extension output
    fn format(&self) -> String;
}

/// A general framework for argumentation
pub trait Framework
where
    Self: Sized,
{
    /// Extension type used by the framework.
    type Extension: GenericExtension;
    /// Iterator over extensions.
    type ExtensionIter: FallibleIterator<Item = Self::Extension, Error = Error>;
    /// Initialize the framework with the raw initial file content.
    fn new(input: &str) -> Result<Self>;
    /// Enumerate all extensions.
    ///
    /// All other extension methods are derived, but may be overriden if necessary.
    fn enumerate_extensions(&mut self) -> Result<IterGuard<'_, Self>>;
    /// Update the framework with the given line from standard input.
    fn update(&mut self, update_line: &str) -> Result<()>;
    /// Drop the extension iter.
    ///
    /// May be used to recycle the iterator if necessary.
    fn drop_extension_iter(&mut self, _iter: Self::ExtensionIter) -> Result<()> {
        Ok(())
    }
    /// Count all extensions.
    fn count_extensions(&mut self) -> Result<usize> {
        self.enumerate_extensions()?.by_ref().count()
    }
    /// Return any extension.
    fn sample_extension(&mut self) -> Result<Option<Self::Extension>> {
        self.enumerate_extensions()?.next()
    }
    /// Check the given argument for credulous acceptance.
    fn is_credulous_accepted(
        &mut self,
        arg: &<<Self as Framework>::Extension as GenericExtension>::Arg,
    ) -> Result<bool> {
        self.enumerate_extensions()?
            .any(|ext| Ok(ext.contains(arg)))
    }
    /// Check the given argument for skeptical acceptance.
    fn is_skeptical_accepted(
        &mut self,
        arg: &<<Self as Framework>::Extension as GenericExtension>::Arg,
    ) -> Result<bool> {
        self.enumerate_extensions()?
            .all(|ext| Ok(ext.contains(arg)))
    }
}
