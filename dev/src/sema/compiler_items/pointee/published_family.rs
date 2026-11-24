//! Reconstruct the Pointee family from the authenticated imported declarations.
use super::{Declarations, LangItem, Report, schemas};
use crate::{
    sema::{
        compiler_items::{
            Role,
            headers::{Flags, Trait},
        },
        declaration_inputs,
    },
    support::constness::Constness,
};

pub(super) fn plan(
    report: &Report,
    facts: &Declarations,
    steps: &mut usize,
) -> Result<Option<schemas::Plan>, &'static str> {
    let catalog = facts
        .source_bindings
        .as_deref()
        .ok_or("imported Pointee family lacks its actual role catalog")?;
    *steps = steps.saturating_add(3);
    let trait_ = catalog.role(LangItem::PointeeTrait)?;
    let member = catalog.role(LangItem::MetadataType)?;
    let nominal = catalog.role(LangItem::DynMetadata)?;
    if trait_.is_none() && member.is_none() && nominal.is_none() {
        return Ok(None);
    }
    let trait_ = trait_.ok_or("imported Metadata lacks its actual Pointee role")?;
    let member = member.ok_or("imported Pointee lacks its actual Metadata role")?;
    let owner = facts
        .associated
        .imported_owner()
        .ok_or("imported Pointee roles lack their actual owner graph")?;
    owner.associated_language_role(facts, trait_, LangItem::PointeeTrait, steps)?;
    owner.associated_language_role(facts, member, LangItem::MetadataType, steps)?;
    let header = Trait::current(report, facts, trait_, steps)?;
    header.profile(
        &facts.types,
        Flags {
            unsafe_: false,
            auto_: false,
            constant: false,
        },
        0,
    )?;
    let pointee = catalog
        .validated_role(LangItem::PointeeSized)?
        .ok_or("imported Pointee requires its actual PointeeSized role")?;
    *steps = steps.saturating_add(header.inputs.len());
    if header.written.parameters != 0
        || header.written.supertraits != 1
        || !header.requires(&facts.types, pointee.definition, None, Constness::Ordinary)
        || header.members != [member.definition]
    {
        return Err("imported Pointee changed its direct bound, parameters or actual member");
    }
    require_member(facts, trait_, member)?;
    let (projection, schema) = schemas::projection_contract(
        facts,
        trait_.definition,
        member.definition,
        header.self_type,
    )?;
    if let Some(nominal) = nominal {
        owner.associated_language_role(facts, nominal, LangItem::DynMetadata, steps)?;
        let target = schemas::nominal_formal(facts, nominal.definition)?;
        let inputs = declaration_inputs::declared_own(
            &facts.generics,
            facts.crates.as_deref(),
            nominal.definition,
        )?;
        *steps = steps.saturating_add(inputs.len());
        schemas::require_nominal_bound(
            facts,
            target,
            pointee.definition,
            inputs.iter().map(|input| input.value),
        )?;
    }
    Ok(Some(schemas::Plan {
        trait_,
        member,
        nominal,
        self_type: header.self_type,
        projection,
        schema,
        attributes: header.attributes,
        denied_implementations: facts
            .associated
            .imported_denies_implementation(trait_.definition)?,
    }))
}

fn require_member(facts: &Declarations, trait_: Role, member: Role) -> Result<(), &'static str> {
    if member.parent != Some(trait_.definition)
        || facts.associated.trait_owner(member.definition) != Some(trait_.definition)
        || facts.generics.parents.get(&member.definition) != Some(&trait_.definition)
        || facts
            .generics
            .parameters
            .get(&member.definition)
            .is_some_and(|formals| !formals.is_empty())
        || facts
            .associated
            .families
            .get(&member.definition)
            .is_some_and(|family| {
                family.captures != 0 || family.total != 0 || family.requirements.is_none()
            })
    {
        return Err("imported Metadata changed its associated owner or formal contract");
    }
    Ok(())
}
