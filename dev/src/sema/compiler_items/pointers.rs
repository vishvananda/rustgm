//! Canonical source identities and closed function-pointer shims remain distinct proofs.
use crate::{
    resolve::{
        bindings::{Declaration, HeaderAttribute},
        body_origin::DeclarationOrigin,
        expansion::Report,
        model::DefId,
        package::Error,
    },
    sema::model::Declarations,
    support::{bindings::LangItem, constness::Constness},
    types::{
        Kind, TypeId, arguments::Argument, callable::Convention, instance::Instance,
        parameters::predicates::Predicate,
    },
};

pub(super) fn validate_member(
    report: &Report,
    facts: &Declarations,
    declaration: Declaration,
) -> Result<(), Error> {
    let checked = || -> Result<(), &'static str> {
        let owner = declaration
            .parent
            .ok_or("source fn_ptr_addr lacks its actual trait owner")?;
        if facts.associated.trait_owner(declaration.definition) != Some(owner) {
            return Err("source fn_ptr_addr has a different actual member owner");
        }
        let source = report
            .resolution
            .definitions
            .get(owner.0 as usize)
            .and_then(|d| d.node)
            .ok_or("source fn_ptr_addr owner has no declaration")?;
        let header = super::headers::Trait::source(
            report,
            facts,
            Declaration {
                definition: owner,
                parent: None,
                origin: source,
            },
        )?;
        header.canonical_inputs(facts)?;
        if !header.members.contains(&declaration.definition) {
            return Err("source fn_ptr_addr is absent from its retained trait members");
        }
        let function = facts
            .declaration_header(declaration.definition)?
            .ok_or("source fn_ptr_addr has no prepared signature")?;
        if function.declaration.source() != Some(declaration.origin) || function.body.is_some() {
            return Err("source fn_ptr_addr changed its required declaration identity");
        }
        Ok(())
    };
    checked().map_err(|message| Declarations::error(report, declaration.origin, message))
}

/// Accept the pinned function-pointer role in a consumer only with its actual
/// required member, declared supertraits, and restrictions.
pub(super) fn imported(
    report: &Report,
    facts: &Declarations,
    catalog: &super::Catalog,
    steps: &mut usize,
) -> Result<bool, &'static str> {
    *steps = steps.saturating_add(6);
    let owner = catalog.role(LangItem::FnPtrTrait)?;
    let member = catalog.role(LangItem::FnPtrAddr)?;
    let (Some(owner), Some(member)) = (owner, member) else {
        return if owner.is_none() && member.is_none() {
            Ok(false)
        } else {
            Err("imported FnPtr requires its actual trait and address member")
        };
    };
    let header = super::headers::Trait::current(report, facts, owner, steps)?;
    header.profile(
        &facts.types,
        super::headers::Flags {
            unsafe_: false,
            constant: false,
            auto_: false,
        },
        0,
    )?;
    if header.written.parameters != 0
        || header.written.supertraits != 2
        || header.members != [member.definition]
        || !header.attributes.contains(HeaderAttribute::DynIncompatible)
        || !facts
            .associated
            .imported_denies_implementation(owner.definition)?
    {
        return Err("imported FnPtr changed its restricted trait declaration");
    }
    *steps = steps.saturating_add(2 * header.inputs.len());
    for role in [LangItem::Copy, LangItem::Clone] {
        let bound = catalog
            .role(role)?
            .ok_or("imported FnPtr lost a required supertrait")?;
        if !header.requires(&facts.types, bound.definition, None, Constness::Ordinary) {
            return Err("imported FnPtr requires the actual Copy and Clone supertraits");
        }
    }
    imported_member(facts, member, &header)?;
    Ok(true)
}

fn imported_member(
    facts: &Declarations,
    member: super::Role,
    owner: &super::headers::Trait<'_>,
) -> Result<(), &'static str> {
    let function = facts
        .declaration_header(member.definition)?
        .ok_or("imported fn_ptr_addr lost its actual required signature")?;
    if member.parent != Some(owner.definition)
        || facts.associated.trait_owner(member.definition) != Some(owner.definition)
        || facts.generics.parents.get(&member.definition) != Some(&owner.definition)
        || facts
            .generics
            .parameters
            .get(&member.definition)
            .is_some_and(|own| !own.is_empty())
        || function.declaration != member.origin
        || function.body.is_some()
    {
        return Err("imported fn_ptr_addr changed its actual required member");
    }
    let Some(Kind::Function {
        parameters,
        result,
        bound_regions: 0,
        unsafe_: false,
        convention: Convention::RUST,
    }) = facts.types.kind(function.signature)
    else {
        return Err("imported fn_ptr_addr requires its ordinary Rust signature");
    };
    if facts.types.list(parameters) != Some(&[owner.self_type][..])
        || !unit_pointer(&facts.types, result)
    {
        return Err("imported fn_ptr_addr requires the exact Self to *const () contract");
    }
    Ok(())
}

