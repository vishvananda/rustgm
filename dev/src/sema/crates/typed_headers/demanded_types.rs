//! One persistent iterative type memo serves eager and privately demanded roots.
use super::{Declarations, Error, Headers, error, signature_shapes};
use crate::{
    resolve::{
        body_origin::{ImportedBodyId, ImportedDeclarationId},
        crates::ReservationWork,
        model::DefId,
    },
    support::crates::{
        generics::{self as g, source as s},
        owner_catalog::DeclarationRef,
    },
    types::{Kind, RegionId, Store, TypeId, arena::Identity, arguments::Argument},
};

#[path = "demanded_ports.rs"]
mod reserved;
pub(super) use reserved::{Maps, Reserved};
mod sealed {
    pub trait Sealed {}
}
use super::mapped_source::Context;
struct Ordinary<'a, 'rows> {
    context: &'a Context<'rows>,
    slots: &'a [Argument],
    regions: &'a [RegionId],
    root: ImportedDeclarationId,
}
impl sealed::Sealed for Ordinary<'_, '_> {}
impl<'rows> Port<'rows, Headers> for Ordinary<'_, 'rows> {
    fn rows(&self) -> s::ValidatedView<'rows> {
        self.context.descriptor.rows()
    }
    fn definition(&self, row: DeclarationRef, budget: &mut Headers) -> Result<DefId, Error> {
        self.context.definition(row, budget)
    }
    fn slot(&mut self, _: &mut Store, row: s::SlotRef, _: &mut Headers) -> Result<Argument, Error> {
        self.slots
            .get(row.0 as usize)
            .copied()
            .ok_or_else(|| error("source type parameter changed its exact canonical slot kind"))
    }
    fn region(&mut self, row: s::RegionRef, _: &mut Headers) -> Result<Demand<RegionId>, Error> {
        self.regions
            .get(row.0 as usize)
            .copied()
            .map(Demand::Ready)
            .ok_or_else(|| error("source reference lost its actual regional remap"))
    }
    fn application(
        &self,
        _: s::FunctionApplicationRef,
        _: &mut Headers,
    ) -> Result<Dependency, Error> {
        Err(error(
            "source callable application needs its wider explicit remap profile",
        ))
    }
    fn availability(&self) -> Availability {
        Availability::Functions
    }
    fn root(&self) -> ImportedDeclarationId {
        self.root
    }
}

