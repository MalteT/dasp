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

semantics!(Admissible);
semantics!(Complete);
semantics!(ConflictFree);
semantics!(Ground);
semantics!(Stable);
