//! Analysis-only keep-going census records. A census is never a compilation result.
//!
//! Records are appended in discovery order. Every isolated loop iterates ordered
//! vectors, so that order is deterministic; hash tables here only answer lookups.
use crate::{
    resolve::{
        expansion::Report,
        model::DefId,
        package::{DiagnosticFile, Error},
    },
    support::hash::HashMap,
};
#[cfg(test)]
#[path = "../../tests/invariants/census_records.rs"]
mod tests;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Frontend,
    Declarations,
    Bodies,
    Completion,
}
impl Phase {
    pub const ALL: [Self; 4] = [
        Self::Frontend,
        Self::Declarations,
        Self::Bodies,
        Self::Completion,
    ];
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Frontend => "frontend",
            Self::Declarations => "declarations",
            Self::Bodies => "bodies",
            Self::Completion => "completion",
        }
    }
    const fn index(self) -> usize {
        self as usize
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Category {
    /// An isolated item failed with its own diagnostic.
    Failed,
    /// An item reached a failed item; `blocked_by` names the root-cause record.
    Blocked,
    /// Budget or capacity exhaustion is sticky and ends the census.
    Budget,
    /// A failure outside an isolated loop ends the census.
    Crate,
    /// Later passes consume completed facts of every item, so they did not run.
    Stopped,
    /// A later crate-wide pass ran after the failed pass that ended the census,
    /// on whatever facts that pass left. Its record follows the terminal
    /// record, names it in `blocked_by` and may be derived from it.
    Lookahead,
}
impl Category {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Failed => "failed",
            Self::Blocked => "blocked",
            Self::Budget => "budget",
            Self::Crate => "crate",
            Self::Stopped => "stopped",
            Self::Lookahead => "lookahead",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Location {
    Source { file: u32, offset: u32 },
    Imported { label: String, offset: u32 },
    Unlocated,
}
impl Location {
    #[must_use]
    pub fn of(error: &Error) -> Self {
        match &error.file {
            DiagnosticFile::Source(file) => Self::Source {
                file: file.0,
                offset: error.span.start,
            },
            DiagnosticFile::Imported(label) => Self::Imported {
                label: label.clone(),
                offset: error.span.start,
            },
            DiagnosticFile::Unlocated => Self::Unlocated,
        }
    }
}

/// One macro call site from a diagnostic's retained expansion trace.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Frame {
    pub macro_name: String,
    pub file: u32,
    pub offset: u32,
}

#[derive(Clone, Debug)]
pub struct Record {
    pub phase: Phase,
    pub category: Category,
    pub item: Option<String>,
    pub location: Location,
    pub expansions: Vec<Frame>,
    pub message: String,
    pub blocked_by: Option<usize>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Counts {
    pub checked: usize,
    pub failed: usize,
    pub blocked: usize,
}

#[derive(Default)]
pub struct Census {
    records: Vec<Record>,
    counts: [Counts; Phase::ALL.len()],
    terminal: Option<usize>,
    seen: HashMap<(Location, String), usize>,
}
impl Census {
    #[must_use]
    pub fn records(&self) -> &[Record] {
        &self.records
    }
    #[must_use]
    pub const fn counts(&self, phase: Phase) -> Counts {
        self.counts[phase.index()]
    }
    /// The record that ended the census, if any. A census that ran every pass
    /// has no terminal record, but it is still analysis-only.
    #[must_use]
    pub fn terminal(&self) -> Option<(usize, &Record)> {
        self.terminal
            .and_then(|index| self.records.get(index).map(|record| (index, record)))
    }
    #[must_use]
    pub const fn has_failures(&self, phase: Phase) -> bool {
        let counts = self.counts(phase);
        counts.failed != 0 || counts.blocked != 0
    }
    /// End the census at a failure outside any isolated loop.
    pub fn stop(&mut self, phase: Phase, error: &Error) {
        let category = if exhausts_budget(&error.message) {
            Category::Budget
        } else {
            Category::Crate
        };
        self.finish(phase, category, error, None);
    }
    /// End the census at a failure whose failing unit an earlier record poisoned.
    pub(crate) fn stop_blocked(&mut self, phase: Phase, error: &Error, cause: usize) {
        self.finish(phase, Category::Blocked, error, Some(cause));
    }
    pub(crate) fn stop_after_failures(&mut self, phase: Phase) {
        let counts = self.counts(phase);
        let message = format!(
            "later passes were not run after {} failed and {} blocked items in {}",
            counts.failed,
            counts.blocked,
            phase.name()
        );
        self.terminal = Some(self.push(Record {
            phase,
            category: Category::Stopped,
            item: None,
            location: Location::Unlocated,
            expansions: Vec::new(),
            message,
            blocked_by: None,
        }));
    }
    /// Name the crate-wide pass whose failure ended the census, then record
    /// each later pass's first error as following that terminal record. No
    /// count changes: a lookahead isolates nothing and qualifies nothing.
    pub(crate) fn look_ahead(
        &mut self,
        phase: Phase,
        pass: &str,
        following: impl IntoIterator<Item = (&'static str, Error)>,
    ) {
        let Some(terminal) = self.terminal else {
            return;
        };
        if let Some(record) = self.records.get_mut(terminal) {
            record.item = Some(format!("pass `{pass}`"));
        }
        for (pass, error) in following {
            let item = Some(format!("pass `{pass}`"));
            let mut record = Self::diagnostic(phase, Category::Lookahead, item, &error);
            record.blocked_by = Some(terminal);
            self.push(record);
        }
    }
    pub(crate) const fn checked(&mut self, phase: Phase, count: usize) {
        let counts = &mut self.counts[phase.index()];
        counts.checked = counts.checked.saturating_add(count);
    }
    /// An earlier failure with this exact location and message is the same failure.
    pub(crate) fn repeated(&self, error: &Error) -> Option<usize> {
        self.seen
            .get(&(Location::of(error), error.message.clone()))
            .copied()
    }
    pub(crate) fn failed(&mut self, phase: Phase, item: String, error: &Error) -> usize {
        let index = self.push(Self::diagnostic(phase, Category::Failed, Some(item), error));
        self.seen
            .entry((Location::of(error), error.message.clone()))
            .or_insert(index);
        self.counts[phase.index()].failed += 1;
        index
    }
    pub(crate) fn blocked(&mut self, phase: Phase, item: String, location: Location, cause: usize) {
        let cause = self.cause(cause);
        self.push(Record {
            phase,
            category: Category::Blocked,
            item: Some(item),
            location,
            expansions: Vec::new(),
            message: "blocked by an earlier census failure".into(),
            blocked_by: Some(cause),
        });
        self.counts[phase.index()].blocked += 1;
    }
    /// Blocked records always name the failure at the root of their chain.
    fn cause(&self, mut index: usize) -> usize {
        while let Some(next) = self.records.get(index).and_then(|record| record.blocked_by) {
            if next == index {
                break;
            }
            index = next;
        }
        index
    }
    fn finish(&mut self, phase: Phase, category: Category, error: &Error, cause: Option<usize>) {
        let mut record = Self::diagnostic(phase, category, None, error);
        record.blocked_by = cause.map(|cause| self.cause(cause));
        self.terminal = Some(self.push(record));
    }
    fn diagnostic(phase: Phase, category: Category, item: Option<String>, error: &Error) -> Record {
        Record {
            phase,
            category,
            item,
            location: Location::of(error),
            expansions: error
                .expansions
                .iter()
                .map(|trace| Frame {
                    macro_name: trace.macro_name.clone(),
                    file: trace.call.file.0,
                    offset: trace.call.span.start,
                })
                .collect(),
            message: error.message.clone(),
            blocked_by: None,
        }
    }
    fn push(&mut self, record: Record) -> usize {
        self.records.push(record);
        self.records.len() - 1
    }
}

/// Budget and representation-capacity failures are sticky crate state. This uses
/// the shared diagnostic vocabulary, including ledgers bounded by their input
/// graph, so a misclassified local limit can only stop the census early; it never
/// lets a census continue past a real exhaustion.
#[must_use]
pub fn exhausts_budget(message: &str) -> bool {
    let message = message.to_ascii_lowercase();
    [
        "budget",
        "exhausted",
        "limit exceeded",
        "capacity exceeded",
        "too many ",
    ]
    .iter()
    .any(|word| message.contains(word))
        || (message.contains("exceed")
            && (message.contains("limit") || message.contains("input-graph bound")))
}

/// The kind of cause a fixed-message caller received, so its fold keeps
/// exhaustion, ambiguity and the unmapped-lifetime invariant apart from every
/// genuinely invalid cause.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CauseKind {
    /// Genuine budget or representation-capacity exhaustion.
    Exhausted,
    /// An inference variable stayed unresolved.
    Ambiguous,
    /// A selected type kept a lifetime its query cannot map back.
    UnmappedLifetime,
    /// Every other cause.
    Invalid,
}
impl CauseKind {
    /// A message names its kind by its single producer's shared text.
    fn of_message(message: &str) -> Self {
        if exhausts_budget(message) {
            Self::Exhausted
        } else if message == crate::types::infer::UNRESOLVED {
            Self::Ambiguous
        } else if message == crate::types::substitute::UNMAPPED_LIFETIME {
            Self::UnmappedLifetime
        } else {
            Self::Invalid
        }
    }
}

/// A cause that a fixed-message caller folds by its kind.
pub(crate) trait Cause {
    fn kind(&self) -> CauseKind;
}
impl Cause for String {
    fn kind(&self) -> CauseKind {
        CauseKind::of_message(self)
    }
}
impl Cause for &str {
    fn kind(&self) -> CauseKind {
        CauseKind::of_message(self)
    }
}
impl Cause for crate::types::infer::Error {
    fn kind(&self) -> CauseKind {
        match self {
            Self::Budget | Self::Overflow => CauseKind::Exhausted,
            Self::Unresolved => CauseKind::Ambiguous,
            Self::InvalidIdentity | Self::Mismatch | Self::Occurs | Self::Normalization => {
                CauseKind::Invalid
            }
        }
    }
}
impl Cause for crate::sema::traits::selection::Failure {
    fn kind(&self) -> CauseKind {
        match self {
            Self::Pending(_) => CauseKind::Invalid,
            Self::Invalid(message) => CauseKind::of_message(message),
        }
    }
}

/// Genuine exhaustion keeps its sticky budget wording, ambiguity and the
/// unmapped-lifetime invariant keep their causes' own texts, and every other
/// cause is an ordinary failure with the caller's wording.
pub(crate) fn fold_cause(
    cause: &impl Cause,
    exhausted: &'static str,
    invalid: &'static str,
) -> &'static str {
    match cause.kind() {
        CauseKind::Exhausted => exhausted,
        CauseKind::Ambiguous => crate::types::infer::UNRESOLVED,
        CauseKind::UnmappedLifetime => crate::types::substitute::UNMAPPED_LIFETIME,
        CauseKind::Invalid => invalid,
    }
}

/// The declaration kind and name of an item, for census records only.
pub(crate) fn describe(report: &Report, id: DefId) -> String {
    report
        .resolution
        .definitions
        .get(id.0 as usize)
        .map_or_else(
            || format!("definition {}", id.0),
            |definition| {
                format!(
                    "{} `{}`",
                    definition.kind.name(),
                    report.package.names.get(definition.name)
                )
            },
        )
}

/// The projected source position of an item's declaration.
pub(crate) fn site(report: &Report, id: DefId) -> Location {
    report
        .resolution
        .definitions
        .get(id.0 as usize)
        .and_then(|definition| definition.node)
        .map_or(Location::Unlocated, |key| {
            Location::of(&report.diagnostic(super::model::Declarations::error(
                report,
                key,
                String::new(),
            )))
        })
}
