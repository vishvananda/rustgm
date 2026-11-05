//! A sealed proof retains the exact immutable templates checked under their assumptions.
mod context;
mod owners;
#[path = "templates/suspended.rs"]
mod retained;
pub(crate) use retained::{Append, Retained};
mod permission_sources;
mod preconditions;
use super::{Environment, Error, loans, regions};
use crate::support::hash::HashMap;
use crate::{
    body::{
        ir::{BodyKey, Owner},
        regions::{Contract, Contracts},
        templates::{Template, Templates},
    },
    types::{ItemId, TypeId, closure::ClosureId, variance::Variances},
};
pub(crate) use context::{Contexts, verify as verify_with_context};
use std::hash::BuildHasher;

pub struct Checked<'a> {
    templates: &'a Templates,
    state: State,
}
// The same owned proof fields move together when a scoped view is suspended.
struct State {
    future_arena: Option<crate::types::arena::Identity>,
    index: HashMap<ItemId, usize>,
    closures: HashMap<ClosureId, usize>,
    anonymous: HashMap<BodyKey, usize>,
    regions: regions::Analysis,
    loans: Vec<loans::Analysis>,
    contracts: Vec<Contract>,
}
const _: [(); 176] = [(); size_of::<State>()];

impl State {
    fn require_current_store(&self, types: &crate::types::Store) -> Result<(), &'static str> {
        if self
            .future_arena
            .as_ref()
            .is_some_and(|arena| !types.has_identity(arena))
        {
            return Err("checked future family belongs to another actual type store");
        }
        Ok(())
    }
}

fn future_arena(
    types: &crate::types::Store,
    templates: &Templates,
    anonymous: &HashMap<BodyKey, usize>,
) -> Result<Option<crate::types::arena::Identity>, Error> {
    let Some((&key, &index)) = anonymous
        .iter()
        .find(|(key, _)| matches!(key, BodyKey::AsyncBody(_)))
    else {
        return Ok(None);
    };
    let template = templates.get(index).ok_or_else(|| Error {
        node: None,
        message: "checked future owner index is stale".into(),
    })?;
    let Owner::AsyncBody {
        identity, future, ..
    } = template.body.owner
    else {
        return Err(Error {
            node: Some(template.body.source.into()),
            message: "checked future arena requires its actual original owner".into(),
        });
    };
    if key != BodyKey::AsyncBody(identity)
        || !matches!(types.kind(future), Some(crate::types::Kind::Future { identity: actual, .. }) if actual == identity)
    {
        return Err(Error {
            node: Some(template.body.source.into()),
            message: "checked future arena changed its actual owner/application".into(),
        });
    }
    types
        .future_proof_arena(future)
        .map(Some)
        .map_err(|message| Error {
            node: Some(template.body.source.into()),
            message: message.into(),
        })
}

