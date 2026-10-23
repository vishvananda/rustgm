//! The actual Proved owner is moved into a cold scope and returned on every exit.
use super::{Constant, Context, Declarations, Engine, Error, Report, Request, Session, Target};
use crate::support::hash::HashMap;
use crate::{
    borrow::incremental::{ParentMember, ScopeOwner},
    const_eval::lease_codes::Scope,
    sema::inline_constants::JournalLease,
};

pub(super) struct Active {
    pub owner: ScopeOwner,
    pub code: Scope,
    journal: JournalLease,
    roots: HashMap<Target, usize>,
    sessions: HashMap<Request, Session>,
    completed: HashMap<Request, Constant>,
    suspended: usize,
    charged: usize,
    inherited_bytes: usize,
    request_bytes: usize,
    previous: Option<Box<Active>>,
}
impl Context {
    pub(crate) fn with_parent(
        &self,
        report: &Report,
        facts: &mut Declarations,
        owner: &ParentMember<'_>,
    ) -> Result<(), Error> {
        let source = owner.body().source;
        let enter: Result<(), &'static str> = (|| {
            let mut engine = self
                .engine
                .try_borrow_mut()
                .map_err(|_| "parent provider is already active")?;
            let active = engine.enter_parent(facts, owner.scope_owner()?)?;
            let rows = engine
                .original_components
                .as_mut()
                .ok_or("parent originals disappeared")?;
            rows.active = Some(active);
            Ok(())
        })();
        if let Err(error) = enter {
            return Err(Declarations::error(report, source, error));
        }
        // The wrapper retains the same immutable Proved while the provider holds
        // only its cold scope guard. No Engine borrow spans executor callbacks.
        let result = crate::body::constants::instance(report, facts, owner.body());
        let restored: Result<(), &'static str> = (|| {
            let mut engine = self
                .engine
                .try_borrow_mut()
                .map_err(|_| "parent provider retained an active executor borrow")?;
            let active = engine
                .original_components
                .as_mut()
                .and_then(|rows| rows.active.take())
                .ok_or("active parent ownership disappeared")?;
            engine.retire_parent(facts, *active, result.is_ok())
        })();
        result.and_then(|()| restored.map_err(|e| Declarations::error(report, source, e)))
    }
}

