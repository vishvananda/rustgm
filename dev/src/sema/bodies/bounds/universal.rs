//! Completed bodies share universal queries, preserving allocation and diagnostic ownership.
use super::{Checker, Error, NodeId};
use crate::{
    sema::{
        model::LifetimeProof,
        nominals::{
            self,
            traits::{QuantifiedGoal, QuantifiedQueries},
        },
        traits::{assumptions::Assumptions, obligations::Reservation, selection::Caller},
    },
    types::{
        RegionId,
        infer::Context,
        lifetime_proof::Obligations,
        parameters::{clauses::Clause, environment::Environment},
        substitute::Bound,
    },
};
use std::sync::Arc;
#[cfg(test)]
#[path = "../../../../tests/invariants/completed_body_requirements.rs"]
mod body_tests;

#[derive(Default)]
pub(in crate::sema::bodies) struct Requirements {
    pub(super) ordinary: nominals::Requirements,
    quantified: Option<Box<Shared>>,
}
struct Shared {
    queries: QuantifiedQueries,
    context: Context,
    evidence: LifetimeProof,
    reservation: Reservation,
    assumptions: Arc<Assumptions>,
    environment: Option<Arc<Environment>>,
}

impl Checker<'_> {
    pub(in crate::sema::bodies) fn prove_quantified_requirement(
        &mut self,
        node: NodeId,
        clause: Clause,
        checked: &mut Requirements,
    ) -> Result<(), Error> {
        let source_node = self.key(node);
        let Clause::ForAll(id) = self.alias_clause(node, clause)? else {
            return Err(self.error(node, "quantified body requirement lost its binder"));
        };
        if checked.quantified.is_none() {
            let assumptions = self.trait_assumptions(node)?.unwrap_or_default();
            let first = self.trait_region_start(node)?;
            let mut reservation = Reservation::default();
            // Even an empty first query reserves its frontier, so all subsequent
            // queries use the same resume contract and exclude intervening IDs.
            reservation
                .reserve(self.facts, self.state.body, source_node, first..first)
                .map_err(|e| self.error(node, e))?;
            checked.quantified = Some(Box::new(Shared {
                queries: QuantifiedQueries::default(),
                context: Context::default(),
                evidence: LifetimeProof::new(self.state.body, Obligations::new(first)),
                reservation,
                assumptions,
                environment: self.state.method_environment.clone(),
            }));
        }
        let shared = checked
            .quantified
            .as_mut()
            .ok_or_else(|| self.error(node, "missing quantified body query state"))?;
        let first = shared
            .reservation
            .resume(self.facts)
            .map_err(|e| self.error(node, e))?;
        let proof = &mut shared.evidence.proof;
        proof.resume_at(first).map_err(|e| self.error(node, e))?;
        let checkpoint = proof.source_checkpoint();
        let goal = QuantifiedGoal {
            id,
            closed: false,
            availability: self.availability,
            caller: Caller {
                assumptions: &shared.assumptions,
                environment: shared.environment.as_ref(),
                first_region: first,
                retain_sized: false,
            },
        };
        if !shared
            .queries
            .prove(self.facts, &mut shared.context, proof, goal)
            .map_err(|e| self.error(node, e))?
        {
            return Err(self.error(
                node,
                "quantified body requirement needs a matching assumption or implementation",
            ));
        }
        let source = u32::try_from(shared.evidence.sources.len().saturating_add(1))
            .map_err(|_| self.error(node, "too many quantified body requirement sources"))?;
        proof
            .label_sources_since(checkpoint, source)
            .map_err(|e| self.error(node, e))?;
        shared.evidence.sources.push(source_node);
        shared
            .reservation
            .reserve(
                self.facts,
                self.state.body,
                source_node,
                proof.regions.clone(),
            )
            .map_err(|e| self.error(node, e))
    }
}

impl Requirements {
    pub(in crate::sema::bodies) fn finish(self, checker: &mut Checker<'_>) -> Result<(), Error> {
        if let Some(mut shared) = self.quantified {
            // The last ordinary requirement may have allocated unrelated IDs.
            let first = shared
                .reservation
                .resume(checker.facts)
                .map_err(|e| checker.error(checker.state.body.node, e))?;
            shared
                .evidence
                .proof
                .resume_at(first)
                .map_err(|e| checker.error(checker.state.body.node, e))?;
            shared.close_captures(checker)?;
            shared
                .reservation
                .publish_sourced(shared.evidence, checker.facts, checker.state.body)
                .map_err(|e| checker.error(checker.state.body.node, e))?;
        }
        Ok(())
    }
}

impl Shared {
    fn close_captures(&mut self, checker: &mut Checker<'_>) -> Result<(), Error> {
        let aliases = checker.state.call_types.universal_arguments();
        if aliases.is_empty() {
            return Ok(());
        }
        self.context
            .account_work(aliases.len())
            .map_err(|e| checker.error(checker.state.body.node, e))?;
        let replacements = aliases
            .into_iter()
            .enumerate()
            .map(|(index, alias)| {
                let index =
                    u16::try_from(index).map_err(|_| "too many body lifetime parameters")?;
                Ok((alias, RegionId::bound(0, index)?))
            })
            .collect::<Result<_, &'static str>>()
            .map_err(|e| checker.error(checker.state.body.node, e))?;
        // Restore the body's declared universal identities once, after all queries.
        // Other allocated IDs keep their identity; this grants no ownership over
        // them. The proof's intervals still exclude foreign allocations.
        let mut fold =
            Bound::restore_owned_regions(replacements, 1..self.evidence.proof.regions.end);
        self.evidence
            .proof
            .substitute(&mut checker.facts.types, &mut fold)
            .map_err(|e| checker.error(checker.state.body.node, e))?;
        self.context
            .account_work(fold.steps)
            .map_err(|e| checker.error(checker.state.body.node, e))
    }
}
