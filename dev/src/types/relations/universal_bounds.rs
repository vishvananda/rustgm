//! Universal outlives premises use indexed complete inputs and conditional matching.
mod recipes;
#[cfg(test)]
#[path = "../../../tests/invariants/premise_matching.rs"]
mod tests;
use super::{
    Constraints, Error, Region, RegionId, Scope,
    identity_guard::View,
    persistent::{Id, Map, Node},
};
use crate::types::{
    Store,
    outlives::ResultId,
    parameters::{
        clauses::QuantifiedId,
        predicates::{Bound, Predicate},
    },
    region_graph::VerificationId,
    structural::Shapes,
};
use std::collections::{HashMap, HashSet};

#[derive(Default)]
pub(super) struct State {
    candidates: HashMap<(ResultId, u64), Vec<QuantifiedId>>,
    seen: HashSet<Predicate>,
    shapes: Shapes,
    recipes: recipes::Recipes,
    bindings: Map<Region>,
    guards: Map<VerificationId>,
    conditions: HashMap<Id, VerificationId>,
}
impl State {
    pub fn steps(&self) -> usize {
        self.shapes
            .steps
            .saturating_add(self.recipes.steps)
            .saturating_add(self.bindings.steps)
            .saturating_add(self.guards.steps)
    }
    pub fn assume(&mut self, types: &Store, id: QuantifiedId) -> Result<(), Error> {
        let value = types.quantified(id).ok_or(Error::Identity)?;
        // An unused top-level slot does not change a caller premise. This
        // deduplication neither merges requirement scopes nor changes the
        // separate definition-time classification of empty/unused binders.
        if self.seen.contains(&value.predicate) {
            return Ok(());
        }
        let Some(identity) = types.result_identity(value.predicate.ty) else {
            return Err(Error::Identity);
        };
        if !matches!(value.predicate.bound, Bound::Outlives(_)) {
            return Err(Error::Identity);
        }
        let shape = self.shapes.get(types, value.predicate.ty)?;
        self.candidates
            .entry((identity, shape))
            .or_default()
            .push(id);
        self.seen.insert(value.predicate);
        Ok(())
    }
    pub fn alternatives(
        &mut self,
        proof: &mut Constraints<'_>,
        view: View,
        shorter: Region,
    ) -> Result<Vec<VerificationId>, Error> {
        let Some(identity) = proof.types.result_identity(view.projection.ty) else {
            return Err(Error::Identity);
        };
        let key = (identity, self.shapes.get(proof.types, view.projection.ty)?);
        let mut result = Vec::new();
        for index in 0..self.candidates.get(&key).map_or(0, Vec::len) {
            proof.steps = proof.steps.saturating_add(1);
            if proof.steps > proof.limit {
                return Err(Error::Budget);
            }
            let clause = proof
                .types
                .quantified(self.candidates[&key][index])
                .ok_or(Error::Identity)?;
            let Bound::Outlives(bound) = clause.predicate.bound else {
                return Err(Error::Identity);
            };
            let Some(summary) = self.recipes.run(
                proof,
                view,
                clause.predicate.ty,
                &mut self.bindings,
                &mut self.guards,
            )?
            else {
                continue;
            };
            let longer = match bound.bound_parts() {
                Some((0, slot)) => self
                    .bindings
                    .get(summary.bindings, u32::from(slot))?
                    .unwrap_or(Region::STATIC),
                Some((depth, slot)) => proof.region(
                    RegionId::bound(depth - 1, slot).map_err(|_| Error::Identity)?,
                    Scope(0),
                )?,
                None => proof.region(bound, Scope(0))?,
            };
            let condition = self.condition(proof, summary.guards)?;
            let outlives = proof.graph.verify_outlives(longer, shorter)?;
            result.push(proof.graph.verify_all(&[condition, outlives])?);
        }
        Ok(result)
    }
    /// The radix tree has at most 32 levels; each immutable conjunction is built once.
    fn condition(
        &mut self,
        proof: &mut Constraints<'_>,
        root: Id,
    ) -> Result<VerificationId, Error> {
        if let Some(&found) = self.conditions.get(&root) {
            return Ok(found);
        }
        let condition = if root == Id::EMPTY {
            proof.graph.verify_all(&[])?
        } else {
            match self.guards.node(root)? {
                Node::Leaf(_, condition) => condition,
                Node::Branch { left, right, .. } => {
                    let left = self.condition(proof, left)?;
                    let right = self.condition(proof, right)?;
                    proof.graph.verify_all(&[left, right])?
                }
            }
        };
        self.conditions.insert(root, condition);
        Ok(condition)
    }
}
