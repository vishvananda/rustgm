//! Source compiler bindings are declaration claims, never path-spelling authority.
use super::crates::CrateIdentity;

#[derive(Clone, Default)]
pub enum Mode {
    #[default]
    Bootstrap,
    Source {
        provider: CrateIdentity,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LangItem {
    Sized,
    MetaSized,
    PointeeSized,
    MemSizeConst,
    MemAlignConst,
}
impl LangItem {
    pub const ALL: [Self; 5] = [
        Self::Sized,
        Self::MetaSized,
        Self::PointeeSized,
        Self::MemSizeConst,
        Self::MemAlignConst,
    ];
    #[must_use]
    pub const fn index(self) -> usize {
        match self {
            Self::Sized => 0,
            Self::MetaSized => 1,
            Self::PointeeSized => 2,
            Self::MemSizeConst => 3,
            Self::MemAlignConst => 4,
        }
    }
    /// Attribute decoding only; uses must resolve the registered declaration.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "sized" => Some(Self::Sized),
            "meta_sized" => Some(Self::MetaSized),
            "pointee_sized" => Some(Self::PointeeSized),
            "mem_size_const" => Some(Self::MemSizeConst),
            "mem_align_const" => Some(Self::MemAlignConst),
            _ => None,
        }
    }
}

/// Actual compiler intrinsics, separate from bootstrap ordinary library APIs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LayoutIntrinsic {
    SizeOf,
    AlignOf,
}
impl LayoutIntrinsic {
    /// Called once for an attributed declaration, never for an ordinary use.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "size_of" => Some(Self::SizeOf),
            "align_of" => Some(Self::AlignOf),
            _ => None,
        }
    }
}

/// Validated source declarations select a compiler contract, not an ordinary API.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SourceIntrinsic {
    Layout(LayoutIntrinsic),
    NeedsDrop,
}
impl SourceIntrinsic {
    /// Declaration attribute decoding only; consumers retain the actual `DefId`.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        if name == "needs_drop" {
            Some(Self::NeedsDrop)
        } else {
            LayoutIntrinsic::from_name(name).map(Self::Layout)
        }
    }
}
