use pretty_assertions::assert_eq;

use crate::{
    macros::{ext, set},
    semantics::{Admissible, Complete, ConflictFree, Ground, Stable},
};

use super::*;

fn extensions<S: ArgumentationFrameworkSemantic>(program: &str) -> BTreeSet<Extension> {
    let mut af = ArgumentationFramework::<S>::new(program).expect("Creating AF");
    extensions_of(&mut af)
}

fn extensions_of<S: ArgumentationFrameworkSemantic>(
    af: &mut ArgumentationFramework<S>,
) -> BTreeSet<Extension> {
    let extensions_vec = af
        .enumerate_extensions()
        .expect("Enumerating extensions")
        .by_ref()
        .inspect(|ext| Ok(log::trace!("Found extension {ext:?}")))
        .collect::<Vec<_>>()
        .expect("Collecting extensions into hashset");
    let extensions_count = extensions_vec.len();
    let extensions_set = extensions_vec.into_iter().collect::<BTreeSet<_>>().clone();
    assert_eq!(
        extensions_count,
        extensions_set.len(),
        "Some extensions occured more than once!"
    );
    extensions_set
}

#[test]
fn the_empty_af() {
    let only_empty_extension = set![Extension::EMPTY];
    assert_eq!(extensions::<Admissible>(""), only_empty_extension);
    assert_eq!(extensions::<Complete>(""), only_empty_extension);
    assert_eq!(extensions::<Ground>(""), only_empty_extension);
    assert_eq!(extensions::<Stable>(""), only_empty_extension)
}

#[test]
fn simple_admissible_af() {
    let extensions = extensions::<Admissible>(
        r#"
            arg(a1).
            arg(a2).
            arg(a3).
            att(a1, a3).
            att(a2, a3).
            att(a3, a2).
        "#,
    );
    assert_eq!(
        extensions,
        set![Extension::EMPTY, ext!("a1"), ext!("a2"), ext!("a1", "a2")]
    )
}

#[ignore = "complete is not adjusted yet"]
#[test]
fn simple_complete_af() {
    let extensions = extensions::<Complete>(
        r#"
            arg(1).
            arg(2).
            arg(3).
            att(1, 3).
            att(2, 3).
            att(3, 2).
        "#,
    );
    assert_eq!(extensions, set![ext!("1", "2")])
}

#[ignore = "ground is not adjusted yet"]
#[test]
fn simple_ground_af() {
    let exts = extensions::<Ground>(
        r#"
            arg(1).
            arg(2).
            arg(3).
            att(1, 3).
            att(2, 3).
            att(3, 2).
        "#,
    );
    assert_eq!(exts, set![ext!("1", "2")]);

    let exts = extensions::<Ground>(
        r#"
            arg(1).
        "#,
    );
    assert_eq!(exts, set![ext!("1")]);

    let exts = extensions::<Ground>(
        r#"
            arg(1).
            arg(2).
            att(1, 2).
            att(2, 1).
        "#,
    );
    assert_eq!(exts, set![Extension::EMPTY]);
}

#[ignore = "stable is not adjusted yet"]
#[test]
fn simple_stable_af() {
    let exts = extensions::<Stable>(
        r#"
            arg(1).
            arg(2).
            arg(3).
            att(1, 3).
            att(2, 3).
            att(3, 2).
        "#,
    );
    assert_eq!(exts, set![ext!("1", "2")]);

    let exts = extensions::<Stable>(
        r#"
            arg(1).
        "#,
    );
    assert_eq!(exts, set![ext!("1")]);

    let exts = extensions::<Stable>(
        r#"
            arg(1).
            arg(2).
            att(1, 2).
            att(2, 1).
        "#,
    );
    assert_eq!(exts, set![ext!("1"), ext!("2")]);

    let exts = extensions::<Stable>(
        r#"
            arg(1).
            arg(2).
            att(1, 2).
            att(1, 1).
        "#,
    );
    assert_eq!(exts, set![]);
}

#[test]
fn update_admissible_af() {
    let mut af = ArgumentationFramework::<Admissible>::new(
        r#"
            arg(alpha).
            arg(beta).

            att(alpha, beta).
            opt(att(alpha, beta)).

            att(alpha, alpha).
            opt(att(alpha, alpha)).
        "#,
    )
    .expect("Creating AF");
    // Enable alpha->beta by default
    af.update("+att(alpha, beta).")
        .expect("Enable attack alpha->beta");
    let exts = extensions_of(&mut af);
    assert_eq!(exts, set![ext!(), ext!("alpha")]);

    af.update("+att(alpha, alpha).")
        .expect("Enable attack alpha->alpha");
    let exts = extensions_of(&mut af);
    assert_eq!(exts, set![ext!()]);

    af.update("-att(alpha, beta).")
        .expect("Disable attack from AF");
    let exts = extensions_of(&mut af);
    assert_eq!(exts, set![ext!(), ext!("beta")]);
}

