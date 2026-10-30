//! Header polarity is declaration data; validation and proof payment are separate.
use super::{Identity, Implementation, Implementations};
use crate::{
    resolve::model::DefId,
    sema::associated::{Inventory, Owner, OwnerKind},
    types::Store,
};

pub(in crate::sema) use crate::sema::associated::ImplPolarity as Polarity;
#[derive(Clone, Copy)]
pub(super) enum Status {
    PositiveHeader,
    PositiveEarlyUnavailable,
    PositiveValidated,
    NegativeHeader,
    NegativeEarlyUnavailable,
    NegativeValidated,
    ReservationHeader,
    ReservationEarlyUnavailable,
    ReservationChecked,
}
impl Status {
    /// Declaration conformance only; reservations never authorize selection.
    pub(super) fn validated(self) -> bool {
        matches!(
            self,
            Self::PositiveValidated | Self::NegativeValidated | Self::ReservationChecked
        )
    }
    pub(super) fn negative(self) -> bool {
        matches!(
            self,
            Self::NegativeHeader | Self::NegativeEarlyUnavailable | Self::NegativeValidated
        )
    }
    pub(super) fn polarity(self) -> Polarity {
        if self.negative() {
            Polarity::Negative
        } else if matches!(
            self,
            Self::ReservationHeader | Self::ReservationEarlyUnavailable | Self::ReservationChecked
        ) {
            Polarity::Reservation
        } else {
            Polarity::Positive
        }
    }
    pub(super) fn header(polarity: Polarity) -> Self {
        match polarity {
            Polarity::Positive => Self::PositiveHeader,
            Polarity::Negative => Self::NegativeHeader,
            Polarity::Reservation => Self::ReservationHeader,
        }
    }
    pub(super) fn early_unavailable(self) -> bool {
        matches!(
            self,
            Self::PositiveEarlyUnavailable
                | Self::NegativeEarlyUnavailable
                | Self::ReservationEarlyUnavailable
        )
    }
    pub(super) fn complete(self) -> Self {
        match self.polarity() {
            Polarity::Positive => Self::PositiveValidated,
            Polarity::Negative => Self::NegativeValidated,
            Polarity::Reservation => Self::ReservationChecked,
        }
    }
}
impl Implementations {
    /// Imported headers stay unpaid until ordinary conformance proves all of
    /// their requirements and explicitly completes the registry entry.
    pub(in crate::sema) fn prepare_imported(
        &mut self,
        store: &mut Store,
        associated: &Inventory,
        identity: Identity,
        owner: Owner,
        value: Implementation,
    ) -> Result<(), &'static str> {
        let status = Self::imported_status(store, associated, owner, &value)?;
        self.prepare_with_status(store, identity, value, status)
    }

    pub(in crate::sema) fn prepare_imported_charged(
        &mut self,
        store: &mut Store,
        associated: &Inventory,
        identity: Identity,
        owner: Owner,
        value: Implementation,
        charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<(usize, std::ops::Range<usize>), &'static str> {
        charge(1, 0)?;
        let status = Self::imported_status(store, associated, owner, &value)?;
        let first = self.pairs.len();
        let retained =
            self.prepare_with_observer::<true>(store, identity, value, status, charge)?;
        Ok((retained, first..self.pairs.len()))
    }

    fn imported_status(
        store: &Store,
        associated: &Inventory,
        owner: Owner,
        value: &Implementation,
    ) -> Result<Status, &'static str> {
        if value.source.is_some()
            || owner.self_definition != value.owner
            || associated
                .owner(value.owner)
                .and_then(|id| associated.get(id))
                .copied()
                != Some(owner)
        {
            return Err("imported implementation changed its actual retained owner");
        }
        let (constant, negative) =
            associated.imported_implementation(store, owner, value.predicate)?;
        check_effect(value.predicate, constant)?;
        let status = if negative {
            Status::NegativeHeader
        } else {
            Status::PositiveHeader
        };
        Ok(status)
    }

    /// This records the source scheduler's profile result, never conformance.
    /// Normal completion remains the only route to validated availability.
    pub(in crate::sema) fn mark_early_unavailable(
        &mut self,
        store: &Store,
        arena: &crate::types::arena::Identity,
        owner: DefId,
        source: crate::resolve::unit::NodeKey,
        predicate: crate::types::parameters::predicates::Predicate,
    ) -> Result<(), &'static str> {
        if !store.has_identity(arena)
            || self
                .profile_arena
                .as_ref()
                .is_some_and(|actual| !store.has_identity(actual))
        {
            return Err("early implementation profile belongs to another actual arena");
        }
        let entry = self
            .owners
            .get(&owner)
            .and_then(|&i| self.entries.get_mut(i))
            .ok_or("early implementation profile lost its actual header")?;
        if entry.value.owner != owner
            || entry.value.source != Some(source)
            || entry.value.predicate != predicate
            || entry.status.validated()
        {
            return Err("early implementation profile changed its source, predicate or validation");
        }
        entry.status = match entry.status.polarity() {
            Polarity::Positive => Status::PositiveEarlyUnavailable,
            Polarity::Negative => Status::NegativeEarlyUnavailable,
            Polarity::Reservation => Status::ReservationEarlyUnavailable,
        };
        self.profile_arena.get_or_insert_with(|| arena.clone());
        Ok(())
    }
    pub(in crate::sema) fn early_unavailable(
        &self,
        store: &Store,
        owner: DefId,
    ) -> Result<bool, &'static str> {
        let marked = self
            .owners
            .get(&owner)
            .is_some_and(|&i| self.entries[i].status.early_unavailable());
        if marked
            && self
                .profile_arena
                .as_ref()
                .is_none_or(|arena| !store.has_identity(arena))
        {
            return Err("early implementation profile belongs to another actual arena");
        }
        Ok(marked)
    }
    /// Source publication must carry the actual retained impl owner. The other
    /// publication entry is reserved for explicitly registered positive factories.
    pub(in crate::sema) fn prepare_source(
        &mut self,
        store: &mut Store,
        associated: &Inventory,
        identity: Identity,
        owner: Owner,
        value: Implementation,
    ) -> Result<(), &'static str> {
        let source = owner.source_owner()?;
        let OwnerKind::Impl {
            polarity, constant, ..
        } = owner.kind
        else {
            return Err("source implementation requires an actual impl owner");
        };
        if associated
            .owner(value.owner)
            .and_then(|id| associated.get(id))
            .copied()
            != Some(owner)
            || owner.self_definition != value.owner
            || Some(source.source) != value.source
        {
            return Err("source implementation changed its retained owner");
        }
        check_effect(value.predicate, constant)?;
        self.prepare_with_status(store, identity, value, Status::header(polarity))
    }

    pub(in crate::sema) fn prepare_factory(
        &mut self,
        store: &mut Store,
        associated: &Inventory,
        identity: impl Into<Identity>,
        value: Implementation,
    ) -> Result<(), &'static str> {
        if value.source.is_none()
            || associated.owner(value.owner).is_some()
            || associated.declaration_owner(value.owner) != Some(value.owner)
            || associated.schema(value.owner).is_none()
        {
            return Err("positive factory requires a registered source-free schema");
        }
        // Existing registered factories carry ordinary contracts only. A source
        // const owner cannot be reconstructed from a source-free schema.
        check_effect(value.predicate, false)?;
        self.prepare_with_status(store, identity.into(), value, Status::PositiveHeader)
    }

    /// Recheck source polarity even on extraction and cache publication. Missing
    /// source owners are legal only for the explicit positive factory entry.
    pub(in crate::sema) fn polarity(
        &self,
        store: &Store,
        associated: &Inventory,
        definition: DefId,
    ) -> Result<Polarity, &'static str> {
        let entry = self
            .owners
            .get(&definition)
            .and_then(|&index| self.entries.get(index))
            .ok_or("missing implementation polarity record")?;
        if let Some(owner) = associated
            .owner(definition)
            .and_then(|id| associated.get(id))
        {
            let (constant, polarity) = match owner.kind {
                OwnerKind::Impl {
                    polarity, constant, ..
                } => {
                    let source = owner.source_owner()?;
                    if Some(source.source) != entry.value.source {
                        return Err("implementation polarity changed its actual source owner");
                    }
                    (constant, polarity)
                }
                OwnerKind::ImportedImpl(_) => {
                    if entry.value.source.is_some() {
                        return Err("imported conformance cannot acquire source syntax provenance");
                    }
                    let (constant, negative) =
                        associated.imported_implementation(store, *owner, entry.value.predicate)?;
                    (constant, Polarity::from_negative(negative))
                }
                OwnerKind::Trait { .. } => {
                    return Err("implementation polarity belongs to a non-impl owner");
                }
            };
            check_effect(entry.value.predicate, constant)?;
            if owner.self_definition != definition || polarity != entry.status.polarity() {
                return Err("implementation polarity disagrees with its source owner");
            }
        } else {
            check_effect(entry.value.predicate, false)?;
            if entry.status.polarity() != Polarity::Positive
                || associated.declaration_owner(definition) != Some(definition)
                || associated.schema(definition).is_none()
            {
                return Err("implementation polarity has no registered source or factory owner");
            }
        }
        Ok(entry.status.polarity())
    }
}

fn check_effect(
    predicate: crate::types::parameters::predicates::Predicate,
    constant: bool,
) -> Result<(), &'static str> {
    use crate::support::constness::Constness;
    let expected = if constant {
        Constness::Const
    } else {
        Constness::Ordinary
    };
    if predicate
        .bound
        .trait_parts()
        .is_some_and(|(_, effect)| effect != expected)
        || (constant && predicate.bound.trait_parts().is_none())
    {
        return Err("implementation effect disagrees with its actual source owner");
    }
    Ok(())
}
