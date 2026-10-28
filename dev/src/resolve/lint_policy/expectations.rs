//! Cold, owner-authenticated expectation identities and declaration annotations.
use super::{
    Lint, Violation,
    producer::{Producer, Tail, invalid},
};
use crate::support::hash::HashMap;
use crate::{
    resolve::{package::Package, unit::NodeKey},
    syntax::token_tree::Span,
};
use std::{cell::Cell, sync::Arc};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct ExpectationId {
    pub(super) owner: Tail,
    pub(super) index: u32,
}
pub(super) struct Expectation {
    pub effect: u32,
    pub ordinal: u32,
    pub span: Span,
    pub lint: Lint,
    pub members: u16,
    pub reason: Option<Box<str>>,
    pub early: Cell<bool>,
}
pub(super) struct Deprecation {
    pub span: Span,
    pub note: Option<Box<str>>,
    pub since: Option<Box<str>>,
    pub staged_unsupported: bool,
}
pub(crate) struct PendingExpectation<'a> {
    pub id: ExpectationId,
    pub owner: NodeKey,
    pub span: Span,
    pub reason: Option<&'a str>,
    pub self_expect: bool,
    pub resolution_complete: bool,
    pub ordinal: u32,
}
pub(super) struct Domain;
#[derive(Clone, Copy, PartialEq, Eq)]
enum State {
    Fulfilled,
    Finalized,
}
pub(crate) struct Progress {
    domain: Arc<Domain>,
    states: HashMap<ExpectationId, State>,
}
impl Producer {
    pub(crate) fn new_progress(&self, key: NodeKey) -> Result<Option<Box<Progress>>, Violation> {
        self.attribute_read(key, 1)?;
        let Some(domain) = &self.domain else {
            return Ok(None);
        };
        self.diagnostic_read(key, size_of::<Progress>())?;
        Ok(Some(Box::new(Progress {
            domain: Arc::clone(domain),
            states: HashMap::default(),
        })))
    }
    pub(crate) fn validate_progress(
        &self,
        progress: &Progress,
        key: NodeKey,
    ) -> Result<(), Violation> {
        self.attribute_read(key, 1)?;
        if self
            .domain
            .as_ref()
            .is_none_or(|domain| !Arc::ptr_eq(domain, &progress.domain))
        {
            return Err(invalid(
                key,
                "lint request progress belongs to another producer",
            ));
        }
        Ok(())
    }
    fn expectation_row(
        &self,
        package: &Package,
        id: ExpectationId,
        key: NodeKey,
    ) -> Result<&Expectation, Violation> {
        self.attribute_read(key, 1)?;
        let owner = self.record(id.owner, key)?;
        owner.validate(package, &self.reads)?;
        let row = owner
            .parsed
            .expectations
            .get(id.index as usize)
            .ok_or_else(|| invalid(key, "unknown lint expectation identity"))?;
        self.attribute_read(
            key,
            1 + owner.parsed.witnesses.len().checked_ilog2().unwrap_or(0) as usize,
        )?;
        let witness = owner
            .parsed
            .witnesses
            .binary_search_by_key(&row.effect, |witness| witness.effect)
            .ok()
            .and_then(|index| owner.parsed.witnesses.get(index));
        if witness.is_none_or(|witness| witness.attribute.span != row.span) {
            return Err(invalid(key, "lint expectation source identity changed"));
        }
        Ok(row)
    }
    fn event_row(
        &self,
        package: &Package,
        id: ExpectationId,
        key: NodeKey,
        lint: Lint,
    ) -> Result<&Expectation, Violation> {
        let row = self.expectation_row(package, id, key)?;
        if lint.mask().is_none_or(|mask| row.members & mask == 0) {
            return Err(invalid(
                key,
                "lint event belongs to another expectation consumer",
            ));
        }
        if row.lint == Lint::UnfulfilledExpectations {
            return Err(invalid(
                key,
                "unfulfilled lint expectations cannot fulfill themselves",
            ));
        }
        Ok(row)
    }
    pub(crate) fn fulfill_early(
        &self,
        package: &Package,
        id: ExpectationId,
        key: NodeKey,
        lint: Lint,
    ) -> Result<(), Violation> {
        self.event_row(package, id, key, lint)?.early.set(true);
        Ok(())
    }
    pub(crate) fn fulfill(
        &self,
        package: &Package,
        id: ExpectationId,
        key: NodeKey,
        lint: Lint,
        progress: &mut Progress,
    ) -> Result<(), Violation> {
        self.validate_progress(progress, key)?;
        self.event_row(package, id, key, lint)?;
        self.record_progress(progress, id, key, State::Fulfilled)
    }
    fn record_progress(
        &self,
        progress: &mut Progress,
        id: ExpectationId,
        key: NodeKey,
        state: State,
    ) -> Result<(), Violation> {
        self.attribute_read(key, 1)?;
        if let Some(&previous) = progress.states.get(&id) {
            return if previous == state {
                Ok(())
            } else {
                Err(invalid(
                    key,
                    "completed lint expectation cannot change its outcome",
                ))
            };
        }
        let growth = crate::support::retained::Growth::map::<ExpectationId, State>(
            progress.states.len(),
            progress.states.capacity(),
            1,
        );
        self.diagnostic_payload(key, 1 + growth.work, growth.bytes)?;
        progress.states.insert(id, state);
        Ok(())
    }
    pub(crate) fn pending<'a>(
        &'a self,
        package: &Package,
        progress: &Progress,
    ) -> Result<Vec<PendingExpectation<'a>>, Violation> {
        let root = NodeKey {
            file: package.modules[0].file,
            node: package.modules[0].root,
        };
        self.validate_progress(progress, root)?;
        let mut result = Vec::new();
        for &tail in &self.expectation_owners {
            self.attribute_read(root, 1)?;
            let owner = self.record(tail, root)?;
            owner.validate(package, &self.reads)?;
            for (index, row) in owner.parsed.expectations.iter().enumerate() {
                self.attribute_read(owner.key, 1)?;
                let id = ExpectationId {
                    owner: tail,
                    index: u32::try_from(index)
                        .map_err(|_| invalid(owner.key, "expectation index overflow"))?,
                };
                if row.early.get() || progress.states.contains_key(&id) {
                    continue;
                }
                let growth = crate::support::retained::Growth::vector::<PendingExpectation<'_>>(
                    result.len(),
                    result.capacity(),
                    1,
                );
                self.diagnostic_payload(owner.key, 1 + growth.work, growth.bytes)?;
                result.push(PendingExpectation {
                    id,
                    owner: owner.key,
                    span: row.span,
                    reason: row.reason.as_deref(),
                    self_expect: row.lint == Lint::UnfulfilledExpectations,
                    resolution_complete: matches!(
                        row.lint,
                        Lint::HiddenGlobReexports | Lint::IneffectiveUnstableTraitImpl
                    ),
                    ordinal: row.ordinal,
                });
            }
        }
        Ok(result)
    }
    pub(crate) fn finalized(
        &self,
        package: &Package,
        id: ExpectationId,
        key: NodeKey,
        progress: &mut Progress,
    ) -> Result<(), Violation> {
        self.validate_progress(progress, key)?;
        self.expectation_row(package, id, key)?;
        self.record_progress(progress, id, key, State::Finalized)
    }
}

