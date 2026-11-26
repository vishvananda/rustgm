//! Route checked rows by their defining artifact in the sealed resolver catalog.
use super::{DefId, Graph};
use crate::{
    resolve::body_origin::{BodyOrigin, ImportedBodyId},
    sema::model::Declarations,
    support::{crates::CrateIdentity, hash::HashMap, retained},
    types::{Store, closure::ClosureId},
};
use std::rc::Rc;

pub(in crate::sema) struct Graphs {
    first: Rc<Graph>,
    // The common single-provider path needs neither a lookup nor a second table.
    remaining: Vec<Rc<Graph>>,
    positions: HashMap<CrateIdentity, usize>,
    owners: usize,
}

impl Graph {
    fn artifact_identity(&self) -> &CrateIdentity {
        self.inventory.artifacts()[self.provider_index as usize].identity()
    }
}

impl Graphs {
    pub(in crate::sema::crates::typed_headers) fn new(
        graphs: Vec<Rc<Graph>>,
        store: &Store,
        budget: &mut super::Headers,
    ) -> Result<Self, super::Error> {
        use super::error;
        let count = graphs.len();
        budget.charge(count.saturating_mul(4))?;
        let mut graphs = graphs.into_iter();
        let first = graphs
            .next()
            .ok_or_else(|| error("typed crate inventory is empty"))?;
        first.require_store(store).map_err(error)?;
        let mut owners = first.associated_count().map_err(error)?;
        let mut positions = HashMap::default();
        let mut remaining = Vec::new();
        if count > 1 {
            budget.reserve(
                retained::new_map::<CrateIdentity, usize>(count)
                    .saturating_add(count.saturating_mul(size_of::<Rc<Graph>>())),
            )?;
            positions.reserve(count);
            remaining.reserve_exact(count - 1);
            for graph in graphs {
                graph.require_store(store).map_err(error)?;
                let descriptor = graph.descriptor(&first.origins).map_err(error)?;
                if !descriptor.profile().bootstrap_bindings() {
                    return Err(error(
                        "multiple typed crates require ordinary compiler bindings",
                    ));
                }
                if descriptor.identity() == first.artifact_identity()
                    || positions
                        .insert(descriptor.identity().clone(), remaining.len())
                        .is_some()
                {
                    return Err(error("typed crate inventory repeated an artifact identity"));
                }
                owners = owners.saturating_add(graph.associated_count().map_err(error)?);
                remaining.push(graph);
            }
        }
        budget.reserve(size_of::<Self>() + 2 * size_of::<usize>())?;
        budget
            .observed
            .add(size_of::<Self>() + 2 * size_of::<usize>());
        budget.observed.map(&positions);
        budget.observed.vector(&remaining);
        Ok(Self {
            first,
            remaining,
            positions,
            owners,
        })
    }

    pub(in crate::sema) fn iter(&self) -> impl Iterator<Item = &Rc<Graph>> {
        std::iter::once(&self.first).chain(self.remaining.iter())
    }

    pub(in crate::sema) const fn associated_count(&self) -> usize {
        self.owners
    }

    pub(in crate::sema) fn require_store(&self, store: &Store) -> Result<(), &'static str> {
        self.first.require_store(store)
    }

    pub(in crate::sema::crates::typed_headers) fn contains(
        &self,
        graph: &Graph,
    ) -> Result<bool, &'static str> {
        Ok(std::ptr::eq(
            self.by_identity(graph.artifact_identity())?.as_ref(),
            graph,
        ))
    }

    pub(in crate::sema::crates::typed_headers) fn by_identity(
        &self,
        identity: &CrateIdentity,
    ) -> Result<&Rc<Graph>, &'static str> {
        if self.first.artifact_identity() == identity {
            return Ok(&self.first);
        }
        self.positions
            .get(identity)
            .and_then(|&position| self.remaining.get(position))
            .ok_or("typed declaration has no graph for its defining artifact")
    }

    pub(in crate::sema) fn for_definition(
        &self,
        definition: DefId,
    ) -> Result<&Rc<Graph>, &'static str> {
        if self.remaining.is_empty() {
            return Ok(&self.first);
        }
        let catalog = &self.first.origins;
        // Local definitions have no imported rows. The per-graph query still
        // performs its own local-versus-imported classification.
        let Some(id) = catalog.declaration_if_imported(definition)? else {
            return Ok(&self.first);
        };
        let (identity, _, _) = catalog.declaration_coordinate(id)?;
        self.by_identity(identity)
    }

    pub(in crate::sema) fn feature_trait_implementation(
        &self,
        store: &Store,
        owner: crate::sema::associated::Owner,
    ) -> Result<Option<crate::types::parameters::predicates::Predicate>, &'static str> {
        self.for_definition(owner.self_definition)?
            .feature_trait_implementation(store, owner)
    }

    pub(in crate::sema::crates::typed_headers) fn for_body(
        &self,
        body: ImportedBodyId,
    ) -> Result<&Rc<Graph>, &'static str> {
        if self.remaining.is_empty() {
            return Ok(&self.first);
        }
        let (identity, _, _) = self.first.origins.coordinate(body)?;
        self.by_identity(identity)
    }

    pub(in crate::sema::crates::typed_headers) fn for_closure(
        &self,
        facts: &Declarations,
        closure: ClosureId,
    ) -> Result<Option<&Rc<Graph>>, &'static str> {
        if self.remaining.is_empty() {
            return Ok(Some(&self.first));
        }
        match facts.closures.origin(closure) {
            Some(BodyOrigin::Imported(origin)) => self.for_body(origin.id()).map(Some),
            _ => Ok(None),
        }
    }
}
