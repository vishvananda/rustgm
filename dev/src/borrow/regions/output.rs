//! Conditional child proofs are consumed once by the exact enclosing body.
use super::{Analysis, Body, Error, check};
use crate::support::hash::HashMap;
use crate::{
    body::{
        ir::{Body as Cfg, BodyOrigin},
        regions::{Contract, Contracts, Kind, Obligation},
    },
    types::{
        RegionId, Store,
        outlives::Queries,
        region_graph,
        relations::{Constraints, Proof},
        variance::Variances,
    },
};
use std::collections::VecDeque;

pub struct Summary {
    pub(super) parent: BodyOrigin,
    pub(super) requirements: Vec<Obligation>,
    conditions: Option<std::sync::Arc<[crate::types::outlives::Pair]>>,
    source: crate::body::ir::BodyOrigin,
    key: crate::body::ir::BodyKey,
    link: std::sync::Arc<crate::body::regions::OutputLink>,
}
impl Summary {
    pub(super) fn requires_parent(&self) -> bool {
        !self.requirements.is_empty() || self.link.formation.is_some()
    }
    pub(crate) fn execution_conditions(
        &self,
        input: &Contract,
    ) -> Result<Option<&std::sync::Arc<[crate::types::outlives::Pair]>>, &'static str> {
        if !self.matches(input) {
            return Err("execution conditions belong to another source proof");
        }
        Ok(self.conditions.as_ref())
    }
    pub(crate) fn parent_goals(
        &self,
        input: &Contract,
    ) -> Result<Option<(BodyOrigin, &[Obligation])>, &'static str> {
        if !self.matches(input) {
            return Err("parent goals belong to another checked source contract");
        }
        Ok(self
            .requires_parent()
            .then_some((self.parent, &self.requirements)))
    }
    fn matches(&self, input: &Contract) -> bool {
        self.source == input.source
            && self.key == input.key
            && input
                .output
                .as_ref()
                .is_some_and(|link| std::sync::Arc::ptr_eq(link, &self.link))
    }
}

pub(super) fn finish(
    constraints: Constraints<'_>,
    input: &Contract,
) -> Result<(Proof, Option<Summary>), region_graph::Error> {
    let Some(link) = &input.output else {
        return constraints.prove().map(|proof| (proof, None));
    };
    let mut interface = Vec::with_capacity(link.bridges.len());
    let mut mapping = HashMap::with_capacity_and_hasher(
        link.bridges.len(),
        crate::support::hash::BuildFxHasher::default(),
    );
    for bridge in &link.bridges {
        let region = RegionId::bound(0, bridge.slot).map_err(|_| region_graph::Error::Identity)?;
        interface.push(region);
        if mapping.insert(region, bridge.parent).is_some() {
            return Err(region_graph::Error::Identity);
        }
    }
    let (proof, conditions) = if link.bridges.is_empty() {
        (constraints.prove()?, Vec::new())
    } else {
        constraints
            .prove_output(&interface)
            .map_err(|error| match error {
                region_graph::OutputError::Constraint(error) => error,
                region_graph::OutputError::Unsupported => region_graph::Error::UnsupportedOutput,
            })?
    };
    let parent = |region| {
        if region == RegionId::STATIC {
            Ok(region)
        } else {
            mapping
                .get(&region)
                .copied()
                .ok_or(region_graph::Error::Identity)
        }
    };
    let formal =
        (!conditions.is_empty()).then(|| conditions.iter().map(|&(a, b, _)| (a, b)).collect());
    let mut requirements = conditions
        .into_iter()
        .map(|(longer, shorter, _)| {
            Ok(Obligation {
                source: input.source.into(),
                kind: Kind::Outlives((parent(longer)?, parent(shorter)?)),
            })
        })
        .collect::<Result<Vec<_>, region_graph::Error>>()?;
    if let Some(formation) = &link.formation {
        requirements.extend(formation.inputs.iter().map(|formation| Obligation {
            source: input.source.into(),
            kind: Kind::WellFormed(formation.parent),
        }));
    }
    Ok((
        proof,
        Some(Summary {
            conditions: formal,
            parent: link.parent,
            requirements,
            source: input.source,
            key: input.key,
            link: std::sync::Arc::clone(link),
        }),
    ))
}

