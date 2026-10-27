//! Ordinary Vec storage and the private fixed memo share canonical publication.
use super::super::super::{error, recipe_cursor::Budget};
use crate::{
    resolve::{
        body_origin::ImportedDeclarationId, crates::unpaid_provided::RecipeView, package::Error,
    },
    support::{crates::generics::source as s, retained},
    types::{Store, arena::Identity, parameters::predicates::Predicate},
};
use std::{cell::Cell, rc::Rc};
mod sealed {
    pub trait Sealed {}
}
pub(in crate::sema::crates::typed_headers) trait Cache:
    sealed::Sealed
{
    fn entry(&self, index: usize) -> Result<Option<Predicate>, Error>;
    fn publish(&mut self, index: usize, value: Predicate) -> Result<(), Error>;
}
impl sealed::Sealed for Vec<Option<Predicate>> {}
impl Cache for Vec<Option<Predicate>> {
    fn entry(&self, index: usize) -> Result<Option<Predicate>, Error> {
        self.get(index)
            .copied()
            .ok_or_else(|| error("source clause has an unknown trait application"))
    }
    fn publish(&mut self, index: usize, value: Predicate) -> Result<(), Error> {
        let cell = self
            .get_mut(index)
            .ok_or_else(|| error("source clause has an unknown trait application"))?;
        *cell = Some(value);
        Ok(())
    }
}
pub(in crate::sema::crates::typed_headers) struct Shared {
    cells: Rc<Vec<Cell<Option<Predicate>>>>,
}
impl sealed::Sealed for Shared {}
impl Cache for Shared {
    fn entry(&self, index: usize) -> Result<Option<Predicate>, Error> {
        self.cells
            .get(index)
            .map(Cell::get)
            .ok_or_else(|| error("source clause has an unknown trait application"))
    }
    fn publish(&mut self, index: usize, value: Predicate) -> Result<(), Error> {
        self.publish_value(index, value)
    }
}
impl Shared {
    fn publish_value(&self, index: usize, value: Predicate) -> Result<(), Error> {
        let cell = self
            .cells
            .get(index)
            .ok_or_else(|| error("private application publication has no actual slot"))?;
        if cell.get().is_some_and(|actual| actual != value) {
            return Err(error(
                "private application cannot replace a canonical predicate",
            ));
        }
        cell.set(Some(value));
        Ok(())
    }
    pub(in crate::sema::crates::typed_headers) fn writer<'a>(
        &'a self,
        reader: &Reader,
        view: &RecipeView<'_>,
        store: &Store,
        work: &mut impl Budget,
    ) -> Result<Writer<'a>, Error> {
        self.require_reader(reader, view, store, work)?;
        Ok(Writer { owner: self })
    }

    #[cfg(test)]
    pub(in crate::sema::crates::typed_headers) fn observations(&self) -> (usize, usize, usize) {
        (
            self.cells.capacity(),
            self.cells
                .iter()
                .filter(|cell| cell.get().is_some())
                .count(),
            self.retained_bytes(),
        )
    }
    pub(in crate::sema::crates::typed_headers) fn new(
        count: usize,
        work: &mut impl Budget,
    ) -> Result<Self, Error> {
        work.charge(count.saturating_add(1))?;
        work.reserve(
            size_of::<Vec<Cell<Option<Predicate>>>>()
                .saturating_add(2 * size_of::<usize>())
                .saturating_add(count.saturating_mul(size_of::<Cell<Option<Predicate>>>())),
        )?;
        let mut cells = Vec::with_capacity(count);
        cells.resize_with(count, || Cell::new(None));
        Ok(Self {
            cells: Rc::new(cells),
        })
    }
    pub(in crate::sema::crates::typed_headers) fn retained_bytes(&self) -> usize {
        size_of::<Vec<Cell<Option<Predicate>>>>()
            .saturating_add(2 * size_of::<usize>())
            .saturating_add(retained::vector(self.cells.as_ref()))
    }
    pub(in crate::sema::crates::typed_headers) fn reader(
        &self,
        view: &RecipeView<'_>,
        store: &Store,
        work: &mut impl Budget,
    ) -> Result<Reader, Error> {
        work.charge(2)?;
        let arena = store
            .existing_arena_identity()
            .ok_or_else(|| error("private application reader lost its actual current arena"))?;
        let reader = Reader {
            cells: Rc::clone(&self.cells),
            root: view.root(),
            arena: arena.clone(),
        };
        reader.require(view, store, work)?;
        Ok(reader)
    }
    pub(in crate::sema::crates::typed_headers) fn require_reader(
        &self,
        reader: &Reader,
        view: &RecipeView<'_>,
        store: &Store,
        work: &mut impl Budget,
    ) -> Result<(), Error> {
        work.charge(1)?;
        if !Rc::ptr_eq(&self.cells, &reader.cells) {
            return Err(error(
                "private applications changed their actual memo allocation",
            ));
        }
        reader.require(view, store, work)
    }
}
/// A scoped writer borrows the existing owner; readers cannot construct one.
pub(in crate::sema::crates::typed_headers) struct Writer<'a> {
    owner: &'a Shared,
}
impl sealed::Sealed for Writer<'_> {}
impl Cache for Writer<'_> {
    fn entry(&self, index: usize) -> Result<Option<Predicate>, Error> {
        self.owner.entry(index)
    }
    fn publish(&mut self, index: usize, value: Predicate) -> Result<(), Error> {
        self.owner.publish_value(index, value)
    }
}
pub(in crate::sema::crates::typed_headers) struct Reader {
    cells: Rc<Vec<Cell<Option<Predicate>>>>,
    root: ImportedDeclarationId,
    arena: Identity,
}
impl Reader {
    pub(in crate::sema::crates::typed_headers) fn require(
        &self,
        view: &RecipeView<'_>,
        store: &Store,
        work: &mut impl Budget,
    ) -> Result<(), Error> {
        work.charge(3)?;
        if self.root != view.root()
            || !store.has_identity(&self.arena)
            || self.cells.len() != view.rows()?.trait_applications.len()
        {
            return Err(error(
                "private applications changed their actual provider/arena/table",
            ));
        }
        Ok(())
    }
    pub(in crate::sema::crates::typed_headers) fn get(
        &self,
        view: &RecipeView<'_>,
        store: &Store,
        reference: s::TraitApplicationRef,
        work: &mut impl Budget,
    ) -> Result<Option<Predicate>, Error> {
        self.require(view, store, work)?;
        work.charge(1)?;
        self.cells
            .get(reference.0 as usize)
            .map(Cell::get)
            .ok_or_else(|| error("private application read has no actual slot"))
    }
}
