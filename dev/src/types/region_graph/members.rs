//! Hidden regions choose captured universals without granting new assumptions.
mod domains;
mod worklist;
use super::{Error, Graph, Kind, Region, Solution};
use crate::support::intern_slice::Slices;

/// A shared list of permitted captures in one constraint graph.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ChoicesId(u32);
/// A retained equality choice, available only after the entire graph proves.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MembershipId(u32);

#[derive(Clone, Copy)]
struct Request {
    region: Region,
    choices: ChoicesId,
    source: u32,
}
#[derive(Default)]
pub(super) struct Members {
    choices: Slices<Region>,
    requests: Vec<Request>,
    selected: Vec<Region>,
    pub(super) cost: usize,
}

impl Graph {
    /// Intern one set of captured defining-context lifetimes. Static is explicit;
    /// local storage and younger quantified placeholders cannot be captures.
    /// Reuse this identity for every hidden region with the same capture list.
    /// # Errors
    /// Rejects noncapture identities and exhausted constraint storage.
    pub fn member_choices(&mut self, choices: &[Region]) -> Result<ChoicesId, Error> {
        for &choice in choices {
            if !matches!(
                self.kind(choice)?,
                Kind::Static | Kind::Universal { universe: 0 }
            ) {
                return Err(Error::Identity);
            }
        }
        if let Some(id) = self.members.as_ref().and_then(|m| m.choices.find(choices)) {
            return Ok(ChoicesId(id));
        }
        self.capacity(choices.len().saturating_add(1))?;
        let members = self.members.get_or_insert_with(Default::default);
        let id = members.choices.intern(choices).map_err(|_| Error::Budget)?;
        members.cost = members
            .choices
            .edge_count()
            .saturating_add(members.requests.len());
        Ok(ChoicesId(id))
    }

    /// Require a hidden region to equal a permitted capture. No equality is
    /// assumed: the selected pair remains in the graph as proof obligations.
    /// # Errors
    /// Rejects foreign identities and exhausted constraint storage.
    pub fn member(
        &mut self,
        region: Region,
        choices: ChoicesId,
        source: u32,
    ) -> Result<MembershipId, Error> {
        self.kind(region)?;
        self.capacity(1)?;
        let members = self.members.as_mut().ok_or(Error::Identity)?;
        members.choices.get(choices.0).ok_or(Error::Identity)?;
        let id = MembershipId(u32::try_from(members.requests.len()).map_err(|_| Error::Budget)?);
        members.requests.push(Request {
            region,
            choices,
            source,
        });
        members.cost = members.cost.saturating_add(1);
        Ok(id)
    }

    pub(super) fn select_members(&mut self) -> Result<usize, Error> {
        let Some(members) = &self.members else {
            return Ok(0);
        };
        if members.requests.is_empty() {
            return Ok(0);
        }
        let (selected, steps) = worklist::select(self, members)?;
        for (index, &choice) in selected.iter().enumerate() {
            let request = self.members.as_ref().ok_or(Error::Identity)?.requests[index];
            self.require(request.region, choice, request.source)?;
            self.require(choice, request.region, request.source)?;
        }
        self.members.as_mut().ok_or(Error::Identity)?.selected = selected;
        Ok(steps)
    }
}
impl Solution {
    /// Return the permitted name for a membership established by this proof.
    /// # Errors
    /// Rejects identities outside this solved graph.
    pub fn member_choice(&self, id: MembershipId) -> Result<Region, Error> {
        self.graph
            .members
            .as_ref()
            .and_then(|m| m.selected.get(id.0 as usize))
            .copied()
            .ok_or(Error::Identity)
    }
}

#[cfg(test)]
#[path = "../../../tests/invariants/hidden_choices.rs"]
mod tests;
