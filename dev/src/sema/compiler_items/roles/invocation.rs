//! Actual callable members retain their tuple ABI and source-owned Output identity.
use super::{
    Bound, CallTrait, Catalog, Declaration, Declarations, DefId, DefKind, Error, ItemKind, Kind,
    LangItem, Report, TypeId, function, item,
};
use crate::{
    support::constness::Constness,
    types::{arguments::Argument, callable::Convention},
};

pub(super) fn validate(
    report: &Report,
    facts: &Declarations,
    catalog: &Catalog,
    declaration: Declaration,
    role: LangItem,
    self_type: TypeId,
) -> Result<(CallTrait, DefId), Error> {
    let error = |message| Declarations::error(report, declaration.origin, message);
    let (kind, name) = match role {
        LangItem::FnOnce => (CallTrait::FnOnce, "call_once"),
        LangItem::FnMut => (CallTrait::FnMut, "call_mut"),
        LangItem::Fn => (CallTrait::Fn, "call"),
        _ => return Err(error("invalid callable source role")),
    };
    let tuple = catalog
        .language(LangItem::Tuple)
        .ok_or_else(|| error("source callable requires actual Tuple"))?;
    let output = catalog
        .language(LangItem::FnOnceOutput)
        .ok_or_else(|| error("source callable requires actual FnOnce Output"))?;
    let once = catalog
        .language(LangItem::FnOnce)
        .ok_or_else(|| error("source callable requires actual FnOnce"))?;
    if output.parent != Some(once.definition)
        || report.resolution.definitions[output.definition.0 as usize].kind != DefKind::TypeAlias
    {
        return Err(error(
            "source callable Output has a different owner or member kind",
        ));
    }
    let [parameter] = facts
        .generics
        .parameters
        .get(&declaration.definition)
        .map_or(&[][..], Vec::as_slice)
    else {
        return Err(error("source callable requires its one Args parameter"));
    };
    let Argument::Type(args) = parameter.argument else {
        return Err(error("source callable Args must be a type"));
    };
    let tuple_bound = facts.generics.predicates.get(&declaration.definition).into_iter().flatten().any(|p| {
        let crate::types::parameters::clauses::Clause::Predicate(p) = p.value else { return false; };
        p.ty == args && matches!(p.bound, Bound::Trait(id) if facts.types.trait_application(id)
            .is_some_and(|a| a.item.0 == tuple.definition.0 && facts.types.arguments(a.arguments).is_some_and(|a| a.len() == 0)))
    });
    if !tuple_bound {
        return Err(error(
            "source callable Args requires its actual Tuple obligation",
        ));
    }
    validate_parent(report, facts, catalog, declaration, kind, self_type, args)?;
    let owner = facts
        .associated
        .owner(declaration.definition)
        .ok_or_else(|| error("missing callable owner"))?;
    let mut methods = facts
        .associated
        .owner_members(owner)
        .ok_or_else(|| error("missing callable members"))?
        .iter()
        .filter_map(|m| {
            let d = &report.resolution.definitions[m.0 as usize];
            (d.kind == DefKind::Function && report.package.names.get(d.name) == name).then_some(*m)
        });
    let method = methods
        .next()
        .ok_or_else(|| error("source callable lacks its pinned method"))?;
    if methods.next().is_some() {
        return Err(error("source callable has duplicate methods"));
    }
    validate_method(
        report,
        facts,
        declaration,
        method,
        kind,
        (self_type, args),
        (once.definition, output.definition),
    )?;
    Ok((kind, method))
}

fn validate_parent(
    report: &Report,
    facts: &Declarations,
    catalog: &Catalog,
    declaration: Declaration,
    kind: CallTrait,
    self_type: TypeId,
    args: TypeId,
) -> Result<(), Error> {
    let error = |message| Declarations::error(report, declaration.origin, message);
    let ItemKind::Trait { bounds, .. } = item(report, declaration)?.kind else {
        return Err(error("missing source callable syntax"));
    };
    let expected = match kind {
        CallTrait::FnOnce => None,
        CallTrait::FnMut => Some((LangItem::FnOnce, Constness::Ordinary)),
        CallTrait::Fn => Some((LangItem::FnMut, Constness::Conditional)),
    };
    if report
        .package
        .unit(declaration.origin.file)
        .configured
        .ast
        .list(bounds)
        .len()
        != usize::from(expected.is_some())
    {
        return Err(error("source callable requires its exact pinned hierarchy"));
    }
    let Some((role, effect)) = expected else {
        return Ok(());
    };
    let parent = catalog
        .language(role)
        .ok_or_else(|| error("source callable parent is missing"))?;
    let found = facts
        .generics
        .predicates
        .get(&declaration.definition)
        .into_iter()
        .flatten()
        .any(|p| {
            let crate::types::parameters::clauses::Clause::Predicate(p) = p.value else {
                return false;
            };
            p.ty == self_type
                && p.bound.trait_parts().is_some_and(|(id, c)| {
                    c == effect
                        && facts.types.trait_application(id).is_some_and(|a| {
                            a.item.0 == parent.definition.0
                                && facts.types.arguments(a.arguments).is_some_and(|mut a| {
                                    a.len() == 1 && a.next() == Some(Argument::Type(args))
                                })
                        })
                })
        });
    if !found {
        return Err(error(
            "source callable hierarchy lost its Args or const qualification",
        ));
    }
    Ok(())
}

fn validate_method(
    report: &Report,
    facts: &Declarations,
    declaration: Declaration,
    method: DefId,
    kind: CallTrait,
    inputs: (TypeId, TypeId),
    identities: (DefId, DefId),
) -> Result<(), Error> {
    let error = |message| Declarations::error(report, declaration.origin, message);
    let (self_type, args) = inputs;
    let (once, output) = identities;
    let bound_regions = u32::from(kind != CallTrait::FnOnce);
    let f =
        function(facts, method).ok_or_else(|| error("source callable method is not prepared"))?;
    let Some(Kind::Function {
        bound_regions: actual,
        parameters,
        result,
        unsafe_: false,
        convention: Convention::RUST_CALL,
    }) = facts.types.kind(f.signature)
    else {
        return Err(error(
            "source callable requires its safe rust-call method signature",
        ));
    };
    let [receiver, actual_args] = facts
        .types
        .list(parameters)
        .ok_or_else(|| error("missing callable inputs"))?
    else {
        return Err(error(
            "source callable requires receiver and tuple arguments",
        ));
    };
    let valid_receiver = if kind == CallTrait::FnOnce {
        *receiver == self_type
    } else {
        matches!(facts.types.kind(*receiver), Some(Kind::Reference { mutable, referent, region })
            if referent == self_type && mutable == (kind == CallTrait::FnMut) && region.bound_parts() == Some((0, 0)))
    };
    let valid_result = matches!(facts.types.kind(result), Some(Kind::Projection(p))
        if p.item.0 == output.0 && p.subject == self_type
            && facts.types.arguments(p.arguments).is_some_and(|a| a.len() == 0)
            && facts.types.trait_application(p.trait_application).is_some_and(|a|
                a.item.0 == once.0 && facts.types.arguments(a.arguments)
                    .is_some_and(|mut a| a.len() == 1 && a.next() == Some(Argument::Type(args)))));
    if actual != bound_regions
        || !valid_receiver
        || *actual_args != args
        || !valid_result
        || facts
            .generics
            .parameters
            .get(&method)
            .is_some_and(|p| !p.is_empty())
    {
        return Err(error(
            "source callable method differs from its exact receiver/Args/Output contract",
        ));
    }
    Ok(())
}
