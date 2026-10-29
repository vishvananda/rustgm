//! Source constant indexes visit only the actual selected owner and trait buckets.
use super::{Declarations, DefId, DefKind, Members, Report};
use crate::{sema::associated::OwnerKind, support::retained};

impl Members {
    pub(in crate::sema::declarations::traits::implementations) fn prepare_source_owner(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        owner: DefId,
    ) -> Result<(), &'static str> {
        crate::sema::header_attributes::charge_source_contract_query(facts, 1)
            .map_err(|_| "source constant member query work exhausted")?;
        facts
            .source_bindings
            .as_deref()
            .ok_or("source constant index has no provider")?
            .require_provider_report(report)?;
        let id = facts
            .associated
            .owner(owner)
            .ok_or("source constant owner is missing")?;
        let actual = facts
            .associated
            .get(id)
            .ok_or("source constant owner disappeared")?;
        let owner = match actual.kind {
            OwnerKind::Trait { definition, .. }
                if definition == owner || actual.self_definition == owner =>
            {
                actual.self_definition
            }
            OwnerKind::Impl { .. } if actual.self_definition == owner => owner,
            _ => return Err("constant member index requires an actual source owner"),
        };
        actual.source_owner()?;
        if self.owners.contains_key(&owner) {
            return Ok(());
        }
        let members = facts
            .associated
            .owner_members(id)
            .ok_or("source constant members are missing")?;
        let count = members.len();
        let growth = retained::Growth::map::<DefId, Vec<DefId>>(
            self.owners.len(),
            self.owners.capacity(),
            1,
        );
        crate::sema::header_attributes::charge_source_retention(
            facts,
            count.saturating_add(growth.work).saturating_add(1),
            growth
                .bytes
                .saturating_add(count.saturating_mul(size_of::<DefId>())),
        )
        .map_err(|_| "source constant member storage or work exhausted")?;
        let mut constants = Vec::new();
        constants
            .try_reserve_exact(count)
            .map_err(|_| "source constant member allocation failed")?;
        for &member in members {
            if facts.associated.owner(member) != Some(id) {
                return Err("source constant member belongs to another owner");
            }
            let definition = report
                .resolution
                .definitions
                .get(member.0 as usize)
                .ok_or("source constant member definition is missing")?;
            if definition.kind == DefKind::Const {
                constants.push(member);
            }
        }
        self.owners.insert(owner, constants);
        Ok(())
    }
    pub(super) fn prepare_source_pair(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        owner: DefId,
        trait_owner: DefId,
    ) -> Result<(), (DefId, &'static str)> {
        for owner in [owner, trait_owner] {
            self.prepare_source_owner(report, facts, owner)
                .map_err(|e| (owner, e))?;
        }
        let count = self.owners.get(&owner).map_or(0, Vec::len);
        let growth = retained::Growth::map::<DefId, crate::support::hash::HashMap<DefId, DefId>>(
            self.matches.len(),
            self.matches.capacity(),
            1,
        );
        crate::sema::header_attributes::charge_source_retention(
            facts,
            count
                .saturating_mul(4)
                .saturating_add(growth.work)
                .saturating_add(1),
            growth
                .bytes
                .saturating_add(retained::new_map::<DefId, DefId>(count)),
        )
        .map_err(|_| (owner, "source constant override storage or work exhausted"))?;
        Ok(())
    }
}

#[cfg(test)]
impl Members {
    pub(in crate::sema::declarations::traits::implementations) fn indexed_storage(
        &self,
    ) -> (usize, usize) {
        let bytes = retained::map(&self.owners)
            + retained::map(&self.matches)
            + self.owners.values().map(retained::vector).sum::<usize>()
            + self.matches.values().map(retained::map).sum::<usize>();
        (self.owners.len(), bytes)
    }
}
