//! Caller-visible opaque promises are mapped without reading hidden representations.
use super::{Assumptions, Declarations};
use crate::types::{
    GenericArgs, ItemId, TypeId,
    infer::{self, Context, Instantiation, TermId, predicates::Predicate, probe::Probe},
    opaque::OpaqueId,
    outlives::ResultId,
};
use std::{ops::Range, sync::Arc};

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

/// Related declared inputs, and whether they changed the caller's input graph.
pub(in crate::sema::traits) struct Inputs {
    pub regions: Range<u32>,
    pub refined: bool,
}

/// A unique published callable clause can constrain its opaque capture inputs.
/// This supplies equations only; the original query still proves the full bound.
pub(in crate::sema::traits) fn infer_inputs(
    facts: &mut Declarations,
    context: &mut Context,
    goal: &Predicate,
    first: u32,
) -> Result<Option<Inputs>, String> {
    let Some((item, _)) = goal.trait_arguments() else {
        return Ok(None);
    };
    let Some((predicate, regions)) = declared_clause(
        facts,
        context,
        goal.subject(),
        item,
        Published::Bound,
        first,
    )?
    else {
        return Ok(None);
    };
    Ok(match relate(facts, context, goal, &predicate)? {
        Related::Incompatible => None,
        related => Some(Inputs {
            regions,
            refined: matches!(related, Related::Refined),
        }),
    })
}

/// A unique published callable output equation exposes its declared result term,
/// which shares the opaque's inferred captures. Its callable inputs are related
/// first; this is forward normalization through the declaration, never inversion
/// of the output, and the original query still proves the complete bound.
pub(in crate::sema::traits) fn infer_output(
    facts: &mut Declarations,
    context: &mut Context,
    goal: &Predicate,
    item: ItemId,
    first: u32,
) -> Result<Option<(TermId, Range<u32>)>, String> {
    let Some((equation, regions)) = declared_clause(
        facts,
        context,
        goal.subject(),
        item,
        Published::Equality,
        first,
    )?
    else {
        return Ok(None);
    };
    let output = equation
        .projection_result()
        .ok_or("missing declared callable output")?;
    let declared = context
        .projection_application(&facts.types, equation.subject())
        .map_err(text)?;
    Ok(match relate(facts, context, goal, &declared.predicate)? {
        Related::Incompatible => None,
        Related::Unchanged | Related::Refined => Some((output, regions)),
    })
}

#[derive(Clone, Copy)]
enum Published {
    Bound,
    Equality,
}

#[derive(Clone, Copy)]
enum Related {
    Incompatible,
    Unchanged,
    Refined,
}

/// Instantiate the opaque's unique indexed clause over the goal's actual captures.
fn declared_clause(
    facts: &mut Declarations,
    context: &mut Context,
    subject: TermId,
    item: ItemId,
    published: Published,
    first: u32,
) -> Result<Option<(Predicate, Range<u32>)>, String> {
    let Some(infer::Constructor::Opaque(id, _)) =
        context.head(&facts.types, subject).map_err(text)?
    else {
        return Ok(None);
    };
    let (templates, steps) = super::templates::for_declaration(facts, ResultId::Opaque(id))?;
    context.account_work(steps).map_err(text)?;
    let clauses = match published {
        Published::Bound => templates.candidates(item),
        Published::Equality => templates.equalities(item),
    };
    let Some(&[clause]) = clauses else {
        return Ok(None);
    };
    let actual = context
        .item_arguments(&facts.types, subject)
        .map_err(text)?;
    let declaration = facts
        .opaques
        .declarations
        .get(&id)
        .ok_or("missing opaque input contract")?;
    let mut captures = declaration.inferred_arguments(&facts.types, &actual)?;
    let clause = captures.regions.clause(&mut facts.types, clause)?;
    context.account_work(captures.regions.steps).map_err(text)?;
    let (predicate, evidence) = super::super::assumptions::Openings::default()
        .prepare(&mut facts.types, context, clause, first)
        .map_err(text)?;
    let mut substitution =
        Instantiation::new(context, &facts.types, &captures.parameters).map_err(text)?;
    let predicate = context
        .instantiate_predicate(&mut facts.types, predicate, &mut substitution)
        .map_err(text)?;
    Ok(Some((predicate, evidence.regions)))
}

/// Trial equations in a probe first, so an incompatible declaration leaves the
/// caller's inference graph unchanged.
fn relate(
    facts: &Declarations,
    context: &mut Context,
    goal: &Predicate,
    declared: &Predicate,
) -> Result<Related, String> {
    let roots = goal
        .input_roots()
        .chain(declared.input_roots())
        .collect::<Vec<_>>();
    let (mut probe, roots) = Probe::new(&facts.types, context, &roots).map_err(text)?;
    let mut roots = roots.into_iter();
    let left = goal
        .map_inputs(|_| roots.next().ok_or(infer::Error::InvalidIdentity))
        .map_err(text)?;
    let right = declared
        .map_inputs(|_| roots.next().ok_or(infer::Error::InvalidIdentity))
        .map_err(text)?;
    if roots.next().is_some() {
        return Err("opaque input trial has surplus roots".into());
    }
    let compatible = probe
        .context()
        .equate_predicates(&facts.types, &left, &right)
        .and_then(|()| probe.preserves_inputs(context));
    context.account_work(probe.context().steps).map_err(text)?;
    match compatible {
        Ok(preserved) => {
            context
                .equate_predicates(&facts.types, goal, declared)
                .map_err(text)?;
            Ok(if preserved {
                Related::Unchanged
            } else {
                Related::Refined
            })
        }
        Err(
            infer::Error::Unresolved
            | infer::Error::Normalization
            | infer::Error::Mismatch
            | infer::Error::Occurs,
        ) => Ok(Related::Incompatible),
        Err(error) => Err(error.to_string()),
    }
}

fn text(error: infer::Error) -> String {
    error.to_string()
}
