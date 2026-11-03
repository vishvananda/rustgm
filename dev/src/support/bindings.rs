//! Source compiler bindings are declaration claims, never path-spelling authority.
use super::crates::CrateIdentity;

#[derive(Clone, Default)]
pub enum Mode {
    #[default]
    Bootstrap,
    Source {
        provider: CrateIdentity,
    },
    SourceClient {
        provider: CrateIdentity,
    },
}

mod lang_contracts;
mod lang_items;
pub(crate) use lang_contracts::{DeclarationProfile, DeclarationTarget, GenericRequirement};
pub use lang_items::LangItem;

/// Actual compiler intrinsics, separate from bootstrap ordinary library APIs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LayoutIntrinsic {
    SizeOf,
    AlignOf,
    AssertInhabited,
}
impl LayoutIntrinsic {
    /// Called once for an attributed declaration, never for an ordinary use.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "size_of" => Some(Self::SizeOf),
            "align_of" => Some(Self::AlignOf),
            "assert_inhabited" => Some(Self::AssertInhabited),
            _ => None,
        }
    }
}

/// Validated source declarations select a compiler contract, not an ordinary API.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SourceIntrinsic {
    Layout(LayoutIntrinsic),
    NeedsDrop,
    ConstEvalSelect,
    UbChecks,
    Abort,
    Transmute,
    CallerLocation,
    PtrMetadata,
    DiscriminantValue,
    AggregateRawPtr,
    OffsetOf,
    ThreeWayCompare,
    ArithOffset,
    UncheckedSub,
    SubWithOverflow,
    ColdPath,
    TransmuteUnchecked,
}
impl SourceIntrinsic {
    /// Declaration attribute decoding only; consumers retain the actual `DefId`.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        if name == "transmute_unchecked" {
            Some(Self::TransmuteUnchecked)
        } else if name == "unchecked_sub" {
            Some(Self::UncheckedSub)
        } else if name == "sub_with_overflow" {
            Some(Self::SubWithOverflow)
        } else if name == "cold_path" {
            Some(Self::ColdPath)
        } else if name == "arith_offset" {
            Some(Self::ArithOffset)
        } else if name == "three_way_compare" {
            Some(Self::ThreeWayCompare)
        } else if name == "offset_of" {
            Some(Self::OffsetOf)
        } else if name == "aggregate_raw_ptr" {
            Some(Self::AggregateRawPtr)
        } else if name == "discriminant_value" {
            Some(Self::DiscriminantValue)
        } else if name == "ptr_metadata" {
            Some(Self::PtrMetadata)
        } else if name == "caller_location" {
            Some(Self::CallerLocation)
        } else if name == "transmute" {
            Some(Self::Transmute)
        } else if name == "abort" {
            Some(Self::Abort)
        } else if name == "const_eval_select" {
            Some(Self::ConstEvalSelect)
        } else if name == "needs_drop" {
            Some(Self::NeedsDrop)
        } else if name == "ub_checks" {
            Some(Self::UbChecks)
        } else {
            LayoutIntrinsic::from_name(name).map(Self::Layout)
        }
    }
}