impl Engine {
    fn enter_parent(
        &mut self,
        facts: &mut Declarations,
        owner: ScopeOwner,
    ) -> Result<Box<Active>, &'static str> {
        owner.validate()?;
        let mut work = 0;
        let validated = owner
            .pending()
            .validate(facts, owner.body(), owner.input(), &mut work);
        facts.body_work.canonical_steps = facts.body_work.canonical_steps.saturating_add(work);
        validated?;
        if self.original_components.is_none() {
            return Err("actual parent has no retained original component");
        }
        let charged = size_of::<Active>().saturating_add(owner.pending().retained_bytes());
        if facts.constant_work.expression_bytes.saturating_add(charged) > 64 * 1024 * 1024 {
            return Err("active parent owner exceeds storage budget");
        }
        let journal = facts
            .inline_constants
            .as_mut()
            .ok_or("conditional values have no registry")
            .and_then(|registry| {
                registry.begin_journal(
                    owner
                        .pending()
                        .ports()
                        .iter()
                        .map(|p| (p.instance, p.result)),
                )
            })?;
        let code = match self.catalog.enter_scope() {
            Ok(scope) => scope,
            Err(error) => {
                if let Some(registry) = facts.inline_constants.as_mut() {
                    let _ = registry.finish_journal(journal, false);
                }
                return Err(error);
            }
        };
        facts.constant_work.expression_bytes =
            facts.constant_work.expression_bytes.saturating_add(charged);
        let inherited_bytes = crate::support::retained::map(&self.roots)
            .saturating_add(crate::support::retained::map(&self.sessions))
            .saturating_add(crate::support::retained::map(&self.completed))
            .saturating_add(
                self.original_components
                    .as_ref()
                    .and_then(|rows| rows.active.as_ref())
                    .map_or(0, |previous| {
                        previous
                            .inherited_bytes
                            .saturating_add(previous.request_bytes)
                            .saturating_add(previous.charged)
                    }),
            );
        Ok(Box::new(Active {
            owner,
            code,
            journal,
            charged,
            inherited_bytes,
            request_bytes: 0,
            roots: std::mem::take(&mut self.roots),
            sessions: std::mem::take(&mut self.sessions),
            completed: std::mem::take(&mut self.completed),
            suspended: self.suspended_slots,
            previous: self
                .original_components
                .as_mut()
                .and_then(|rows| rows.active.take()),
        }))
    }
    fn retire_parent(
        &mut self,
        facts: &mut Declarations,
        active: Active,
        commit: bool,
    ) -> Result<(), &'static str> {
        let Active {
            owner,
            code,
            journal,
            roots,
            sessions,
            completed,
            suspended,
            charged,
            inherited_bytes: _,
            request_bytes: _,
            previous,
        } = active;
        self.roots = roots;
        self.sessions = sessions;
        self.completed = completed;
        self.suspended_slots = suspended;
        if let Some(rows) = &mut self.original_components {
            rows.active = previous;
        }
        facts.constant_work.expression_bytes =
            facts.constant_work.expression_bytes.saturating_sub(charged);
        let code_result = self.catalog.retire_scope(code);
        let values = facts
            .inline_constants
            .as_mut()
            .ok_or("conditional values disappeared")
            .and_then(|registry| registry.finish_journal(journal, commit && code_result.is_ok()));
        drop(owner);
        code_result.and(values)
    }
    pub(super) fn validate_parent_scope(
        &self,
        facts: &mut Declarations,
        target: Target,
    ) -> Result<(), &'static str> {
        let mut work = 0;
        let checked = (|| {
            let Some(active) = self
                .original_components
                .as_ref()
                .and_then(|rows| rows.active.as_ref())
            else {
                return Ok(());
            };
            if !self.catalog.current_scope(active.code)
                || !facts
                    .inline_constants
                    .as_deref()
                    .is_some_and(|registry| registry.current_journal(active.journal))
            {
                return Err("conditional request has a retired or foreign scope");
            }
            let pending = active.owner.pending();
            if let Target::Anonymous { instance, result } = target
                && pending.has_source(instance.source())
            {
                pending.request(
                    facts,
                    active.owner.body(),
                    active.owner.input(),
                    instance,
                    result,
                    &mut work,
                )?;
            } else {
                pending.validate_context(
                    facts,
                    active.owner.body(),
                    active.owner.input(),
                    &mut work,
                )?;
            }
            Ok(())
        })();
        facts.body_work.canonical_steps = facts.body_work.canonical_steps.saturating_add(work);
        checked
    }
    pub(super) fn reserve_parent_request(&mut self, request: &Request) -> Result<(), &'static str> {
        let Some(active) = self
            .original_components
            .as_mut()
            .and_then(|rows| rows.active.as_mut())
        else {
            return Ok(());
        };
        active.request_bytes = active.request_bytes.saturating_add(
            request
                .arguments
                .len()
                .saturating_mul(3 * size_of::<Constant>()),
        );
        let bytes = active
            .charged
            .saturating_add(active.inherited_bytes)
            .saturating_add(active.request_bytes)
            .saturating_add(crate::support::retained::map(&self.roots))
            .saturating_add(crate::support::retained::map(&self.sessions))
            .saturating_add(crate::support::retained::map(&self.completed))
            .saturating_add(4 * (size_of::<Request>() + size_of::<Session>()));
        if bytes > 64 * 1024 * 1024 {
            return Err("conditional request caches exceed storage budget");
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "../../../tests/invariants/parent_retirement.rs"]
mod tests;
