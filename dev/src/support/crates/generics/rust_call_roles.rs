//! Version thirteen admits the actual Tuple marker and the three callable
//! roles as declarations only. Consumers re-authenticate every header, member
//! and signature; these structural checks grant no selection or call proof.
use super::{Constness, Convention, FunctionBody, HeaderIndex, SourceRows, TraitFlags, Work};
use crate::support::crates::{generics::source as s, owner_catalog::DeclarationRef};

/// The `Tuple` marker, then `FnOnce`, `FnMut` and `Fn`, in role order.
#[derive(Clone, Copy, Default)]
pub(super) struct Roles(pub [Option<DeclarationRef>; 4]);
impl Roles {
    pub(super) fn callable(self, declaration: DeclarationRef) -> bool {
        self.0[1..].contains(&Some(declaration))
    }
}

/// `None` means the header belongs to no callable role.
pub(super) fn trait_header(
    rows: &SourceRows,
    headers: &HeaderIndex<'_>,
    header: &s::TraitHeader,
    roles: Roles,
    work: &mut Work,
) -> Result<Option<()>, &'static str> {
    let Some(position) = roles.0.iter().position(|r| *r == Some(header.declaration)) else {
        return Ok(None);
    };
    work.charge(8)?;
    let flags = header.flags.bits();
    let (expected, parameters, members) = if position == 0 {
        // The marker's dyn-incompatibility claim is optional in source core.
        (
            flags & !TraitFlags::DYN_INCOMPATIBLE == TraitFlags::DENY_IMPL,
            0,
            0,
        )
    } else {
        (
            flags == TraitFlags::CONST,
            1,
            if position == 1 { 2 } else { 1 },
        )
    };
    if !expected
        || header.region_captures != 0
        || !header.region_bounds.slice(&rows.region_bounds)?.is_empty()
        || header.written_parameters != parameters
        || header.members.slice(&rows.members)?.len() != members
        || headers.scopes().complete_slots(header.scope, work)? != parameters + 1
        || headers.scopes().complete_type_slots(header.scope, work)? != parameters + 1
    {
        return Err("source callable role changed its exact closed declaration profile");
    }
    let supertraits = header.written_supertraits.slice(&rows.clause_indices)?;
    if supertraits.len() != usize::from(position >= 2) {
        return Err("source callable role changed its exact written hierarchy");
    }
    Ok(Some(()))
}

/// A required call method of a const callable role is conditional and uses
/// the rust-call convention for both its original and its public callable.
pub(super) fn required_member(
    function: &s::FunctionHeader,
    roles: Roles,
    work: &mut Work,
) -> Result<bool, &'static str> {
    let Some(parent) = function.parent else {
        return Ok(false);
    };
    if !roles.callable(parent) {
        return Ok(false);
    }
    work.charge(2)?;
    if function.body != FunctionBody::Absent
        || function.constness != Constness::Conditional
        || function.original.convention != Convention::RustCall
        || function.callable.convention != Convention::RustCall
        || function.original.unsafe_
        || function.callable.unsafe_
        || function.original.variadic
        || function.callable.variadic
    {
        return Err("source callable role method changed its required rust-call declaration");
    }
    Ok(true)
}
