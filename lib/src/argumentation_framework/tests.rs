use pretty_assertions::assert_eq;

use crate::{
    macros::{ext, set},
    semantics::{Admissible, Complete, Ground, Stable},
};

use super::*;

fn extensions<S: ArgumentationFrameworkSemantic>(program: &str) -> BTreeSet<Extension> {
    let mut af = ArgumentationFramework::<S>::new(program).expect("Creating AF");
    extensions_of(&mut af)
}

fn extensions_of<S: ArgumentationFrameworkSemantic>(
    af: &mut ArgumentationFramework<S>,
) -> BTreeSet<Extension> {
    let extensions = af
        .enumerate_extensions()
        .expect("Enumerating extensions")
        .by_ref()
        .collect::<BTreeSet<_>>()
        .expect("Collecting extensions into hashset")
        .clone();
    extensions
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
            arg(1).
            arg(2).
            arg(3).
            att(1, 3).
            att(2, 3).
            att(3, 2).
        "#,
    );
    assert_eq!(
        extensions,
        set![Extension::EMPTY, ext!("1"), ext!("2"), ext!("1", "2")]
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
        "#,
    )
    .expect("Creating AF");
    let exts = extensions_of(&mut af);
    assert_eq!(exts, set![ext!(), ext!("alpha")]);

    af.update("+att(alpha, alpha).")
        .expect("Added attack to AF");
    let exts = extensions_of(&mut af);
    assert_eq!(exts, set![ext!()]);

    af.update("-att(alpha, beta).")
        .expect("Removed attack from AF");
    let exts = extensions_of(&mut af);
    assert_eq!(exts, set![ext!(), ext!("beta")]);
}

#[test]
fn re_adding_arguments_in_admissible_af() {
    let mut af = ArgumentationFramework::<Admissible>::new(
        r#"
            arg(1).
            arg(2).
        "#,
    )
    .expect("Creating AF");
    let exts = extensions_of(&mut af);
    assert_eq!(exts, set![ext!(), ext!("1"), ext!("2"), ext!("1", "2")]);

    af.update("-arg(1).").expect("Remove argument 1");
    let exts = extensions_of(&mut af);
    assert_eq!(exts, set![ext!(), ext!("2")]);

    af.update("+arg(1).").expect("Re-Add argument 1");
    let exts = extensions_of(&mut af);
    assert_eq!(exts, set![ext!(), ext!("1"), ext!("2"), ext!("1", "2")]);
}

#[test]
fn re_adding_attacks_in_admissible_af() {
    let mut af = ArgumentationFramework::<Admissible>::new(
        r#"
            arg(1).
            arg(2).
            att(1, 2).
        "#,
    )
    .expect("Creating AF");
    let exts = extensions_of(&mut af);
    assert_eq!(exts, set![ext!(), ext!("1")]);

    af.update("-att(1, 2).").expect("Remove attack from 1 to 2");
    let exts = extensions_of(&mut af);
    assert_eq!(exts, set![ext!(), ext!("1"), ext!("2"), ext!("1", "2")]);

    af.update("+att(1, 2).").expect("Re-Add attack from 1 to 2");
    let exts = extensions_of(&mut af);
    assert_eq!(exts, set![ext!(), ext!("1")]);
}
