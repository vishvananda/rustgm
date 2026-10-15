//! Obligations have typed subjects and canonical trait applications, never spelling keys.
use crate::types::TypeId;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Bound {
    Copy,
    Clone,
    Sized,
    Trait(crate::types::traits::ApplicationId),
    Outlives(crate::types::RegionId),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Predicate {
    pub ty: TypeId,
    pub bound: Bound,
}
impl Predicate {
    /// A trait argument can be generic even when its Self subject is concrete.
    /// # Errors
    /// Rejects unknown subject or application identities.
    pub fn contains_parameters(self, store: &crate::types::Store) -> Result<bool, &'static str> {
        let subject = store
            .contains_parameters(self.ty)
            .ok_or("invalid predicate subject")?;
        let arguments = if let Bound::Trait(id) = self.bound {
            store
                .trait_properties(id)
                .ok_or("invalid predicate trait application")?
                .has(crate::types::Property::Parameters)
        } else {
            false
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
            Bound::Copy | Bound::Clone | Bound::Sized => 0,
        };
        Ok(subject.max(arguments))
    }
}
