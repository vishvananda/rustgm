//! Only a result of this complete formal parent is an owned associated promise.
use crate::types::{
    GenericArgs, ItemId, Kind, Store,
    parameters::predicates::{Bound, Predicate},
};

impl super::super::Header {
    pub(in crate::sema) fn partition_results(
        &mut self,
        store: &Store,
        owner: ItemId,
    ) -> Result<(), &'static str> {
        let mut predicates = std::mem::take(&mut self.predicates);
        let mut retained = 0;
        for index in 0..predicates.len() {
            let predicate = predicates[index];
            if let Some(member) = owned_header_result(store, owner, self, predicate.value)? {
                self.results.push((member, predicate));
            } else {
                predicates[retained] = predicate;
                retained += 1;
            }
        }
        predicates.truncate(retained);
        self.predicates = predicates;
        Ok(())
    }
}

fn owned_header_result(
    store: &Store,
    owner: ItemId,
    header: &super::super::Header,
    predicate: Predicate,
) -> Result<Option<ItemId>, &'static str> {
    if matches!(predicate.bound, Bound::ProjectionEqual(_)) {
        return Ok(None);
    }
    let Some(Kind::Projection(projection)) = store.kind(predicate.ty) else {
        return Ok(None);
    };
    if projection.subject != header.self_type || projection.arguments != GenericArgs::EMPTY {
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
    if actual.eq(header.parameters.iter().map(|p| p.argument)) {
        Ok(Some(projection.item))
    } else {
        Ok(None)
    }
}
