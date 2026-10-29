//! Indexed unsafe-call context is unpaid body data bound to the actual use.
use super::{
    CallContexts, Checker, Error, MAX_RETAINED, MAX_ROWS, MAX_TRANSIENT, SourceCallUse, charge,
};
use crate::{
    resolve::unit::NodeKey,
    sema::model::Declarations,
    support::{crates::generics::source::CallSafety, retained},
    syntax::ast::NodeId,
    types::{TypeId, infer::Solved},
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) struct Site {
    pub context: SourceCallUse,
    pub node: NodeId,
    pub signature: TypeId,
    pub kind: CallSafety,
}
impl Site {
    fn key(self) -> (u32, u32, u32) {
        (self.context.0.get(), self.node.get(), self.signature.0)
    }
}
#[derive(Default)]
pub(super) struct Rows {
    sites: Vec<Site>,
}
impl Rows {
    pub(super) fn bytes(&self) -> usize {
        size_of::<Self>().saturating_add(retained::vector(&self.sites))
    }
}

pub(in crate::sema::bodies) fn call_safety_scratch(
    facts: &Declarations,
    bytes: usize,
    count: usize,
) -> Result<(), &'static str> {
    let retained = facts
        .source_bindings
        .as_deref()
        .and_then(|catalog| catalog.call_contexts.as_deref())
        .map_or(Ok(size_of::<CallContexts>()), |contexts| {
            contexts.bytes(contexts.rows.capacity())
        })?;
    if count > MAX_ROWS || bytes > MAX_RETAINED || retained.saturating_add(bytes) > MAX_TRANSIENT {
        return Err("unsafe call scratch exceeds its actual source context envelope");
    }
    Ok(())
}
impl SourceCallUse {
    /// The caller charges this borrowed indexed lookup; absence supplies no unsafe permission.
    pub(crate) fn call_safety(
        self,
        facts: &Declarations,
        owner: NodeKey,
        node: NodeId,
        signature: TypeId,
    ) -> Result<CallSafety, &'static str> {
        self.effect_for(facts, owner)?;
        let rows = facts
            .source_bindings
            .as_deref()
            .and_then(|catalog| catalog.call_contexts.as_deref())
            .and_then(|contexts| contexts.safety.as_deref());
        let key = (self.0.get(), node.get(), signature.0);
        Ok(rows
            .and_then(|rows| {
                rows.sites
                    .binary_search_by_key(&key, |site| site.key())
                    .ok()
                    .map(|i| rows.sites[i].kind)
            })
            .unwrap_or(CallSafety::Safe))
    }
}

