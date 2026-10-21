//! Canonical comparison trait and member vocabulary, independent of source spelling.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Comparison {
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
}
impl Comparison {
    pub const ALL: [Self; 4] = [Self::PartialEq, Self::Eq, Self::PartialOrd, Self::Ord];
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::PartialEq => "PartialEq",
            Self::Eq => "Eq",
            Self::PartialOrd => "PartialOrd",
            Self::Ord => "Ord",
        }
    }
    #[must_use]
    pub const fn has_rhs(self) -> bool {
        matches!(self, Self::PartialEq | Self::PartialOrd)
    }
    #[must_use]
    pub const fn parents(self) -> &'static [Self] {
        match self {
            Self::PartialEq => &[],
            Self::Eq | Self::PartialOrd => &[Self::PartialEq],
            Self::Ord => &[Self::Eq, Self::PartialOrd],
        }
    }
    #[must_use]
    pub const fn methods(self) -> &'static [Method] {
        match self {
            Self::PartialEq => &[Method::Eq, Method::Ne],
            Self::Eq => &[],
            Self::PartialOrd => &[
                Method::PartialCmp,
                Method::Lt,
                Method::Le,
                Method::Gt,
                Method::Ge,
            ],
            Self::Ord => &[Method::Cmp, Method::Min, Method::Max, Method::Clamp],
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Method {
    Eq,
    Ne,
    PartialCmp,
    Lt,
    Le,
    Gt,
    Ge,
    Cmp,
    Min,
    Max,
    Clamp,
}
impl Method {
    pub const ALL: [Self; 11] = [
        Self::Eq,
        Self::Ne,
        Self::PartialCmp,
        Self::Lt,
        Self::Le,
        Self::Gt,
        Self::Ge,
        Self::Cmp,
        Self::Min,
        Self::Max,
        Self::Clamp,
    ];
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Eq => "eq",
            Self::Ne => "ne",
            Self::PartialCmp => "partial_cmp",
            Self::Lt => "lt",
            Self::Le => "le",
            Self::Gt => "gt",
            Self::Ge => "ge",
            Self::Cmp => "cmp",
            Self::Min => "min",
            Self::Max => "max",
            Self::Clamp => "clamp",
        }
    }
    #[must_use]
    pub const fn owner(self) -> Comparison {
        match self {
            Self::Eq | Self::Ne => Comparison::PartialEq,
            Self::PartialCmp | Self::Lt | Self::Le | Self::Gt | Self::Ge => Comparison::PartialOrd,
            Self::Cmp | Self::Min | Self::Max | Self::Clamp => Comparison::Ord,
        }
    }
    #[must_use]
    pub const fn owned(self) -> bool {
        matches!(self, Self::Min | Self::Max | Self::Clamp)
    }
    #[must_use]
    pub const fn relation(self) -> bool {
        matches!(self, Self::Lt | Self::Le | Self::Gt | Self::Ge)
    }
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|member| member.name() == name)
    }
}
