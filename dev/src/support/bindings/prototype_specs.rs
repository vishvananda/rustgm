//! Unique pinned intrinsic identities share immutable declaration-signature profiles.
mod prototype_tables;
use super::LangItem;
use prototype_tables::{NAMES, PROFILES};

/// A known intrinsic declaration identity, independently of its executable contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Declaration(u16);
impl Declaration {
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        let index = NAMES.binary_search_by_key(&name, |(name, _)| *name).ok()?;
        u16::try_from(index).ok().map(Self)
    }
    #[must_use]
    pub fn name(self) -> &'static str {
        NAMES[usize::from(self.0)].0
    }
    pub(crate) fn profile(self) -> &'static Profile {
        &PROFILES[usize::from(NAMES[usize::from(self.0)].1)]
    }
}
#[derive(Clone, Copy)]
pub(crate) enum Region {
    Static,
    Bound(u8),
}
#[derive(Clone, Copy)]
pub(crate) enum Scalar {
    Bool,
    F16,
    F32,
    F64,
    F128,
    I32,
    Isize,
    Str,
    U8,
    U32,
    Usize,
}
impl Scalar {
    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::Bool => "bool",
            Self::F16 => "f16",
            Self::F32 => "f32",
            Self::F64 => "f64",
            Self::F128 => "f128",
            Self::I32 => "i32",
            Self::Isize => "isize",
            Self::Str => "str",
            Self::U8 => "u8",
            Self::U32 => "u32",
            Self::Usize => "usize",
        }
    }
}
#[derive(Clone, Copy)]
pub(crate) enum Term {
    Unit,
    Never,
    Scalar(Scalar),
    Parameter(u8),
    Region(u8),
    Pointer(bool, &'static Self),
    Reference(bool, Region, &'static Self),
    Tuple(&'static [Self]),
    Language(LangItem, &'static [Self]),
    Array(&'static Self, u64),
    Function(bool, &'static [Self], &'static Self),
    OwnedBox,
}
pub(crate) struct Profile {
    pub types: u8,
    pub consts: u8,
    pub unsafe_: bool,
    pub inputs: &'static [Term],
    pub output: Term,
    pub work: u8,
}