struct Order {
    children: Vec<Vec<usize>>,
    parents: Vec<Option<usize>>,
    remaining: Vec<usize>,
    queue: VecDeque<usize>,
}
fn ordering(inputs: &[(&Contract, Option<&Cfg>)]) -> Result<Order, Error> {
    let error = |index: usize, message: &str| Error {
        source: Some(inputs[index].0.source.into()),
        message: message.into(),
    };
    let mut index = HashMap::with_capacity_and_hasher(
        inputs.len(),
        crate::support::hash::BuildFxHasher::default(),
    );
    for (ordinal, (input, cfg)) in inputs.iter().enumerate() {
        index
            .entry(input.source)
            .and_modify(|(_, unique): &mut (usize, bool)| *unique = false)
            .or_insert((ordinal, true));
        if input.output.is_some() && cfg.is_some_and(|body| body.instance.is_some()) {
            return Err(error(
                ordinal,
                "specialized nested output proof is not implemented",
            ));
        }
    }
    let mut children = vec![Vec::new(); inputs.len()];
    let mut parents = vec![None; inputs.len()];
    let mut remaining = vec![0; inputs.len()];
    for (child, (input, _)) in inputs.iter().enumerate() {
        if let Some(&(parent, unique)) = input
            .output
            .as_ref()
            .and_then(|link| index.get(&link.parent))
        {
            if !unique {
                return Err(error(
                    child,
                    "specialized nested output parent is not implemented",
                ));
            }
            children[parent].push(child);
            parents[child] = Some(parent);
            remaining[parent] += 1;
        }
    }
    let queue: VecDeque<_> = remaining
        .iter()
        .enumerate()
        .filter_map(|(i, &count)| (count == 0).then_some(i))
        .collect();
    Ok(Order {
        children,
        parents,
        remaining,
        queue,
    })
}

pub(super) fn prove<'a>(
    types: &Store,
    variances: &Variances,
    contracts: &Contracts<'_>,
    inputs: impl Iterator<Item = Result<(&'a Contract, Option<&'a Cfg>), Error>>,
) -> Result<Analysis, Error> {
    let inputs = inputs.collect::<Result<Vec<_>, _>>()?;
    let error = |index: usize, message: &str| Error {
        source: Some(inputs[index].0.source.into()),
        message: message.into(),
    };
    let Order {
        children,
        parents,
        mut remaining,
        mut queue,
    } = ordering(&inputs)?;
    let mut results: Vec<Option<Body>> =
        std::iter::repeat_with(|| None).take(inputs.len()).collect();
    let mut queries = Queries::new(types, contracts.nominal_requirements, contracts.callables);
    let mut visited = 0;
    while let Some(ordinal) = queue.pop_front() {
        let (input, cfg) = inputs[ordinal];
        let summaries = children[ordinal]
            .iter()
            .map(|&child| {
                results[child]
                    .as_ref()
                    .and_then(|body| body.output.as_ref())
                    .ok_or_else(|| error(child, "missing conditional child proof"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let body = check::body(types, variances, &mut queries, input, cfg, &summaries)?;
        if let Some(output) = &body.output {
            if !output.matches(input) {
                return Err(error(
                    ordinal,
                    "nested output interface changed after proof",
                ));
            }
            if parents[ordinal].is_none() && output.requires_parent() {
                return Err(error(
                    ordinal,
                    "nested output requires its checked parent contract",
                ));
            }
        }
        results[ordinal] = Some(body);
        visited += 1;
        if let Some(parent) = parents[ordinal] {
            remaining[parent] -= 1;
            if remaining[parent] == 0 {
                queue.push_back(parent);
            }
        }
    }
    if visited != inputs.len() {
        return Err(Error {
            source: None,
            message: "cyclic nested output proof dependencies".into(),
        });
    }
    let bodies = results
        .into_iter()
        .map(|body| {
            body.ok_or_else(|| Error {
                source: None,
                message: "missing completed nested output proof".into(),
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Analysis { bodies })
}

#[cfg(test)]
#[path = "../../../tests/invariants/bridge_seals.rs"]
mod tests;