pub(super) trait Budget {
    fn charge(&mut self, count: usize) -> Result<(), Error>;
    fn reserve(&mut self, bytes: usize) -> Result<(), Error>;
}
impl Budget for Headers {
    fn charge(&mut self, count: usize) -> Result<(), Error> {
        Headers::charge(self, count)
    }
    fn reserve(&mut self, bytes: usize) -> Result<(), Error> {
        Headers::reserve(self, bytes)
    }
}
impl Budget for ReservationWork {
    fn charge(&mut self, count: usize) -> Result<(), Error> {
        ReservationWork::charge(self, count)
    }
    fn reserve(&mut self, bytes: usize) -> Result<(), Error> {
        ReservationWork::reserve(self, bytes)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Dependency {
    OwnedBodyDomain(ImportedBodyId),
    CallableTemplate(ImportedDeclarationId),
    DeclaredLifetime(ImportedDeclarationId),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Demand<T> {
    Ready(T),
    Blocked(Dependency),
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Availability {
    // A future producer must introduce its own availability and explicitly
    // invalidate old blocked state; neither port can mutate capabilities now.
    Functions,
    DeclaredSignatures,
}

// Only the ordinary authenticated adapter and the owned-session adapter implement
// this module-private protocol. Neither receives arbitrary canonical type maps.
pub(super) trait Port<'rows, B: Budget>: sealed::Sealed {
    fn rows(&self) -> s::ValidatedView<'rows>;
    fn definition(&self, row: DeclarationRef, budget: &mut B) -> Result<DefId, Error>;
    fn slot(
        &mut self,
        store: &mut Store,
        row: s::SlotRef,
        budget: &mut B,
    ) -> Result<Argument, Error>;
    fn region(&mut self, row: s::RegionRef, budget: &mut B) -> Result<Demand<RegionId>, Error>;
    fn application(
        &self,
        row: s::FunctionApplicationRef,
        budget: &mut B,
    ) -> Result<Dependency, Error>;
    fn availability(&self) -> Availability;
    fn root(&self) -> ImportedDeclarationId;
}
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub(super) struct Counters {
    pub expanded: usize,
    pub edges: usize,
    pub hits: usize,
}
pub(super) struct Cursor {
    nodes: Vec<Option<TypeId>>,
    visiting: Vec<bool>,
    children: Vec<usize>,
    stack: Vec<usize>,
    blocked: Option<Dependency>,
    availability: Availability,
    arena: Identity,
    root: ImportedDeclarationId,
    counters: Counters,
}
impl Cursor {
    /// The ordinary Graph producer already charges these exact vectors and the
    /// type-node envelope. The private owner charges scratch_bytes before entry.
    pub(super) fn prepaid(
        count: usize,
        availability: Availability,
        arena: Identity,
        root: ImportedDeclarationId,
    ) -> Self {
        Self {
            nodes: vec![None; count],
            visiting: vec![false; count],
            children: vec![0; count],
            stack: Vec::with_capacity(count),
            blocked: None,
            availability,
            arena,
            root,
            counters: Counters::default(),
        }
    }
    pub(super) fn scratch_bytes(count: usize) -> usize {
        count
            .saturating_mul(
                size_of::<Option<TypeId>>() + size_of::<bool>() + 2 * size_of::<usize>(),
            )
            .saturating_add(size_of::<Self>())
    }
    #[cfg(test)]
    pub(super) fn counters(&self) -> Counters {
        self.counters
    }
    pub(super) fn blocked(&self) -> Option<Dependency> {
        self.blocked
    }
    pub(super) fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            + crate::support::retained::vector(&self.nodes)
            + crate::support::retained::vector(&self.visiting)
            + crate::support::retained::vector(&self.children)
            + crate::support::retained::vector(&self.stack)
    }
    pub(super) fn require_owner<'rows, B: Budget, P: Port<'rows, B>>(
        &self,
        port: &P,
        store: &Store,
        budget: &mut B,
    ) -> Result<(), Error> {
        budget.charge(1)?;
        if !store.has_identity(&self.arena) || self.root != port.root() {
            return Err(error(
                "source recipe memo belongs to another actual catalog or arena",
            ));
        }
        if port.availability() != self.availability {
            return Err(error(
                "recipe availability changed without explicit memo invalidation",
            ));
        }
        Ok(())
    }
    pub(super) fn demand<'rows, B: Budget, P: Port<'rows, B>>(
        &mut self,
        port: &mut P,
        store: &mut Store,
        root: g::TypeRef,
        budget: &mut B,
    ) -> Result<Demand<TypeId>, Error> {
        self.require_owner(port, store, budget)?;
        let slot = self
            .nodes
            .get(root.0 as usize)
            .ok_or_else(|| error("source recipe has an unknown demanded root"))?;
        if let Some(ready) = slot {
            self.counters.hits = self.counters.hits.saturating_add(1);
            return Ok(Demand::Ready(*ready));
        }
        if let Some(dependency) = self.blocked {
            return Ok(Demand::Blocked(dependency));
        }
        self.stack.push(root.0 as usize);
        while let Some(&index) = self.stack.last() {
            budget.charge(1)?;
            let row = *port
                .rows()
                .types
                .get(index)
                .ok_or_else(|| error("source recipe refers outside its actual type graph"))?;
            if !self.visiting[index] {
                self.visiting[index] = true;
                self.counters.expanded = self.counters.expanded.saturating_add(1);
            }
            if let s::Type::FunctionItem(application) | s::Type::ApplicationSignature(application) =
                row
            {
                let dependency = port.application(application, budget)?;
                return Ok(self.block(dependency));
            }
            let child = next(row, self.children[index], port.rows())?;
            self.children[index] = self.children[index]
                .checked_add(1)
                .ok_or_else(|| error("source recipe cursor overflow"))?;
            if let Some(child) = child {
                self.counters.edges = self.counters.edges.saturating_add(1);
                let child = child.0 as usize;
                let ready = self
                    .nodes
                    .get(child)
                    .ok_or_else(|| error("source recipe has an unknown actual type dependency"))?;
                if ready.is_some() {
                    continue;
                }
                if self.visiting[child] {
                    return Err(error("source canonical type recipes contain a cycle"));
                }
                self.stack.push(child);
            } else {
                match self.intern(port, store, row, budget)? {
                    Demand::Ready(mapped) => {
                        self.nodes[index] = Some(mapped);
                        self.visiting[index] = false;
                        self.stack.pop();
                    }
                    Demand::Blocked(dependency) => return Ok(self.block(dependency)),
                }
            }
        }
        self.nodes[root.0 as usize]
            .map(Demand::Ready)
            .ok_or_else(|| error("source demanded recipe did not complete"))
    }
    fn block(&mut self, dependency: Dependency) -> Demand<TypeId> {
        self.blocked = Some(dependency);
        Demand::Blocked(dependency)
    }
    pub(super) fn into_complete(self, budget: &mut impl Budget) -> Result<Vec<TypeId>, Error> {
        // The final dense vector may coexist with the optional memo during
        // transfer. Do not rely on a standard-library in-place optimization.
        budget.charge(self.nodes.len())?;
        budget.reserve(self.nodes.len().saturating_mul(4 * size_of::<TypeId>()))?;
        self.nodes
            .into_iter()
            .map(|node| node.ok_or_else(|| error("source canonical graph is incomplete")))
            .collect()
    }
    fn child(&self, reference: g::TypeRef) -> Result<TypeId, Error> {
        self.nodes
            .get(reference.0 as usize)
            .copied()
            .flatten()
            .ok_or_else(|| error("source type recipe has an unresolved actual dependency"))
    }
    fn intern<'rows, B: Budget, P: Port<'rows, B>>(
        &self,
        port: &mut P,
        store: &mut Store,
        row: s::Type,
        budget: &mut B,
    ) -> Result<Demand<TypeId>, Error> {
        let kind = match row {
            s::Type::Scalar(scalar) => return Ok(Demand::Ready(super::super::scalar(scalar))),
            s::Type::Parameter(slot) => {
                return match port.slot(store, slot, budget)? {
                    Argument::Type(ty) => Ok(Demand::Ready(ty)),
                    _ => Err(error(
                        "source type parameter changed its exact canonical slot kind",
                    )),
                };
            }
            s::Type::Reference {
                mutable,
                region,
                referent,
            } => {
                let region = match port.region(region, budget)? {
                    Demand::Ready(region) => region,
                    Demand::Blocked(dependency) => return Ok(Demand::Blocked(dependency)),
                };
                Kind::Reference {
                    mutable,
                    region,
                    referent: self.child(referent)?,
                }
            }
            s::Type::Function(signature) => {
                signature_shapes::check(port.rows(), None, signature)?;
                let frame = port.rows().binders[signature.binder.0 as usize];
                port.definition(frame.owner, budget)?;
                let rows = port.rows();
                let roots = signature.parameters.slice(&rows.roots).map_err(error)?;
                budget.charge(roots.len().saturating_add(1))?;
                budget.reserve(roots.len().saturating_mul(4 * size_of::<TypeId>()))?;
                let parameters = roots
                    .iter()
                    .map(|&root| self.child(root))
                    .collect::<Result<Vec<_>, _>>()?;
                return signature_shapes::intern(
                    store,
                    frame.count,
                    signature,
                    &parameters,
                    self.child(signature.result)?,
                )
                .map(Demand::Ready);
            }
            s::Type::FunctionItem(_) | s::Type::ApplicationSignature(_) => {
                return Err(error(
                    "source application recipe escaped its explicit profile",
                ));
            }
        };
        store.intern(kind).map(Demand::Ready).map_err(error)
    }
}
fn next(
    row: s::Type,
    cursor: usize,
    rows: s::ValidatedView<'_>,
) -> Result<Option<g::TypeRef>, Error> {
    Ok(match row {
        s::Type::Scalar(_) | s::Type::Parameter(_) => None,
        s::Type::Reference { referent, .. } => (cursor == 0).then_some(referent),
        s::Type::Function(signature) => {
            let parameters = signature.parameters.slice(&rows.roots).map_err(error)?;
            if cursor == parameters.len() {
                Some(signature.result)
            } else {
                parameters.get(cursor).copied()
            }
        }
        s::Type::FunctionItem(_) | s::Type::ApplicationSignature(_) => Err(error(
            "source callable application needs its wider explicit remap profile",
        ))?,
    })
}

pub(super) fn prepare_functions(
    context: &Context<'_>,
    facts: &mut Declarations,
    slots: &[Argument],
    regions: &[RegionId],
    root: ImportedDeclarationId,
    budget: &mut Headers,
) -> Result<Vec<TypeId>, Error> {
    context.require_functions(budget)?;
    // Graph::prepare already reserves these exact memo/frontier arrays and the
    // complete node envelope. Do not pay a second time for the same allocation.
    budget.charge(context.descriptor.rows().types.len().saturating_mul(3))?;
    let mut cursor = Cursor::prepaid(
        context.descriptor.rows().types.len(),
        Availability::Functions,
        facts.types.arena_identity(),
        root,
    );
    let mut port = Ordinary {
        context,
        slots,
        regions,
        root,
    };
    for index in 0..context.descriptor.rows().types.len() {
        let root = g::TypeRef(
            u32::try_from(index).map_err(|_| error("source type recipe ordinal overflow"))?,
        );
        if matches!(
            cursor.demand(&mut port, &mut facts.types, root, budget)?,
            Demand::Blocked(_)
        ) {
            return Err(error(
                "ordinary source recipe unexpectedly requires a wider dependency",
            ));
        }
    }
    cursor.into_complete(budget)
}
