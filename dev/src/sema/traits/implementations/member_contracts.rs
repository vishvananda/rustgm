//! A checked constant member does not authorize methods or complete its implementation.
use super::{Availability, DefId, Headers, Implementations};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(in crate::sema) struct ConstantRequest {
    pub owner: DefId,
    pub declaration: DefId,
}

impl Implementations {
    pub(in crate::sema) fn finish_constant_headers(&mut self) -> Result<(), &'static str> {
        if self
            .pending
            .as_ref()
            .is_some_and(|pending| !pending.only_collections())
        {
            return Err("implementation candidate inventory still has pending headers");
        }
        if self
            .pending
            .as_ref()
            .is_some_and(|pending| pending.is_empty())
        {
            self.pending = None;
        }
        if self.headers == Headers::Open {
            self.headers = Headers::Constants;
        }
        Ok(())
    }

    pub(in crate::sema) fn constant_headers_ready(&self) -> bool {
        self.headers != Headers::Open
    }
    pub(in crate::sema) fn selection_headers_ready(&self) -> bool {
        self.constant_headers_ready()
            && self
                .pending
                .as_deref()
                .is_none_or(super::pending::Pending::is_empty)
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
        // This records one owner's checked contract, not selection authority.
        // The requester selects that owner through the exhaustive candidate query;
        // unrelated pending headers need not block this member's validation.
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
