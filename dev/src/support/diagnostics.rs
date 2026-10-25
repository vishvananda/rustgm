//! Phase-neutral warning identities have no printing or semantic permission.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WarningCode {
    UnusedMustUse,
    UnusedAttributes,
    DuplicateMacroAttributes,
    MalformedDiagnosticAttributes,
    MalformedDiagnosticFormatLiterals,
    MisplacedDiagnosticAttributes,
}
impl WarningCode {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::UnusedMustUse => "unused_must_use",
            Self::UnusedAttributes => "unused_attributes",
            Self::DuplicateMacroAttributes => "duplicate_macro_attributes",
            Self::MalformedDiagnosticAttributes => "malformed_diagnostic_attributes",
            Self::MalformedDiagnosticFormatLiterals => "malformed_diagnostic_format_literals",
            Self::MisplacedDiagnosticAttributes => "misplaced_diagnostic_attributes",
        }
    }
}
