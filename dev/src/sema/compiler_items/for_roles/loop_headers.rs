//! Prove the required iteration declarations before publishing their identities.
use super::{Catalog, Declaration, Declarations, DefId, ItemId, Kind, LangItem, Report};
use crate::{
    resolve::body_origin::DeclarationOrigin,
    sema::compiler_items::{
        Role,
        headers::{Flags, Trait},
    },
    support::constness::Constness,
    types::{
        GenericArgs, TypeId,
        callable::Convention,
        parameters::{clauses::Clause, predicates::Bound},
    },
};

pub(in crate::sema) fn source_role(
    report: &Report,
    facts: &Declarations,
    role: LangItem,
    enabled: bool,
) -> Result<Option<Declaration>, &'static str> {
    let catalog = facts
        .source_bindings
        .as_deref()
        .ok_or("source iteration lost its actual catalog")?;
    let Some(declaration) = catalog.language(role) else {
        return Ok(None);
    };
    if !enabled {
        return Err("source iteration declarations require version fourteen");
    }
    catalog.require_provider_report(report)?;
    let mut steps = 20;
    let checked = super::super::enum_roles::Group::Option
        .check(facts, catalog)
        .and_then(|option| check(report, facts, catalog, &mut steps, option));
    crate::sema::header_attributes::charge_source_contract_query(facts, steps)
        .map_err(|_| "iteration declaration query exceeds its source work budget")?;
    if !checked? {
        return Err("source iteration lost its actual protocol declarations");
    }
    Ok(Some(declaration))
}

pub(in crate::sema::compiler_items) fn imported(
    report: &Report,
    facts: &Declarations,
    catalog: &Catalog,
    steps: &mut usize,
    complete: &super::super::roles::RoleSet,
) -> Result<bool, &'static str> {
    check(
        report,
        facts,
        catalog,
        steps,
        LangItem::FOR_OPTION_ROLES
            .iter()
            .all(|&role| complete.contains(role)),
    )
}

fn check(
    report: &Report,
    facts: &Declarations,
    catalog: &Catalog,
    steps: &mut usize,
    option_complete: bool,
) -> Result<bool, &'static str> {
    *steps = steps.saturating_add(4);
    let roles = [
        catalog.role(LangItem::Iterator)?,
        catalog.role(LangItem::IteratorNext)?,
        catalog.role(LangItem::IntoIter)?,
    ];
    let [Some(iterator), Some(next), Some(into)] = roles else {
        if roles == [None; 3] {
            return Ok(false);
        }
        return Err("iteration requires its complete actual declaration group");
    };
    if !option_complete {
        return Err("iteration requires its completed Option declaration group");
    }
    let option = catalog
        .role(LangItem::Option)?
        .ok_or("iteration requires its actual Option declaration")?;
    let into_owner = trait_owner(facts, into)?;
    if next.parent != Some(iterator.definition) || into_owner.definition == iterator.definition {
        return Err("iteration methods have the wrong actual trait owners");
    }
    let iterator = trait_header(report, facts, iterator, steps)?;
    let into_iter = trait_header(report, facts, into_owner, steps)?;
    if iterator.origin.source().is_none() {
        super::defaults::imported(
            report,
            facts,
            catalog,
            &iterator,
            ItemId(option.definition.0),
            steps,
        )?;
    }
    let item = next_item(facts, next, &iterator, ItemId(option.definition.0))?;
    let (input, result) = function(facts, into, 0)?;
    if input != into_iter.self_type {
        return Err("iteration into_iter requires its actual by-value Self input");
    }
    let into_result = projection(facts, result, into_iter.self_type, into_iter.definition)?;
    check_results(
        facts,
        &into_iter,
        iterator.definition,
        into_result,
        item,
        result,
        steps,
    )?;
    Ok(true)
}

fn trait_owner(facts: &Declarations, method: Role) -> Result<Role, &'static str> {
    let definition = method
        .parent
        .ok_or("iteration method lacks its actual trait parent")?;
    let owner = facts
        .associated
        .owner(definition)
        .and_then(|owner| facts.associated.get(owner))
        .ok_or("iteration method lacks its actual trait owner")?;
    let origin = if let Some(source) = owner.source {
        source.into()
    } else {
        let catalog = facts
            .crates
            .as_deref()
            .and_then(crate::sema::crates::Linkage::origin_catalog)
            .ok_or("imported iteration lacks its actual trait declaration catalog")?;
        DeclarationOrigin::Imported(catalog.declaration(definition)?)
    };
    Ok(Role {
        definition,
        parent: None,
        origin,
    })
}

