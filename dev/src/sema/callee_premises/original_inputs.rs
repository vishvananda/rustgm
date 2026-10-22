//! Prepared pattern callers retain one immutable declared header, never body goals.
use super::{Declarations, Inputs};
use crate::{
    resolve::{model::DefId, unit::NodeKey},
    support::retained::Allocation,
    types::{
        Kind, TypeId,
        arguments::Argument,
        outlives::Pair,
        parameters::{ParameterId, clauses::Clause},
    },
};
pub(in crate::sema) struct OriginalInputs {
    source: NodeKey,
    universals: u32,
    signature: Option<TypeId>,
    parameters: Vec<ParameterId>,
    assumptions: Vec<Pair>,
    type_assumptions: Vec<Clause>,
    require_result: bool,
}
impl OriginalInputs {
    pub(in crate::sema) fn preflight(facts: &Declarations, owner: Option<DefId>) -> (usize, usize) {
        let Some(owner) = owner else {
            return (size_of::<Self>(), 1);
        };
        let count = facts
            .generics
            .parameters_in_scope(owner)
            .count()
            .saturating_add(facts.generics.predicates_in_scope(owner).count());
        let bytes = size_of::<Self>()
            .saturating_add(count.saturating_mul(4).saturating_mul(size_of::<Clause>()))
            .saturating_add(
                facts
                    .lifetimes
                    .functions
                    .get(&owner)
                    .map_or(0, |s| s.bounds.len())
                    .saturating_add(facts.associated.schema(owner).map_or(0, |s| s.bounds.len()))
                    .saturating_mul(4 * size_of::<Pair>()),
            );
        (bytes, count.saturating_add(1))
    }
    pub(in crate::sema) fn identity(
        facts: &Declarations,
        source: NodeKey,
        owner: Option<DefId>,
    ) -> Result<(Option<TypeId>, u32), &'static str> {
        Ok(if let Some(closure) = facts.closures.get(source) {
            if closure.owner != owner {
                return Err("prepared input closure owner changed");
            }
            (
                Some(closure.execution.map_or(closure.signature, |e| e.signature)),
                closure
                    .execution
                    .map_or(closure.universals, |e| e.universals),
            )
        } else if let Some(inline) = facts
            .inline_constants
            .as_deref()
            .and_then(|r| r.get(source))
        {
            if inline.declaration != owner {
                return Err("prepared input inline owner changed");
            }
            (None, inline.universals)
        } else if let Some(function) = owner
            .and_then(|d| facts.function_indices.get(&d))
            .and_then(|&n| facts.functions.get(n))
        {
            if function.body != Some(source) {
                return Err("prepared input function source changed");
            }
            (
                Some(function.signature),
                facts
                    .lifetimes
                    .functions
                    .get(&function.definition)
                    .map_or(0, |s| s.total),
            )
        } else if let Some(initializer) = owner
            .and_then(|d| facts.initializer_index(d))
            .and_then(|n| facts.initializers.get(n))
        {
            if initializer.semantic_source() != source {
                return Err("prepared input initializer source changed");
            }
            (
                None,
                facts
                    .associated
                    .schema(initializer.definition)
                    .map_or(0, |s| s.captures),
            )
        } else {
            return Err("prepared caller has no actual input header");
        })
    }
    pub(in crate::sema) fn new(
        facts: &mut Declarations,
        source: NodeKey,
        owner: Option<DefId>,
        universals: u32,
    ) -> Result<Self, &'static str> {
        let (signature, count) = Self::identity(facts, source, owner)?;
        if count != universals {
            return Err("prepared caller input binder changed");
        }
        let mut result = Self {
            source,
            universals,
            signature,
            parameters: Vec::new(),
            assumptions: Vec::new(),
            type_assumptions: Vec::new(),
            require_result: facts
                .nested_outputs
                .as_deref()
                .is_some_and(|outputs| outputs.get(&source).is_some()),
        };
        if let Some(owner) = owner {
            for parameter in facts.generics.parameters_in_scope(owner) {
                facts.body_work.canonical_steps += 1;
                if let Argument::Type(ty) = parameter.argument {
                    let Some(Kind::Parameter(id)) = facts.types.kind(ty) else {
                        return Err("invalid prepared caller type parameter");
                    };
                    result.parameters.push(id);
                }
            }
            result.assumptions.extend(
                facts
                    .lifetimes
                    .functions
                    .get(&owner)
                    .map(|s| s.bounds.as_slice())
                    .or_else(|| facts.associated.schema(owner).map(|s| s.bounds.as_slice()))
                    .into_iter()
                    .flatten()
                    .copied(),
            );
            facts.body_work.canonical_steps += result.assumptions.len();
            for predicate in facts.generics.predicates_in_scope(owner) {
                facts.body_work.canonical_steps += 1;
                if let Some(clause) = predicate.value.caller_outlives(&facts.types)? {
                    result.type_assumptions.push(clause);
                }
            }
        }
        result.view().prepare(facts)?;
        Ok(result)
    }
    pub(in crate::sema) fn view(&self) -> Inputs<'_> {
        Inputs {
            source: self.source,
            universals: self.universals,
            signature: self.signature,
            parameters: &self.parameters,
            assumptions: &self.assumptions,
            type_assumptions: &self.type_assumptions,
            require_result: self.require_result,
        }
    }
    pub(in crate::sema) fn allocation(&self) -> Allocation {
        let mut total = Allocation::default();
        total.boxed(self);
        total.vector(&self.parameters);
        total.vector(&self.assumptions);
        total.vector(&self.type_assumptions);
        total
    }
}
