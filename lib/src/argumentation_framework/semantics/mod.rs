//! Semantics supported by the argumentation framework solver

use crate::semantics::Semantics;

/// A semantics for Dung's Argumentation Frameworks.
///
/// # Predicates
/// The following predicates should be understood by the programs. Where `{revision}` is the
/// current revision number starting at `0` for the initial AF.
///
///   - `argument({name}, {revision})` :: Define argument `{name}` in `{revision}`
///   - `attack({from}, {to}, {revision})` :: Define attack `{from}` -> `{to}` in `{revision}`
///   - `delete({predicate}, {revision})` :: Delete an attack or an argument in `{revision}`
///
/// The solving happens in the following steps:
///   1. Create a `base` program that will be added and grounded once.
///   2. Create a `theory` program that will be added and grounded with the `base` and every `update_{x}` program
///   3. Enable/Disable the respective external atoms
///   4. Ground and Solve now
///   5. For every update to the AF:
///      1. Add an `update_{x}` program, if necessary, ground with `theory` and adjust the truthiness of external atoms
///      2. Solve again
pub trait ArgumentationFrameworkSemantic: Semantics {
    /// The theory that will be grounded with every update
    ///
    /// `#program theory(revision).`
    const THEORY: &'static str;
    /// The base program, only ground before the first solving
    ///
    /// `#program base.`
    const BASE: &'static str;
    /// The groundwork for every update that takes place
    ///
    /// `#program update_{revision}.`
    const UPDATE: &'static str;
}

macro_rules! impl_program {
    ($name:path, $path:literal) => {
        impl ArgumentationFrameworkSemantic for $name {
            const THEORY: &'static str = include_str!($path);
            const BASE: &'static str = "#show. #show in/2.";
            const UPDATE: &'static str = "";
        }
    };
}

impl ArgumentationFrameworkSemantic for crate::semantics::Admissible {
    const THEORY: &'static str = r#"
        % Arguments and attacks that have not been deleted are still part of this revision
        argument(X, revision) :- argument(X, revision - 1), not delete(argument(X, _), revision).
        attack(X, Y, revision) :- attack(X, Y, revision - 1), not delete(attack(X, Y, _), revision).

        % Guess a set S \subseteq A
        in(X, revision) :- not out(X, revision), argument(X, revision).
        out(X, revision) :- not in(X, revision), argument(X, revision).

        % S has to be conflict-free
        :- in(X, revision), in(Y, revision), attack(X, Y, revision).

        % The argument x is defeated by the set S
        defeated(X, revision) :- in(Y, revision), attack(Y, X, revision).

        % The argument x is not defended by S
        not_defended(X, revision) :- attack(Y, X, revision), not defeated(Y, revision).

        % All arguments x \in S need to be defended by S
        :- in(X, revision), not_defended(X, revision).
    "#;
    const BASE: &'static str = "#show. #show in/2.";
    const UPDATE: &'static str = "";
}

impl_program!(crate::semantics::Complete, "./complete.dl");
impl_program!(crate::semantics::Stable, "./stable.dl");
impl_program!(crate::semantics::Ground, "./ground.dl");
