//! Source-owned lexical lint rules are shared by expansion diagnostics and semantics.
mod attribute_changes;
pub(crate) mod producer;
use crate::resolve::{package::Error, unit::NodeKey};
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Level {
    Allow,
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
    pub(crate) duplicate_macro_attributes: Level,
    pub(crate) diagnostic_attributes: [Level; 3],
}
impl Default for Policy {
    fn default() -> Self {
        Self {
            operation: Level::Warn,
            reference: Level::Deny,
            atomic_ordering: Level::Deny,
            unreachable_patterns: Level::Warn,
            must_use: Level::Warn,
            unused_attributes: Level::Warn,
            duplicate_macro_attributes: Level::Warn,
            diagnostic_attributes: [Level::Warn; 3],
        }
    }
}
#[derive(Clone, Copy)]
pub(crate) enum Lint {
    Operation,
    Reference,
    AtomicOrdering,
    UnreachablePatterns,
    MustUse,
    UnusedAttributes,
    DuplicateMacroAttributes,
    DiagnosticAttributes(u8),
}
pub(crate) struct Change {
    pub(crate) lint: Lint,
    pub(crate) level: Level,
    pub(crate) offset: u32,
}

#[derive(Clone, Copy)]
pub(crate) struct Violation { pub key: NodeKey, pub offset: u32, pub message: &'static str }
impl From<Violation> for Error {
    fn from(error: Violation) -> Self { Self::at(error.key.file, error.offset, error.message) }
}
impl Policy {
    pub(crate) fn apply(&mut self, key: NodeKey, change: &Change) -> Result<(), Violation> {
        let level = match change.lint {
            Lint::Operation => &mut self.operation,
            Lint::Reference => &mut self.reference,
            Lint::AtomicOrdering => &mut self.atomic_ordering,
            Lint::UnreachablePatterns => &mut self.unreachable_patterns,
            Lint::MustUse => &mut self.must_use,
            Lint::UnusedAttributes => &mut self.unused_attributes,
            Lint::DuplicateMacroAttributes => &mut self.duplicate_macro_attributes,
            Lint::DiagnosticAttributes(index) => &mut self.diagnostic_attributes[usize::from(index)],
        };
        if *level == Level::Forbid {
            if matches!(change.level, Level::Allow | Level::Warn) {
                return Err(Violation {
                    key,
                    offset: change.offset,
                    message: match change.lint {
                        Lint::AtomicOrdering => "forbidden atomic ordering lint cannot be weakened",
                        Lint::UnreachablePatterns => {
                            "forbidden unreachable pattern lint cannot be weakened"
                        }
                        Lint::MustUse => "forbidden unused_must_use lint cannot be weakened",
                        Lint::UnusedAttributes => {
                            "forbidden unused_attributes lint cannot be weakened"
                        }
                        Lint::DuplicateMacroAttributes => {
                            "forbidden duplicate_macro_attributes lint cannot be weakened"
                        }
                        Lint::DiagnosticAttributes(_) => "forbidden diagnostic attribute lint cannot be weakened",
                        _ => "forbidden unsafe lint cannot be weakened",
                    },
                });
            }
            return Ok(());
        }
        *level = change.level;
        Ok(())
    }
}
