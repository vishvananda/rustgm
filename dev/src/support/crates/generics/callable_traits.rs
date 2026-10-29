//! V7 admits only the genuine declaration-only Destruct/Freeze protocols, never const calls.
use super::{HeaderIndex,SourceRows};
use crate::support::{constness::Constness,crates::{generics::{Work,source as s},owner_catalog::DeclarationRef}};
pub(super) fn destruct(rows:&SourceRows,headers:&HeaderIndex<'_>,header:&s::TraitHeader,pointee:DeclarationRef,work:&mut Work)->Result<(),&'static str>{
    work.charge(12)?;
    if header.flags.bits() != (s::TraitFlags::CONST | s::TraitFlags::DENY_IMPL | s::TraitFlags::DYN_INCOMPATIBLE)
        || header.region_captures!=0 || !header.region_bounds.slice(&rows.region_bounds)?.is_empty()
        || header.written_parameters!=0 || header.canonical_inputs!=1 || header.declared_inputs!=2
        || !header.members.slice(&rows.members)?.is_empty() || headers.scopes().complete_slots(header.scope,work)?!=1 {
        return Err("source Destruct requires its exact closed const marker declaration");
    }
    let [literal]=header.written_supertraits.slice(&rows.clause_indices)? else {return Err("source Destruct requires one actual PointeeSized supertrait");};
    let clause=rows.clauses.get(*literal as usize).ok_or("source Destruct supertrait input is absent")?;
    let s::Requirement::Trait{application,effect:Constness::Ordinary}=clause.requirement else{return Err("source Destruct supertrait changed its ordinary effect");};
    let actual=rows.trait_applications.get(application.0 as usize).ok_or("source Destruct supertrait application is absent")?;
    if actual.declaration != (s::DeclarationAddress{artifact:s::ArtifactRef(0),declaration:pointee}) || actual.subject!=header.self_type || !actual.arguments.slice(&rows.arguments)?.is_empty() {
        return Err("source Destruct supertrait differs from its canonical Self/PointeeSized identity");
    }
    let Some(s::Type::Parameter(slot))=rows.types.get(header.self_type.0 as usize) else{return Err("source Destruct requires its actual Self slot");};
    let slot=rows.slots.get(slot.0 as usize).ok_or("source Destruct Self slot is absent")?;
    if slot.owner!=header.declaration || slot.ordinal!=0 || slot.index!=0 || !matches!(slot.kind,s::ParameterKind::Type{sized:None}) {
        return Err("source Destruct changed its unsized, unqualified Self parameter");
    }
    Ok(())
}

pub(super) fn freeze(rows:&SourceRows,headers:&HeaderIndex<'_>,header:&s::TraitHeader,work:&mut Work)->Result<(),&'static str>{
    work.charge(8)?;
    if header.flags.bits() != (s::TraitFlags::UNSAFE | s::TraitFlags::AUTO)
        || header.region_captures!=0 || !header.region_bounds.slice(&rows.region_bounds)?.is_empty()
        || header.written_parameters!=0 || header.canonical_inputs!=0 || header.declared_inputs!=1
        || !header.members.slice(&rows.members)?.is_empty()
        || !header.written_supertraits.slice(&rows.clause_indices)?.is_empty()
        || headers.scopes().complete_slots(header.scope,work)?!=1 {
        return Err("source Freeze requires its exact empty unsafe auto declaration");
    }
    let Some(s::Type::Parameter(slot))=rows.types.get(header.self_type.0 as usize) else{return Err("source Freeze requires its actual Self slot");};
    let slot=rows.slots.get(slot.0 as usize).ok_or("source Freeze Self slot is absent")?;
    if slot.owner!=header.declaration || slot.ordinal!=0 || slot.index!=0 || !matches!(slot.kind,s::ParameterKind::Type{sized:None}) {
        return Err("source Freeze changed its unsized, unqualified Self parameter");
    }
    Ok(())
}
