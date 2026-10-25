//! Each complete application is remapped once; clauses retain their own effect.
use super::{Context, Error, Headers, s, translation};
use crate::types::{Store, parameters::predicates::Predicate};

impl Context<'_> {
    pub(in crate::sema::crates::typed_headers) fn application(
        &mut self,
        reference: s::TraitApplicationRef,
        store: &mut Store,
        budget: &mut Headers,
    ) -> Result<Predicate, Error> {
        let mut port = self.port(None);
        translation::application(&mut port, &mut self.applications, reference, store, budget)
    }
}