#[test]
fn re_enabling_arguments_in_admissible_af() {
    let mut af = ArgumentationFramework::<Admissible>::new(
        r#"
            arg(a1).
            arg(a2).

            opt(arg(a1)).
        "#,
    )
    .expect("Creating AF");
    // Enable arg(1) by default
    af.update("+arg(a1).").expect("Enable argument a1");
    let exts = extensions_of(&mut af);
    assert_eq!(exts, set![ext!(), ext!("a1"), ext!("a2"), ext!("a1", "a2")]);

    af.update("-arg(a1).").expect("Disable argument 1");
    let exts = extensions_of(&mut af);
    assert_eq!(exts, set![ext!(), ext!("a2")]);

    af.update("+arg(a1).").expect("Re-Enable argument 1");
    let exts = extensions_of(&mut af);
    assert_eq!(exts, set![ext!(), ext!("a1"), ext!("a2"), ext!("a1", "a2")]);
}

#[test]
fn re_enabling_attacks_in_admissible_af() {
    let mut af = ArgumentationFramework::<Admissible>::new(
        r#"
            arg(a1).
            arg(a2).

            att(a1, a2).
            opt(att(a1, a2)).
        "#,
    )
    .expect("Creating AF");
    // Enable a1->a2 by default
    af.update("+att(a1, a2).")
        .expect("Enable attack from a1 to a2");
    let exts = extensions_of(&mut af);
    assert_eq!(exts, set![ext!(), ext!("a1")]);

    af.update("-att(a1, a2).")
        .expect("Disable attack from a1 to a2");
    let exts = extensions_of(&mut af);
    assert_eq!(exts, set![ext!(), ext!("a1"), ext!("a2"), ext!("a1", "a2")]);

    af.update("+att(a1, a2).")
        .expect("Re-Enable attack from a1 to a2");
    let exts = extensions_of(&mut af);
    assert_eq!(exts, set![ext!(), ext!("a1")]);
}

#[test]
fn enabling_arguments_in_admissible_afs() {
    let mut af = ArgumentationFramework::<Admissible>::new(
        r#"
            arg(a).
            opt(arg(a)).

            arg(b).
            opt(arg(b)).
        "#,
    )
    .expect("Creating AF");
    assert_eq!(extensions_of(&mut af), set![ext!()]);
    af.update("+arg(a).").expect("Enable argument a");
    assert_eq!(extensions_of(&mut af), set![ext!(), ext!("a")]);
    af.update("+arg(b).").expect("Enable argument b");
    assert_eq!(
        extensions_of(&mut af),
        set![ext!(), ext!("a"), ext!("b"), ext!("a", "b")]
    );

    let mut af = ArgumentationFramework::<Admissible>::new(
        r#"
            arg(a).
            arg(b).
            att(a, b).

            arg(c).
            opt(arg(c)).
        "#,
    )
    .expect("Creating AF");
    assert_eq!(extensions_of(&mut af), set![ext!(), ext!("a")]);
    af.update("+arg(c).").expect("Enable argument c");
    assert_eq!(
        extensions_of(&mut af),
        set![ext!(), ext!("a"), ext!("c"), ext!("a", "c")]
    );
}

#[test]
fn enabling_attacks_in_admissible_afs() {
    let mut af = ArgumentationFramework::<Admissible>::new(
        r#"
            arg(a).
            arg(b).

            att(b, a).
            opt(att(b, a)).

            att(a, b).
            opt(att(a, b)).
        "#,
    )
    .expect("Creating AF");
    assert_eq!(
        extensions_of(&mut af),
        set![ext!(), ext!("a"), ext!("b"), ext!("a", "b")]
    );
    af.update("+att(b, a).").expect("Enable attack from b to a");
    assert_eq!(extensions_of(&mut af), set![ext!(), ext!("b")]);
    af.update("+att(a, b).").expect("Enable attack from a to b");
    assert_eq!(extensions_of(&mut af), set![ext!(), ext!("a"), ext!("b")]);
}