pub(super) fn trait_header<'a>(
    report: &Report,
    facts: &'a Declarations,
    role: Role,
    steps: &mut usize,
) -> Result<Trait<'a>, &'static str> {
    let header = Trait::current(report, facts, role, steps)?;
    header.profile(
        &facts.types,
        Flags {
            unsafe_: false,
            constant: true,
            auto_: false,
        },
        0,
    )?;
    if header.written.parameters != 0 || header.written.supertraits != 0 {
        return Err("iteration traits require their actual parameterless declarations");
    }
    Ok(header)
}

fn function(
    facts: &Declarations,
    role: Role,
    regions: u32,
) -> Result<(TypeId, TypeId), &'static str> {
    let header = facts
        .declaration_header(role.definition)?
        .ok_or("iteration method lacks its actual function header")?;
    let Some(Kind::Function {
        parameters,
        result,
        unsafe_: false,
        bound_regions,
        convention: Convention::RUST,
    }) = facts.types.kind(header.signature)
    else {
        return Err("iteration method requires its actual safe Rust signature");
    };
    let Some([input]) = facts.types.list(parameters) else {
        return Err("iteration method requires one actual input");
    };
    if header.body.is_some()
        || header.declaration != role.origin
        || facts.associated.trait_owner(role.definition) != role.parent
    {
        return Err("iteration method changed its actual required declaration");
    }
    if header.universals != regions || bound_regions != regions {
        return Err("iteration method changed its actual lifetime binder");
    }
    if facts
        .generics
        .parameters
        .get(&role.definition)
        .is_some_and(|parameters| !parameters.is_empty())
        || role.origin.source().is_some() && facts.nonconst_trait_member(role.definition)?
    {
        return Err("iteration method changed its actual type formals or conditional effect");
    }
    Ok((*input, result))
}

fn next_item(
    facts: &Declarations,
    role: Role,
    iterator: &Trait<'_>,
    option: ItemId,
) -> Result<DefId, &'static str> {
    let (input, result) = function(facts, role, 1)?;
    if !matches!(facts.types.kind(input), Some(Kind::Reference { referent, region, mutable: true })
        if referent == iterator.self_type && region.bound_parts() == Some((0, 0)))
    {
        return Err("iteration next requires its actual late-bound mutable Self input");
    }
    let Some(Kind::Adt(owner, arguments)) = facts.types.kind(result) else {
        return Err("iteration next requires its actual Option result");
    };
    let mut arguments = facts
        .types
        .arguments(arguments)
        .ok_or("iteration next lost its actual Option arguments")?;
    let (Some(crate::types::arguments::Argument::Type(item)), None) =
        (arguments.next(), arguments.next())
    else {
        return Err("iteration next requires its actual Option Item result");
    };
    if owner != option {
        return Err("iteration next returned a different actual Option declaration");
    }
    projection(facts, item, iterator.self_type, iterator.definition)
}

fn projection(
    facts: &Declarations,
    ty: TypeId,
    subject: TypeId,
    trait_: DefId,
) -> Result<DefId, &'static str> {
    let Some(Kind::Projection(projection)) = facts.types.kind(ty) else {
        return Err("iteration requires its actual associated type projection");
    };
    let member = DefId(projection.item.0);
    if projection.subject != subject
        || projection.arguments != GenericArgs::EMPTY
        || !matches!(facts.types.trait_application(projection.trait_application), Some(application)
            if application.item == ItemId(trait_.0) && application.arguments == GenericArgs::EMPTY)
        || facts.associated.trait_owner(member) != Some(trait_)
        || !facts.associated.types.contains_key(&member)
    {
        return Err("iteration projection changed its actual owner or inputs");
    }
    Ok(member)
}

