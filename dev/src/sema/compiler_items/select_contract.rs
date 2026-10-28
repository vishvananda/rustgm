//! The source selector's four parameters and callable obligations are fixed data.
use crate::{
    resolve::model::DefId,
    sema::model::{Declarations, Function},
    support::{bindings::LangItem, constness::Constness},
    types::{
        Kind, TypeId,
        arguments::Argument,
        callable::Convention,
        parameters::{
            ParameterKind,
            clauses::Clause,
            predicates::{Bound, Predicate},
        },
    },
};

pub(super) fn validate(facts: &Declarations, function: Function) -> Result<(), &'static str> {
    let types = parameters(facts, function.definition)?;
    let Some(Kind::Function {
        bound_regions: 0,
        parameters,
        result,
        unsafe_: false,
        convention: Convention::RUST,
    }) = facts.types.kind(function.signature)
    else {
        return Err(
            "source const_eval_select requires its safe Rust signature without late lifetimes",
        );
    };
    if facts.types.list(parameters) != Some(&types[..3])
        || result != types[3]
        || !facts.is_const_function(function.definition)
    {
        return Err("source const_eval_select requires (ARG,F,G) to RET and const eligibility");
    }
    requirements(facts, function.definition, types)
}

fn parameters(facts: &Declarations, definition: DefId) -> Result<[TypeId; 4], &'static str> {
    let [a, f, g, r] = facts
        .generics
        .parameters
        .get(&definition)
        .map_or(&[][..], Vec::as_slice)
    else {
        return Err("source const_eval_select requires exactly four type parameters");
    };
    let [
        Argument::Type(a),
        Argument::Type(f),
        Argument::Type(g),
        Argument::Type(r),
    ] = [a.argument, f.argument, g.argument, r.argument]
    else {
        return Err("source const_eval_select requires four owned type parameters");
    };
    let types = [a, f, g, r];
    for (ty, ordinal) in types.into_iter().zip(0_u32..) {
        let Some(Kind::Parameter(id)) = facts.types.kind(ty) else {
            return Err("source const_eval_select lost its own type parameter");
        };
        if facts.types.parameter(id).is_none_or(|p| {
            p.owner.0 != definition.0
                || p.index != ordinal
                || p.kind != (ParameterKind::Type { sized: true })
        }) {
            return Err("source const_eval_select has a foreign or unsized parameter slot");
        }
    }
    Ok(types)
}

fn requirements(
    facts: &Declarations,
    definition: DefId,
    types: [TypeId; 4],
) -> Result<(), &'static str> {
    let catalog = facts
        .source_bindings
        .as_deref()
        .ok_or("source selector catalog disappeared")?;
    // Reservation identifies source headers only. All role completion and caller
    // proofs remain mandatory before any selected operation may execute.
    let role = |kind| {
        catalog
            .language(kind)
            .map(|d| d.definition)
            .ok_or("source const_eval_select requires actual Tuple/FnOnce/Output declarations")
    };
    let identities = [
        role(LangItem::Tuple)?,
        role(LangItem::FnOnce)?,
        role(LangItem::FnOnceOutput)?,
    ];
    // Trait-header preparation validates the actual sizing hierarchy before
    // ordinary function publication. These clauses describe that header only;
    // they supply neither caller proof nor callable execution permission.
    let sizing = catalog
        .sizing
        .ok_or("source selector requires completed actual sizing declarations")?;
    if catalog
        .language(LangItem::FnOnceOutput)
        .and_then(|d| d.parent)
        != Some(identities[1])
    {
        return Err("source selector Output belongs to another FnOnce owner");
    }
    let mut found = 0_u8;
    for clause in facts.generics.declared_predicates(definition) {
        let predicate = match clause.value {
            Clause::UnstableFeature(_) => {
                return Err("source selector feature conditions require an explicit contract");
            }
            Clause::Predicate(p) => p,
            Clause::ForAll(id) => {
                let p = facts
                    .types
                    .quantified(id)
                    .ok_or("source selector clause disappeared")?;
                if p.bound_regions != 0 {
                    return Err("source selector header has an unsupported quantified clause");
                }
                p.predicate
            }
        };
        found |= requirement(facts, predicate, types, identities, sizing)?;
    }
    if found != 31 {
        return Err(
            "source selector requires ARG:Tuple and exact ordinary/const FnOnce Output equalities",
        );
    }
    Ok(())
}

fn requirement(
    facts: &Declarations,
    p: Predicate,
    types: [TypeId; 4],
    ids: [DefId; 3],
    sizing: [DefId; 3],
) -> Result<u8, &'static str> {
    if let Some((application, mode)) = p.bound.trait_parts() {
        let app = facts
            .types
            .trait_application(application)
            .ok_or("invalid source selector trait clause")?;
        let args = facts
            .types
            .arguments(app.arguments)
            .ok_or("invalid source selector trait arguments")?;
        if mode == Constness::Ordinary
            && sizing.contains(&DefId(app.item.0))
            && args.len() == 0
            && types.contains(&p.ty)
        {
            return Ok(0);
        }
        if p.ty == types[0]
            && mode == Constness::Ordinary
            && app.item.0 == ids[0].0
            && args.len() == 0
        {
            return Ok(1);
        }
        if app.item.0 == ids[1].0
            && args.len() == 1
            && args.into_iter().next() == Some(Argument::Type(types[0]))
        {
            if p.ty == types[1] && mode == Constness::Const {
                return Ok(2);
            }
            if p.ty == types[2] && mode == Constness::Ordinary {
                return Ok(4);
            }
        }
    } else if let Bound::ProjectionEqual(result) = p.bound {
        if result == types[3]
            && let Some(Kind::Projection(projection)) = facts.types.kind(p.ty)
            && projection.item.0 == ids[2].0
            && facts
                .types
                .arguments(projection.arguments)
                .is_some_and(|a| a.len() == 0)
            && facts
                .types
                .trait_application(projection.trait_application)
                .is_some_and(|app| {
                    app.item.0 == ids[1].0
                        && facts.types.arguments(app.arguments).is_some_and(|mut a| {
                            a.len() == 1 && a.next() == Some(Argument::Type(types[0]))
                        })
                })
        {
            if projection.subject == types[1] {
                return Ok(8);
            }
            if projection.subject == types[2] {
                return Ok(16);
            }
        }
    } else if p.bound == Bound::Sized && types.contains(&p.ty) {
        return Ok(0);
    }
    Err("source selector has an unsupported defining constraint")
}

#[cfg(test)]
#[path = "../../../tests/invariants/selector_contracts.rs"]
mod tests;
