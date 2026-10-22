//! Hidden result inference belongs exclusively to the defining body.
use super::{Checker, Error, NodeId, TermId};
use crate::types::{
    infer::{Instantiation, Solved},
    parameters::{clauses::Clause, predicates::Bound},
};

/// Inferred representations remain private until every defining-body requirement completes.
#[derive(Default)]
pub(super) struct HiddenResults {
    values: Vec<Hidden>,
    result: Option<(super::DefId, super::TypeId)>,
}
struct Hidden {
    id: crate::types::opaque::OpaqueId,
    source: super::NodeKey,
    inferred: super::TypeId,
    free_regions: Vec<crate::types::RegionId>,
    closed: Option<super::TypeId>,
}

impl Checker<'_> {
    pub(super) fn prepare_opaque_results(&mut self, node: NodeId) -> Result<(), Error> {
        self.state.opaque_terms.clear();
        self.state.opaque_callables = None;
        self.state.result_term = None;
        let ids = self
            .state
            .environment_owner
            .and_then(|owner| self.facts.opaques.owners.get(&owner));
        let Some(ids) = ids else {
            return Ok(());
        };
        let mut substitution =
            Instantiation::opaque_results(&mut self.state.infer, &self.facts.types, ids)
                .map_err(|e| self.error(node, e))?;
        let result = self
            .state
            .infer
            .instantiate_with(&mut self.facts.types, self.state.result, &mut substitution)
            .map_err(|e| self.error(node, e))?;
        self.state.opaque_terms.extend(substitution.opaque_terms());
        self.state.opaque_terms.sort_unstable_by_key(|(id, _)| id.0);
        for index in 0..self.state.opaque_terms.len() {
            let (id, term) = self.state.opaque_terms[index];
            let application = self
                .facts
                .opaques
                .declarations
                .get(&id)
                .ok_or_else(|| self.error(node, "missing defining opaque application"))?
                .application;
            let application = self.alias_type(node, application)?;
            self.state
                .infer
                .define_opaque(&self.facts.types, application, term)
                .map_err(|e| self.error(node, e))?;
            // Callable result bounds provide input context before nested closure
            // bodies finish. They are hints, not early proof obligations: the
            // hidden representation must still prove every bound after inference.
            let bounds = self.facts.opaques.declarations[&id].bounds.clone();
            for clause in bounds {
                self.declared_closure_signature(node, id, term, clause)?;
            }
        }
        self.state.result_term = Some(result);
        Ok(())
    }

    pub(super) fn result_term(&mut self, node: NodeId) -> Result<TermId, Error> {
        if let Some(term) = self.state.result_term {
            Ok(term)
        } else {
            self.known(node, self.state.result)
        }
    }

    pub(super) fn check_opaque_results(&mut self, solved: &Solved) -> Result<HiddenResults, Error> {
        let mut results = HiddenResults::default();
        if self.state.opaque_terms.is_empty() {
            return Ok(results);
        }
        let mut closing = self.hidden_region_closure(solved)?;
        for index in 0..self.state.opaque_terms.len() {
            let (id, term) = self.state.opaque_terms[index];
            let hidden = solved.types[term.0 as usize];
            let declaration = self.facts.opaques.declarations.get(&id).ok_or_else(|| {
                self.error(self.state.body.node, "missing opaque result declaration")
            })?;
            let source = declaration.source;
            let bounds = declaration.bounds.clone();
            let mut substitution = crate::types::substitute::Bound::defining_results(
                &self.facts.types,
                crate::support::hash::HashMap::from_iter([(declaration.application, hidden)]),
            )
            .map_err(|e| self.error(source.node, e))?;
            self.prove_bound(source.node, hidden, Bound::Sized)?;
            let mut checked = super::bounds::Requirements::default();
            for clause in bounds {
                let clause = substitution
                    .clause(&mut self.facts.types, clause)
                    .map_err(|e| self.error(source.node, e))?;
                match clause {
                    Clause::Predicate(value) => {
                        self.prove_bound(source.node, value.ty, value.bound)?;
                    }
                    Clause::ForAll(_) => {
                        self.prove_quantified_requirement(source.node, clause, &mut checked)?;
                    }
                }
            }
            checked.finish(self)?;
            let inferred = hidden;
            let mut uses = crate::types::bound_uses::Uses::including_free();
            uses.collect(&self.facts.types, hidden)
                .map_err(|e| self.error(source.node, e))?;
            let mut free_regions: Vec<_> = uses.free_regions().collect();
            free_regions.sort_unstable_by_key(|r| r.0);
            let hidden = if free_regions.is_empty() {
                Some(
                    closing
                        .apply(&mut self.facts.types, hidden)
                        .map_err(|e| self.error(source.node, e))?,
                )
            } else {
                None
            };
            results.values.push(Hidden {
                id,
                source,
                inferred,
                free_regions,
                closed: hidden,
            });
        }
        if let Some(term) = self.state.result_term {
            let owner = self.state.environment_owner.ok_or_else(|| {
                self.error(self.state.body.node, "opaque body has no defining owner")
            })?;
            results.result = Some((owner, solved.types[term.0 as usize]));
        }
        Ok(results)
    }

    pub(super) fn publish_hidden_results(&mut self, results: HiddenResults) -> Result<(), Error> {
        for value in results.values {
            let declaration = self
                .facts
                .opaques
                .declarations
                .get_mut(&value.id)
                .ok_or_else(|| {
                    super::Declarations::error(
                        self.report,
                        value.source,
                        "missing opaque declaration",
                    )
                })?;
            if let Some(hidden) = value.closed {
                declaration
                    .check_hidden_captures(&self.facts.types, hidden)
                    .map_err(|e| super::Declarations::error(self.report, value.source, e))?;
            }
            declaration.inferred = Some(value.inferred);
            declaration.free_regions = value.free_regions;
            declaration.hidden = value.closed;
        }
        if let Some((owner, result)) = results.result {
            self.facts.opaques.body_results.insert(owner, result);
        }
        Ok(())
    }

    fn hidden_region_closure(
        &self,
        solved: &Solved,
    ) -> Result<crate::types::substitute::Bound, Error> {
        let error = |message| self.error(self.state.body.node, message);
        let mut equations = crate::types::substitute::RegionEquations::default();
        equations
            .extend(solved.region_equalities.iter().copied())
            .map_err(error)?;
        let mut closing = crate::types::substitute::Bound::close_projection_regions(
            1,
            0,
            crate::support::hash::HashMap::default(),
        )
        .map_err(error)?;
        closing
            .exchange_projection_equations(&mut equations)
            .map_err(error)?;
        Ok(closing)
    }
}
