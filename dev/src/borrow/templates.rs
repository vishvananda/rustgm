//! A sealed proof retains the exact immutable templates checked under their assumptions.
mod context;
mod owners;
mod preconditions;
use super::{Environment, Error, loans, regions};
use crate::support::hash::HashMap;
use crate::{
    body::{
        ir::{BodyKey, Owner, Selection},
        regions::{Contract, Contracts},
        templates::{Template, Templates},
    },
    types::{ItemId, TypeId, closure::ClosureId, variance::Variances},
};
pub(crate) use context::{Contexts, verify as verify_with_context};
use std::hash::BuildHasher;

pub struct Checked<'a> {
    templates: &'a Templates,
    index: HashMap<ItemId, usize>,
    closures: HashMap<ClosureId, usize>,
    anonymous: HashMap<BodyKey, usize>,
    regions: regions::Analysis,
    loans: Vec<loans::Analysis>,
    contracts: Vec<Contract>,
}
impl Checked<'_> {
    /// Publish only conditions from these exact, fully checked source contracts.
    /// They grant no caller permission; selection must require their actual images.
    pub(crate) fn visit_closure_conditions(
        &self,
        types: &crate::types::Store,
        bounds: &crate::types::outlives::Bounds,
        callables: &crate::types::callable::Templates,
        mut publish: impl FnMut(
            ClosureId,
            crate::body::ir::NodeKey,
            Option<&std::sync::Arc<[crate::types::outlives::Pair]>>,
        ) -> Result<(), &'static str>,
    ) -> Result<usize, &'static str> {
        let mut collector = preconditions::Collector::new(types, bounds, callables);
        for (&identity, &index) in &self.closures {
            let contract = &self.contracts[index];
            if !matches!(
                self.templates.bodies[index].body.owner,
                Owner::Closure {
                    execution: Some(_),
                    ..
                }
            ) {
                continue;
            }
            let conditions = match &self.regions.bodies[index].output {
                Some(summary) => summary.execution_conditions(contract)?,
                None if contract.output.is_some() => {
                    return Err("closure conditions require their checked output summary");
                }
                None => None,
            };
            let source = self.templates.bodies[index].body.source;
            let kind = types
                .closure_kind(identity)
                .ok_or("closure conditions require complete call permission")?;
            let borrowed_receiver = kind != crate::types::closure::CallKind::Once;
            let universals = contract
                .universals
                .checked_sub(u32::from(borrowed_receiver))
                .ok_or("captured source has no execution receiver lifetime")?;
            let conditions = collector.source(contract, universals, conditions)?;
            publish(identity, source, conditions.as_ref())?;
        }
        Ok(collector.steps())
    }
    /// Transport data is derived from an exact checked original, never a runtime permit.
    pub(crate) fn conditional_sources(
        &self,
        sources: &mut crate::body::conditional_inputs::Sources,
    ) -> Result<(), &'static str> {
        for (&key, &index) in &self.anonymous {
            sources.steps = sources.steps.saturating_add(1);
            let input = &self.contracts[index];
            if key != input.key {
                return Err("conditional original index changed its contract owner");
            }
            let Some(summary) = &self.regions.bodies[index].output else {
                continue;
            };
            let Some((parent, goals)) = summary.parent_goals(input)? else {
                continue;
            };
            let link = input
                .output
                .as_ref()
                .ok_or("conditional original lost its link")?;
            sources.insert(
                input.source,
                parent,
                link,
                goals,
                summary.execution_conditions(input)?,
            )?;
        }
        Ok(())
    }
    pub(crate) fn conditional_original(
        &self,
        source: crate::body::ir::NodeKey,
    ) -> Result<(&Contract, &regions::OutputSummary), &'static str> {
        let index = *self
            .anonymous
            .get(&crate::body::ir::BodyKey::Anonymous(source))
            .ok_or("conditional child has no checked executable original")?;
        let input = &self.contracts[index];
        let summary = self.regions.bodies[index]
            .output
            .as_ref()
            .ok_or("conditional original has no checked output summary")?;
        summary.parent_goals(input)?;
        Ok((input, summary))
    }
    pub(crate) fn is_empty(&self) -> bool {
        self.templates.bodies.is_empty()
    }
    #[must_use]
    pub fn get(&self, item: ItemId) -> Option<&Template> {
        self.index
            .get(&item)
            .map(|&index| &self.templates.bodies[index])
    }
    #[must_use]
    pub const fn regions(&self) -> &regions::Analysis {
        &self.regions
    }
    #[must_use]
    pub fn loans(&self) -> &[loans::Analysis] {
        &self.loans
    }
    #[must_use]
    pub fn input(&self, item: ItemId) -> Option<&Contract> {
        self.index.get(&item).map(|&index| &self.contracts[index])
    }
    /// Borrow the exact function, contract and full region result sealed together.
    pub(crate) fn function_proof(
        &self,
        item: ItemId,
    ) -> Option<(&Template, &Contract, &regions::Body)> {
        let &index = self.index.get(&item)?;
        Some((
            &self.templates.bodies[index],
            &self.contracts[index],
            &self.regions.bodies[index],
        ))
    }
    #[must_use]
    pub fn closure(&self, identity: ClosureId) -> Option<(&Template, &Contract)> {
        let &index = self.closures.get(&identity)?;
        Some((&self.templates.bodies[index], &self.contracts[index]))
    }
    pub(super) fn contract(&self, body: &crate::body::ir::Body) -> Option<&regions::Body> {
        let &index = match body.key() {
            BodyKey::Declaration(definition) => self.index.get(&ItemId(definition.0)),
            BodyKey::Closure(identity) => self.closures.get(&identity),
            key @ BodyKey::Anonymous(_) => self.anonymous.get(&key),
            BodyKey::InitializerInstance(_)
            | BodyKey::AnonymousInstance(_)
            | BodyKey::ClosureAdapter(_) => None,
        }?;
        (self.templates.bodies[index].body.key() == body.key()).then(|| &self.regions.bodies[index])
    }
}

/// Check every template, including unused bodies, before permitting proof transfer.
/// The token borrows the original CFGs so they cannot change after verification.
/// # Errors
/// Rejects incomplete promotion, duplicate owners and invalid lifetime/ownership/loan proofs.
pub fn verify<'a, S: BuildHasher>(
    environment: &mut Environment<'_, S>,
    variances: &Variances,
    contracts: &Contracts<'_>,
    templates: &'a Templates,
    custom: &[ItemId],
    position: impl Fn(TypeId, Selection) -> Result<(u32, u32), &'static str>,
) -> Result<Checked<'a>, Error> {
    let owners::Indices {
        functions: index,
        closures,
        anonymous,
    } = owners::indices(templates)?;
    let regions = regions::check_templates(environment.types, variances, contracts, templates)
        .map_err(|e| Error {
            node: e.source,
            message: e.message,
        })?;
    let loans = loans::templates::verify(environment, templates, &regions, custom, position)?;
    let mut inputs: HashMap<_, _> = contracts.bodies.iter().map(|c| (c.key, c)).collect();
    let contracts = templates
        .bodies
        .iter()
        .map(|template| {
            inputs
                .remove(&template.body.key())
                .cloned()
                .ok_or_else(|| Error {
                    node: Some(template.body.source),
                    message: "checked template lost its input contract".into(),
                })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Checked {
        templates,
        index,
        closures,
        anonymous,
        regions,
        loans,
        contracts,
    })
}
