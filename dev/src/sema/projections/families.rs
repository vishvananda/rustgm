//! A complete family application carries input conditions separately from result promises.
use crate::{
    resolve::model::DefId,
    sema::model::Declarations,
    types::{
        Kind,
        arguments::Argument,
        outlives::Pair,
        parameters::{
            ParameterKind,
            clauses::Clause,
            predicates::{Bound, Predicate},
        },
        projection::Application,
    },
};

pub(in crate::sema) struct Requirements {
    pub clauses: Vec<Clause>,
    pub regions: Vec<Pair>,
    pub steps: usize,
}

/// The enclosing normalizer retains formation of the request's type inputs.
/// This maps the member's implicit Sized bounds and explicit where contract.
/// It grants no proof and must not substitute projected result bounds as premises.
pub(in crate::sema) fn requirements(
    facts: &mut Declarations,
    application: Application,
) -> Result<Option<Requirements>, &'static str> {
    let member = DefId(application.item.0);
    if !facts.associated.families.contains_key(&member) {
        return Ok(None);
    }
    let mut fold = facts.trait_substitution(
        Predicate {
            ty: application.subject,
            bound: Bound::Trait(application.trait_application),
        },
        Some((member, application.arguments)),
    )?;
    inputs(facts, member, &mut fold).map(Some)
}

/// Map only the member's input contract through an already checked joint folder.
pub(in crate::sema) fn inputs(
    facts: &mut Declarations,
    member: DefId,
    fold: &mut crate::types::substitute::Bound,
) -> Result<Requirements, &'static str> {
    let family = facts.associated.families.get(&member);
    let templates = match family {
        Some(family) => family
            .requirements
            .as_deref()
            .ok_or("associated family input contract is not complete")?,
        None => &[],
    };
    let mut clauses = Vec::new();
    let parameters = facts
        .generics
        .parameters
        .get(&member)
        .map_or(&[][..], Vec::as_slice);
    for parameter in parameters {
        let Argument::Type(ty) = parameter.argument else {
            continue;
        };
        let Some(Kind::Parameter(id)) = facts.types.kind(ty) else {
            return Err("missing family type parameter identity");
        };
        if matches!(
            facts.types.parameter(id).map(|p| p.kind),
            Some(ParameterKind::Type { sized: true })
        ) {
            clauses.push(
                fold.clause(
                    &mut facts.types,
                    Predicate {
                        ty,
                        bound: Bound::Sized,
                    }
                    .into(),
                )?,
            );
        }
    }
    for &clause in templates {
        clauses.push(fold.clause(&mut facts.types, clause)?);
    }
    let regions = family
        .into_iter()
        .flat_map(|f| &f.regions)
        .map(|&(a, b)| Ok((fold.region(a, 0)?, fold.region(b, 0)?)))
        .collect::<Result<Vec<_>, &'static str>>()?;
    let steps = fold
        .steps
        .saturating_add(parameters.len())
        .saturating_add(templates.len())
        .saturating_add(regions.len());
    Ok(Requirements {
        clauses,
        regions,
        steps,
    })
}
