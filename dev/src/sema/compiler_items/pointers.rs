//! Canonical source identities and closed function-pointer shims remain distinct proofs.
use crate::{
    resolve::{
        bindings::Declaration, expansion::Report, model::DefId, package::Error, unit::NodeKey,
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
            .function_header(declaration.definition)?
            .ok_or("source fn_ptr_addr has no prepared signature")?;
        if function.declaration.source() != Some(declaration.origin) || function.body.is_some() {
            return Err("source fn_ptr_addr changed its required declaration identity");
        }
        Ok(())
    };
    checked().map_err(|message| Declarations::error(report, declaration.origin, message))
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
    let Some(declaration) = catalog.language(LangItem::FnPtrTrait) else {
        return Ok(false);
    };
    if declaration.definition.0 != application.item.0 {
        return Ok(false);
    }
    catalog.validated_language(LangItem::FnPtrTrait)?;
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
    pub source: NodeKey,
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
    let Some(member) = catalog.language(LangItem::FnPtrAddr) else {
        return Ok(None);
    };
    if member.definition.0 != item.0 {
        return Ok(None);
    }
    catalog.validated_language(LangItem::FnPtrAddr)?;
    let owner = catalog
        .validated_language(LangItem::FnPtrTrait)?
        .ok_or("source fn_ptr_addr requires its completed actual FnPtr owner")?;
    if member.parent != Some(owner.definition)
        || facts.associated.trait_owner(member.definition) != Some(owner.definition)
    {
        return Err("source fn_ptr_addr shim belongs to a different actual trait");
    }
    let function = facts
        .function_header(member.definition)?
        .ok_or("source fn_ptr_addr lost its actual header")?;
    if function.declaration.source() != Some(member.origin) || function.body.is_some() {
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
        source: member.origin,
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
    if types.list(parameters) != Some(&[receiver][..])
        || !matches!(types.kind(result),Some(Kind::Pointer { mutable:false, pointee }) if pointee==TypeId::UNIT)
    {
        return Err("source fn_ptr_addr shim requires the exact Self to *const () contract");
    }
    Ok((receiver, result))
}
