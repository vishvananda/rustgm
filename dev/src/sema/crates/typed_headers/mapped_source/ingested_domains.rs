//! Version nine keeps body-owned recipes dormant in the existing typed cursor.
use super::{Context, Declarations, Error, Headers, error};
use crate::{
    resolve::{
        body_origin::ImportedDeclarationId,
        crates::{ReservationWork, unpaid_provided::RecipeView},
    },
    sema::crates::typed_headers::{
        recipe_cursor::{Availability, Cursor, Demand, Maps, MemoReader, Port, Recorded, Reserved},
        unpaid_provided::{OrdinaryDomain, OrdinaryRecords, ordinary_requests},
    },
    support::crates::generics::{self as g, source as s},
    types::{RegionId, TypeId, arguments::Argument},
};
use std::{cell::RefCell, rc::Rc};

pub(super) struct Lazy {
    reader: MemoReader,
    maps: Rc<Maps>,
    writer: RefCell<Writer>,
}
struct Writer {
    cursor: Cursor,
    records: Box<OrdinaryRecords>,
}
impl Lazy {
    #[cfg(test)]
    pub(super) fn observed(&self) -> Result<(usize, usize, usize), Error> {
        let writer = self
            .writer
            .try_borrow()
            .map_err(|_| error("reentrant ordinary test observation"))?;
        let counters = writer.cursor.counters();
        Ok((writer.records.completed(), counters.expanded, counters.hits))
    }
    pub(super) fn prepare_body(
        &self,
        graph: &super::Graph,
        facts: &mut Declarations,
        definition: crate::resolve::model::DefId,
        work: &mut ReservationWork,
    ) -> Result<OrdinaryDomain, Error> {
        graph.require_store(&facts.types).map_err(error)?;
        let view = graph.recipe_view();
        let header = facts
            .crates
            .as_deref()
            .and_then(|linkage| linkage.originals.as_deref())
            .and_then(|headers| headers.get(definition))
            .ok_or_else(|| error("ordinary original lost its current installed header"))?;
        let mut writer = self
            .writer
            .try_borrow_mut()
            .map_err(|_| error("reentrant ordinary original preparation"))?;
        writer
            .cursor
            .require_reader(&self.reader, &view, &facts.types, work)?;
        let domain = writer
            .records
            .prepare(&view, facts, definition, header, work)?;
        if domain.environment().is_some() {
            return Ok(domain);
        }
        let raw = domain.raw()?;
        let rows = view.rows()?;
        let original = rows
            .originals
            .get(raw.0 as usize)
            .ok_or_else(|| error("ordinary original lost its current raw body"))?;
        let mut port = Recorded::ordinary(
            Reserved::new(view, &self.maps, &facts.types, work)?,
            &domain,
            &facts.lifetimes.callables,
        );
        work.charge(
            original
                .locals
                .len()
                .saturating_add(original.values.len().saturating_mul(2))
                .saturating_add(original.projections.len())
                .saturating_add(original.calls.len())
                .saturating_add(1),
        )?;
        // A Call's result type does not include its separate applied signature.
        // Reconstruct only this actual original's complete operation frontier.
        for root in original
            .locals
            .iter()
            .map(|local| local.ty)
            .chain(original.values.iter().flat_map(|value| {
                std::iter::once(value.ty).chain(match value.kind {
                    s::ValueKind::Call { signature, .. } => Some(signature),
                    _ => None,
                })
            }))
            .chain(original.projections.iter().map(|projection| projection.ty))
            .chain(original.calls.iter().map(|call| call.signature))
            .chain(std::iter::once(original.result))
        {
            if matches!(
                writer
                    .cursor
                    .demand(&mut port, &mut facts.types, root, work)?,
                Demand::Blocked(_)
            ) {
                return Err(error(
                    "ordinary original requires its actual typed recipe dependency",
                ));
            }
        }
        let super::Applications::Lazy { memo, reader } = &graph.applications else {
            return Err(error(
                "ordinary original lost its actual lazy application owner",
            ));
        };
        let view = graph.recipe_view();
        let mut applications = memo.writer(reader, &view, &facts.types, work)?;
        ordinary_requests(
            raw,
            port,
            &mut writer.cursor,
            &mut applications,
            &mut facts.types,
            work,
        )?;
        Ok(domain)
    }
    pub(super) fn body(
        &self,
        graph: &super::Graph,
        facts: &Declarations,
        definition: crate::resolve::model::DefId,
        header: &super::super::Header,
        work: &mut ReservationWork,
    ) -> Result<OrdinaryDomain, Error> {
        graph.require_store(&facts.types).map_err(error)?;
        let view = graph.recipe_view();
        let writer = self
            .writer
            .try_borrow()
            .map_err(|_| error("reentrant ordinary original lookup"))?;
        writer
            .cursor
            .require_reader(&self.reader, &view, &facts.types, work)?;
        writer.records.get(&view, facts, definition, header, work)
    }
    pub(super) fn prepare(
        context: &Context<'_>,
        facts: &mut Declarations,
        root: ImportedDeclarationId,
        budget: &mut Headers,
    ) -> Result<Self, Error> {
        let mut work = begin(budget);
        let result = Self::new(context, facts, root, &mut work);
        finish(budget, &work);
        result
    }
    fn new(
        context: &Context<'_>,
        facts: &mut Declarations,
        root: ImportedDeclarationId,
        work: &mut ReservationWork,
    ) -> Result<Self, Error> {
        let rows = context.descriptor.rows();
        let view = RecipeView::retained_parts(
            context.imported.dependencies(),
            context.provider_index,
            context.catalog,
            root,
        );
        work.charge(rows.types.len().saturating_mul(3).saturating_add(1))?;
        work.reserve(
            size_of::<Self>()
                .saturating_add(
                    Cursor::scratch_bytes(rows.types.len()).saturating_sub(size_of::<Cursor>()),
                )
                .saturating_add(Maps::allocation_bound(rows.slots.len(), rows.regions.len()))
                .saturating_add(
                    facts
                        .types
                        .node_allocation_bound(rows.types.len().saturating_add(rows.slots.len())),
                )
                .saturating_add(facts.types.parameter_allocation_bound(rows.slots.len())),
        )?;
        let mut cursor = Cursor::prepaid(
            rows.types.len(),
            Availability::DeclaredSignatures,
            facts.types.arena_identity(),
            root,
        );
        let maps = Maps::prepaid(&view, &facts.types)?;
        let records = Box::new(OrdinaryRecords::new(&view, work)?);
        let mut port = Reserved::new(view, &maps, &facts.types, work)?;
        work.charge(
            rows.slots
                .len()
                .saturating_add(rows.functions.len().saturating_mul(4))
                .saturating_add(rows.traits.len())
                .saturating_add(rows.implementations.len())
                .saturating_add(rows.clauses.len())
                .saturating_add(rows.roots.len())
                .saturating_add(rows.arguments.len()),
        )?;
        for (index, slot) in rows.slots.iter().enumerate() {
            if matches!(slot.kind, s::ParameterKind::Type { .. }) {
                port.slot(
                    &mut facts.types,
                    s::SlotRef(
                        u32::try_from(index)
                            .map_err(|_| error("ordinary slot ordinal overflow"))?,
                    ),
                    work,
                )?;
            }
        }
        for range in rows
            .functions
            .iter()
            .map(|header| header.regions.bounds)
            .chain(rows.traits.iter().map(|header| header.region_bounds))
            .chain(
                rows.implementations
                    .iter()
                    .map(|header| header.region_bounds),
            )
        {
            for bound in range.slice(&rows.region_bounds).map_err(error)? {
                for region in [bound.longer, bound.shorter] {
                    if matches!(port.region(region, work)?, Demand::Blocked(_)) {
                        return Err(error(
                            "ordinary header requires its actual declared lifetime",
                        ));
                    }
                }
            }
        }
        super::ingest_roots::prepare(rows, &mut |reference| match cursor.demand(
            &mut port,
            &mut facts.types,
            reference,
            work,
        )? {
            Demand::Ready(_) => Ok(()),
            Demand::Blocked(_) => Err(error(
                "ordinary header requires its actual declared recipe dependency",
            )),
        })?;
        let view = RecipeView::retained_parts(
            context.imported.dependencies(),
            context.provider_index,
            context.catalog,
            root,
        );
        let reader = cursor.reader(&view, &facts.types, work)?;
        Ok(Self {
            reader,
            maps,
            writer: RefCell::new(Writer { cursor, records }),
        })
    }
    pub(super) fn ty(&self, reference: g::TypeRef) -> Result<TypeId, Error> {
        self.writer
            .try_borrow()
            .map_err(|_| error("reentrant ordinary type memo lookup"))?
            .cursor
            .child(reference)
    }
    pub(super) fn slot(&self, reference: s::SlotRef) -> Result<Argument, Error> {
        self.maps
            .slot(reference.0 as usize)
            .map_err(error)?
            .ok_or_else(|| error("ordinary formal requires its actual pending declaration"))
    }
    pub(super) fn region(&self, reference: s::RegionRef) -> Result<RegionId, Error> {
        self.maps
            .region(reference.0 as usize)
            .map_err(error)?
            .ok_or_else(|| error("ordinary region requires its actual owning body domain"))
    }
}
pub(super) fn begin(budget: &Headers) -> ReservationWork {
    ReservationWork {
        steps: budget.work.get(),
        retained: budget.base.saturating_add(budget.bytes),
        peak: 0,
    }
}
pub(super) fn finish(budget: &mut Headers, work: &ReservationWork) {
    budget.work.set(budget.work.get().max(work.steps));
    budget.bytes = budget.bytes.max(work.retained.saturating_sub(budget.base));
}
