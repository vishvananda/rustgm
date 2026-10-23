//! Header polarity is declaration data; validation and proof payment are separate.
use super::{Identity, Implementation, Implementations};
use crate::{
    resolve::model::DefId,
    sema::associated::{Inventory, Owner, OwnerKind},
    types::Store,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::sema) enum Polarity {
    Positive,
    Negative,
}
#[derive(Clone, Copy)]
pub(super) enum Status {
    PositiveHeader,
    PositiveValidated,
    NegativeHeader,
    NegativeValidated,
}
impl Status {
    pub(super) fn validated(self) -> bool {
        matches!(self, Self::PositiveValidated | Self::NegativeValidated)
    }
    pub(super) fn negative(self) -> bool {
        matches!(self, Self::NegativeHeader | Self::NegativeValidated)
    }
    pub(super) fn complete(self) -> Self {
        if self.negative() {
            Self::NegativeValidated
        } else {
            Self::PositiveValidated
        }
    }
}
impl Implementations {
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
        let OwnerKind::Impl { negative, .. } = owner.kind else {
            return Err("source implementation requires an actual impl owner");
        };
        if associated
            .owner(value.owner)
            .and_then(|id| associated.get(id))
            .copied()
            != Some(owner)
            || owner.self_definition != value.owner
            || owner.source != value.source
        {
            return Err("source implementation changed its retained owner");
        }
        let status = if negative {
            Status::NegativeHeader
        } else {
            Status::PositiveHeader
        };
        self.prepare_with_status(store, identity, value, status)
    }

    pub(in crate::sema) fn prepare_factory(
        &mut self,
        store: &mut Store,
        associated: &Inventory,
        identity: impl Into<Identity>,
        value: Implementation,
    ) -> Result<(), &'static str> {
        if associated.owner(value.owner).is_some()
            || associated.declaration_owner(value.owner) != Some(value.owner)
            || associated.schema(value.owner).is_none()
        {
            return Err("positive factory requires a registered source-free schema");
        }
        self.prepare_with_status(store, identity.into(), value, Status::PositiveHeader)
    }

    /// Recheck source polarity even on extraction and cache publication. Missing
    /// source owners are legal only for the explicit positive factory entry.
    pub(in crate::sema) fn polarity(
        &self,
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
            let OwnerKind::Impl { negative, .. } = owner.kind else {
                return Err("implementation polarity belongs to a non-impl owner");
            };
            if owner.self_definition != definition
                || owner.source != entry.value.source
                || negative != entry.status.negative()
            {
                return Err("implementation polarity disagrees with its source owner");
            }
        } else if entry.status.negative()
            || associated.declaration_owner(definition) != Some(definition)
            || associated.schema(definition).is_none()
        {
            return Err("implementation polarity has no registered source or factory owner");
        }
        Ok(if entry.status.negative() {
            Polarity::Negative
        } else {
            Polarity::Positive
        })
    }
}
