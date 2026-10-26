//! Owned numbering records are private data, never a defining environment or proof.
use super::{Progress, Ready, Session};
use crate::{
    resolve::{
        body_origin::{ImportedBodyId, ImportedDeclarationId, ProofSite},
        crates::{ReservationWork, unpaid_provided::RecipeView},
        package::Error,
    },
    sema::{
        crates::typed_headers::{error, recipe_cursor::Demand},
        numbered_regions::record_next,
    },
    support::{
        crates::{
            generics::{self as g, source as s},
            owner_catalog::{BodyRef, DeclarationRef},
        },
        retained::{self, Growth},
    },
    types::{Kind, RegionId, Store, arena::Identity},
};
use std::sync::Arc;

#[path = "neutral_aliases.rs"]
mod neutral_finish;
pub(super) use neutral_finish::{Completed, Key, Request};

#[cfg(test)]
#[path = "../../../../tests/invariants/recorded_domains.rs"]
pub(super) mod tests;

struct Seal {
    root: ImportedDeclarationId,
    body: ImportedBodyId,
    raw: BodyRef,
    expected: Ready,
    arena: Identity,
}
struct DomainStage {
    seal: Seal,
    variables: usize,
    aliases: Option<usize>,
}
struct Allocation {
    slot: u32,
    id: RegionId,
    site: ProofSite,
}
struct AllocationLog {
    ordered: Vec<Allocation>,
    by_slot: Vec<Option<RegionId>>,
}
struct RecordedDomain {
    seal: Seal,
    log: AllocationLog,
    aliases: Option<Vec<RegionId>>,
}
impl DomainStage {
    fn new(
        view: &RecipeView<'_>,
        store: &Store,
        expected: &Ready,
        work: &mut ReservationWork,
    ) -> Result<Self, Error> {
        work.charge(12)?;
        let rows = view.rows()?;
        if rows.functions.get(expected.pair.function_row as usize) != Some(&expected.header)
            || rows.scopes.get(expected.scope.0) != Some(&expected.scope.1)
            || store.kind(expected.pair.original) != Some(expected.original)
            || store.kind(expected.pair.callable) != Some(expected.callable)
            || view.declaration(expected.header.declaration, work)? != expected.pair.declaration
        {
            return Err(error(
                "recorded domain changed its actual callable or scope",
            ));
        }
        let s::FunctionBody::Original(raw) = expected.header.body else {
            return Err(error("recorded domain requires its actual original"));
        };
        let body = view.body(raw, work)?;
        if expected.pair.body != Some(body) {
            return Err(error("recorded domain changed its actual Catalog body"));
        }
        let original = rows
            .originals
            .get(raw.0 as usize)
            .ok_or_else(|| error("recorded domain lost its actual original"))?;
        let input = rows
            .body_inputs
            .get(raw.0 as usize)
            .ok_or_else(|| error("recorded domain lost its actual inputs"))?;
        let unit = rows
            .body_units
            .get(raw.0 as usize)
            .ok_or_else(|| error("recorded domain lost its actual numbering unit"))?;
        let frame = rows
            .binders
            .get(expected.header.original.binder.0 as usize)
            .ok_or_else(|| error("recorded domain lost its actual original binder"))?;
        let Kind::Function { bound_regions, .. } = expected.original else {
            return Err(error(
                "recorded domain lost its canonical function signature",
            ));
        };
        if original.body != raw
            || original.declaration != expected.header.declaration
            || input.body != raw
            || input.frame != expected.header.original.binder
            || frame.owner != expected.header.declaration
            || frame.kind != s::BinderKind::OriginalFunction
            || frame.count != bound_regions
            || unit.body != raw
            || unit.universals != frame.count
            || unit.universals != expected.header.regions.total
            || unit.allocation_order.len() != original.variables.len()
            || unit
                .aliases
                .as_ref()
                .is_some_and(|aliases| aliases.len() != unit.universals as usize)
        {
            return Err(error(
                "recorded domain changed its complete original/binder/allocation identity",
            ));
        }
        let arena = store
            .existing_arena_identity()
            .ok_or_else(|| error("recorded domain has no actual type arena"))?
            .clone();
        Ok(Self {
            seal: Seal {
                root: view.root(),
                body,
                raw,
                expected: *expected,
                arena,
            },
            variables: original.variables.len(),
            aliases: unit.aliases.as_ref().map(Vec::len),
        })
    }
    fn record(
        self,
        view: &RecipeView<'_>,
        work: &mut ReservationWork,
    ) -> Result<RecordedDomain, Error> {
        let before = work.retained;
        let rows = view.rows()?;
        let n = self.variables;
        let u = self.aliases.unwrap_or(0);
        let records = Growth::vector::<Allocation>(0, 0, n);
        let slots = Growth::vector::<Option<RegionId>>(0, 0, n);
        let aliases = Growth::vector::<RegionId>(0, 0, u);
        let persistent = size_of::<RecordedDomain>()
            .saturating_add(2 * size_of::<usize>())
            .saturating_add(records.bytes)
            .saturating_add(slots.bytes)
            .saturating_add(aliases.bytes);
        let inverse = s::AllocationOrdinals::allocation_bound(n);
        work.charge(n.saturating_mul(3).saturating_add(u).saturating_add(1))?;
        work.reserve(persistent.saturating_add(inverse))?;
        let order = metadata(work, |budget| {
            rows.allocation_ordinals(self.seal.raw, before.saturating_add(persistent), 0, budget)
        })?;
        let log = allocate(view, self.seal.raw, n, work)?;
        let aliases = self
            .aliases
            .map(|count| {
                metadata(work, |budget| {
                    let mut aliases = Vec::with_capacity(count);
                    for universal in 0..count {
                        let ordinal = order
                            .alias(
                                u32::try_from(universal)
                                    .map_err(|_| "recorded alias index overflow")?,
                                budget,
                            )?
                            .ok_or("recorded alias lost its actual present ledger")?;
                        let allocation = log
                            .ordered
                            .get(ordinal as usize)
                            .ok_or("recorded alias is outside its actual allocation order")?;
                        aliases.push(allocation.id);
                    }
                    Ok(aliases)
                })
            })
            .transpose()?;
        drop(order);
        let result = RecordedDomain {
            seal: self.seal,
            log,
            aliases,
        };
        let actual = result.retained_bytes();
        if actual > work.retained.saturating_sub(before) {
            return Err(error("recorded domain exceeded its selected prepayment"));
        }
        work.retained = before.saturating_add(actual);
        Ok(result)
    }
}
fn allocate(
    view: &RecipeView<'_>,
    body: BodyRef,
    n: usize,
    work: &mut ReservationWork,
) -> Result<AllocationLog, Error> {
    let rows = view.rows()?;
    let mut allocations = Vec::with_capacity(n);
    let mut by_slot = Vec::with_capacity(n);
    by_slot.resize_with(n, || None);
    let original = &rows.originals[body.0 as usize];
    let unit = &rows.body_units[body.0 as usize];
    for &slot in &unit.allocation_order {
        let raw = original
            .variables
            .get(slot as usize)
            .ok_or_else(|| error("recorded allocation lost its checked raw slot"))?;
        let site = view.body_site(body, raw.site, work)?;
        record_next(allocations.len(), |id| {
            allocations.push(Allocation { slot, id, site });
            Ok(())
        })
        .map_err(error)?;
        let actual = allocations
            .last()
            .ok_or_else(|| error("recorded allocation lost its real appended cell"))?;
        if !matches!(actual.site, ProofSite::Imported(_)) {
            return Err(error(
                "recorded allocation cannot acquire source syntax provenance",
            ));
        }
        let mapped = by_slot
            .get_mut(actual.slot as usize)
            .ok_or_else(|| error("recorded allocation is outside its actual slot domain"))?;
        if mapped.replace(actual.id).is_some() {
            return Err(error("recorded allocation repeated its actual raw slot"));
        }
    }
    Ok(AllocationLog {
        ordered: allocations,
        by_slot,
    })
}
fn metadata<T>(
    work: &mut ReservationWork,
    action: impl FnOnce(&mut g::Work) -> Result<T, &'static str>,
) -> Result<T, Error> {
    let before = work.steps;
    let mut budget = g::Work {
        steps: before,
        ..g::Work::default()
    };
    let result = action(&mut budget);
    work.charge(budget.steps.saturating_sub(before))?;
    result.map_err(error)
}
impl RecordedDomain {
    fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            .saturating_add(2 * size_of::<usize>())
            .saturating_add(retained::vector(&self.log.ordered))
            .saturating_add(retained::vector(&self.log.by_slot))
            .saturating_add(self.aliases.as_ref().map_or(0, retained::vector))
    }
    fn require(
        &self,
        stage: &DomainStage,
        store: &Store,
        work: &mut ReservationWork,
    ) -> Result<(), Error> {
        work.charge(3)?;
        if !store.has_identity(&self.seal.arena)
            || self.seal.root != stage.seal.root
            || self.seal.body != stage.seal.body
            || self.seal.raw != stage.seal.raw
            || self.seal.expected != stage.seal.expected
            || self.log.ordered.len() != stage.variables
            || self.log.by_slot.len() != stage.variables
            || self.aliases.as_ref().map(Vec::len) != stage.aliases
        {
            return Err(error(
                "recorded domain belongs to another actual body, arena or signature",
            ));
        }
        Ok(())
    }
    #[cfg(test)]
    fn region(
        &self,
        view: &RecipeView<'_>,
        store: &Store,
        expected: &Ready,
        slot: u32,
        work: &mut ReservationWork,
    ) -> Result<RegionId, Error> {
        let stage = DomainStage::new(view, store, expected, work)?;
        self.require(&stage, store, work)?;
        work.charge(1)?;
        self.log
            .by_slot
            .get(slot as usize)
            .copied()
            .flatten()
            .ok_or_else(|| error("recorded domain has no actual allocated slot"))
    }
}
pub(super) struct Owner {
    entries: Vec<Option<Arc<RecordedDomain>>>,
    bytes: usize,
}
impl Owner {
    fn new(rows: s::ValidatedView<'_>, work: &mut ReservationWork) -> Result<Self, Error> {
        let before = work.retained;
        let count = rows.functions.len();
        let growth = Growth::vector::<Option<Arc<RecordedDomain>>>(0, 0, count);
        work.charge(count.saturating_add(growth.work))?;
        work.reserve(size_of::<Self>().saturating_add(growth.bytes))?;
        let mut entries = Vec::with_capacity(count);
        entries.resize_with(count, || None);
        let bytes = size_of::<Self>().saturating_add(retained::vector(&entries));
        work.retained = before.saturating_add(bytes);
        Ok(Self { entries, bytes })
    }
    pub(super) fn retained_bytes(&self) -> usize {
        self.bytes
    }
    fn complete(
        &mut self,
        view: &RecipeView<'_>,
        store: &Store,
        expected: &Ready,
        work: &mut ReservationWork,
    ) -> Result<(), Error> {
        let stage = DomainStage::new(view, store, expected, work)?;
        let cell = self
            .entries
            .get_mut(expected.pair.function_row as usize)
            .ok_or_else(|| error("recorded domain has no actual function cell"))?;
        if let Some(actual) = cell {
            return actual.require(&stage, store, work);
        }
        let recorded = stage.record(view, work)?;
        self.bytes = self.bytes.saturating_add(recorded.retained_bytes());
        *cell = Some(Arc::new(recorded));
        Ok(())
    }
}
impl Session {
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "private recorded numbering awaits reviewed defining/default and original/instance consumers; it grants no bound domain or public Provided permission"
        )
    )]
    pub(in crate::sema::crates::typed_headers::unpaid_provided) fn record_numbering(
        self: Box<Self>,
        owner: DeclarationRef,
        work: &mut ReservationWork,
    ) -> Result<Progress, Error> {
        let (mut session, status) = self.callable_template(owner, work)?;
        if status != Demand::Ready(()) {
            return Ok((session, status));
        }
        session.require_owner(work)?;
        let view = session
            .inputs
            .recipe_view(&session.inventory, session.provider_index, work)?;
        let prepared = session
            .recipes
            .as_mut()
            .and_then(|state| state.callables.as_mut())
            .ok_or_else(|| error("recording demand lost its actual named template"))?;
        let rows = view.rows()?;
        let Some(s::HeaderRef::Function(index)) = rows.header(owner).map_err(error)? else {
            return Err(error("recording demand lacks its actual function header"));
        };
        let expected = prepared
            .ready
            .get(index as usize)
            .copied()
            .flatten()
            .ok_or_else(|| error("recording demand lost its actual callable readiness"))?;
        prepared.require(&view, &session.facts.types, owner, expected.pair, work)?;
        if prepared.recordings.is_none() {
            prepared.recordings = Some(Box::new(Owner::new(rows, work)?));
        }
        prepared
            .recordings
            .as_mut()
            .ok_or_else(|| error("recording owner disappeared"))?
            .complete(&view, &session.facts.types, &expected, work)?;
        Ok((session, Demand::Ready(())))
    }
}

impl super::Prepared {
    pub(super) fn record_selected(
        &mut self,
        view: &RecipeView<'_>,
        store: &Store,
        expected: &Ready,
        work: &mut ReservationWork,
    ) -> Result<(), Error> {
        if self.recordings.is_none() {
            self.recordings = Some(Box::new(Owner::new(view.rows()?, work)?));
        }
        self.recordings
            .as_mut()
            .ok_or_else(|| error("recording owner disappeared"))?
            .complete(view, store, expected, work)
    }
}
