//! Direct trait contracts keep static-only members out of dynamic dispatch.
mod self_types;
use super::{Contract, Error, Rule, charge, predicate};
use crate::{
    resolve::{
        expansion::Report,
        model::{DefId, DefKind},
    },
    sema::{associated::OwnerKind, model::Declarations},
    types::{ItemId, Kind, TypeId, arguments::Argument, parameters::predicates::Bound},
};
use self_types::Occurrences;

pub(super) fn collect(
    report: &Report,
    facts: &Declarations,
    item: DefId,
    steps: &mut usize,
) -> Result<Contract, Error> {
    if crate::sema::compiler_items::unsizing_role(facts, item)?
        == Some(crate::support::bindings::LangItem::Unsize)
    {
        return Err("actual source Unsize is not dyn compatible".into());
    }
    if let Some(catalog) = facts.source_bindings.as_deref()
        && catalog.forces_dyn_incompatible(item)?
    {
        return Err("actual source trait requires dyn incompatibility".into());
    }
    if let Some(kind) = facts.associated.callable(item) {
        return callable(facts, item, kind, steps);
    }
    let owner = facts
        .associated
        .owner(item)
        .ok_or("missing dyn trait owner")?;
    if !matches!(facts.associated.get(owner).map(|o| o.kind),
        Some(OwnerKind::Trait { definition, .. }) if definition == item)
    {
        return Err("dyn compatibility requires a trait declaration".into());
    }
    let header = facts
        .generics
        .traits
        .as_ref()
        .and_then(|t| t.header(ItemId(item.0)))
        .ok_or("missing completed dyn trait header")?;
    if sized(facts, item, header.self_type)? {
        return Err(reject(item, Rule::Sized));
    }
    let mut contract = Contract {
        supertraits: Vec::new(),
        methods: Vec::new(),
        associated: Vec::new(),
    };
    let mut uses = Occurrences::new(header.self_type, false);
    for (index, bound) in header.inputs.view().iter().enumerate() {
        charge(steps, 1)?;
        let value = predicate(&facts.types, bound.value)?;
        if let Some((id, _)) = value.bound.trait_parts() {
            let application = facts
                .types
                .trait_application(id)
                .ok_or("invalid dyn trait requirement")?;
            if uses.arguments(&facts.types, application.arguments, steps)? {
                return Err(reject(item, Rule::SelfType));
            }
            if header.implies_input(&facts.types, index, bound)? {
                contract.supertraits.push(bound.value);
            }
        }
    }
    let mut uses = Occurrences::new(header.self_type, true);
    for &member in facts
        .associated
        .owner_members(owner)
        .ok_or("missing dyn trait members")?
    {
        charge(steps, 1)?;
        match report
            .resolution
            .definitions
            .get(member.0 as usize)
            .map(|d| d.kind)
        {
            Some(DefKind::Const) => return Err(reject(member, Rule::Constant)),
            Some(DefKind::TypeAlias) => {
                if sized(facts, member, header.self_type)? {
                    continue;
                }
                if facts
                    .generics
                    .parameters
                    .get(&member)
                    .is_some_and(|p| !p.is_empty())
                {
                    return Err(reject(member, Rule::GenericAssociated));
                }
                contract.associated.push(member);
            }
            Some(DefKind::Function) => {
                if crate::sema::declarations::is_final_trait_method(report, facts, member)
                    .map_err(|error| error.to_string())?
                {
                    continue;
                }
                if method(facts, member, header.self_type, &mut uses, steps)? {
                    contract.methods.push(member);
                }
            }
            _ => return Err("unexpected associated dyn declaration".into()),
        }
    }
    Ok(contract)
}

