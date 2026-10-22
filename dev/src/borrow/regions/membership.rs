//! Share capture lists while consuming the existing body-obligation stream.
use super::constraints::Constraints;
use crate::support::hash::HashMap;
use crate::types::{
    GenericArgs, RegionId, Store,
    arguments::Argument,
    region_graph::{ChoicesId, MembershipId},
};

pub(super) struct Requests<'a> {
    types: &'a Store,
    choices: HashMap<GenericArgs, ChoicesId>,
    members: Vec<(GenericArgs, RegionId, MembershipId)>,
}
impl<'a> Requests<'a> {
    pub(super) fn new(types: &'a Store) -> Self {
        Self {
            types,
            choices: HashMap::default(),
            members: Vec::new(),
        }
    }
    pub(super) fn require(
        &mut self,
        constraints: &mut Constraints<'_>,
        region: RegionId,
        captures: GenericArgs,
        source: u32,
    ) -> Result<(), String> {
        let choice = if let Some(&choice) = self.choices.get(&captures) {
            choice
        } else {
            let mut regions = vec![RegionId::STATIC];
            regions.extend(
                self.types
                    .arguments(captures)
                    .ok_or("invalid opaque capture list")?
                    .filter_map(|argument| match argument {
                        Argument::Lifetime(region) => Some(region),
                        _ => None,
                    }),
            );
            let choice = constraints
                .member_choices(&regions)
                .map_err(|e| e.to_string())?;
            self.choices.insert(captures, choice);
            choice
        };
        self.members.push((
            captures,
            region,
            constraints
                .member(region, choice, source)
                .map_err(|e| e.to_string())?,
        ));
        Ok(())
    }
    pub(super) fn finish(
        self,
        proof: &crate::types::relations::Proof,
    ) -> Result<Vec<crate::types::opaque::RegionChoice>, crate::types::region_graph::Error> {
        self.members
            .into_iter()
            .map(|(captures, region, member)| {
                proof
                    .member_choice(member)
                    .map(|choice| crate::types::opaque::RegionChoice {
                        captures,
                        region,
                        choice,
                    })
            })
            .collect()
    }
}
