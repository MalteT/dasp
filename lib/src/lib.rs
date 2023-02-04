pub mod argumentation_framework;
mod error;
pub mod framework;
pub mod semantics;

pub use error::{Error, Result};
pub use framework::{Framework, GenericExtension};

/// Macro definitions used throughout the crate
///
/// The language server marks them unused, so let's ignore that.
// TODO: Remove (someday)
#[allow(unused)]
mod macros {
    /// Macro to easily construct a [`std::collections::BTreeSet`]
    macro_rules! set {
        () => {
            ::std::collections::BTreeSet::new()
        };
        ($($exp:expr),*) => {
            [
                $( $exp ),*
            ].into_iter().collect::<::std::collections::BTreeSet<_>>()
        }
    }
    pub(crate) use set;

    /// Macro to easily construct an extension
    macro_rules! ext {
        () => {
            crate::argumentation_framework::Extension {
                atoms: crate::macros::set!()
            }
        };
        ($($arg:literal),*) => {
            [
                $( String::from($arg) ),*
            ].into_iter().collect::<crate::argumentation_framework::Extension>()
        }
    }
    pub(crate) use ext;

    /// Create an argument for Dung's [`crate::argumentation_framework::ArgumentationFramework`]
    macro_rules! arg {
        ($name:literal) => {
            crate::argumentation_framework::symbols::Arg { id: $name.into() }
        };
    }
    pub(crate) use arg;

    /// Create an attack for Dung's [`crate::argumentation_framework::ArgumentationFramework`]
    macro_rules! att {
        ($from:literal, $to:literal) => {
            crate::argumentation_framework::symbols::Att {
                from: $from.into(),
                to: $to.into(),
            }
        };
    }
    pub(crate) use att;
}
