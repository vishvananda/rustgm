//! A header that consumes a trait associated type's contract before the fixed
//! associated-type pass prepares that one member on demand.
use super::{Checker, DefId, Error, ItemId, Owner};
use crate::{
    resolve::model::DefKind,
    sema::associated::OwnerKind,
    syntax::item::{ItemKind, Visibility},
};

impl Checker<'_> {
    /// The source trait owner of an associated type, or `None` for any other
    /// definition.
    fn source_trait_member(&self, id: DefId) -> Result<Option<(Owner, DefId)>, Error> {
        if self.report.resolution.definitions[id.0 as usize].kind != DefKind::TypeAlias {
            return Ok(None);
        }
        let Some(index) = self.facts.associated.owner(id) else {
            return Ok(None);
        };
        let Some(owner) = self
            .facts
            .associated
            .source_at(index.0 as usize)
            .map_err(Error::unlocated)?
        else {
            return Ok(None);
        };
        Ok(match owner.kind {
            OwnerKind::Trait { definition, .. } => Some((owner, definition)),
            _ => None,
        })
    }

    /// A source trait associated type whose result contract a consumer needs
    /// and no pass has prepared yet.
    pub(in crate::sema::declarations) fn unprepared_contract(
        &self,
        item: ItemId,
    ) -> Result<Option<DefId>, Error> {
        let id = DefId(item.0);
        if self.facts.associated.types.contains_key(&id)
            || self.demanded_contracts.contains_key(&id)
        {
            return Ok(None);
        }
        Ok(self.source_trait_member(id)?.map(|_| id))
    }

    /// Prepare one demanded trait associated type, as the fixed pass would.
    /// A declared contract whose bounds are still being lowered is the same
    /// partial view that pass gives its other members. Returns false for any
    /// other definition.
    pub(in crate::sema::declarations) fn demand_associated_contract(
        &mut self,
        id: DefId,
    ) -> Result<bool, Error> {
        let Some((owner, trait_item)) = self.source_trait_member(id)? else {
            return Ok(false);
        };
        if self.facts.associated.types.contains_key(&id) {
            return Ok(true);
        }
        let key = self.report.resolution.definitions[id.0 as usize]
            .node
            .ok_or_else(|| self.error(owner.source, "missing associated type source"))?;
        if self.demanded_contracts.contains_key(&id) {
            return Err(self.error(key, "demanded associated type contract was withdrawn"));
        }
        let item = self.item(key)?;
        let ItemKind::TypeAlias { ty, .. } = item.kind else {
            return Err(self.error(key, "missing associated type syntax"));
        };
        if item.visibility != Visibility::Private {
            return Err(self.error(key, "associated types require inherited visibility"));
        }
        let pending = self.declare_trait_member(id, key, owner, trait_item, ty)?;
        if !self.facts.associated.types.contains_key(&id) {
            return Err(self.error(
                key,
                "demanded associated type contract belongs to a compiler role",
            ));
        }
        self.facts.generics.parents.insert(id, trait_item);
        self.demanded_contracts.insert(id, !pending);
        if pending {
            // The member stays active until its result bounds are lowered.
            self.states[id.0 as usize] = super::super::super::State::Active;
            self.census_activate(id);
            self.work
                .push(super::super::super::Work::AssociatedBounds(id));
            self.begin_associated_result_bounds(id, key)?;
        }
        Ok(true)
    }

    pub(in crate::sema::declarations) fn finish_demanded_bounds(
        &mut self,
        id: DefId,
    ) -> Result<(), Error> {
        let key = self.report.resolution.definitions[id.0 as usize]
            .node
            .ok_or_else(|| Error::unlocated("missing demanded associated type source"))?;
        if self.demanded_contracts.get(&id) != Some(&false) {
            return Err(self.error(key, "associated result bounds finished twice"));
        }
        self.finish_associated_result_bounds(id, key)?;
        self.states[id.0 as usize] = super::super::super::State::Complete;
        self.demanded_contracts.insert(id, true);
        Ok(())
    }

    /// Census only: a demanded contract whose bounds did not finish is
    /// withdrawn, so its later readers wait on the poisoned member.
    pub(in crate::sema::declarations) fn withdraw_unfinished_contracts(&mut self) {
        for (id, complete) in &self.demanded_contracts {
            if !complete {
                self.facts.associated.types.remove(id);
            }
        }
    }
}

#[cfg(test)]
#[path = "../../../../../tests/invariants/demand_header_facts.rs"]
mod tests;