pub(super) struct Early {
    pub key: NodeKey,
    pub target: NodeKey,
    pub origin: Tail,
    pub offset: u32,
    pub introduction: crate::expand::marks::ExpansionId,
}
impl Producer {
    pub(crate) fn early_warning(
        &mut self,
        key: NodeKey,
        target: (NodeKey, Tail),
        offset: u32,
        introduction: crate::expand::marks::ExpansionId,
    ) -> Result<(), Violation> {
        let growth = crate::support::retained::Growth::vector::<Early>(
            self.early.len(),
            self.early.capacity(),
            1,
        );
        self.reserve_diagnostic(key, growth.bytes)?;
        self.attribute_read(key, 1 + growth.work)?;
        self.early.push(Early {
            key,
            target: target.0,
            origin: target.1,
            offset,
            introduction,
        });
        Ok(())
    }
    pub(crate) fn early_warnings(
        &self,
        package: &Package,
        project: impl Fn(crate::resolve::package::Error) -> crate::resolve::package::Error,
    ) -> Result<Vec<(NodeKey, crate::resolve::package::Warning)>, Violation> {
        let mut result = Vec::new();
        for event in &self.early {
            let note = self.deprecation_note(package, event.origin, event.target)?;
            self.diagnostic_read(event.key, note.len().saturating_add(40))?;
            let mut raw = crate::resolve::package::Error::at(
                event.key.file,
                event.offset,
                format!("use of deprecated macro: {note}"),
            );
            raw.introduction = event.introduction;
            let diagnostic = project(raw);
            let (steps, bytes) = diagnostic
                .warning_payload()
                .ok_or_else(|| invalid(event.key, "early warning has a nested carrier"))?;
            self.diagnostic_payload(
                event.key,
                steps,
                bytes.saturating_add(4 * size_of::<(NodeKey, crate::resolve::package::Warning)>()),
            )?;
            result.push((
                event.key,
                crate::resolve::package::Warning {
                    code: crate::support::diagnostics::WarningCode::Deprecated,
                    diagnostic,
                },
            ));
        }
        Ok(result)
    }
}
#[cfg(test)]
#[path = "../../../tests/invariants/lint_expectations.rs"]
mod tests;

