use std::ops::Add;

const ARGUMENT_PREFIX: &str = "a";

pub type ArgumentWithState = (Argument, State);
pub type AttackWithState = (Attack, State);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum State {
    Dead,
    Alive,
}

#[derive(Debug, Clone, Copy, Eq)]
pub struct Argument {
    id: usize,
    pub optional: bool,
}

#[derive(Debug, Clone, Copy, Eq)]
pub struct Attack {
    from: usize,
    to: usize,
    pub optional: bool,
}

impl Argument {
    pub fn new(id: usize, optional: bool) -> Self {
        Self { id, optional }
    }
    pub fn name(&self) -> String {
        let Argument { id, .. } = self;
        format!("{ARGUMENT_PREFIX}{id}")
    }
}

impl Attack {
    pub fn from_raw(from: usize, to: usize, optional: bool) -> Self {
        Self { from, to, optional }
    }
    pub fn from(&self) -> String {
        let Attack { from, .. } = self;
        format!("{ARGUMENT_PREFIX}{from}")
    }
    pub fn to(&self) -> String {
        let Attack { to, .. } = self;
        format!("{ARGUMENT_PREFIX}{to}")
    }

    pub fn contains(&self, argument: &Argument) -> bool {
        self.from == argument.id || self.to == argument.id
    }
}

impl PartialEq for Argument {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl PartialEq for Attack {
    fn eq(&self, other: &Self) -> bool {
        self.from.eq(&other.from) && self.to.eq(&other.to)
    }
}

impl PartialOrd for Argument {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.id.partial_cmp(&other.id)
    }
}

impl Ord for Argument {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

impl Add<usize> for Argument {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        let Argument { id, optional } = self;
        Self {
            id: id + rhs,
            optional,
        }
    }
}

impl Default for Argument {
    fn default() -> Self {
        Self {
            id: 1,
            optional: false,
        }
    }
}
