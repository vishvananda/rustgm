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
    Freeze,
    DropGlue,
    Unsize,
    CoerceUnsized,
    FormatArguments,
    FormatArgument,
    Not,
    Panic,
    PanicFmt,
    PanicImpl,
    PanicDisplay,
    ConstPanicFmt,
    PointeeTrait,
    MetadataType,
    DynMetadata,
    PhantomData,
    PanicLocation,
    PanicInfo,
    TrivialClone,
    TryTrait,
    TryFromResidual,
    TryFromOutput,
    TryBranch,
    IntoTryType,
    ControlFlowContinue,
    ControlFlowBreak,
}
impl LangItem {
    pub const ALL: [Self; 42] = [
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
        Self::Freeze,
        Self::DropGlue,
        Self::Unsize,
        Self::CoerceUnsized,
        Self::FormatArguments,
        Self::FormatArgument,
        Self::Not,
        Self::Panic,
        Self::PanicFmt,
        Self::PanicImpl,
        Self::PanicDisplay,
        Self::ConstPanicFmt,
        Self::PointeeTrait,
        Self::MetadataType,
        Self::DynMetadata,
        Self::PhantomData,
        Self::PanicLocation,
        Self::PanicInfo,
        Self::TrivialClone,
        Self::TryTrait,
        Self::TryFromResidual,
        Self::TryFromOutput,
        Self::TryBranch,
        Self::IntoTryType,
        Self::ControlFlowContinue,
        Self::ControlFlowBreak,
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
            Self::Freeze => 16,
            Self::DropGlue => 17,
            Self::Unsize => 18,
            Self::CoerceUnsized => 19,
            Self::FormatArguments => 20,
            Self::FormatArgument => 21,
            Self::Not => 22,
            Self::Panic => 23,
            Self::PanicFmt => 24,
            Self::PanicImpl => 25,
            Self::PanicDisplay => 26,
            Self::ConstPanicFmt => 27,
            Self::PointeeTrait => 28,
            Self::MetadataType => 29,
            Self::DynMetadata => 30,
            Self::PhantomData => 31,
            Self::PanicLocation => 32,
            Self::PanicInfo => 33,
            Self::TrivialClone => 34,
            Self::TryTrait => 35,
            Self::TryFromResidual => 36,
            Self::TryFromOutput => 37,
            Self::TryBranch => 38,
            Self::IntoTryType => 39,
            Self::ControlFlowContinue => 40,
            Self::ControlFlowBreak => 41,
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
            "freeze" => Some(Self::Freeze),
            "drop_glue" => Some(Self::DropGlue),
            "unsize" => Some(Self::Unsize),
            "coerce_unsized" => Some(Self::CoerceUnsized),
            "format_arguments" => Some(Self::FormatArguments),
            "format_argument" => Some(Self::FormatArgument),
            "not" => Some(Self::Not),
            "panic" => Some(Self::Panic),
            "panic_fmt" => Some(Self::PanicFmt),
            "panic_impl" => Some(Self::PanicImpl),
            "panic_display" => Some(Self::PanicDisplay),
            "const_panic_fmt" => Some(Self::ConstPanicFmt),
            "pointee_trait" => Some(Self::PointeeTrait),
            "metadata_type" => Some(Self::MetadataType),
            "dyn_metadata" => Some(Self::DynMetadata),
            "phantom_data" => Some(Self::PhantomData),
            "panic_location" => Some(Self::PanicLocation),
            "panic_info" => Some(Self::PanicInfo),
            "trivial_clone" => Some(Self::TrivialClone),
            "Try" => Some(Self::TryTrait),
            "from_residual" => Some(Self::TryFromResidual),
            "from_output" => Some(Self::TryFromOutput),
            "branch" => Some(Self::TryBranch),
            "into_try_type" => Some(Self::IntoTryType),
            "Continue" => Some(Self::ControlFlowContinue),
            "Break" => Some(Self::ControlFlowBreak),
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
    ConstEvalSelect,
    UbChecks,
    Abort,
    Transmute,
}
impl SourceIntrinsic {
    /// Declaration attribute decoding only; consumers retain the actual `DefId`.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        if name == "transmute" {
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