impl Producer {
    pub(crate) fn deprecation_owner(
        &self,
        package: &Package,
        key: NodeKey,
    ) -> Result<Option<Tail>, Violation> {
        self.attribute_read(key, 1)?;
        let Some(&tail) = self.index.get(&key) else {
            return Ok(None);
        };
        let owner = self.record(tail, key)?;
        owner.validate(package, &self.reads)?;
        Ok(owner.parsed.deprecated.as_ref().map(|_| tail))
    }
    pub(crate) fn deprecation_in<'a>(
        &'a self,
        package: &Package,
        key: NodeKey,
        mut module: crate::resolve::package::ModuleId,
    ) -> Result<Option<(Tail, &'a str)>, Violation> {
        if let Some(tail) = self.deprecation_owner(package, key)? {
            return self
                .deprecation_note(package, tail, key)
                .map(|note| Some((tail, note)));
        }
        for _ in 0..package.modules.len() {
            self.attribute_read(key, 1)?;
            let entry = package
                .modules
                .get(module.0 as usize)
                .ok_or_else(|| invalid(key, "deprecation module is foreign"))?;
            for source in [
                Some(NodeKey {
                    file: entry.file,
                    node: entry.root,
                }),
                entry.declaration,
            ]
            .into_iter()
            .flatten()
            {
                if let Some(tail) = self.deprecation_owner(package, source)? {
                    return self
                        .deprecation_note(package, tail, key)
                        .map(|note| Some((tail, note)));
                }
            }
            let Some(parent) = entry.parent else {
                return Ok(None);
            };
            module = parent;
        }
        Err(invalid(
            key,
            "deprecation module ancestry exceeds actual inventory",
        ))
    }
    pub(crate) fn deprecation_note<'a>(
        &'a self,
        package: &Package,
        tail: Tail,
        key: NodeKey,
    ) -> Result<&'a str, Violation> {
        let owner = self.record(tail, key)?;
        owner.validate(package, &self.reads)?;
        let row = owner
            .parsed
            .deprecated
            .as_ref()
            .ok_or_else(|| invalid(key, "deprecation origin lost its annotation"))?;
        if package.source_core().is_some() && row.staged_unsupported && row.since.is_some() {
            return Err(Violation {
                key: owner.key,
                offset: row.span.start,
                message: "staged future or nonnumeric deprecation policy is not implemented",
            });
        }
        Ok(row
            .note
            .as_deref()
            .unwrap_or("use of deprecated declaration"))
    }
}

impl Producer {
    pub(crate) fn has_expectations(&self) -> bool {
        !self.expectation_owners.is_empty()
    }
    pub(crate) fn require_deprecation_transport(&self, package: &Package) -> Result<(), Violation> {
        for owner in &self.owners {
            self.attribute_read(owner.key, 1)?;
            if let Some(row) = &owner.parsed.deprecated {
                owner.validate(package, &self.reads)?;
                return Err(Violation {
                    key: owner.key,
                    offset: row.span.start,
                    message: "deprecation declaration transport is not implemented for this metadata profile",
                });
            }
        }
        Ok(())
    }
}
