//! A checked constant member does not authorize methods or complete its implementation.
use super::{Availability, DefId, Implementations};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(in crate::sema) struct ConstantRequest {
    pub owner: DefId,
    pub declaration: DefId,
}

impl Implementations {
    pub(in crate::sema) fn finish_constant_headers(&mut self) {
        self.constant_headers_ready = true;
    }

    pub(in crate::sema) fn constant_headers_ready(&self) -> bool {
        self.constant_headers_ready
    }
    pub(in crate::sema) fn available_constant(
        &self,
        owner: DefId,
        declaration: DefId,
        availability: Availability,
    ) -> Option<DefId> {
        self.constant(owner, declaration).or_else(|| {
            if matches!(availability, Availability::Declarations) {
                self.entries[*self.owners.get(&owner)?]
                    .checked_constants
                    .get(&declaration)
                    .copied()
            } else {
                None
            }
        })
    }

    pub(in crate::sema) fn complete_constant(
        &mut self,
        request: ConstantRequest,
        definition: DefId,
    ) -> Result<(), &'static str> {
        if !self.constant_headers_ready {
            return Err("constant readiness requires complete candidate headers");
        }
        let index = *self
            .owners
            .get(&request.owner)
            .ok_or("constant readiness requires a prepared implementation")?;
        let entry = &mut self.entries[index];
        match entry.checked_constants.entry(request.declaration) {
            std::collections::hash_map::Entry::Vacant(slot) => {
                slot.insert(definition);
            }
            std::collections::hash_map::Entry::Occupied(slot) if *slot.get() != definition => {
                return Err("checked constant changed its selected definition");
            }
            std::collections::hash_map::Entry::Occupied(_) => {}
        }
        Ok(())
    }
}
