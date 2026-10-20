//! Opaque choices use the same scoped identities as ordinary typed coercions.
use super::{Constraints, Error, Proof, Region, RegionId, Scope};
use crate::types::region_graph::{ChoicesId, MembershipId};

impl Constraints<'_> {
    /// Share one capture set across the free regions of a hidden result.
    /// Captures name the defining body's root binder, never an inference alias.
    /// # Errors
    /// Rejects unbound or nonuniversal names and exhausted graph storage.
    pub fn member_choices(&mut self, choices: &[RegionId]) -> Result<ChoicesId, Error> {
        let choices = choices
            .iter()
            .map(|&r| self.region(r, Scope(0)))
            .collect::<Result<Vec<_>, _>>()?;
        self.graph.member_choices(&choices)
    }

    /// Constrain a typed body lifetime to one of the explicitly permitted captures.
    /// Coercions, provenance and declared bounds remain independent requirements.
    /// # Errors
    /// Rejects missing identities and exhausted graph storage.
    pub fn member(
        &mut self,
        region: RegionId,
        choices: ChoicesId,
        source: u32,
    ) -> Result<MembershipId, Error> {
        self.graph
            .member(self.region(region, Scope(0))?, choices, source)
    }
}

impl Proof {
    /// Restore the declared lifetime name selected by a completed membership proof.
    /// Root binder IDs are allocated contiguously, so restoration costs O(1) per
    /// hidden region and does not scan the function's complete capture list.
    /// # Errors
    /// Rejects foreign memberships and choices outside the defining root binder.
    pub fn member_choice(&self, member: MembershipId) -> Result<RegionId, Error> {
        let choice = self.solution.member_choice(member)?;
        if choice == Region::STATIC {
            return Ok(RegionId::STATIC);
        }
        let start = self.identities.universals.first().ok_or(Error::Identity)?;
        let index = choice.0.checked_sub(start.0).ok_or(Error::Identity)?;
        if self.identities.universals.get(index as usize) != Some(&choice) {
            return Err(Error::Identity);
        }
        RegionId::bound(0, u16::try_from(index).map_err(|_| Error::Identity)?)
            .map_err(|_| Error::Identity)
    }
}

#[cfg(test)]
#[path = "../../../tests/invariants/capture_relations.rs"]
mod tests;
