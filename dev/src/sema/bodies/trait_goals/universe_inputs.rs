//! Universal call checks reuse only their actual declared caller inputs and aliases.
use super::{Checker, Error, NodeId};
use crate::{
    resolve::unit::NodeKey,
    sema::{callee_premises::OriginalInputs, model::Declarations},
    support::hash::HashSet,
    types::{
        RegionId,
        lifetime_proof::Obligations,
        outlives::Queries,
        relations::{BodyAliases, Constraints, ProofScope},
    },
};

pub(in crate::sema::bodies) struct CallerInputs {
    original: OriginalInputs,
    aliases: BodyAliases,
}
impl CallerInputs {
    pub(super) fn payload(&self) -> usize {
        self.original
            .allocation()
            .bytes
            .saturating_add(self.aliases.allocation().bytes)
            .saturating_add(self.aliases.control_allocation())
    }
    pub(super) fn graph<'a>(
        &self,
        facts: &'a Declarations,
        queries: &mut Queries<'a>,
        limit: usize,
    ) -> Result<Constraints<'a>, &'static str> {
        let mut graph = self.original.view().seed_limited(facts, queries, limit)?;
        for (slot, &alias) in self.aliases.ordered().iter().enumerate() {
            graph
                .scoped_alias(
                    alias,
                    ProofScope::ROOT,
                    u16::try_from(slot).map_err(|_| "too many universal caller aliases")?,
                )
                .map_err(|_| "invalid universal caller alias")?;
        }
        Ok(graph)
    }
}
impl Checker<'_> {
    pub(super) fn prepare_universal_call_inputs(&mut self, node: NodeId) -> Result<(), Error> {
        if self.state.call_types.caller_inputs().is_some() {
            return Ok(());
        }
        let count = self.state.call_types.universal_aliases().len();
        if u32::try_from(count).ok() != Some(self.state.body_binders) {
            return Err(self.error(node, "universal caller input aliases are incomplete"));
        }
        let alias = BodyAliases::construction_bound(count).map_err(|e| self.error(node, e))?;
        let (bytes, work) = OriginalInputs::preflight(self.facts, self.state.environment_owner);
        if bytes.saturating_add(alias.bytes) > 64 * 1024 * 1024 {
            return Err(self.error(node, "universal caller input storage exceeds 64 MB"));
        }
        self.state
            .infer
            .account_work(work.saturating_add(alias.work))
            .map_err(|e| self.error(node, e))?;
        let original = OriginalInputs::new(
            self.facts,
            self.state.body,
            self.state.environment_owner,
            self.state.body_binders,
        )
        .map_err(|e| self.error(node, e))?;
        let aliases = BodyAliases::new(self.state.call_types.universal_arguments())
            .map_err(|e| self.error(node, e))?;
        self.state
            .call_types
            .retain_caller_inputs(CallerInputs { original, aliases });
        Ok(())
    }
}

pub(super) fn foreign_regions(
    facts: &Declarations,
    source: NodeKey,
    proof: &Obligations,
    caller: Option<&CallerInputs>,
    graph: &mut Constraints<'_>,
    queries: &mut Queries<'_>,
) -> Result<(), String> {
    let mut seen = HashSet::default();
    let mut register = |region: RegionId| -> Result<(), String> {
        if region == RegionId::STATIC
            || region.bound_parts().is_some()
            || proof.owns_region(region)
            || caller.is_some_and(|c| c.aliases.slot(region).is_some())
            || !seen.insert(region)
        {
            return Ok(());
        }
        if facts.body_regions.get(source, region).is_none() {
            return Err("universal call proof has a foreign body lifetime".into());
        }
        graph.variable(region).map_err(|e| e.to_string())
    };
    for (_, _, _, block) in proof.blocks() {
        for &(a, b) in block.equalities.iter().chain(&block.outlives) {
            register(a)?;
            register(b)?;
        }
        for &(ty, region) in &block.types {
            register(region)?;
            for &component in queries.components(ty)?.scalars.regions {
                register(component)?;
            }
        }
    }
    Ok(())
}
