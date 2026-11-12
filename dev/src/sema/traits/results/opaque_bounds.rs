//! Caller-visible opaque promises are mapped without reading hidden representations.
use super::{Assumptions, Declarations};
use crate::types::{GenericArgs, ItemId, TypeId, opaque::OpaqueId, outlives::ResultId};
use std::sync::Arc;

pub(super) fn traits(
    facts: &mut Declarations,
    ty: TypeId,
    id: OpaqueId,
    arguments: GenericArgs,
    item: ItemId,
) -> Result<(Arc<Assumptions>, usize), String> {
    let (templates, template_steps) =
        super::templates::for_declaration(facts, ResultId::Opaque(id))?;
    let mut result = Assumptions::default();
    let mut cost = 0;
    if let Some(predicates) = templates.candidates(item) {
        let declaration = facts
            .opaques
            .declarations
            .get(&id)
            .ok_or("missing opaque result contract")?;
        let mut fold = declaration.substitution(&facts.types, arguments)?;
        for &clause in predicates {
            let mapped = fold.clause(&mut facts.types, clause)?;
            result
                .insert_clause(&facts.types, mapped)
                .map_err(|e| e.to_string())?;
        }
        cost = fold.steps.saturating_add(predicates.len());
    }
    let result = Arc::new(result);
    facts
        .generics
        .traits
        .as_mut()
        .ok_or("missing opaque rule contracts")?
        .result_rules
        .insert(ty, item, Arc::clone(&result), cost)?;
    Ok((result, cost.saturating_add(template_steps)))
}

pub(in crate::sema) fn prepare_outlives(facts: &mut Declarations) -> Result<(), String> {
    let mut ids: Vec<_> = facts.opaques.declarations.keys().copied().collect();
    ids.sort_unstable_by_key(|id| id.0);
    for id in ids {
        let (templates, _) = super::templates::for_declaration(facts, ResultId::Opaque(id))?;
        let application = facts.opaques.declarations[&id].application;
        facts
            .types
            .declare_opaque_outlives(application, &templates.outlives)?;
    }
    Ok(())
}

/// A unique published callable clause can constrain its opaque capture inputs.
/// This supplies equations only; the original query still proves the full bound.
pub(in crate::sema::traits) fn infer_inputs(
    facts: &mut Declarations,
    context: &mut crate::types::infer::Context,
    goal: &crate::types::infer::predicates::Predicate,
    first: u32,
) -> Result<Option<std::ops::Range<u32>>, String> {
    use crate::types::infer::{self, Instantiation, probe::Probe};
    let Some(infer::Constructor::Opaque(id, _)) = context
        .head(&facts.types, goal.subject())
        .map_err(|e| e.to_string())?
    else {
        return Ok(None);
    };
    let Some((item, _)) = goal.trait_arguments() else {
        return Ok(None);
    };
    let (templates, steps) = super::templates::for_declaration(facts, ResultId::Opaque(id))?;
    context.account_work(steps).map_err(|e| e.to_string())?;
    let Some(&[clause]) = templates.candidates(item) else {
        return Ok(None);
    };
    let actual = context
        .item_arguments(&facts.types, goal.subject())
        .map_err(|e| e.to_string())?;
    let declaration = facts
        .opaques
        .declarations
        .get(&id)
        .ok_or("missing opaque input contract")?;
    let mut captures = declaration.inferred_arguments(&facts.types, &actual)?;
    let clause = captures.regions.clause(&mut facts.types, clause)?;
    context
        .account_work(captures.regions.steps)
        .map_err(|e| e.to_string())?;
    let (predicate, evidence) = super::super::assumptions::Openings::default()
        .prepare(&mut facts.types, context, clause, first)
        .map_err(|e| e.to_string())?;
    let mut substitution = Instantiation::new(context, &facts.types, &captures.parameters)
        .map_err(|e| e.to_string())?;
    let predicate = context
        .instantiate_predicate(&mut facts.types, predicate, &mut substitution)
        .map_err(|e| e.to_string())?;
    let roots = goal
        .input_roots()
        .chain(predicate.input_roots())
        .collect::<Vec<_>>();
    let (mut probe, roots) =
        Probe::new(&facts.types, context, &roots).map_err(|e| e.to_string())?;
    let mut roots = roots.into_iter();
    let left = goal
        .map_inputs(|_| roots.next().ok_or(infer::Error::InvalidIdentity))
        .map_err(|e| e.to_string())?;
    let right = predicate
        .map_inputs(|_| roots.next().ok_or(infer::Error::InvalidIdentity))
        .map_err(|e| e.to_string())?;
    if roots.next().is_some() {
        return Err("opaque input trial has surplus roots".into());
    }
    let compatible = probe
        .context()
        .equate_predicates(&facts.types, &left, &right);
    context
        .account_work(probe.context().steps)
        .map_err(|e| e.to_string())?;
    match compatible {
        Ok(()) => {
            context
                .equate_predicates(&facts.types, goal, &predicate)
                .map_err(|e| e.to_string())?;
            Ok(Some(evidence.regions))
        }
        Err(
            infer::Error::Unresolved
            | infer::Error::Normalization
            | infer::Error::Mismatch
            | infer::Error::Occurs,
        ) => Ok(None),
        Err(error) => Err(error.to_string()),
    }
}
