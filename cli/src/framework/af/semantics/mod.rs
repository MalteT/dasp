macro_rules! impl_program {
    ($name:ident, $path:literal) => {
        impl super::Program for $name {
            const PROGRAM: &'static str = include_str!($path);
        }
    };
}

pub trait Program {
    const PROGRAM: &'static str;
}

pub struct Complete;
pub struct Stable;
pub struct Ground;
pub struct Admissible;

impl_program!(Complete, "./comp.dl");
impl_program!(Stable, "./stable.dl");
impl_program!(Ground, "./ground.dl");
impl_program!(Admissible, "./adm.dl");
