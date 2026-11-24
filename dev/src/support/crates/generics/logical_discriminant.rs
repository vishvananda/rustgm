//! Authenticate the linked trait and type rows for logical discriminants.
use super::{HeaderIndex, SourceRows, Work};
use crate::support::crates::generics::source as s;
use crate::support::crates::owner_catalog::DeclarationRef;

pub(super) fn owned_members(
    rows: &SourceRows,
    headers: &HeaderIndex<'_>,
    roles: [Option<DeclarationRef>; 2],
    work: &mut Work,
) -> Result<(), &'static str> {
    let (owner, member) = match roles {
        [None, None] => return Ok(()),
        [Some(owner), Some(member)] => (owner, member),
        _ => return Err("source discriminant requires its actual trait and associated member"),
    };
    let Some(s::HeaderRef::Trait(index)) = headers.header(owner, work)? else {
        return Err("source discriminant requires its actual trait header");
    };
    let header = &rows.traits[index as usize];
    work.charge(6)?;
    if header.flags.bits() != (s::TraitFlags::DENY_IMPL | s::TraitFlags::DYN_INCOMPATIBLE)
        || header.written_parameters != 0
        || header.region_captures != 0
        || header.region_bounds.start != header.region_bounds.end
        || header.written_supertraits.start != header.written_supertraits.end
        || header.members.slice(&rows.members)? != [member]
    {
        return Err("source DiscriminantKind changed its exact marker or member contract");
    }
    let Some(s::HeaderRef::AssociatedType(index)) = headers.header(member, work)? else {
        return Err("source discriminant requires its actual associated type header");
    };
    let associated = &rows.associated_types[index as usize];
    if associated.parent != owner || associated.declared_inputs != 0 || !associated.sized {
        return Err("source discriminant changed its actual required associated member");
    }
    Ok(())
}
