//! Completed calls reference one current, unmet semantic-body context.
use super::{Checker, Declarations, Error};
use crate::{resolve::unit::NodeKey, support::constness::Constness, types::infer::Solved};
use std::num::NonZeroU32;
#[path = "effect_facts/site_locations.rs"]
mod caller_sites;
mod discharged_sites;
mod displacement_rows;
mod protocol_rows;
pub(super) use discharged_sites::call_safety_scratch;

/// A source-node plan location; actual source, member and call checks remain separate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OffsetPlanId(NonZeroU32);
impl OffsetPlanId {
    pub(crate) const fn index(self) -> u32 {
        self.0.get() - 1
    }
}

/// Current-table location; it grants no call, source or const permission.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TryPlanId(NonZeroU32);
impl TryPlanId {
    pub(crate) const fn index(self) -> u32 {
        self.0.get() - 1
    }
}

/// A current-table location, never eligibility, proof payment or a foreign effect.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SourceCallUse(NonZeroU32);
#[derive(Clone, Copy)]
struct Descriptor {
    owner: NodeKey,
    effect: Constness,
}
#[derive(Default)]
pub(in crate::sema) struct CallContexts {
    rows: Vec<Descriptor>,
    plans: Vec<crate::sema::try_plans::Plan>,
    sites: Vec<caller_sites::Site>,
    offsets: Option<Box<displacement_rows::Rows>>,
    safety: Option<Box<discharged_sites::Rows>>,
}
const MAX_ROWS: usize = 1_000_000;
const MAX_RETAINED: usize = 16 * 1024 * 1024;
const MAX_TRANSIENT: usize = 32 * 1024 * 1024;
impl CallContexts {
    fn bytes(&self, capacity: usize) -> Result<usize, &'static str> {
        capacity
            .checked_mul(size_of::<Descriptor>())
            .and_then(|bytes| {
                bytes.checked_add(
                    self.plans
                        .capacity()
                        .checked_mul(size_of::<crate::sema::try_plans::Plan>())?,
                )
            })
            .and_then(|bytes| {
                bytes.checked_add(
                    self.sites
                        .capacity()
                        .checked_mul(size_of::<caller_sites::Site>())?,
                )
            })
            .and_then(|bytes| bytes.checked_add(size_of::<Self>() + size_of::<Option<Box<Self>>>()))
            .and_then(|bytes| {
                bytes.checked_add(self.offsets.as_deref().map_or(0, |rows| {
                    size_of::<displacement_rows::Rows>()
                        + rows.rows.capacity() * size_of::<crate::sema::displacement_plans::Plan>()
                }))
            })
            .and_then(|bytes| {
                bytes.checked_add(
                    self.safety
                        .as_deref()
                        .map_or(0, discharged_sites::Rows::bytes),
                )
            })
            .ok_or("source call context storage overflow")
    }
    fn reserve(&mut self, steps: &mut usize, limit: usize) -> Result<SourceCallUse, &'static str> {
        if self.bytes(self.rows.capacity())? > limit.min(MAX_RETAINED) {
            return Err("source call context storage budget exceeded");
        }
        if self.rows.len() >= MAX_ROWS {
            return Err("source call context inventory exhausted");
        }
        let row = u32::try_from(self.rows.len())
            .ok()
            .and_then(|row| row.checked_add(1))
            .and_then(NonZeroU32::new)
            .map(SourceCallUse)
            .ok_or("source call context identity exhausted")?;
        if self.rows.len() < self.rows.capacity() {
            return Ok(row);
        }
        let capacity = self.rows.len().saturating_mul(2).clamp(4, MAX_ROWS);
        let retained = self.bytes(capacity)?;
        let transient = retained
            .checked_add(self.rows.capacity().saturating_mul(size_of::<Descriptor>()))
            .and_then(|bytes| bytes.checked_add(size_of::<Vec<Descriptor>>()))
            .ok_or("source call context storage overflow")?;
        if retained > limit.min(MAX_RETAINED) || transient > MAX_TRANSIENT {
            return Err("source call context storage budget exceeded");
        }
        charge(steps, self.rows.len())?;
        let mut staged = Vec::new();
        staged
            .try_reserve_exact(capacity)
            .map_err(|_| "source call context allocation failed")?;
        if self.bytes(staged.capacity())? > limit.min(MAX_RETAINED)
            || self
                .bytes(staged.capacity())?
                .saturating_add(self.rows.capacity().saturating_mul(size_of::<Descriptor>()))
                .saturating_add(size_of::<Vec<Descriptor>>())
                > MAX_TRANSIENT
        {
            return Err("source call context allocation exceeds storage budget");
        }
        staged.extend_from_slice(&self.rows);
        self.rows = staged;
        Ok(row)
    }
    fn append(
        &mut self,
        owner: NodeKey,
        effect: Constness,
        steps: &mut usize,
    ) -> Result<SourceCallUse, &'static str> {
        let row = self.reserve(steps, MAX_RETAINED)?;
        // Reservation completes identity/capacity checks before this commit.
        self.rows.push(Descriptor { owner, effect });
        Ok(row)
    }
}
impl SourceCallUse {
    fn descriptor(self, facts: &Declarations) -> Result<&Descriptor, &'static str> {
        facts
            .source_bindings
            .as_deref()
            .ok_or("source call use has no current source catalog")?
            .call_contexts
            .as_deref()
            .and_then(|contexts| contexts.rows.get((self.0.get() - 1) as usize))
            .ok_or("source call use has no current body descriptor")
    }
    /// This lookup is data ownership only. The querying work owner charges it.
    /// # Errors
    /// Rejects missing current catalog data or an invalid row.
    pub fn owner(self, facts: &Declarations) -> Result<NodeKey, &'static str> {
        Ok(self.descriptor(facts)?.owner)
    }
    /// Callers charge successful and rejected lookups in their mutable work budget.
    /// The effect comes from the current table, never from the supplied row handle.
    /// # Errors
    /// Rejects missing data and a use copied from another semantic body.
    pub fn effect_for(
        self,
        facts: &Declarations,
        owner: NodeKey,
    ) -> Result<Constness, &'static str> {
        let descriptor = self.descriptor(facts)?;
        if descriptor.owner != owner {
            return Err("source call use belongs to another semantic body");
        }
        Ok(descriptor.effect)
    }
}
fn charge(steps: &mut usize, amount: usize) -> Result<(), &'static str> {
    *steps = steps
        .checked_add(amount)
        .ok_or("source call context work overflow")?;
    if *steps > 1_000_000 {
        return Err("body type work budget exceeded");
    }
    Ok(())
}
impl Checker<'_> {
    fn existing_call_context(
        &mut self,
        solved: &Solved,
        effect: Constness,
        allowances: &mut crate::resolve::expansion::MacroAllowanceSession<'_>,
    ) -> Result<(bool, Option<SourceCallUse>), Error> {
        if self.state.file != self.state.body.file {
            return Err(Declarations::error(
                self.report,
                self.state.body,
                "source call body changed source unit",
            ));
        }
        let mut existing = None;
        let mut unpublished = false;
        let mut any = false;
        for index in 0..self.state.calls.len() {
            let (node, term) = self.state.calls[index];
            charge(&mut self.state.steps, 1)
                .map_err(|e| Declarations::error(self.report, self.state.body, e))?;
            let key = self.key(node);
            let operation = self
                .facts
                .node(key)
                .ok_or_else(|| {
                    Declarations::error(self.report, self.state.body, "invalid call node")
                })?
                .operation;
            // ForPlan retains distinct into_iter/next signatures, not one Call.
            if matches!(self.kind(node), crate::syntax::ast::Kind::For { .. })
                || self.erased_source_operator(node)?
            {
                continue;
            }
            any = true;
            if matches!(self.kind(node), crate::syntax::ast::Kind::OffsetOf { .. }) {
                if let Some(source) = self.replayed_offset_context(key, operation, effect)? {
                    if existing.is_some_and(|old| old != source) {
                        return Err(self.error(node, "offset calls have different body contexts"));
                    }
                    existing = Some(source);
                } else {
                    unpublished = true;
                }
                continue;
            }
            if self.is_try_call(node) {
                if let Some(source) = self.replayed_try_context(key, operation, effect)? {
                    if existing.is_some_and(|old| old != source) {
                        return Err(Declarations::error(
                            self.report,
                            key,
                            "Try calls have different body contexts",
                        ));
                    }
                    existing = Some(source);
                } else {
                    unpublished = true;
                }
                continue;
            }
            let signature = solved.types.get(term.0 as usize).ok_or_else(|| {
                Declarations::error(self.report, key, "invalid completed call signature")
            })?;
            self.check_const_library_call(node, solved, allowances)?;
            self.check_transmute_size(node, solved)?;
            if let Some(crate::sema::model::Operation::Call {
                source_use: Some(source),
                signature: existing_signature,
            }) = operation
            {
                charge(&mut self.state.steps, 1)
                    .map_err(|e| Declarations::error(self.report, key, e))?;
                let current = source
                    .effect_for(self.facts, self.state.body)
                    .map_err(|e| Declarations::error(self.report, key, e))?;
                if effect != current
                    || *signature != existing_signature
                    || existing.is_some_and(|old| old != source)
                {
                    return Err(Declarations::error(
                        self.report,
                        key,
                        "source call publication changed its current body context",
                    ));
                }
                existing = Some(source);
            } else {
                unpublished = true;
            }
        }
        if existing.is_some() && unpublished {
            return Err(self.error(
                self.state.body.node,
                "source call body was partially published",
            ));
        }
        Ok((any, existing))
    }
    fn check_transmute_size(
        &mut self,
        node: crate::syntax::ast::NodeId,
        solved: &Solved,
    ) -> Result<(), Error> {
        let crate::syntax::ast::Kind::Call { callee, .. } = self.kind(node) else {
            return Ok(());
        };
        let term = self.term(callee)?;
        let ty = *solved
            .types
            .get(term.0 as usize)
            .ok_or_else(|| self.error(node, "source transmute lost its solved callee"))?;
        if !matches!(
            self.facts.types.kind(ty),
            Some(crate::types::Kind::FunctionItem { .. })
        ) {
            return Ok(());
        }
        let instance = crate::types::instance::Instance::new(&self.facts.types, ty)
            .map_err(|e| self.error(node, e))?;
        let selected = self
            .facts
            .source_bindings
            .as_deref()
            .ok_or_else(|| self.error(node, "source call catalog disappeared"))?
            .transmute_selection(self.facts, instance, self.state.body.into())
            .map_err(|e| self.error(node, e))?;
        if let Some(selected) = selected {
            self.facts
                .source_bindings
                .as_deref()
                .ok_or_else(|| self.error(node, "source transmute catalog disappeared"))?
                .require_transmute_report(self.report, self.facts, &selected)
                .map_err(|e| self.error(node, e))?;
            crate::sema::compiler_items::require_transmute_size(
                self.facts,
                selected.source,
                selected.destination,
            )
            .map_err(|e| self.error(node, e))?;
        }
        Ok(())
    }
    pub(super) fn publish_calls(&mut self, solved: &Solved) -> Result<(), Error> {
        if self.facts.source_bindings.is_none() {
            return self.publish_ordinary_calls(solved);
        }
        let effect = self.callee_use_effect()?;
        let mut allowances = self.report.macro_allowances();
        let (any, mut source_use) = self.existing_call_context(solved, effect, &mut allowances)?;
        // Charge all commit ports before retaining a descriptor or changing a node.
        charge(&mut self.state.steps, self.state.calls.len())
            .map_err(|e| self.error(self.state.body.node, e))?;
        let warm = source_use.is_some();
        if any && !warm {
            if self
                .facts
                .source_bindings
                .as_deref()
                .is_some_and(|catalog| catalog.call_contexts.is_none())
            {
                // The current call-context budget includes its enclosing owner.
                // A source client has no local source-header retention producer.
                charge(&mut self.state.steps, 1)
                    .map_err(|error| self.error(self.state.body.node, error))?;
                self.facts
                    .source_bindings
                    .as_deref_mut()
                    .ok_or_else(|| {
                        Declarations::error(
                            self.report,
                            self.state.body,
                            "source catalog disappeared",
                        )
                    })?
                    .call_contexts = Some(Box::default());
            }
            source_use = Some(
                self.facts
                    .source_bindings
                    .as_deref_mut()
                    .and_then(|catalog| catalog.call_contexts.as_deref_mut())
                    .ok_or_else(|| {
                        Declarations::error(
                            self.report,
                            self.state.body,
                            "source call context disappeared",
                        )
                    })?
                    .reserve(&mut self.state.steps, MAX_RETAINED)
                    .map_err(|error| Declarations::error(self.report, self.state.body, error))?,
            );
        }
        let sites = self.prepare_caller_sites(solved, source_use, warm)?;
        self.preflight_site_commit(source_use, &sites, warm)?;
        let safety = self.prepare_safety_sites(solved, source_use)?;
        self.preflight_safety_sites(source_use, &safety, warm)?;
        if any && !warm {
            let actual = self
                .facts
                .source_bindings
                .as_deref_mut()
                .and_then(|catalog| catalog.call_contexts.as_deref_mut())
                .ok_or_else(|| {
                    Declarations::error(
                        self.report,
                        self.state.body,
                        "source call context disappeared before commit",
                    )
                })?
                .append(self.state.body, effect, &mut self.state.steps)
                .map_err(|error| Declarations::error(self.report, self.state.body, error))?;
            if Some(actual) != source_use {
                return Err(self.error(
                    self.state.body.node,
                    "caller preparation changed its reserved context",
                ));
            }
        }
        self.publish_try_plans(solved, source_use, &mut allowances)?;
        self.publish_displacement_plans(solved, source_use, &mut allowances)?;
        self.commit_caller_sites(&sites, warm);
        self.commit_safety_sites(&safety, warm);
        for (node, term) in &self.state.calls {
            if matches!(self.kind(*node), crate::syntax::ast::Kind::For { .. })
                || self.erased_source_operator(*node)?
                || self.is_try_call(*node)
                || matches!(self.kind(*node), crate::syntax::ast::Kind::OffsetOf { .. })
            {
                continue;
            }
            let key = self.key(*node);
            // Preflight checked both indices; only the separate context table
            // changed since then. This commit has no fallible lookup or growth.
            self.facts.nodes[key.file.0 as usize][key.node.get() as usize].operation =
                Some(crate::sema::model::Operation::Call {
                    signature: solved.types[term.0 as usize],
                    source_use,
                });
        }
        Ok(())
    }
    fn publish_ordinary_calls(&mut self, solved: &Solved) -> Result<(), Error> {
        let mut allowances = self.report.macro_allowances();
        for index in 0..self.state.calls.len() {
            let (node, term) = self.state.calls[index];
            if matches!(self.kind(node), crate::syntax::ast::Kind::For { .. }) {
                // publish_for_plans checked each retained callable separately.
                continue;
            }
            self.check_const_library_call(node, solved, &mut allowances)?;
            let key = self.key(node);
            self.facts
                .node_mut(key)
                .ok_or_else(|| Declarations::error(self.report, key, "invalid call node"))?
                .operation = Some(crate::sema::model::Operation::Call {
                signature: solved.types[term.0 as usize],
                source_use: None,
            });
        }
        Ok(())
    }
}
#[cfg(test)]
#[path = "../../../tests/invariants/effect_snapshots.rs"]
mod tests;
