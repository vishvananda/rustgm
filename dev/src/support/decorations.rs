//! Attribute recognition names an existing consumer, never a semantic permission.
//! Consumers retain their owner, placement, input, feature and use validation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Consumer {
    Configuration,
    Loading,
    Requests,
    Expansion,
    SourceBindings,
    Stability,
    Representation,
    Linkage,
    TargetFeature,
    SafetyLint,
    Documentation,
    MustUse,
    Inlining,
    TraitParentheses,
    CrateOptions,
    TraitDiagnostics,
    InherentPolicy,
    IncoherentTarget,
    InsignificantDestructor,
    NonExhaustive,
    UnsafeSpecialization,
    MarkerTrait,
    DerivedImplementation,
    CallerTracking,
    IntrinsicConstPolicy,
    IntrinsicNoUnwind,
    ConstPanicPolicy,
    Coverage,
    InheritedOverflow,
    NoImplicitAutorefs,
    PointerLintMarker,
    ColdAdvisory,
    PreludeImport,
    PublicTransparency,
    Promotion,
    Comptime,
    NoMirInline,
    ReservationImpl,
    NonConstTraitMethod,
}
impl Consumer {
    pub(crate) fn tool_member(tool: &str, name: &str) -> Option<Self> {
        match (tool, name) {
            ("diagnostic", "on_unimplemented") => Some(Self::TraitDiagnostics),
            _ => None,
        }
    }

    pub(crate) fn named(name: &str) -> Option<Self> {
        Some(match name {
            "cfg" | "cfg_attr" => Self::Configuration,
            "no_core" | "no_std" | "no_implicit_prelude" | "path" => Self::Loading,
            "feature" => Self::Requests,
            "macro_use"
            | "macro_export"
            | "rustc_macro_transparency"
            | "recursion_limit"
            | "crate_name" => Self::Expansion,
            "lang"
            | "rustc_builtin_macro"
            | "allow_internal_unstable"
            | "allow_internal_unsafe"
            | "rustc_intrinsic"
            | "rustc_deny_explicit_impl"
            | "rustc_diagnostic_item"
            | "fundamental"
            | "rustc_specialization_trait"
            | "rustc_coinductive"
            | "rustc_dyn_incompatible_trait"
            | "rustc_trivial_field_reads"
            | "rustc_nonnull_optimization_guaranteed"
            | "rustc_never_returns_null_ptr"
            | "panic_handler" => Self::SourceBindings,
            "stable"
            | "unstable"
            | "rustc_allowed_through_unstable_modules"
            | "unstable_feature_bound"
            | "rustc_const_stable"
            | "rustc_const_unstable"
            | "rustc_default_body_unstable"
            | "rustc_allow_const_fn_unstable" => Self::Stability,
            "rustc_const_stable_indirect" | "rustc_do_not_const_check" => Self::ConstPanicPolicy,
            "repr" | "derive" | "derive_const" => Self::Representation,
            "coverage" => Self::Coverage,
            "rustc_inherit_overflow_checks" => Self::InheritedOverflow,
            "rustc_no_implicit_autorefs" => Self::NoImplicitAutorefs,
            "rustc_as_ptr" => Self::PointerLintMarker,
            "cold" => Self::ColdAdvisory,
            "prelude_import" => Self::PreludeImport,
            "rustc_pub_transparent" => Self::PublicTransparency,
            "rustc_promotable" => Self::Promotion,
            "rustc_comptime" => Self::Comptime,
            "rustc_no_mir_inline" => Self::NoMirInline,
            "rustc_reservation_impl" => Self::ReservationImpl,
            "rustc_non_const_trait_method" => Self::NonConstTraitMethod,
            "no_mangle" | "export_name" | "link_name" => Self::Linkage,
            "target_feature" => Self::TargetFeature,
            "allow" | "warn" | "deny" | "forbid" | "expect" | "deprecated" => Self::SafetyLint,
            "doc" => Self::Documentation,
            "rustc_nounwind" => Self::IntrinsicNoUnwind,
            "track_caller" => Self::CallerTracking,
            "rustc_intrinsic_const_stable_indirect" => Self::IntrinsicConstPolicy,
            "must_use" => Self::MustUse,
            "inline" => Self::Inlining,
            "non_exhaustive" => Self::NonExhaustive,
            "marker" => Self::MarkerTrait,
            "automatically_derived" => Self::DerivedImplementation,
            "rustc_unsafe_specialization_marker" => Self::UnsafeSpecialization,
            "rustc_paren_sugar" => Self::TraitParentheses,
            "rustc_coherence_is_core" | "rustc_preserve_ub_checks" => Self::CrateOptions,
            "rustc_on_unimplemented" => Self::TraitDiagnostics,
            "rustc_allow_incoherent_impl" => Self::InherentPolicy,
            "rustc_has_incoherent_inherent_impls" => Self::IncoherentTarget,
            "rustc_insignificant_dtor" => Self::InsignificantDestructor,
            _ => return None,
        })
    }
}

/// Actual compiler-predefined tool registration, pinned to Rust 1.98's resolver.
/// This classification retains tool metadata; it grants no feature/API use.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PredefinedTool {
    Clippy,
    Rustfmt,
    Diagnostic,
    Miri,
    RustAnalyzer,
}
impl PredefinedTool {
    pub(crate) fn named(name: &str) -> Option<Self> {
        Some(match name {
            "clippy" => Self::Clippy,
            "rustfmt" => Self::Rustfmt,
            "diagnostic" => Self::Diagnostic,
            "miri" => Self::Miri,
            "rust_analyzer" => Self::RustAnalyzer,
            _ => return None,
        })
    }
}