impl Checked<'_> {
    /// Check the store used by this family's real deferred-body verification.
    /// This read-only query grants no constructor, selected body or CFG permission.
    /// # Errors
    /// Rejects an absent deferred family or a foreign/replaced type store.
    pub fn require_future_store(&self, types: &crate::types::Store) -> Result<(), &'static str> {
        self.future_store(types).map(|_| ())
    }
    pub(crate) fn future_store(
        &self,
        types: &crate::types::Store,
    ) -> Result<&crate::types::arena::Identity, &'static str> {
        self.state.require_current_store(types)?;
        self.state
            .future_arena
            .as_ref()
            .ok_or("checked family has no actual deferred source body")
    }
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
        for (&identity, &index) in &self.state.closures {
            let contract = &self.state.contracts[index];
            let template = self
                .templates
                .get(index)
                .ok_or("checked closure template index is stale")?;
            if !matches!(
                template.body.owner,
                Owner::Closure {
                    execution: Some(_),
                    ..
                }
            ) {
                continue;
            }
            let conditions = match &self.state.regions.bodies[index].output {
                Some(summary) => summary.execution_conditions(contract)?,
                None if contract.output.is_some() => {
                    return Err("closure conditions require their checked output summary");
                }
                None => None,
            };
            let source = template.body.source;
            let kind = types
                .closure_kind(identity)
                .ok_or("closure conditions require complete call permission")?;
            let borrowed_receiver = kind != crate::types::closure::CallKind::Once;
            let universals = contract
                .universals
                .checked_sub(u32::from(borrowed_receiver))
                .ok_or("captured source has no execution receiver lifetime")?;
            let conditions = collector.source(contract, universals, conditions)?;
            publish(
                identity,
                source
                    .source()
                    .ok_or("imported closure conditions require their imported header catalog")?,
                conditions.as_ref(),
            )?;
        }
        Ok(collector.steps())
    }
    /// Transport data is derived from an exact checked original, never a runtime permit.
    pub(crate) fn conditional_sources(
        &self,
        sources: &mut crate::body::conditional_inputs::Sources,
    ) -> Result<(), &'static str> {
        for (&key, &index) in &self.state.anonymous {
            sources.steps = sources.steps.saturating_add(1);
            let input = &self.state.contracts[index];
            if key != input.key {
                return Err("conditional original index changed its contract owner");
            }
            let Some(summary) = &self.state.regions.bodies[index].output else {
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
            .state
            .anonymous
            .get(&crate::body::ir::BodyKey::Anonymous(source))
            .ok_or("conditional child has no checked executable original")?;
        let input = &self.state.contracts[index];
        let summary = self.state.regions.bodies[index]
            .output
            .as_ref()
            .ok_or("conditional original has no checked output summary")?;
        summary.parent_goals(input)?;
        Ok((input, summary))
    }
    pub(crate) fn is_empty(&self) -> bool {
        self.templates.len() == 0
    }
    #[must_use]
    pub fn get(&self, item: ItemId) -> Option<&Template> {
        self.state
            .index
            .get(&item)
            .and_then(|&index| self.templates.get(index))
    }
    #[must_use]
    pub const fn regions(&self) -> &regions::Analysis {
        &self.state.regions
    }
    #[must_use]
    pub fn loans(&self) -> &[loans::Analysis] {
        &self.state.loans
    }
    #[must_use]
    pub fn input(&self, item: ItemId) -> Option<&Contract> {
        self.state
            .index
            .get(&item)
            .map(|&index| &self.state.contracts[index])
    }
    /// Borrow the exact function, contract and full region result sealed together.
    pub(crate) fn function_proof(
        &self,
        item: ItemId,
    ) -> Option<(&Template, &Contract, &regions::Body)> {
        let &index = self.state.index.get(&item)?;
        Some((
            self.templates.get(index)?,
            &self.state.contracts[index],
            &self.state.regions.bodies[index],
        ))
    }
    pub(crate) fn async_body(
        &self,
        identity: crate::types::opaque::OpaqueId,
    ) -> Option<(&Template, &Contract, &regions::Body)> {
        let key = BodyKey::AsyncBody(identity);
        let &index = self.state.anonymous.get(&key)?;
        let template = self.templates.get(index)?;
        let contract = &self.state.contracts[index];
        (template.body.key() == key && contract.key == key).then_some((
            template,
            contract,
            &self.state.regions.bodies[index],
        ))
    }
    #[must_use]
    pub fn closure(&self, identity: ClosureId) -> Option<(&Template, &Contract)> {
        let &index = self.state.closures.get(&identity)?;
        Some((self.templates.get(index)?, &self.state.contracts[index]))
    }
    pub(super) fn contract(&self, body: &crate::body::ir::Body) -> Option<&regions::Body> {
        let &index = match body.key() {
            BodyKey::Declaration(definition) => self.state.index.get(&ItemId(definition.0)),
            BodyKey::Closure(identity) => self.state.closures.get(&identity),
            key @ (BodyKey::Anonymous(_) | BodyKey::Imported(_) | BodyKey::AsyncBody(_)) => {
                self.state.anonymous.get(&key)
            }
            BodyKey::AsyncApplication(_)
            | BodyKey::FutureEntry(_)
            | BodyKey::InitializerInstance(_)
            | BodyKey::AnonymousInstance(_)
            | BodyKey::ClosureAdapter(_) => None,
        }?;
        (self.templates.get(index)?.body.key() == body.key())
            .then(|| &self.state.regions.bodies[index])
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
    position: impl Fn(
        TypeId,
        crate::body::drops::MemberQuery,
    ) -> Result<crate::body::drops::MemberPosition, &'static str>,
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
    let mut inputs: HashMap<_, _> = templates
        .inputs(&contracts.bodies)
        .map(|c| (c.key, c))
        .collect();
    let contracts = templates
        .iter()
        .map(|template| {
            inputs
                .remove(&template.body.key())
                .cloned()
                .ok_or_else(|| Error {
                    node: Some(template.body.source.into()),
                    message: "checked template lost its input contract".into(),
                })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Checked {
        templates,
        state: State {
            future_arena: future_arena(environment.types, templates, &anonymous)?,
            index,
            closures,
            anonymous,
            regions,
            loans,
            contracts,
        },
    })
}