#[test]
fn enabling_arguments_and_attacks_in_admissible_afs() {
    let mut af = ArgumentationFramework::<Admissible>::new(
        r#"
            arg(a).
            opt(arg(a)).

            arg(b).
            opt(arg(b)).

            att(a, b).
            opt(att(a, b)).

            att(b, a).
            opt(att(b, a)).

            arg(c).
            opt(arg(c)).

            att(c, a).
            opt(att(c, a)).
        "#,
    )
    .expect("Creating AF");
    assert_eq!(extensions_of(&mut af), set![ext!()]);
    af.update("+arg(a).").expect("Enable argument a");
    assert_eq!(extensions_of(&mut af), set![ext!(), ext!("a")]);
    af.update("+arg(b).").expect("Enable argument b");
    assert_eq!(
        extensions_of(&mut af),
        set![ext!(), ext!("a"), ext!("b"), ext!("a", "b")]
    );
    af.update("+att(a, b).").expect("Enable attack from a to b");
    assert_eq!(extensions_of(&mut af), set![ext!(), ext!("a")]);
    af.update("+att(b, a).").expect("Enable attack from b to a");
    assert_eq!(extensions_of(&mut af), set![ext!(), ext!("a"), ext!("b")]);

    af.update("+arg(c).").expect("Enable argument c");
    assert_eq!(
        extensions_of(&mut af),
        set![
            ext!(),
            ext!("a"),
            ext!("b"),
            ext!("c"),
            ext!("a", "c"),
            ext!("b", "c")
        ]
    );
    af.update("+att(c, a).").expect("Enable attack from c to a");
    assert_eq!(
        extensions_of(&mut af),
        set![ext!(), ext!("b"), ext!("c"), ext!("b", "c")]
    );
}

#[test]
fn simple_conflict_free_af() {
    let mut af = ArgumentationFramework::<ConflictFree>::new(
        r#"
            arg(a).
            arg(b).
            att(a, b).
        "#,
    )
    .expect("Creating AF");
    assert_eq!(extensions_of(&mut af), set![ext!(), ext!("a"), ext!("b")]);
}

#[test]
fn simple_conflict_free_af_with_enabling_arguments_and_attacks() {
    let mut af = ArgumentationFramework::<ConflictFree>::new(
        r#"
            arg(a).
            arg(b).
            att(a, b).

            att(b, a).
            opt(att(b, a)).

            arg(c).
            opt(arg(c)).

            att(c, a).
            opt(att(c, a)).
        "#,
    )
    .expect("Creating AF");
    assert_eq!(extensions_of(&mut af), set![ext!(), ext!("a"), ext!("b")]);
    af.update("+att(b, a).").expect("Enable attack from b to a");
    assert_eq!(extensions_of(&mut af), set![ext!(), ext!("a"), ext!("b")]);
    af.update("+arg(c).").expect("Enable argument c");
    assert_eq!(
        extensions_of(&mut af),
        set![
            ext!(),
            ext!("a"),
            ext!("b"),
            ext!("c"),
            ext!("a", "c"),
            ext!("b", "c")
        ]
    );
    af.update("+att(c, a).").expect("Enable attack from c to a");
    assert_eq!(
        extensions_of(&mut af),
        set![ext!(), ext!("a"), ext!("b"), ext!("c"), ext!("b", "c")]
    )
}

#[test]
fn simple_conflict_free_af_with_general_updates() {
    let mut af = ArgumentationFramework::<ConflictFree>::new(
        r#"
            arg(a).
            arg(b).
            arg(c).
            att(a, b).
            att(c, a).

            opt(att(a, b)).
            opt(arg(a)).
        "#,
    )
    .expect("Creating AF");
    // Enable a, and a->b by default
    af.update("+att(a, b).")
        .expect("Enabling attack from a to b");
    af.update("+arg(a).").expect("Enabling argument a");
    assert_eq!(
        extensions_of(&mut af),
        set![ext!(), ext!("a"), ext!("b"), ext!("c"), ext!("b", "c")]
    );
    af.update("-att(a, b).")
        .expect("Removing attack from a to b");
    assert_eq!(
        extensions_of(&mut af),
        set![
            ext!(),
            ext!("a"),
            ext!("b"),
            ext!("c"),
            ext!("a", "b"),
            ext!("b", "c")
        ]
    );
    af.update("-arg(a).").expect("Removing argument a");
    assert_eq!(
        extensions_of(&mut af),
        set![ext!(), ext!("b"), ext!("c"), ext!("b", "c")]
    )
}