pub(in crate::sema) fn application(
    facts: &Declarations,
    predicate: Predicate,
) -> Result<bool, &'static str> {
    let Some((id, effect)) = predicate.bound.trait_parts() else {
        return Ok(false);
    };
    let Some(catalog) = facts.source_bindings.as_deref() else {
        return Ok(false);
    };
    let Some(application) = facts.types.trait_application(id) else {
        return Err("invalid source FnPtr application");
    };
    let Some(declaration) = catalog.role(LangItem::FnPtrTrait)? else {
        return Ok(false);
    };
    if declaration.definition.0 != application.item.0 {
        return Ok(false);
    }
    catalog.validated_role(LangItem::FnPtrTrait)?;
    if facts
        .types
        .arguments(application.arguments)
        .is_none_or(|args| args.len() != 0)
    {
        return Err("source FnPtr has unexpected trait arguments");
    }
    Ok(effect == Constness::Ordinary)
}

pub(in crate::sema) fn predicate(
    facts: &Declarations,
    goal: Predicate,
) -> Result<bool, &'static str> {
    Ok(application(facts, goal)?
        && matches!(facts.types.kind(goal.ty), Some(Kind::Function { .. })))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Address {
    pub instance: Instance,
    pub definition: DefId,
    pub origin: DeclarationOrigin,
    pub signature: TypeId,
    pub receiver: TypeId,
    pub result: TypeId,
}

/// Exact canonical method data for a compiler shim, never an ordinary impl body.
pub(crate) fn address(
    facts: &mut Declarations,
    instance: Instance,
) -> Result<Option<Address>, &'static str> {
    let Some(Kind::FunctionItem {
        item,
        arguments,
        signature,
    }) = facts.types.kind(instance.ty())
    else {
        return Ok(None);
    };
    let Some(catalog) = facts.source_bindings.as_deref() else {
        return Ok(None);
    };
    let Some(member) = catalog.role(LangItem::FnPtrAddr)? else {
        return Ok(None);
    };
    if member.definition.0 != item.0 {
        return Ok(None);
    }
    catalog.validated_role(LangItem::FnPtrAddr)?;
    let owner = catalog
        .validated_role(LangItem::FnPtrTrait)?
        .ok_or("source fn_ptr_addr requires its completed actual FnPtr owner")?;
    if member.parent != Some(owner.definition)
        || facts.associated.trait_owner(member.definition) != Some(owner.definition)
    {
        return Err("source fn_ptr_addr shim belongs to a different actual trait");
    }
    let function = facts
        .declaration_header(member.definition)?
        .ok_or("source fn_ptr_addr lost its actual header")?;
    if function.declaration != member.origin || function.body.is_some() {
        return Err("source fn_ptr_addr changed its required actual declaration");
    }
    let (receiver, result) = operands(&facts.types, arguments, signature)?;
    let original = facts
        .lifetimes
        .callables
        .get(item)
        .ok_or("source fn_ptr_addr lacks its canonical callable")?
        .signature;
    if original != function.signature {
        return Err("source fn_ptr_addr canonical callable changed its declaration signature");
    }
    crate::sema::header_attributes::charge_source_contract_query(facts, 4)
        .map_err(|_| "source fn_ptr_addr query lacks or exhausted its actual producer budget")?;
    let mut fold = crate::sema::dispatch::callable_substitution(facts, instance)?;
    if fold.apply(&mut facts.types, original)? != signature {
        return Err("source fn_ptr_addr signature differs from its actual scoped substitution");
    }
    Ok(Some(Address {
        instance,
        definition: member.definition,
        origin: member.origin,
        signature,
        receiver,
        result,
    }))
}

fn operands(
    types: &crate::types::Store,
    arguments: crate::types::GenericArgs,
    signature: TypeId,
) -> Result<(TypeId, TypeId), &'static str> {
    let mut arguments = types
        .arguments(arguments)
        .ok_or("source fn_ptr_addr has invalid arguments")?;
    let (Some(Argument::Type(receiver)), None) = (arguments.next(), arguments.next()) else {
        return Err("source fn_ptr_addr shim requires only its actual Self argument");
    };
    if !matches!(types.kind(receiver), Some(Kind::Function { .. }))
        || types.contains_parameters(receiver) != Some(false)
        || types.contains_projections(receiver) != Some(false)
    {
        return Err("source fn_ptr_addr shim requires a closed actual function pointer");
    }
    let Some(Kind::Function {
        parameters,
        result,
        bound_regions: 0,
        unsafe_: false,
        convention: Convention::RUST,
    }) = types.kind(signature)
    else {
        return Err("source fn_ptr_addr shim requires its safe ordinary Rust signature");
    };
    if types.list(parameters) != Some(&[receiver][..]) || !unit_pointer(types, result) {
        return Err("source fn_ptr_addr shim requires the exact Self to *const () contract");
    }
    Ok((receiver, result))
}

fn unit_pointer(types: &crate::types::Store, ty: TypeId) -> bool {
    matches!(
        types.kind(ty),
        Some(Kind::Pointer {
            mutable: false,
            pointee: TypeId::UNIT
        })
    )
}