fn check_results(
    facts: &Declarations,
    into: &Trait<'_>,
    iterator: DefId,
    into_result: DefId,
    item: DefId,
    result: TypeId,
    steps: &mut usize,
) -> Result<(), &'static str> {
    let contract = facts
        .associated
        .types
        .get(&into_result)
        .ok_or("IntoIterator has no actual iterator result contract")?;
    *steps = steps.saturating_add(contract.bounds.len());
    let mut bound = false;
    let mut equal = false;
    for &clause in &contract.bounds {
        let Clause::Predicate(predicate) = clause else {
            continue;
        };
        match predicate.bound {
            Bound::Trait(application) if predicate.ty == result => {
                bound |= matches!(facts.types.trait_application(application), Some(application)
                    if application.item == ItemId(iterator.0) && application.arguments == GenericArgs::EMPTY);
            }
            Bound::ProjectionEqual(expected)
                if projection(facts, predicate.ty, result, iterator) == Ok(item) =>
            {
                equal |= projection(facts, expected, into.self_type, into.definition)
                    .is_ok_and(|member| member != into_result);
            }
            _ => {}
        }
    }
    if !bound || !equal {
        return Err("IntoIterator changed its actual Iterator<Item = Self::Item> result bounds");
    }
    Ok(())
}

/// The selected override can acquire the unwritten const effect of the actual
/// provider blanket. Body and associated-result proof remain separate.
pub(in crate::sema) fn source_blanket_member(
    facts: &Declarations,
    catalog: &Catalog,
    actual: &crate::sema::traits::implementations::Implementation,
    target: ItemId,
    member: DefId,
) -> Result<bool, &'static str> {
    use crate::types::arguments::Argument;
    let (Some(iterator), Some(into)) = (
        catalog.language(LangItem::Iterator),
        catalog.language(LangItem::IntoIter),
    ) else {
        return Ok(false);
    };
    if into.parent.map(|owner| ItemId(owner.0)) != Some(target) {
        return Ok(false);
    }
    if actual.methods.as_slice() != [(into.definition, Some(member))]
        || actual.types.len() != 2
        || !actual.constants.is_empty()
    {
        return Err(
            "source iterator blanket requires its checked override and two associated results",
        );
    }
    let Some([parameter]) = facts
        .generics
        .parameters
        .get(&actual.owner)
        .map(Vec::as_slice)
    else {
        return Err("source iterator blanket requires its single actual generic parameter");
    };
    if parameter.argument != Argument::Type(actual.predicate.ty) || parameter.default.is_some() {
        return Err("source iterator blanket changed its impl-owned Self formal");
    }
    let declared = crate::sema::declaration_inputs::declared_own(
        &facts.generics,
        facts.crates.as_deref(),
        actual.owner,
    )?;
    let meta_sized = catalog
        .language(LangItem::MetaSized)
        .ok_or("source iterator blanket lost its actual MetaSized declaration")?;
    let exact = |index, trait_: Declaration, effect| {
        let Some(crate::sema::declaration_inputs::Input {
            value: Clause::Predicate(premise),
            ..
        }) = declared.get(index)
        else {
            return false;
        };
        premise.ty == actual.predicate.ty
            && premise
                .bound
                .trait_parts()
                .is_some_and(|(application, actual)| {
                    actual == effect
                        && facts
                            .types
                            .trait_application(application)
                            .is_some_and(|application| {
                                application.item.0 == trait_.definition.0
                                    && application.arguments == GenericArgs::EMPTY
                            })
                })
    };
    if declared.len() != 2
        || !exact(0, iterator, Constness::Conditional)
        || !exact(1, meta_sized, Constness::Ordinary)
    {
        return Err(
            "source iterator blanket requires its actual conditional Iterator and MetaSized premises",
        );
    }
    Ok(true)
}

/// The role marks the method. Its checked imported parent owns the impl.
pub(in crate::sema) fn imported_into_trait(
    facts: &Declarations,
    definition: DefId,
) -> Result<Option<Role>, &'static str> {
    let Some(catalog) = facts.source_bindings.as_deref() else {
        return Ok(None);
    };
    let Some(method) = catalog.validated_role(LangItem::IntoIter)? else {
        return Ok(None);
    };
    if method.parent != Some(definition) || method.origin.source().is_some() {
        return Ok(None);
    }
    trait_owner(facts, method).map(Some)
}
