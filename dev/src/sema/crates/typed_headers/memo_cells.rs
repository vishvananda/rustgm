//! A private writer shares its actual fixed memo; readers cannot publish values.
use super::{Budget, Error, Identity, ImportedDeclarationId, Store, TypeId, error, g};
use crate::resolve::crates::unpaid_provided::RecipeView;
use std::{cell::Cell, rc::Rc};

mod sealed {
    pub trait Sealed {}
}
pub(in crate::sema::crates::typed_headers) enum CellState {
    Missing,
    Pending,
    Ready(TypeId),
}
pub(in crate::sema::crates::typed_headers) trait Storage:
    sealed::Sealed
{
    fn get(&self, index: usize) -> CellState;
    fn publish(&mut self, index: usize, value: TypeId) -> Result<(), Error>;
    fn retained_bytes(&self) -> usize;
}
impl sealed::Sealed for Vec<Option<TypeId>> {}
impl Storage for Vec<Option<TypeId>> {
    fn get(&self, index: usize) -> CellState {
        match self.as_slice().get(index) {
            Some(Some(ty)) => CellState::Ready(*ty),
            Some(None) => CellState::Pending,
            None => CellState::Missing,
        }
    }
    fn publish(&mut self, index: usize, value: TypeId) -> Result<(), Error> {
        let cell = self
            .get_mut(index)
            .ok_or_else(|| error("source recipe publication has an unknown actual row"))?;
        *cell = Some(value);
        Ok(())
    }
    fn retained_bytes(&self) -> usize {
        crate::support::retained::vector(self)
    }
}

pub(in crate::sema::crates::typed_headers) struct Shared {
    nodes: Rc<Vec<Cell<Option<TypeId>>>>,
}
impl Shared {
    pub(super) fn prepaid(count: usize) -> Self {
        Self {
            nodes: Rc::new(vec![Cell::new(None); count]),
        }
    }
    pub(super) fn allocation_bound(count: usize) -> usize {
        count
            .saturating_mul(size_of::<Cell<Option<TypeId>>>())
            .saturating_add(size_of::<Vec<Cell<Option<TypeId>>>>())
            .saturating_add(2 * size_of::<usize>())
    }
    pub(super) fn require_reader(
        &self,
        reader: &Reader,
        budget: &mut impl Budget,
    ) -> Result<(), Error> {
        budget.charge(1)?;
        if !Rc::ptr_eq(&self.nodes, &reader.nodes) {
            return Err(error(
                "source memo reader changed its actual writer allocation",
            ));
        }
        Ok(())
    }
    pub(super) fn reader(
        &self,
        root: ImportedDeclarationId,
        arena: &Identity,
        budget: &mut impl Budget,
    ) -> Result<Reader, Error> {
        // Both reference-count operations share already prepaid allocations.
        budget.charge(2)?;
        Ok(Reader {
            nodes: Rc::clone(&self.nodes),
            root,
            arena: arena.clone(),
        })
    }
}
impl sealed::Sealed for Shared {}
impl Storage for Shared {
    fn get(&self, index: usize) -> CellState {
        match self.nodes.get(index).map(Cell::get) {
            Some(Some(ty)) => CellState::Ready(ty),
            Some(None) => CellState::Pending,
            None => CellState::Missing,
        }
    }
    fn publish(&mut self, index: usize, value: TypeId) -> Result<(), Error> {
        let cell = self
            .nodes
            .get(index)
            .ok_or_else(|| error("source shared recipe publication has an unknown actual row"))?;
        if cell.get().is_some() {
            return Err(error(
                "source shared recipe cannot replace a completed canonical row",
            ));
        }
        cell.set(Some(value));
        Ok(())
    }
    fn retained_bytes(&self) -> usize {
        size_of::<Vec<Cell<Option<TypeId>>>>()
            .saturating_add(2 * size_of::<usize>())
            .saturating_add(crate::support::retained::vector(self.nodes.as_ref()))
    }
}

/// The opaque private header mapping owns this handle. It is data correspondence,
/// never declaration, region, body or instance proof.
pub(in crate::sema::crates::typed_headers) struct Reader {
    nodes: Rc<Vec<Cell<Option<TypeId>>>>,
    root: ImportedDeclarationId,
    arena: Identity,
}
pub(in crate::sema::crates::typed_headers) enum Lookup {
    Ready(TypeId),
    Pending {
        owner: ImportedDeclarationId,
        root: g::TypeRef,
    },
}
impl Reader {
    pub(in crate::sema::crates::typed_headers) fn require<B: Budget>(
        &self,
        view: &RecipeView<'_>,
        store: &Store,
        budget: &mut B,
    ) -> Result<(), Error> {
        budget.charge(3)?;
        if self.root != view.root()
            || !store.has_identity(&self.arena)
            || self.nodes.len() != view.rows()?.types.len()
        {
            return Err(error(
                "source memo reader changed its actual catalog/arena/table owner",
            ));
        }
        Ok(())
    }
    pub(in crate::sema::crates::typed_headers) fn get(
        &self,
        view: &RecipeView<'_>,
        store: &Store,
        root: g::TypeRef,
        budget: &mut impl Budget,
    ) -> Result<Lookup, Error> {
        self.require(view, store, budget)?;
        budget.charge(1)?;
        Ok(
            match self
                .nodes
                .get(root.0 as usize)
                .map(Cell::get)
                .ok_or_else(|| error("source memo reader has an unknown actual type row"))?
            {
                Some(ty) => Lookup::Ready(ty),
                None => Lookup::Pending {
                    owner: self.root,
                    root,
                },
            },
        )
    }
}
