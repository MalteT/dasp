//! Semantics supported by the argumentation framework solver

use crate::semantics::Semantics;

pub trait ArgumentationFrameworkSemantic: Semantics {
    const PROGRAM: &'static str;
}

macro_rules! impl_program {
    ($name:path, $path:literal) => {
        impl ArgumentationFrameworkSemantic for $name {
            const PROGRAM: &'static str = include_str!($path);
        }
    };
}

impl_program!(crate::semantics::Complete, "./complete.dl");
impl_program!(crate::semantics::Stable, "./stable.dl");
impl_program!(crate::semantics::Ground, "./ground.dl");
impl_program!(crate::semantics::Admissible, "./admissible.dl");
