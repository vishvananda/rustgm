//! Borrow installed inputs from the same fixed private scope allocation.
use super::*;
use crate::{
    resolve::body_origin::{Catalog, ImportedDeclarationId},
    sema::generics::Generics,
    types::{Store, arena::Identity},
};

pub(in crate::sema::crates::typed_headers) struct Reader {
    scopes: Rc<Vec<OnceCell<Box<Scope>>>>,
    index: Rc<Vec<Option<usize>>>,
    root: ImportedDeclarationId,
    arena: Identity,
}
impl Owner {
    pub(super) fn scope_reader(
        &self,
        prepared: &Prepared,
        view: &RecipeView<'_>,
        store: &Store,
        work: &mut ReservationWork,
    ) -> Result<Reader, Error> {
        work.charge(3)?;
        let arena = store
            .existing_arena_identity()
            .ok_or_else(|| error("scope reader lost its actual current arena"))?;
        let result = Reader {
            scopes: Rc::clone(&self.scopes),
            index: Rc::clone(&prepared.scopes),
            root: view.root(),
            arena: arena.clone(),
        };
        result.require(view, store, work)?;
        Ok(result)
    }
    pub(super) fn require_scope_reader(
        &self,
        prepared: &Prepared,
        reader: &Reader,
        view: &RecipeView<'_>,
        store: &Store,
        work: &mut ReservationWork,
    ) -> Result<(), Error> {
        work.charge(2)?;
        if !Rc::ptr_eq(&self.scopes, &reader.scopes) || !Rc::ptr_eq(&prepared.scopes, &reader.index)
        {
            return Err(error(
                "scope reader changed its actual scope/index allocations",
            ));
        }
        reader.require(view, store, work)
    }
}
impl Reader {
    pub(in crate::sema::crates::typed_headers) fn require(
        &self,
        view: &RecipeView<'_>,
        store: &Store,
        work: &mut ReservationWork,
    ) -> Result<(), Error> {
        work.charge(4)?;
        let rows = view.rows()?;
        if self.root != view.root()
            || !store.has_identity(&self.arena)
            || self.scopes.len() != rows.scopes.len()
            || self.index.len() != rows.declarations.len()
        {
            return Err(error(
                "scope reader changed its actual root/arena/dense extents",
            ));
        }
        Ok(())
    }
    /// Data only: execution also requires the actual current Store/body checks.
    pub(in crate::sema::crates::typed_headers) fn inputs<'a>(
        &'a self,
        view: &RecipeView<'_>,
        catalog: &Catalog,
        generics: &Generics,
        owner: DeclarationRef,
        work: &mut ReservationWork,
    ) -> Result<(&'a Arc<[Input]>, usize), Error> {
        work.charge(8)?;
        if self.root != view.root() {
            return Err(error("scope inputs changed their actual catalog root"));
        }
        let rows = view.rows()?;
        let index = self
            .index
            .get(owner.0 as usize)
            .copied()
            .flatten()
            .ok_or_else(|| error("scope inputs require their actual declared scope index"))?;
        let raw = rows
            .scopes
            .get(index)
            .ok_or_else(|| error("scope input index is outside its actual rows"))?;
        let scope = self
            .scopes
            .get(index)
            .and_then(OnceCell::get)
            .map(Box::as_ref)
            .ok_or_else(|| {
                error("scope inputs require their actual pending declaration producer")
            })?;
        if raw.owner != owner || scope.raw != *raw || !scope.installed.get() {
            return Err(error("scope inputs require their actual installed owner"));
        }
        scope.require_data(rows, generics, Some(catalog))?;
        let definition = view.definition(owner, work)?;
        let parent = raw
            .parent
            .map(|parent| view.definition(parent, work))
            .transpose()?;
        if generics.parents.get(&definition).copied() != parent {
            return Err(error("scope inputs changed their actual normalized parent"));
        }
        if generics
            .environments
            .get(&definition)
            .is_none_or(|actual| !Arc::ptr_eq(actual, &scope.environment))
            || generics
                .parameters
                .get(&definition)
                .is_none_or(|actual| actual.len() != scope.declared.len())
        {
            return Err(error(
                "scope inputs changed their installed formal/environment relationship",
            ));
        }
        let prefix = match rows.header(owner).map_err(error)? {
            Some(s::HeaderRef::Function(index)) => {
                rows.functions[index as usize].declared_inputs as usize
            }
            Some(s::HeaderRef::Trait(index)) => {
                rows.traits[index as usize].declared_inputs as usize
            }
            Some(s::HeaderRef::Implementation(index)) => {
                rows.implementations[index as usize].declared_inputs as usize
            }
            None => {
                let (range, _) = originals::input_range(rows, *raw)?;
                if range.start != range.end {
                    return Err(error(
                        "Self scope inputs changed their actual empty construction",
                    ));
                }
                0
            }
        };
        let all = scope.inputs.all();
        if prefix > all.len() {
            return Err(error("scope inputs truncated their actual declared prefix"));
        }
        Ok((all, prefix))
    }
}
