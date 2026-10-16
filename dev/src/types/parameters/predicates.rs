//! Obligations have typed subjects and canonical trait applications, never spelling keys.
use crate::types::TypeId;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Bound {
    Copy,
    Clone,
    Sized,
    Trait(crate::types::traits::ApplicationId),
    Outlives(crate::types::RegionId),
    /// The projected subject equals this result; its inputs are not injective.
    /// A discharged normalizer may replace either root with its normal form.
    ProjectionEqual(TypeId),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Predicate {
    pub ty: TypeId,
    pub bound: Bound,
}
impl Predicate {
    /// Concrete inputs may contain static lifetimes and closed callable binders,
    /// but no type/const parameters, unresolved projections or external region variables.
    /// # Errors
    /// Rejects unknown subject, result or trait application identities.
    pub fn is_concrete(self, store: &crate::types::Store) -> Result<bool, &'static str> {
        let concrete = |ty: TypeId| {
            store
                .properties
                .get(ty.0 as usize)
                .map(|p| p.is_concrete())
                .ok_or("invalid predicate type")
        };
        if !concrete(self.ty)? {
            return Ok(false);
        }
        Ok(match self.bound {
            Bound::Trait(id) => store
                .trait_properties(id)
                .ok_or("invalid predicate trait application")?
                .is_concrete(),
            Bound::ProjectionEqual(result) => concrete(result)?,
            Bound::Outlives(region) => region == crate::types::RegionId::STATIC,
            Bound::Copy | Bound::Clone | Bound::Sized => true,
        })
    }

    /// Projections may occur in the subject or any complete trait argument.
    /// # Errors
    /// Rejects unknown subject or application identities.
    pub fn contains_projections(self, store: &crate::types::Store) -> Result<bool, &'static str> {
        let subject = store
            .contains_projections(self.ty)
            .ok_or("invalid predicate subject")?;
        let arguments = match self.bound {
            Bound::Trait(id) => store
                .trait_properties(id)
                .ok_or("invalid predicate trait application")?
                .has(crate::types::Property::Projections),
            Bound::ProjectionEqual(result) => store
                .contains_projections(result)
                .ok_or("invalid projection equality result")?,
            Bound::Copy | Bound::Clone | Bound::Sized | Bound::Outlives(_) => false,
        };
        Ok(subject || arguments)
    }
    /// Whether any argument lifetime needs opening outside its local binders.
    /// # Errors
    /// Rejects unknown subject or application identities.
    pub fn has_free_regions(self, store: &crate::types::Store) -> Result<bool, &'static str> {
        let subject = store
            .has_free_regions(self.ty, 0)
            .ok_or("invalid predicate subject")?;
        let arguments = match self.bound {
            Bound::Trait(id) => {
                let properties = store
                    .trait_properties(id)
                    .ok_or("invalid predicate trait application")?;
                properties.escaping != 0 || properties.has(crate::types::Property::FreeRegions)
            }
            Bound::Outlives(region) => region != crate::types::RegionId::STATIC,
            Bound::ProjectionEqual(result) => store
                .has_free_regions(result, 0)
                .ok_or("invalid projection equality result")?,
            Bound::Copy | Bound::Clone | Bound::Sized => false,
        };
        Ok(subject || arguments)
    }
    /// A trait argument can be generic even when its Self subject is concrete.
    /// # Errors
    /// Rejects unknown subject or application identities.
    pub fn contains_parameters(self, store: &crate::types::Store) -> Result<bool, &'static str> {
        let subject = store
            .contains_parameters(self.ty)
            .ok_or("invalid predicate subject")?;
        let arguments = match self.bound {
            Bound::Trait(id) => store
                .trait_properties(id)
                .ok_or("invalid predicate trait application")?
                .has(crate::types::Property::Parameters),
            Bound::ProjectionEqual(result) => store
                .contains_parameters(result)
                .ok_or("invalid projection equality result")?,
            Bound::Copy | Bound::Clone | Bound::Sized | Bound::Outlives(_) => false,
        };
        Ok(subject || arguments)
    }
    /// Include escaping lifetimes from Self, the bound and all written trait arguments.
    /// # Errors
    /// Rejects unknown subject or application identities.
    pub fn escaping_binders(self, store: &crate::types::Store) -> Result<u16, &'static str> {
        let subject = store
            .escaping_binders(self.ty)
            .ok_or("invalid predicate subject")?;
        let arguments = match self.bound {
            Bound::Trait(id) => {
                store
                    .trait_properties(id)
                    .ok_or("invalid predicate trait application")?
                    .escaping
            }
            Bound::Outlives(region) => region.bound_parts().map_or(0, |(depth, _)| depth + 1),
            Bound::ProjectionEqual(result) => store
                .escaping_binders(result)
                .ok_or("invalid projection equality result")?,
            Bound::Copy | Bound::Clone | Bound::Sized => 0,
        };
        Ok(subject.max(arguments))
    }
}
