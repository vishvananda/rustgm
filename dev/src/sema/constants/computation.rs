//! A source value keeps its actual caller proof owner without a materialized permit.
use super::{Declarations, Error, InitializerKind, NodeKey, Queries, Query, TypeId};
use crate::{
    sema::values::Instance,
    types::{
        Kind, RegionId,
        arguments::{Argument, ConstKind},
    },
};

/// An opaque source-computation request, minted from a retained caller dispatcher.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SourceComputation {
    selected: Instance,
    original: NodeKey,
    caller: NodeKey,
}
impl SourceComputation {
    pub(crate) const fn selected(self) -> Instance {
        self.selected
    }
    pub(crate) const fn original(self) -> NodeKey {
        self.original
    }
    pub(crate) fn validate(
        self,
        facts: &mut Declarations,
        result: TypeId,
    ) -> Result<(), &'static str> {
        Self::schema(facts, self.selected, self.original, result)?;
        let mut nonstatic = false;
        for argument in facts
            .types
            .arguments(self.selected.arguments)
            .ok_or("missing source arguments")?
        {
            if let Argument::Lifetime(region) = argument {
                if region == RegionId::STATIC {
                    continue;
                }
                nonstatic = true;
                if region.bound_parts().is_some()
                    || !facts
                        .body_regions
                        .get(region.0 as usize - 1)
                        .is_some_and(|owner| {
                            owner.body == self.caller && owner.source == self.caller
                        })
                {
                    return Err("source computation has a foreign caller lifetime");
                }
            }
        }
        if !nonstatic {
            return Err("source computation requires an owned open lifetime");
        }
        Ok(())
    }
    fn schema(
        facts: &mut Declarations,
        selected: Instance,
        original: NodeKey,
        result: TypeId,
    ) -> Result<(), &'static str> {
        let initializer = facts
            .initializer_index(selected.definition)
            .and_then(|index| facts.initializers.get(index))
            .copied()
            .ok_or("source computation has no initializer")?;
        if !initializer.type_checked()
            || !matches!(initializer.kind, InitializerKind::Const)
            || initializer.semantic_source() != original
        {
            return Err("source computation requires its original checked initializer");
        }
        // Validate exact parameter arity/kinds without resolving or interning a result.
        selected.selected_arguments(facts)?;
        for argument in facts
            .types
            .arguments(selected.arguments)
            .ok_or("missing source arguments")?
        {
            let closed = match argument {
                Argument::Lifetime(_) => true,
                Argument::Type(ty) => {
                    facts.types.contains_parameters(ty) == Some(false)
                        && facts.types.contains_projections(ty) == Some(false)
                        && facts.types.needs_region_context(ty) == Some(false)
                }
                Argument::Const(id) => {
                    matches!(facts.types.const_kind(id), Some(ConstKind::Value(_)))
                }
            };
            if !closed {
                return Err("source computation retains a non-lifetime parameter");
            }
        }
        selected_result(facts, selected, initializer.ty, result)
    }
}
fn selected_result(
    facts: &mut Declarations,
    selected: Instance,
    declared: TypeId,
    result: TypeId,
) -> Result<(), &'static str> {
    if declared == result && closed_result(&facts.types, result) {
        return Ok(());
    }
    if facts.types.contains_projections(declared) != Some(false)
        || !source_result(&facts.types, result)
    {
        return Err("source computation result normalization is not implemented");
    }
    let mut substitution = selected.substitution(facts)?;
    let actual = substitution.apply(&mut facts.types, declared)?;
    facts.body_work.canonical_steps += substitution.steps;
    if actual != result {
        return Err("source computation result differs from its selected declaration");
    }
    Ok(())
}
fn closed_result(types: &crate::types::Store, result: TypeId) -> bool {
    source_result(types, result) && types.needs_region_context(result) == Some(false)
}
fn source_result(types: &crate::types::Store, result: TypeId) -> bool {
    matches!(
        types.kind(result),
        Some(
            Kind::Primitive(_)
                | Kind::Tuple(_)
                | Kind::Array { .. }
                | Kind::Adt(_, _)
                | Kind::Reference { mutable: false, .. }
        )
    ) && types.contains_parameters(result) == Some(false)
        && types.contains_projections(result) == Some(false)
}
impl Queries<'_> {
    pub(super) fn source_computation(
        &mut self,
        selected: Instance,
        original: NodeKey,
        result: TypeId,
    ) -> Result<(SourceComputation, TypeId), Error> {
        SourceComputation::schema(self.typing.facts, selected, original, result)
            .map_err(|e| self.error(Query::Node(original), e))?;
        let aliases = self
            .typing
            .facts
            .types
            .arguments(selected.arguments)
            .ok_or_else(|| self.error(Query::Node(original), "missing source arguments"))?
            .len();
        self.with_parked_caller(aliases, |dispatcher, facts, scope, reservation, regions| {
            dispatcher.retain_source_aliases(facts, selected, regions)?;
            let selected = selected.substitute(&mut facts.types, regions)?;
            let result = regions.apply(&mut facts.types, result)?;
            let selected = dispatcher.source_instance(facts, selected)?;
            let result = dispatcher.source_result(facts, result)?;
            dispatcher.reserve_pattern(facts, scope.body, reservation)?;
            let witness = SourceComputation {
                selected,
                original,
                caller: scope.body,
            };
            witness.validate(facts, result)?;
            Ok((witness, result))
        })
        .map_err(|e| self.error(Query::Node(original), e))
    }
}

#[cfg(test)]
#[path = "../../../tests/invariants/computed_witnesses.rs"]
mod tests;
