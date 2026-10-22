//! A source value keeps its actual caller proof owner without a materialized permit.
mod catalog;
mod namespaces;
mod observed_arguments;
mod prepared_callers;
mod queries;
pub(crate) use namespaces::{
    available as caller_namespace_available, published as caller_published, reserve_normalization,
    with_namespace, with_registered_namespace,
};
mod seals;
use super::{Declarations, Error, InitializerKind, NodeKey, Queries, Query, TypeId};
use crate::{
    sema::values::Instance,
    types::{
        Kind, RegionId,
        arguments::{Argument, ConstKind},
    },
};
pub(crate) use catalog::Enclosing as CallerInstance;
pub(crate) use prepared_callers::{BodyCaller, PreparedConstant};

/// An opaque source-computation request scoped to one declaration catalog.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SourceComputation(catalog::Handle);

pub(in crate::sema) struct Computations(catalog::Catalog);
#[derive(Clone, Copy)]
pub(crate) struct Resolved {
    pub selected: Instance,
    pub original: NodeKey,
}
impl Declarations {
    pub(in crate::sema) fn retain_source_caller(
        &mut self,
        source: NodeKey,
        declaration: Option<crate::resolve::model::DefId>,
        universals: u32,
        ranges: [std::ops::Range<usize>; 2],
    ) -> Result<(), &'static str> {
        let publication = catalog::Publication::new(ranges)?;
        publication.validate(self, source)?;
        if self.source_computations.is_none() {
            self.source_computations = Some(Box::new(Computations(catalog::Catalog::new()?)));
        }
        let catalog = &mut self
            .source_computations
            .as_deref_mut()
            .ok_or("missing source caller catalog")?
            .0;
        catalog.caller(
            catalog::CallerKey {
                source,
                declaration,
                universals,
                enclosing: catalog::Enclosing::Original,
            },
            publication,
        )?;
        self.constant_work.source_context_bytes = catalog.retained_bytes();
        Ok(())
    }
}
impl SourceComputation {
    /// Observation shares immutable graph facts, never caller proof authority.
    pub(in crate::sema) fn with_argument_regions<T, E: From<&'static str>>(
        facts: &mut Declarations,
        selected: Instance,
        operation: impl FnOnce(&Declarations, &[RegionId]) -> Result<T, E>,
    ) -> Result<T, E> {
        if facts.source_computations.is_none() {
            facts.body_work.canonical_steps = facts.body_work.canonical_steps.saturating_add(1);
            let catalog = catalog::Catalog::new().map_err(E::from)?;
            facts.constant_work.source_context_bytes = catalog.retained_bytes();
            facts.source_computations = Some(Box::new(Computations(catalog)));
        }
        let context = CallerInstance::Initializer(selected);
        observed_arguments::prepare(facts, context, 0).map_err(E::from)?;
        let regions = observed_arguments::regions(facts, context).map_err(E::from)?;
        operation(facts, regions)
    }
    pub(crate) fn context_order(self, facts: &Declarations) -> Result<[u32; 8], &'static str> {
        let (entry, caller) = self.entry(facts)?;
        Ok([
            entry.selected.definition.0,
            entry.selected.arguments.0,
            caller.source.file.0,
            caller.source.node.get(),
            entry.input.definition.0,
            entry.input.arguments.0,
            entry.actual_result.0,
            entry.requested.0,
        ])
    }
    pub(crate) fn input(self, facts: &Declarations) -> Result<Instance, &'static str> {
        Ok(self.entry(facts)?.0.input)
    }
    pub(crate) fn validate_use(
        self,
        facts: &Declarations,
        source: NodeKey,
        enclosing: CallerInstance,
        result: TypeId,
    ) -> Result<(), &'static str> {
        let (entry, caller) = self.entry(facts)?;
        if caller.source != source || caller.enclosing != enclosing || entry.actual_result != result
        {
            return Err("source associated operand belongs to another caller or result");
        }
        Ok(())
    }

    pub(crate) fn result(self, facts: &Declarations) -> Result<TypeId, &'static str> {
        Ok(self.entry(facts)?.0.requested)
    }
    pub(crate) fn identity(
        self,
        facts: &Declarations,
    ) -> Result<(Instance, NodeKey), &'static str> {
        let entry = self.entry(facts)?.0;
        Ok((entry.selected, entry.original))
    }
    #[cfg(test)]
    pub(crate) fn selected(self, facts: &Declarations) -> Result<Instance, &'static str> {
        Ok(self.entry(facts)?.0.selected)
    }
    /// The body whose numbering unit holds this computation's caller regions.
    #[cfg(test)]
    pub(crate) fn caller_source(self, facts: &Declarations) -> Result<NodeKey, &'static str> {
        Ok(self.entry(facts)?.1.source)
    }
    fn entry(
        self,
        facts: &Declarations,
    ) -> Result<(catalog::Use, catalog::CallerKey), &'static str> {
        let catalog = &facts
            .source_computations
            .as_deref()
            .ok_or("missing source computation catalog")?
            .0;
        let entry = catalog.lookup(self.0)?;
        catalog.publication(entry.0.caller)?.within(facts)?;
        Ok(entry)
    }
    pub(crate) fn validate(
        self,
        facts: &mut Declarations,
        result: TypeId,
    ) -> Result<(), &'static str> {
        self.resolve(facts, result).map(|_| ())
    }
    pub(crate) fn resolve(
        self,
        facts: &mut Declarations,
        result: TypeId,
    ) -> Result<Resolved, &'static str> {
        let (entry, caller) = self.entry(facts)?;
        if !matches!(caller.enclosing, CallerInstance::Original)
            && !namespaces::contains(facts, caller)
        {
            return Err("source computation lost its exact caller namespace");
        }
        if entry.requested != result {
            return Err("source computation request result changed");
        }
        if let Some(seal) = entry.seal {
            Self::shape(facts, entry.selected, entry.original)?;
            seal.validate(facts, entry.selected, result)?;
        } else {
            Self::schema(facts, entry.selected, entry.original, result)?;
        }
        let mut nonstatic = false;
        for argument in facts
            .types
            .arguments(entry.selected.arguments)
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
                        .get(caller.source, region)
                        .is_some_and(|owner| {
                            owner.body == caller.source && owner.source == caller.source
                        })
                {
                    return Err("source computation has a foreign caller lifetime");
                }
            }
        }
        let nested = facts
            .types
            .arguments(entry.selected.arguments)
            .ok_or("missing source arguments")?
            .any(|argument| {
                matches!(argument, Argument::Type(ty)
                if facts.types.needs_region_context(ty) == Some(true))
            });
        if nested {
            let mut visits = 0;
            let observed = Self::with_argument_regions(facts, entry.selected, |facts, regions| {
                for region in regions {
                    visits += 1;
                    if !facts
                        .body_regions
                        .get(caller.source, *region)
                        .is_some_and(|owner| {
                            owner.body == caller.source && owner.source == caller.source
                        })
                    {
                        return Err("source computation has a foreign caller lifetime");
                    }
                }
                Ok(!regions.is_empty())
            });
            facts.body_work.canonical_steps =
                facts.body_work.canonical_steps.saturating_add(visits);
            nonstatic |= observed?;
        }
        if !nonstatic && !entry.seal.is_some_and(|seal| seal.projected(facts)) {
            return Err("source computation requires an owned open lifetime");
        }
        Ok(Resolved {
            selected: entry.selected,
            original: entry.original,
        })
    }
    fn schema(
        facts: &mut Declarations,
        selected: Instance,
        original: NodeKey,
        result: TypeId,
    ) -> Result<(), &'static str> {
        let declared = Self::shape(facts, selected, original)?;
        selected_result(facts, selected, declared, result)
    }
    fn shape(
        facts: &mut Declarations,
        selected: Instance,
        original: NodeKey,
    ) -> Result<TypeId, &'static str> {
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
                        && facts.types.escaping_binders(ty) == Some(0)
                }
                Argument::Const(id) => {
                    matches!(facts.types.const_kind(id), Some(ConstKind::Value(_)))
                }
            };
            if !closed {
                return Err("source computation retains a non-lifetime parameter");
            }
        }
        Ok(initializer.ty)
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
/// A routing decision only; normalization and final witness admission remain mandatory.
fn source_result_candidate(types: &crate::types::Store, result: TypeId) -> bool {
    if source_result(types, result) {
        return true;
    }
    if types.contains_parameters(result) != Some(false)
        || types.contains_projections(result) != Some(true)
    {
        return false;
    }
    match types.kind(result) {
        Some(
            Kind::Projection(_)
            | Kind::Tuple(_)
            | Kind::Array { .. }
            | Kind::Adt(_, _)
            | Kind::Reference { mutable: false, .. },
        ) => true,
        Some(Kind::Opaque(id, _)) => types.opaque_declaration(id).is_some_and(|d| d.trait_result),
        _ => false,
    }
}

