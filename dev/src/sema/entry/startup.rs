//! An executable startup root specializes the actual Source start declaration.
use super::{Declarations, DefId};
use crate::types::{
    GenericArgs, ItemId, Kind, TypeId,
    arguments::Argument,
    instance::Instance,
    outlives::Queries,
    parameters::predicates::{Bound, Predicate},
    relations::Constraints,
    substitute,
};

#[derive(Clone, Copy)]
pub(crate) struct Startup {
    pub target: Instance,
    pub callback: Instance,
    pub source: crate::resolve::body_origin::ProofSite,
}

pub(crate) fn prepare(
    report: &crate::resolve::expansion::Report,
    facts: &mut Declarations,
    entry: DefId,
) -> Result<Option<Startup>, String> {
    let Some(start) = super::source_start(facts)? else {
        return Ok(None);
    };
    let source = facts
        .function_indices
        .get(&start)
        .and_then(|&index| facts.functions.get(index))
        .ok_or("source start requires its actual body definition")?
        .node;
    crate::sema::declarations::prepare_callable_template(report, facts, source, start)
        .map_err(|error| error.to_string())?;
    let (callback, callback_signature) = super::main_callable(facts, entry)?;
    let Some(Kind::Function { result, .. }) = facts.types.kind(callback_signature) else {
        return Err("main lost its checked function signature".into());
    };
    let (_, start_signature) = item(facts, start)?;
    let [formal] = facts.generics.parameter_ids(&facts.types, start)?[..] else {
        return Err("source start lost its single type parameter".into());
    };
    let argument = Argument::Type(result);
    let mut substitution = substitute::Bound::parameters(&facts.types, &[(formal, argument)])?;
    let signature = substitution.signature(&mut facts.types, start_signature, 0)?;
    let Some(Kind::Function { parameters, .. }) = facts.types.kind(signature) else {
        return Err("source start lost its specialized signature".into());
    };
    if facts.types.list(parameters).and_then(|list| list.first()) != Some(&callback_signature) {
        return Err("main pointer does not match the selected Source start callback".into());
    }
    prove(facts, start, result, &mut substitution)?;
    let arguments = facts.types.intern_arguments(&[argument])?;
    let target = facts.types.intern(Kind::FunctionItem {
        item: ItemId(start.0),
        arguments,
        signature,
    })?;
    Ok(Some(Startup {
        target: Instance::new(&facts.types, target)?,
        callback: Instance::new(&facts.types, callback)?,
        source: source.into(),
    }))
}

fn item(facts: &Declarations, owner: DefId) -> Result<(TypeId, TypeId), &'static str> {
    let ty = facts
        .definitions
        .get(owner.0 as usize)
        .copied()
        .flatten()
        .ok_or("startup definition has no type")?;
    let Some(Kind::FunctionItem { signature, .. }) = facts.types.kind(ty) else {
        return Err("startup definition has no function signature");
    };
    Ok((ty, signature))
}

fn prove(
    facts: &mut Declarations,
    start: DefId,
    result: TypeId,
    substitution: &mut substitute::Bound,
) -> Result<(), String> {
    let source =
        crate::sema::declaration_inputs::declared(&facts.generics, facts.crates.as_deref(), start)?
            .collect::<Vec<_>>();
    let mut state = crate::sema::nominals::traits::State::instance(facts, 0)?;
    if let Some(termination) = super::source_role(facts, super::LangItem::Termination)? {
        let id = facts
            .types
            .intern_trait_application(crate::types::traits::Application {
                item: ItemId(termination.0),
                arguments: GenericArgs::EMPTY,
            })?;
        if !state.prove(
            facts,
            Predicate {
                ty: result,
                bound: Bound::Trait(id),
            },
            true,
        )? {
            return Err(
                "main result does not implement the actual Source Termination trait".into(),
            );
        }
    }
    for requirement in source {
        let clause = substitution.clause(&mut facts.types, requirement.value)?;
        if !state.prove_clause(facts, clause, true)? {
            return Err("selected Source start requires an unmet main result bound".into());
        }
    }
    facts.body_work.canonical_steps += state.selection_steps().saturating_add(substitution.steps);
    let proof = state.finish();
    if !proof.is_empty() {
        for (_, ty) in proof.type_roots() {
            facts
                .lifetimes
                .nominal_requirements
                .prepare(&mut facts.types, ty)?;
        }
        let mut queries = Queries::new(
            &facts.types,
            &facts.lifetimes.nominal_requirements,
            &facts.lifetimes.callables,
        );
        let mut graph = Constraints::new(&facts.types, &facts.lifetimes.variances, 0, 1_000_000)
            .map_err(|error| error.to_string())?;
        proof.require(&mut graph, &mut queries)?;
        graph.solve().map_err(|error| error.to_string())?;
    }
    Ok(())
}
