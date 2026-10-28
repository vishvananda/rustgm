//! Phase-neutral warning identities have no printing or semantic permission.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WarningCode {
    Deprecated,
    HiddenGlobReexports,
    IneffectiveUnstableTraitImpl,
    UnfulfilledLintExpectations,
    InvalidAtomicOrdering,
    UnreachablePatterns,
    UnusedMustUse,
    UnusedAttributes,
    UnusedVisibilities,
    UnstableSyntaxPreExpansion,
    DuplicateMacroAttributes,
    AmbiguousDeriveHelpers,
    MalformedDiagnosticAttributes,
    MalformedDiagnosticFormatLiterals,
    MisplacedDiagnosticAttributes,
    UnsafeOpInUnsafeFn,
    StaticMutRefs,
}
impl WarningCode {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Deprecated => "deprecated",
            Self::HiddenGlobReexports => "hidden_glob_reexports",
            Self::IneffectiveUnstableTraitImpl => "ineffective_unstable_trait_impl",
            Self::UnfulfilledLintExpectations => "unfulfilled_lint_expectations",
            Self::InvalidAtomicOrdering => "invalid_atomic_ordering",
            Self::UnreachablePatterns => "unreachable_patterns",
            Self::UnusedMustUse => "unused_must_use",
            Self::UnusedAttributes => "unused_attributes",
            Self::UnusedVisibilities => "unused_visibilities",
            Self::UnstableSyntaxPreExpansion => "unstable_syntax_pre_expansion",
            Self::DuplicateMacroAttributes => "duplicate_macro_attributes",
            Self::AmbiguousDeriveHelpers => "ambiguous_derive_helpers",
            Self::MalformedDiagnosticAttributes => "malformed_diagnostic_attributes",
            Self::MalformedDiagnosticFormatLiterals => "malformed_diagnostic_format_literals",
            Self::MisplacedDiagnosticAttributes => "misplaced_diagnostic_attributes",
            Self::UnsafeOpInUnsafeFn => "unsafe_op_in_unsafe_fn",
            Self::StaticMutRefs => "static_mut_refs",
        }
    }
}
