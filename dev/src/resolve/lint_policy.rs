//! Source-owned lexical lint rules are shared by expansion diagnostics and semantics.
mod attribute_changes;
mod expectations;
pub(crate) use expectations::{ExpectationId, Progress};
pub(crate) mod producer;
use crate::resolve::{package::Error, unit::NodeKey};
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Level {
    Allow,
    Expect,
    Warn,
    Deny,
    Forbid,
}
impl Level {
    pub(crate) fn denies(self) -> bool {
        matches!(self, Self::Deny | Self::Forbid)
    }
}
#[derive(Clone, Copy)]
pub(crate) struct Policy {
    pub(crate) operation: Level,
    pub(crate) reference: Level,
    pub(crate) atomic_ordering: Level,
    pub(crate) unreachable_patterns: Level,
    pub(crate) must_use: Level,
    pub(crate) unused_attributes: Level,
    pub(crate) unused_visibilities: Level,
    pub(crate) duplicate_macro_attributes: Level,
    pub(crate) ambiguous_derive_helpers: Level,
    pub(crate) diagnostic_attributes: [Level; 3],
    pub(crate) deprecated: Level,
    pub(crate) unfulfilled: Level,
    expectations: [Option<ExpectationId>; 14],
    pub(crate) deprecated_origin: Option<producer::Tail>,
    pub(crate) deprecation_explicit: bool,
}
impl Policy {
    pub(crate) const DEFAULT: Self = Self {
        operation: Level::Warn,
        reference: Level::Deny,
        atomic_ordering: Level::Deny,
        unreachable_patterns: Level::Warn,
        must_use: Level::Warn,
        unused_attributes: Level::Warn,
        unused_visibilities: Level::Warn,
        duplicate_macro_attributes: Level::Warn,
        ambiguous_derive_helpers: Level::Warn,
        diagnostic_attributes: [Level::Warn; 3],
        deprecated: Level::Warn,
        unfulfilled: Level::Warn,
        expectations: [None; 14],
        deprecated_origin: None,
        deprecation_explicit: false,
    };
}
impl Default for Policy {
    fn default() -> Self {
        Self::DEFAULT
    }
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Lint {
    Operation,
    Reference,
    AtomicOrdering,
    UnreachablePatterns,
    MustUse,
    UnusedAttributes,
    UnusedVisibilities,
    DuplicateMacroAttributes,
    AmbiguousDeriveHelpers,
    DiagnosticAttributes(u8),
    Deprecated,
    UnfulfilledExpectations,
}
pub(crate) struct Change {
    pub(crate) lint: Lint,
    pub(crate) level: Level,
    pub(crate) offset: u32,
    pub(crate) expectation: Option<ExpectationId>,
}

#[derive(Clone, Copy)]
pub(crate) struct Violation {
    pub key: NodeKey,
    pub offset: u32,
    pub message: &'static str,
}
impl From<Violation> for Error {
    fn from(error: Violation) -> Self {
        Self::at(error.key.file, error.offset, error.message)
    }
}
impl Policy {
    pub(crate) fn apply(&mut self, key: NodeKey, change: &Change) -> Result<(), Violation> {
        if change.lint == Lint::Deprecated {
            self.deprecation_explicit = true;
        }
        let level = match change.lint {
            Lint::Deprecated => &mut self.deprecated,
            Lint::UnfulfilledExpectations => &mut self.unfulfilled,
            Lint::Operation => &mut self.operation,
            Lint::Reference => &mut self.reference,
            Lint::AtomicOrdering => &mut self.atomic_ordering,
            Lint::UnreachablePatterns => &mut self.unreachable_patterns,
            Lint::MustUse => &mut self.must_use,
            Lint::UnusedAttributes => &mut self.unused_attributes,
            Lint::UnusedVisibilities => &mut self.unused_visibilities,
            Lint::DuplicateMacroAttributes => &mut self.duplicate_macro_attributes,
            Lint::AmbiguousDeriveHelpers => &mut self.ambiguous_derive_helpers,
            Lint::DiagnosticAttributes(index) => {
                &mut self.diagnostic_attributes[usize::from(index)]
            }
        };
        if *level == Level::Forbid {
            if matches!(change.level, Level::Allow | Level::Warn | Level::Expect) {
                return Err(Violation {
                    key,
                    offset: change.offset,
                    message: match change.lint {
                        Lint::Deprecated => "forbidden deprecated lint cannot be weakened",
                        Lint::UnfulfilledExpectations => {
                            "forbidden unfulfilled_lint_expectations lint cannot be weakened"
                        }
                        Lint::AtomicOrdering => "forbidden atomic ordering lint cannot be weakened",
                        Lint::UnreachablePatterns => {
                            "forbidden unreachable pattern lint cannot be weakened"
                        }
                        Lint::MustUse => "forbidden unused_must_use lint cannot be weakened",
                        Lint::UnusedAttributes => {
                            "forbidden unused_attributes lint cannot be weakened"
                        }
                        Lint::UnusedVisibilities => {
                            "forbidden unused_visibilities lint cannot be weakened"
                        }
                        Lint::DuplicateMacroAttributes => {
                            "forbidden duplicate_macro_attributes lint cannot be weakened"
                        }
                        Lint::AmbiguousDeriveHelpers => {
                            "forbidden ambiguous_derive_helpers lint cannot be weakened"
                        }
                        Lint::DiagnosticAttributes(_) => {
                            "forbidden diagnostic attribute lint cannot be weakened"
                        }
                        _ => "forbidden unsafe lint cannot be weakened",
                    },
                });
            }
            return Ok(());
        }
        if change.lint != Lint::UnfulfilledExpectations || change.level != Level::Expect {
            *level = change.level;
            self.expectations[change.lint.index()] = change.expectation;
        }
        Ok(())
    }
}

impl Lint {
    pub(crate) const fn index(self) -> usize {
        match self {
            Self::Operation => 0,
            Self::Reference => 1,
            Self::AtomicOrdering => 2,
            Self::UnreachablePatterns => 3,
            Self::MustUse => 4,
            Self::UnusedAttributes => 5,
            Self::UnusedVisibilities => 6,
            Self::DuplicateMacroAttributes => 7,
            Self::AmbiguousDeriveHelpers => 8,
            Self::DiagnosticAttributes(index) => 9 + index as usize,
            Self::Deprecated => 12,
            Self::UnfulfilledExpectations => 13,
        }
    }
}
impl Policy {
    pub(crate) fn expectation(&self, lint: Lint) -> Option<ExpectationId> {
        self.expectations[lint.index()]
    }
}

impl Lint {
    pub(crate) const fn warning(code: crate::support::diagnostics::WarningCode) -> Option<Self> {
        use crate::support::diagnostics::WarningCode as W;
        Some(match code {
            W::Deprecated => Self::Deprecated,
            W::UnfulfilledLintExpectations => Self::UnfulfilledExpectations,
            W::InvalidAtomicOrdering => Self::AtomicOrdering,
            W::UnreachablePatterns => Self::UnreachablePatterns,
            W::UnusedMustUse => Self::MustUse,
            W::UnusedAttributes => Self::UnusedAttributes,
            W::UnusedVisibilities => Self::UnusedVisibilities,
            W::DuplicateMacroAttributes => Self::DuplicateMacroAttributes,
            W::AmbiguousDeriveHelpers => Self::AmbiguousDeriveHelpers,
            W::MalformedDiagnosticAttributes => Self::DiagnosticAttributes(0),
            W::MalformedDiagnosticFormatLiterals => Self::DiagnosticAttributes(1),
            W::MisplacedDiagnosticAttributes => Self::DiagnosticAttributes(2),
            W::UnsafeOpInUnsafeFn => Self::Operation,
            W::StaticMutRefs => Self::Reference,
            W::UnstableSyntaxPreExpansion => return None,
        })
    }
}

impl Lint {
    pub(super) fn mask(self) -> Option<u16> {
        1_u16.checked_shl(u32::try_from(self.index()).ok()?)
    }
}