impl Queries<'_> {
    pub(super) fn source_computation(
        &mut self,
        selected: Instance,
        original: NodeKey,
        result: TypeId,
    ) -> Result<super::values::execution::Target, Error> {
        SourceComputation::shape(self.typing.facts, selected, original)
            .map_err(|e| self.error(Query::Node(original), e))?;
        let actual_result = result;
        let aliases = self
            .typing
            .facts
            .types
            .arguments(selected.arguments)
            .ok_or_else(|| self.error(Query::Node(original), "missing source arguments"))?
            .len();
        let input = selected;
        self.with_parked_caller(
            aliases,
            |dispatcher, facts, scope, reservation, regions, publication| {
                dispatcher.retain_source_aliases(facts, selected, regions)?;
                let selected = selected.substitute(&mut facts.types, regions)?;
                let result = regions.apply(&mut facts.types, result)?;
                let selected = dispatcher.source_instance(facts, selected)?;
                let result = dispatcher.source_result(facts, result)?;
                dispatcher.reserve_pattern(facts, scope.body, reservation)?;
                if facts.types.arguments_are_concrete(selected.arguments)? {
                    if selected.result_type(facts)? != result {
                        return Err("canonical initializer request result changed".into());
                    }
                    return Ok(super::values::execution::Target::Associated {
                        instance: selected,
                        result,
                    });
                }
                let seal = seals::Seal::prepare(facts, dispatcher, selected, result)?;
                let requested = seal.result();
                dispatcher.reserve_pattern(facts, scope.body, reservation)?;
                let publication = catalog::Publication::new(publication)?;
                let known = facts
                    .source_computations
                    .as_deref()
                    .and_then(|catalog| catalog.0.original(scope.body).ok());
                if let Some((key, previous)) = known {
                    if key.declaration != scope.owner
                        || key.universals != scope.binders
                        || previous != publication
                    {
                        return Err("source caller publication changed before use".into());
                    }
                } else {
                    publication.validate(facts, scope.body)?;
                }
                if facts.source_computations.is_none() {
                    facts.source_computations =
                        Some(Box::new(Computations(catalog::Catalog::new()?)));
                }
                let catalog = &mut facts
                    .source_computations
                    .as_deref_mut()
                    .ok_or("missing source computation catalog")?
                    .0;
                let caller = catalog.caller(
                    catalog::CallerKey {
                        source: scope.body,
                        declaration: scope.owner,
                        universals: scope.binders,
                        enclosing: catalog::Enclosing::Original,
                    },
                    publication,
                )?;
                let witness = SourceComputation(catalog.retain_use(catalog::Use {
                    caller,
                    input,
                    selected,
                    original,
                    actual_result,
                    requested,
                    seal: Some(seal),
                })?);
                facts.constant_work.source_context_bytes = catalog.retained_bytes();
                witness.validate(facts, requested)?;
                Ok(super::values::execution::Target::SourceAssociated {
                    source: witness,
                    result: requested,
                })
            },
        )
        .map_err(|e| self.error(Query::Node(original), e))
    }
}

#[cfg(test)]
#[path = "../../../tests/invariants/computed_witnesses.rs"]
mod tests;
