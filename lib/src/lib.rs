#![feature(try_find)]
pub mod argumentation_framework;
mod error;
pub mod framework;
pub mod semantics;
#[cfg(test)]
mod tests;

pub use error::{Error, Result};
pub use framework::{Framework, GenericExtension};

/// Try setting up logging for unit tests
#[cfg(test)]
#[ctor::ctor]
fn setup_logging() {
    pretty_env_logger::formatted_builder()
        .is_test(true)
        .filter_level(log::LevelFilter::Trace)
        .try_init()
        .ok();
    log::trace!("Test logger setup");
}

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
        ($name:literal, $optional:literal) => {{
            let optional: bool = $optional;
            crate::argumentation_framework::symbols::Argument {
                id: $name.into(),
                optional,
            }
        }};
        ($name:literal) => {
            arg!($name, false)
        };
        ($name:literal opt) => {
            arg!($name, true)
        };
    }
    pub(crate) use arg;

    /// Create an attack for Dung's [`crate::argumentation_framework::ArgumentationFramework`]
    macro_rules! att {
        ($from:literal, $to:literal, $optional:literal) => {{
            let optional: bool = $optional;
            crate::argumentation_framework::symbols::Attack {
                from: $from.into(),
                to: $to.into(),
                optional,
            }
        }};
        ($from:literal, $to:literal) => {
            att!($from, $to, false)
        };
        ($from:literal, $to:literal opt) => {
            att!($from, $to, true)
        };
    }
    pub(crate) use att;
}
