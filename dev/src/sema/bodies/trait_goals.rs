//! Resolve call goals before inference completion, retaining actual assumption evidence.
use super::{Checker, Error, NodeId};
use crate::sema::traits::assumptions::{Assumptions, Evidence};
use std::collections::HashMap;

impl Checker<'_> {
    pub(super) fn infer_trait_obligations(
        &mut self,
        body: NodeId,
    ) -> Result<HashMap<usize, Evidence>, Error> {
        let mut pending: Vec<_> = self
            .call_types
            .obligations
            .iter()
            .enumerate()
            .filter_map(|(index, (_, predicate))| predicate.trait_arguments().map(|_| index))
            .collect();
        let mut evidence = HashMap::new();
        if pending.is_empty() {
            return Ok(evidence);
        }
        let assumptions = self.trait_assumptions(body)?;
        while !pending.is_empty() {
            let before = pending.len();
            let mut next = Vec::new();
            for index in pending {
                let (node, predicate) = &self.call_types.obligations[index];
                if let Some(proof) = assumptions
                    .prove(&self.facts.types, &mut self.infer, predicate)
                    .map_err(|e| self.error(*node, e))?
                {
                    evidence.insert(index, proof);
                } else {
                    next.push(index);
                }
            }
            if next.len() == before {
                let node = self.call_types.obligations[next[0]].0;
                return Err(self.error(node,"trait obligation requires an unambiguous matching assumption or implementation"));
            }
            pending = next;
        }
        Ok(evidence)
    }

    fn trait_assumptions(&mut self, node: NodeId) -> Result<Assumptions, Error> {
        let predicates: Vec<_> = self
            .method_environment
            .iter()
            .flat_map(|environment| environment.trait_predicates())
            .copied()
            .collect();
        let mut assumptions = Assumptions::default();
        for predicate in predicates {
            let predicate = if predicate
                .escaping_binders(&self.facts.types)
                .map_err(|e| self.error(node, e))?
                != 0
            {
                self.alias_predicate(node, predicate)?
            } else {
                predicate
            };
            assumptions
                .insert(&self.facts.types, predicate)
                .map_err(|e| self.error(node, e))?;
        }
        Ok(assumptions)
    }
}