/// Language call traits own completed typed headers, without source AST owners.
fn callable(
    facts: &Declarations,
    item: DefId,
    kind: crate::support::call_traits::CallTrait,
    steps: &mut usize,
) -> Result<Contract, Error> {
    let header = facts
        .generics
        .traits
        .as_ref()
        .and_then(|traits| traits.header(ItemId(item.0)))
        .ok_or("missing completed callable object header")?;
    charge(steps, header.inputs.view().len().saturating_add(1))?;
    let associated = if kind == crate::support::call_traits::CallTrait::FnOnce {
        vec![
            facts
                .associated
                .callable_output()
                .ok_or("missing callable object Output declaration")?,
        ]
    } else {
        Vec::new()
    };
    Ok(Contract {
        supertraits: header.inputs.view().iter().map(|p| p.value).collect(),
        methods: Vec::new(),
        associated,
    })
}

fn sized(facts: &Declarations, item: DefId, self_type: TypeId) -> Result<bool, Error> {
    let Some(environment) = facts.generics.body_environment(item) else {
        return Ok(false);
    };
    environment
        .matching_sized(&facts.types, self_type, 0, &mut None)
        .map_err(Error::from)
}
fn reject(definition: DefId, rule: Rule) -> Error {
    Error::Incompatible { definition, rule }
}

fn method(
    facts: &Declarations,
    item: DefId,
    self_type: TypeId,
    uses: &mut Occurrences,
    steps: &mut usize,
) -> Result<bool, Error> {
    if sized(facts, item, self_type)? {
        return Ok(false);
    }
    let Some(&(receiver, references)) = facts.methods.receivers.get(&item) else {
        return Err(reject(item, Rule::Receiver));
    };
    if references == 0 && receiver == self_type {
        return Ok(false);
    }
    if references != 1
        || !matches!(facts.types.kind(receiver), Some(Kind::Reference { referent, .. }) if referent == self_type)
    {
        return Err(reject(item, Rule::Receiver));
    }
    if facts
        .generics
        .parameters
        .get(&item)
        .into_iter()
        .flatten()
        .any(|p| !matches!(p.argument, Argument::Lifetime(_)))
    {
        return Err(reject(item, Rule::GenericMethod));
    }
    let ty = facts
        .definitions
        .get(item.0 as usize)
        .copied()
        .flatten()
        .ok_or("missing dyn method type")?;
    let Some(Kind::FunctionItem { signature, .. }) = facts.types.kind(ty) else {
        return Err("dyn method has no callable identity".into());
    };
    let Some(Kind::Function {
        parameters, result, ..
    }) = facts.types.kind(signature)
    else {
        return Err("dyn method has no completed signature".into());
    };
    let parameters = facts
        .types
        .list(parameters)
        .ok_or("invalid dyn method parameters")?;
    if facts.types.contains_opaques(result) == Some(true) {
        return Err(reject(item, Rule::OpaqueReturn));
    }
    for &ty in parameters.iter().skip(1).chain(std::iter::once(&result)) {
        if uses.forbidden(&facts.types, ty, steps)? {
            return Err(reject(item, Rule::SelfType));
        }
    }
    requirements(facts, item, self_type, uses, steps)?;
    Ok(true)
}

fn requirements(
    facts: &Declarations,
    item: DefId,
    self_type: TypeId,
    uses: &mut Occurrences,
    steps: &mut usize,
) -> Result<(), Error> {
    for requirement in facts.generics.declared_predicates(item) {
        charge(steps, 1)?;
        let value = predicate(&facts.types, requirement.value)?;
        if matches!(value.bound, Bound::Outlives(_) | Bound::Auto(_)) && value.ty == self_type {
            continue;
        }
        let mut forbidden = uses.forbidden(&facts.types, value.ty, steps)?;
        match value.bound {
            Bound::Trait(id) | Bound::ConstTrait(id) | Bound::MaybeConstTrait(id) => {
                let application = facts
                    .types
                    .trait_application(id)
                    .ok_or("invalid dyn method bound")?;
                forbidden |= uses.arguments(&facts.types, application.arguments, steps)?;
            }
            Bound::ProjectionEqual(result) => {
                forbidden |= uses.forbidden(&facts.types, result, steps)?;
            }
            Bound::Outlives(_) | Bound::Copy | Bound::Clone | Bound::Sized | Bound::Auto(_) => {}
        }
        if forbidden {
            return Err(reject(item, Rule::SelfWhereClause));
        }
    }
    Ok(())
}