impl Checker<'_> {
    pub(in crate::sema::bodies::effect_facts) fn prepare_safety_sites(
        &mut self,
        solved: &Solved,
        context: Option<SourceCallUse>,
    ) -> Result<Vec<Site>, Error> {
        let Some(mut pending) = self.state.call_types.safety.take() else {
            return Ok(Vec::new());
        };
        let count = pending.rows.len();
        let logarithm = count.checked_ilog2().unwrap_or(0) as usize + 1;
        let work = count
            .saturating_mul(logarithm)
            .saturating_mul(2)
            .saturating_add(
                self.state
                    .calls
                    .len()
                    .saturating_mul(logarithm.saturating_add(1)),
            );
        charge(&mut self.state.steps, work).map_err(|e| self.error(self.state.body.node, e))?;
        pending.rows.sort_unstable_by_key(|row| row.node.get());
        if pending
            .rows
            .windows(2)
            .any(|p| p[0].node == p[1].node && p[0].kind != p[1].kind)
        {
            return Err(self.error(
                self.state.body.node,
                "unsafe call replay changed its lexical context",
            ));
        }
        pending.rows.dedup_by_key(|row| row.node);
        let context = context.ok_or_else(|| {
            self.error(
                self.state.body.node,
                "unsafe calls lost their actual body context",
            )
        })?;
        let growth = retained::Growth::vector::<Site>(0, 0, pending.rows.len());
        call_safety_scratch(
            self.facts,
            pending.bytes().saturating_add(growth.bytes),
            count,
        )
        .map_err(|e| self.error(self.state.body.node, e))?;
        charge(
            &mut self.state.steps,
            growth.work.saturating_add(pending.rows.len()),
        )
        .map_err(|e| self.error(self.state.body.node, e))?;
        let mut sites = Vec::with_capacity(pending.rows.len());
        if retained::vector(&sites) > growth.bytes {
            return Err(self.error(
                self.state.body.node,
                "unsafe call staging exceeded prepaid capacity",
            ));
        }
        for &(node, term) in &self.state.calls {
            let Ok(index) = pending
                .rows
                .binary_search_by_key(&node.get(), |row| row.node.get())
            else {
                continue;
            };
            let row = pending.rows[index];
            if row.kind == CallSafety::Safe {
                return Err(self.error(node, "unsafe call retained an absent lexical obligation"));
            }
            let signature = solved
                .types
                .get(term.0 as usize)
                .copied()
                .ok_or_else(|| self.error(node, "unsafe call lost its completed signature"))?;
            if sites.len() == sites.capacity() {
                return Err(self.error(node, "unsafe call request duplicated its actual node"));
            }
            sites.push(Site {
                context,
                node,
                signature,
                kind: row.kind,
            });
        }
        if sites.len() != pending.rows.len() {
            return Err(self.error(
                self.state.body.node,
                "unsafe call record has no completed call operation",
            ));
        }
        sites.sort_unstable_by_key(|site| site.key());
        self.state.call_types.safety = Some(pending);
        Ok(sites)
    }

    pub(in crate::sema::bodies::effect_facts) fn preflight_safety_sites(
        &mut self,
        context: Option<SourceCallUse>,
        sites: &[Site],
        warm: bool,
    ) -> Result<(), Error> {
        let Some(context) = context else {
            return Ok(());
        };
        let contexts = self
            .facts
            .source_bindings
            .as_deref_mut()
            .and_then(|catalog| catalog.call_contexts.as_deref_mut())
            .ok_or_else(|| {
                Error::unlocated("unsafe call publication lost its source context owner")
            })?;
        if sites.is_empty() && contexts.safety.is_none() {
            return Ok(());
        }
        if warm {
            let retained = contexts
                .safety
                .as_deref()
                .map_or(&[][..], |rows| rows.sites.as_slice());
            let start = retained.partition_point(|site| site.context.0 < context.0);
            let end = retained.partition_point(|site| site.context.0 <= context.0);
            charge(
                &mut self.state.steps,
                sites
                    .len()
                    .saturating_add(2 * (retained.len().checked_ilog2().unwrap_or(0) as usize + 1)),
            )
            .map_err(Error::unlocated)?;
            if retained[start..end] != *sites {
                return Err(Error::unlocated(
                    "unsafe call replay changed its actual signature or site inventory",
                ));
            }
            return Ok(());
        }
        if contexts
            .safety
            .as_deref()
            .and_then(|rows| rows.sites.last())
            .is_some_and(|last| last.context.0 >= context.0)
        {
            return Err(Error::unlocated(
                "unsafe call publication cannot replace another body context",
            ));
        }
        let growth = contexts
            .safety_growth(sites.len())
            .map_err(Error::unlocated)?;
        charge(
            &mut self.state.steps,
            growth.work.saturating_add(sites.len()),
        )
        .map_err(Error::unlocated)?;
        let scratch = self
            .state
            .call_types
            .safety
            .as_deref()
            .map_or(0, super::super::safety::Pending::bytes)
            .saturating_add(sites.len().saturating_mul(size_of::<Site>()));
        if contexts
            .bytes(contexts.rows.capacity())
            .map_err(Error::unlocated)?
            .saturating_add(growth.bytes)
            .saturating_add(scratch)
            > MAX_TRANSIENT
        {
            return Err(Error::unlocated(
                "unsafe call publication exceeds its transient storage budget",
            ));
        }
        contexts
            .reserve_safety(sites.len(), growth)
            .map_err(Error::unlocated)
    }

    pub(in crate::sema::bodies::effect_facts) fn commit_safety_sites(
        &mut self,
        sites: &[Site],
        warm: bool,
    ) {
        if !warm
            && !sites.is_empty()
            && let Some(rows) = self
                .facts
                .source_bindings
                .as_deref_mut()
                .and_then(|catalog| catalog.call_contexts.as_deref_mut())
                .and_then(|contexts| contexts.safety.as_deref_mut())
        {
            rows.sites.extend_from_slice(sites);
        }
    }
}

impl CallContexts {
    fn safety_growth(&self, count: usize) -> Result<retained::Growth, &'static str> {
        if count == 0 {
            return Ok(retained::Growth::default());
        }
        let (length, capacity) = self
            .safety
            .as_deref()
            .map_or((0, 0), |rows| (rows.sites.len(), rows.sites.capacity()));
        if length.saturating_add(count) > MAX_ROWS {
            return Err("unsafe call inventory exhausted");
        }
        let mut growth = retained::Growth::vector::<Site>(length, capacity, count);
        growth.bytes = growth
            .bytes
            .saturating_add(usize::from(self.safety.is_none()) * size_of::<Rows>());
        let current = self.bytes(self.rows.capacity())?;
        let projected = if growth.bytes == 0 {
            current
        } else {
            current
                .saturating_sub(capacity.saturating_mul(size_of::<Site>()))
                .saturating_add(growth.bytes)
        };
        if projected > MAX_RETAINED || current.saturating_add(growth.bytes) > MAX_TRANSIENT {
            return Err("unsafe call publication exceeds its retained storage budget");
        }
        Ok(growth)
    }
    fn reserve_safety(
        &mut self,
        count: usize,
        growth: retained::Growth,
    ) -> Result<(), &'static str> {
        if count == 0 {
            return Ok(());
        }
        let before = self.bytes(self.rows.capacity())?;
        let rows = self.safety.get_or_insert_with(Default::default);
        rows.sites
            .try_reserve(count)
            .map_err(|_| "unsafe call publication allocation failed")?;
        let after = self.bytes(self.rows.capacity())?;
        if after > MAX_RETAINED || after > before.saturating_add(growth.bytes) {
            return Err("unsafe call publication exceeded prepaid capacity");
        }
        Ok(())
    }
}
