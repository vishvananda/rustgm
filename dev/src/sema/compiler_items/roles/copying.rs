//! Copying roles validate declared contracts; no type gains a Copy permission here.
use super::{
    Bound, Catalog, Declaration, Declarations, DefId, Error, ItemKind, Kind, LangItem, Report,
    TypeId, function, item,
};
use crate::{support::constness::Constness, types::callable::Convention};

pub(super) fn validate(
    report: &Report,
    facts: &Declarations,
    catalog: &Catalog,
    declaration: Declaration,
    role: LangItem,
    self_type: TypeId,
    complete: &mut u16,
) -> Result<(), Error> {
    let error = |message| Declarations::error(report, declaration.origin, message);
    let expected = match role {
        LangItem::Copy => LangItem::Clone,
        LangItem::Clone => LangItem::Sized,
        LangItem::LegacyReceiver => LangItem::PointeeSized,
        _ => return Err(error("invalid copying role")),
    };
    require_super(
        report,
        facts,
        catalog,
        declaration,
        self_type,
        expected,
        Constness::Ordinary,
    )?;
    let owner = facts
        .associated
        .owner(declaration.definition)
        .ok_or_else(|| error("source copying role lost its owner"))?;
    let members = facts
        .associated
        .owner_members(owner)
        .ok_or_else(|| error("source copying role lost its members"))?;
    if role != LangItem::Clone {
        if !members.is_empty() {
            return Err(error("source copying marker must not declare members"));
        }
        return Ok(());
    }
    let method = catalog
        .language(LangItem::CloneFn)
        .ok_or_else(|| error("source Clone requires its actual clone_fn declaration"))?;
    if method.parent != Some(declaration.definition) || !members.contains(&method.definition) {
        return Err(error(
            "source clone_fn is not a member of its actual Clone owner",
        ));
    }
    let function = function(facts, method.definition)
        .ok_or_else(|| error("source clone_fn signature is not prepared"))?;
    let Some(Kind::Function {
        bound_regions: 1,
        parameters,
        result,
        unsafe_: false,
        convention: Convention::RUST,
    }) = facts.types.kind(function.signature)
    else {
        return Err(error(
            "source clone_fn requires a safe Rust signature with one borrow binder",
        ));
    };
    let parameters = facts
        .types
        .list(parameters)
        .ok_or_else(|| error("missing clone_fn inputs"))?;
    let [receiver] = parameters else {
        return Err(error("source clone_fn requires one shared receiver"));
    };
    if result != self_type
        || !matches!(facts.types.kind(*receiver), Some(Kind::Reference {
        mutable: false, referent, region,
    }) if referent == self_type && region.bound_parts() == Some((0, 0)))
        || facts
            .generics
            .parameters
            .get(&method.definition)
            .is_some_and(|p| !p.is_empty())
    {
        return Err(error(
            "source clone_fn does not retain the exact Self input/result contract",
        ));
    }
    *complete |= 1 << LangItem::CloneFn.index();
    Ok(())
}

pub(super) fn require_super(
    report: &Report,
    facts: &Declarations,
    catalog: &Catalog,
    declaration: Declaration,
    self_type: TypeId,
    expected: LangItem,
    constness: Constness,
) -> Result<(), Error> {
    let error = |message| Declarations::error(report, declaration.origin, message);
    let ItemKind::Trait { bounds, .. } = item(report, declaration)?.kind else {
        return Err(error("source role has no trait syntax"));
    };
    if report
        .package
        .unit(declaration.origin.file)
        .configured
        .ast
        .list(bounds)
        .len()
        != 1
    {
        return Err(error("source role requires its exact pinned supertrait"));
    }
    let expected = catalog
        .language(expected)
        .ok_or_else(|| error("source role is missing its prerequisite declaration"))?;
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
            if p.ty != self_type {
                return false;
            }
            if expected.definition
                == catalog
                    .language(LangItem::Sized)
                    .map_or(DefId(u32::MAX), |d| d.definition)
                && constness == Constness::Ordinary
                && p.bound == Bound::Sized
            {
                return true;
            }
            p.bound.trait_parts().is_some_and(|(id, effect)| {
                effect == constness
                    && facts.types.trait_application(id).is_some_and(|a| {
                        a.item.0 == expected.definition.0
                            && facts
                                .types
                                .arguments(a.arguments)
                                .is_some_and(|a| a.len() == 0)
                    })
            })
        });
    if !found {
        return Err(error("source role is missing its exact typed prerequisite"));
    }
    Ok(())
}
