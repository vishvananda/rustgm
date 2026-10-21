//! Canonical equality trait and member vocabulary, independent of source spelling.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Comparison {
    PartialEq,
    Eq,
}
impl Comparison {
    pub const ALL: [Self; 2] = [Self::PartialEq, Self::Eq];
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::PartialEq => "PartialEq",
            Self::Eq => "Eq",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Method {
    Eq,
    Ne,
}
impl Method {
    pub const ALL: [Self; 2] = [Self::Eq, Self::Ne];
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Eq => "eq",
            Self::Ne => "ne",
        }
    }
}
