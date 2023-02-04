//! Relevant semantics

/// Marker trait for semantics
pub trait Semantics: ::std::fmt::Debug + Clone + Copy + Default {}

macro_rules! semantics {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, Default)]
        pub struct $name;

        impl Semantics for $name {}
    };
}

semantics!(Complete);
semantics!(Stable);
semantics!(Ground);
semantics!(Admissible);
