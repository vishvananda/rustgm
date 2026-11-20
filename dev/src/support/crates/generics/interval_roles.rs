//! The range helper roles share one exact const-trait declaration.
use super::{FunctionBody, HeaderIndex, HeaderRef, Profile, SourceRows, TraitFlags};
use crate::support::{
    bindings::LangItem,
    constness::Constness,
    crates::{
        generics::{Convention, TypeRef, Work},
        owner_catalog::DeclarationRef,
    },
};

pub(super) const ROLES: [LangItem; 3] =
    [LangItem::RangeMin, LangItem::RangeMax, LangItem::RangeSub];

#[derive(Clone, Copy)]
pub(super) struct Family {
    pub owner: DeclarationRef,
    pub method: DeclarationRef,
}

pub(super) fn bound_roles(
    rows: &SourceRows,
    profile: Profile,
    work: &mut Work,
) -> Result<[Option<DeclarationRef>; 3], &'static str> {
    let mut declarations = [None; 3];
    if profile.declaration_policies() {
        for row in &rows.roles {
            work.charge(1)?;
            if let Some(index) = ROLES.iter().position(|&role| row.role == role)
                && (row.declaration.artifact.0 != 0
                    || declarations[index]
                        .replace(row.declaration.declaration)
                        .is_some())
            {
                return Err("source range transport changed its exact role binding");
            }
        }
    }
    Ok(declarations)
}

pub(super) fn family(
    rows: &SourceRows,
    headers: &HeaderIndex<'_>,
    members: [Option<DeclarationRef>; 3],
    work: &mut Work,
) -> Result<Option<Family>, &'static str> {
    if members == [None; 3] {
        return Ok(None);
    }
    let [Some(min), Some(max), Some(method)] = members else {
        return Err("source range transport requires all three completed roles");
    };
    work.charge(6)?;
    let parent = |member: DeclarationRef| {
        rows.declarations
            .get(member.0 as usize)
            .and_then(|record| record.definition.parent)
    };
    let owner = parent(method).ok_or("source RangeSub requires its actual trait owner")?;
    let Some(HeaderRef::Trait(index)) = headers.header(owner, work)? else {
        return Err("source range roles require their actual trait header");
    };
    let header = &rows.traits[index as usize];
    let actual = header.members.slice(&rows.members)?;
    let flags = header.flags.bits() & (TraitFlags::CONST | TraitFlags::UNSAFE | TraitFlags::AUTO);
    if flags != TraitFlags::CONST
        || header.region_captures != 0
        || header.written_parameters != 0
        || header.written_supertraits.start != header.written_supertraits.end
        || actual.len() != 3
        || ![min, max, method]
            .iter()
            .all(|id| actual.contains(id) && parent(*id) == Some(owner))
    {
        return Err("source range roles changed their actual const trait member family");
    }
    for bound in [min, max] {
        let Some(HeaderRef::Constant(index)) = headers.header(bound, work)? else {
            return Err("source range bound requires its actual constant header");
        };
        let constant = &rows.constants()[index as usize];
        if constant.body != super::ConstantBody::Required
            || !same_type(rows, constant.ty, header.self_type)
        {
            return Err("source range bound requires its actual required Self declaration");
        }
    }
    let Some(HeaderRef::Function(index)) = headers.header(method, work)? else {
        return Err("source RangeSub requires its actual method header");
    };
    let raw = &rows.functions[index as usize];
    if raw.parent != Some(owner)
        || raw.body != FunctionBody::Absent
        || raw.constness != Constness::Conditional
        || raw.regions.total != 0
        || raw.declared_inputs != 0
        || headers.scopes().complete_slots(raw.scope, work)?
            != headers.scopes().complete_slots(header.scope, work)?
        || raw.formals.slice(&rows.formals)?.len() != 1
    {
        return Err("source RangeSub requires its actual conditional required method");
    }
    for signature in [raw.original, raw.callable] {
        let [subject] = signature.parameters.slice(&rows.roots)? else {
            return Err("source RangeSub requires one actual Self input");
        };
        if signature.unsafe_
            || signature.variadic
            || signature.convention != Convention::Rust
            || !same_type(rows, *subject, header.self_type)
            || !same_type(rows, signature.result, header.self_type)
        {
            return Err("source RangeSub requires its safe Rust Self signature");
        }
    }
    Ok(Some(Family { owner, method }))
}

fn same_type(rows: &SourceRows, left: TypeRef, right: TypeRef) -> bool {
    rows.types
        .get(left.0 as usize)
        .is_some_and(|left| rows.types.get(right.0 as usize) == Some(left))
}
