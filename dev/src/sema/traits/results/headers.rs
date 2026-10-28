//! Only a result of this complete formal parent is an owned associated promise.
use crate::types::{
    GenericArgs, ItemId, Kind, Store,
    arguments::Argument,
    parameters::{clauses::Clause, predicates::Bound},
};

impl super::super::Header {
    pub(in crate::sema) fn partition_results(
        &mut self,
        store: &Store,
        owner: ItemId,
    ) -> Result<(), &'static str> {
        let mut predicates = std::mem::take(&mut self.source_inputs_mut()?.predicates);
        let mut retained = 0;
        for index in 0..predicates.len() {
            let predicate = predicates[index];
            if let Some(member) = self.owned_result(store, owner, predicate.value)? {
                self.results.push((member, predicate));
            } else {
                predicates[retained] = predicate;
                retained += 1;
            }
        }
        predicates.truncate(retained);
        self.source_inputs_mut()?.predicates = predicates;
        Ok(())
    }
    pub(in crate::sema) fn owned_result(
        &self,
        store: &Store,
        owner: ItemId,
        clause: Clause,
    ) -> Result<Option<ItemId>, &'static str> {
        // Only inspect ownership here. The returned member retains the whole
        // clause; its binder is never discarded from the resulting contract.
        let (predicate, depth) = match clause {
            Clause::UnstableFeature(_) => return Ok(None),
            Clause::Predicate(predicate) => (predicate, 0),
            Clause::ForAll(id) => (
                store
                    .quantified(id)
                    .ok_or("invalid associated result clause")?
                    .predicate,
                1,
            ),
        };
        // A binding in `Self::Out: Other<Value = X>` belongs to the contract
        // of this owner's Out, while its equality subject remains Other::Value.
        let subject = if matches!(predicate.bound, Bound::ProjectionEqual(_)) {
            let Some(Kind::Projection(equality)) = store.kind(predicate.ty) else {
                return Err("associated equality requires a projected subject");
            };
            equality.subject
        } else {
            predicate.ty
        };
        let Some(Kind::Projection(projection)) = store.kind(subject) else {
            return Ok(None);
        };
        if projection.subject != self.self_type || projection.arguments != GenericArgs::EMPTY {
            return Ok(None);
        }
        let application = store
            .trait_application(projection.trait_application)
            .ok_or("invalid projected trait header application")?;
        if application.item != owner {
            return Ok(None);
        }
        let actual = store
            .arguments(application.arguments)
            .ok_or("invalid projected trait header arguments")?;
        if actual.len() != self.parameters.len() {
            return Ok(None);
        }
        for (actual, formal) in actual.zip(&self.parameters) {
            // Formal header inputs are parameter identities, not default values.
            // Type/const parameter IDs are binder-independent; lifetimes capture.
            let expected = match formal.argument {
                Argument::Lifetime(region) => {
                    let region = if let Some((binder, index)) = region.bound_parts() {
                        crate::types::RegionId::bound(
                            binder
                                .checked_add(depth)
                                .ok_or("trait header capture depth overflow")?,
                            index,
                        )?
                    } else {
                        region
                    };
                    Argument::Lifetime(region)
                }
                argument => argument,
            };
            if actual != expected {
                return Ok(None);
            }
        }
        Ok(Some(projection.item))
    }
}
