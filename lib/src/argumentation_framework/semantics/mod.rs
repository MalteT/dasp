//! Semantics supported by the argumentation framework solver

use crate::semantics::Semantics;

/// A semantics for Dung's Argumentation Frameworks.
///
/// # Predicates
/// The following predicates should be understood by the programs.
///
///   - `argument({name})` :: Define argument `{name}`
///   - `attack({from}, {to})` :: Define attack `{from}` -> `{to}`
///
/// The solving happens in the following steps:
///   1. Create a `base` program that will be added and grounded once.
///   2. Enable/Disable the respective external atoms
///   3. Solve
pub trait ArgumentationFrameworkSemantic: Semantics {
    /// The base program, only ground before the first solving
    ///
    /// `#program base.`
    const BASE: &'static str;
}

macro_rules! impl_program {
    ($name:path, $path:literal) => {
        impl ArgumentationFrameworkSemantic for $name {
            const BASE: &'static str = r#""#;
        }
    };
}

impl ArgumentationFrameworkSemantic for crate::semantics::Admissible {
    const BASE: &'static str = r#"
        %% Guess a set S \subseteq A
        in(X) :- not out(X), argument(X).
        out(X) :- not in(X), argument(X).

        %% S has to be conflict-free
        :- in(X), in(Y), attack(X,Y).

        %% The argument x is defeated by the set S
        defeated(X) :- in(Y), attack(Y,X).

        %% The argument x is not defended by S
        not_defended(X) :- attack(Y,X), not defeated(Y).

        %% All arguments x \in S need to be defended by S
        :- in(X), not_defended(X).
    "#;
}

impl ArgumentationFrameworkSemantic for crate::semantics::ConflictFree {
    const BASE: &'static str = r#"
        %% Guess a set S \subseteq A
        in(X) :- not out(X), argument(X).
        out(X) :- not in(X), argument(X).

        %% S has to be conflict-free
        :- in(X), in(Y), attack(X, Y).
    "#;
}

impl_program!(crate::semantics::Complete, "./complete.dl");
impl_program!(crate::semantics::Stable, "./stable.dl");
impl_program!(crate::semantics::Ground, "./ground.dl");
