//! Rebuild the imported discriminator family against its actual typed owner.
use super::{Contracts, Declarations, Error, LangItem, Rc, Report, authority, crates, roles};
use crate::{
    resolve::bindings::HeaderAttribute,
    sema::compiler_items::headers::{Flags, Trait},
    types::Kind,
};

pub(in crate::sema::compiler_items) fn prepare(
    report: &Report,
    facts: &mut Declarations,
    complete: &mut roles::RoleSet,
) -> Result<Option<Rc<Contracts>>, Error> {
    let mut steps = 0;
    let checked = imported_contract(report, facts, &mut steps);
    let bytes = if matches!(checked, Ok(Some(_))) {
        size_of::<Contracts>() + 2 * size_of::<usize>()
    } else {
        0
    };
    crates::charge_source_headers(facts, steps, bytes)?;
    let Some(contract) = checked.map_err(Error::unlocated)? else {
        return Ok(None);
    };
    complete.insert(LangItem::DiscriminantKind);
    complete.insert(LangItem::DiscriminantType);
    Ok(Some(Rc::new(contract)))
}

fn imported_contract(
    report: &Report,
    facts: &Declarations,
    steps: &mut usize,
) -> Result<Option<Contracts>, &'static str> {
    let catalog = facts
        .source_bindings
        .as_deref()
        .ok_or("imported discriminant lost its role catalog")?;
    *steps = steps.saturating_add(2);
    let (trait_, member) = match (
        catalog.role(LangItem::DiscriminantKind)?,
        catalog.role(LangItem::DiscriminantType)?,
    ) {
        (None, None) => return Ok(None),
        (Some(trait_), Some(member)) => (trait_, member),
        _ => return Err("imported discriminant requires its actual trait and associated member"),
    };
    let owner = facts
        .associated
        .imported_owner()
        .ok_or("imported discriminant lost its actual owner graph")?;
    owner.associated_language_role(facts, trait_, LangItem::DiscriminantKind, steps)?;
    owner.associated_language_role(facts, member, LangItem::DiscriminantType, steps)?;
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
    *steps = steps.saturating_add(6);
    if header.written.parameters != 0
        || header.written.supertraits != 0
        || header.members != [member.definition]
        || !header.attributes.contains(HeaderAttribute::DynIncompatible)
        || !facts
            .associated
            .imported_denies_implementation(trait_.definition)?
    {
        return Err("imported DiscriminantKind changed its actual marker or required result");
    }
    let contract = facts
        .associated
        .require_projection_header(member.definition)?;
    let Some(Kind::Projection(projection)) = facts.types.kind(contract.projection) else {
        return Err("imported discriminant lacks its actual canonical projection");
    };
    authority::validate_member(facts, trait_, member, header.self_type, projection)?;
    let arena = facts
        .types
        .existing_arena_identity()
        .ok_or("imported discriminator lost its checked type arena")?;
    Ok(Some(Contracts {
        arena: arena.clone(),
        trait_,
        member,
        self_type: header.self_type,
        projection,
        images: None,
    }))
}
