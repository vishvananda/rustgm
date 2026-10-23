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
    Copy,
    Clone,
    CloneFn,
    LegacyReceiver,
    Tuple,
    FnOnce,
    FnMut,
    Fn,
    FnOnceOutput,
    Drop,
    Destruct,
}
impl LangItem {
    pub const ALL: [Self; 16] = [
        Self::Sized,
        Self::MetaSized,
        Self::PointeeSized,
        Self::MemSizeConst,
        Self::MemAlignConst,
        Self::Copy,
        Self::Clone,
        Self::CloneFn,
        Self::LegacyReceiver,
        Self::Tuple,
        Self::FnOnce,
        Self::FnMut,
        Self::Fn,
        Self::FnOnceOutput,
        Self::Drop,
        Self::Destruct,
    ];
    #[must_use]
    pub const fn index(self) -> usize {
        match self {
            Self::Sized => 0,
            Self::MetaSized => 1,
            Self::PointeeSized => 2,
            Self::MemSizeConst => 3,
            Self::MemAlignConst => 4,
            Self::Copy => 5,
            Self::Clone => 6,
            Self::CloneFn => 7,
            Self::LegacyReceiver => 8,
            Self::Tuple => 9,
            Self::FnOnce => 10,
            Self::FnMut => 11,
            Self::Fn => 12,
            Self::FnOnceOutput => 13,
            Self::Drop => 14,
            Self::Destruct => 15,
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
            "copy" => Some(Self::Copy),
            "clone" => Some(Self::Clone),
            "clone_fn" => Some(Self::CloneFn),
            "legacy_receiver" => Some(Self::LegacyReceiver),
            "tuple_trait" => Some(Self::Tuple),
            "fn_once" => Some(Self::FnOnce),
            "fn_mut" => Some(Self::FnMut),
            "fn" => Some(Self::Fn),
            "fn_once_output" => Some(Self::FnOnceOutput),
            "drop" => Some(Self::Drop),
            "destruct" => Some(Self::Destruct),
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
