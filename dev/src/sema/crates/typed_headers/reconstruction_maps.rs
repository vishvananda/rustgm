//! Original reconstruction reads the same selected memo and canonical applications.
use super::{ProvidedOwners, ReservationWork, Store, s};
use crate::{
    sema::crates::typed_headers::recipe_cursor::MemoLookup,
    support::crates::generics::TypeRef,
    types::{RegionId, TypeId, arguments::Argument, parameters::predicates::Predicate},
};
impl ProvidedOwners {
    pub(in crate::sema::crates::typed_headers) fn original_type(
        &self,
        store: &Store,
        root: TypeRef,
        steps: &mut usize,
    ) -> Result<TypeId, &'static str> {
        let mut work = ReservationWork::default();
        let result = (|| {
            self.require_current(store, &mut work)?;
            match self
                .reader
                .get(&self.view(), store, root, &mut work)
                .map_err(|_| "selected original changed its actual type memo")?
            {
                MemoLookup::Ready(ty) => Ok(ty),
                MemoLookup::Pending { .. } => {
                    Err("selected original requires its actual pending type root")
                }
            }
        })();
        *steps = steps.saturating_add(work.steps);
        if *steps > crate::support::crates::MAX_COMPONENTS {
            return Err("selected original type work budget exceeded");
        }
        result
    }
    pub(in crate::sema::crates::typed_headers) fn original_slot(
        &self,
        reference: s::SlotRef,
    ) -> Result<Argument, &'static str> {
        self.maps
            .slot(reference.0 as usize)?
            .ok_or("selected original requires its actual pending formal")
    }
    pub(in crate::sema::crates::typed_headers) fn original_region(
        &self,
        store: &Store,
        reference: s::RegionRef,
        steps: &mut usize,
    ) -> Result<RegionId, &'static str> {
        *steps = steps.saturating_add(1);
        self.require_store(store)?;
        if !matches!(
            self.descriptor()?.rows().regions.get(reference.0 as usize),
            Some(s::Region::Static | s::Region::Bound { .. })
        ) {
            return Err("selected original requires its actual body-owned regional mapping");
        }
        self.maps
            .region(reference.0 as usize)?
            .ok_or("selected original requires its actual pending declared region")
    }
    pub(in crate::sema::crates::typed_headers) fn original_application(
        &self,
        store: &Store,
        reference: s::TraitApplicationRef,
        steps: &mut usize,
    ) -> Result<Predicate, &'static str> {
        use crate::sema::crates::typed_headers::predicate_remap::translation::applications::Cache;
        *steps = steps.saturating_add(1);
        self.require_store(store)?;
        self.applications
            .entry(reference.0 as usize)?
            .ok_or("selected original requires its actual pending canonical application")
    }
}
