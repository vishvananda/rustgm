//! Exact template relocation is distinct from each Call's actual late opening.
use super::{
    Context, DomainIndex, Work, g,
    nodes::{Arena, Key, Region, Rewrite},
    s,
};
use crate::support::crates::owner_catalog::DeclarationRef;

pub(super) fn relocations(
    domains: &DomainIndex<'_, '_>,
    arena: &mut Arena,
    work: &mut Work,
) -> Result<(), &'static str> {
    let rows = domains.headers.rows();
    arena.reserve(
        rows.region_slots.len().saturating_mul(size_of::<u32>()),
        0,
        work,
    )?;
    work.charge(rows.region_slots.len())?;
    arena.relocations = vec![0; rows.region_slots.len()];
    for header in &rows.functions {
        work.charge(1)?;
        for (public, slot) in header
            .regions
            .late
            .slice(&rows.region_slots)?
            .iter()
            .enumerate()
        {
            work.charge(1)?;
            let canonical = rows.slots[slot.0 as usize].index;
            let index = (header.regions.late.start as usize)
                .checked_add(canonical as usize)
                .ok_or("source public relocation index overflow")?;
            *arena
                .relocations
                .get_mut(index)
                .ok_or("source public relocation exceeds its exact slot map")? =
                u32::try_from(public).map_err(|_| "source public relocation slot overflow")?;
        }
    }
    Ok(())
}

pub(super) fn key(
    domains: &DomainIndex<'_, '_>,
    root: g::TypeRef,
    rewrite: Rewrite,
    caller: Option<Context>,
    work: &mut Work,
) -> Result<Key, &'static str> {
    // Only the already memoized exact dependence summary can remove context.
    // Application substitution can make an otherwise rigid template closed.
    let closed = match rewrite {
        Rewrite::Application(application) => domains.types.closed_application(application, work)?,
        _ => domains.types.closed_root(root, work)?,
    };
    // Actual function recipes retain formation ownership even when their
    // lifetime domain is empty. Applications retain their exact target instead.
    let function = matches!(
        domains.headers.rows().types.get(root.0 as usize),
        Some(s::Type::Function(_))
    ) && !matches!(rewrite, Rewrite::Application(_));
    if (!closed || function) && caller.is_none() {
        return Err("source contextual rewrite omitted its actual caller/frame owner");
    }
    Ok(Key {
        root,
        rewrite: if closed && !function && rewrite != Rewrite::Original {
            match rewrite {
                Rewrite::Public(_) => Rewrite::Original,
                _ => rewrite,
            }
        } else {
            rewrite
        },
        caller: if closed && !function { None } else { caller },
    })
}

pub(super) fn argument(
    domains: &DomainIndex<'_, '_>,
    input: Key,
    slot: s::SlotRef,
    work: &mut Work,
) -> Result<Option<Key>, &'static str> {
    let Rewrite::Application(application) = input.rewrite else {
        return Ok(None);
    };
    work.charge(1)?;
    let application = domains
        .headers
        .rows()
        .function_applications
        .get(application.0 as usize)
        .ok_or("source signature application is unknown")?;
    let target = domains.function(application.declaration.declaration, work)?;
    let position = domains
        .headers
        .scopes()
        .type_argument_position(target.scope, slot, work)?;
    let arguments = application
        .arguments
        .slice(&domains.headers.rows().arguments)?;
    let Some(&s::Argument::Type(root)) = arguments.get(position as usize) else {
        return Err("source signature application omitted its complete actual type argument");
    };
    key(domains, root, Rewrite::Original, input.caller, work).map(Some)
}

pub(super) fn region(
    domains: &DomainIndex<'_, '_>,
    arena: &Arena,
    reference: s::RegionRef,
    rewrite: Rewrite,
    work: &mut Work,
) -> Result<Region, &'static str> {
    work.charge(1)?;
    let rows = domains.headers.rows();
    Ok(
        match *rows
            .regions
            .get(reference.0 as usize)
            .ok_or("source signature region is unknown")?
        {
            s::Region::Static => Region::Static,
            s::Region::Variable { body, slot } => Region::Variable { body, slot },
            s::Region::Parameter { .. } => {
                return Err("source signature early-region transport is unsupported");
            }
            s::Region::Bound { slot, .. } => {
                let binder = domains
                    .binders
                    .target(reference, work)?
                    .ok_or("source signature bound region has no actual ancestor")?;
                match rewrite {
                    Rewrite::Original => Region::Bound { binder, slot },
                    Rewrite::Public(owner) => {
                        public_region(domains, arena, owner, binder, slot, work)?
                    }
                    Rewrite::Application(application) => {
                        let target = rows
                            .function_applications
                            .get(application.0 as usize)
                            .ok_or("source signature application is unknown")?;
                        let header = domains.function(target.declaration.declaration, work)?;
                        if binder != header.callable.binder {
                            return Err("source callable template captured another actual binder");
                        }
                        Region::Late { binder, slot }
                    }
                }
            }
        },
    )
}
fn public_region(
    domains: &DomainIndex<'_, '_>,
    arena: &Arena,
    owner: DeclarationRef,
    binder: g::BinderRef,
    slot: u32,
    work: &mut Work,
) -> Result<Region, &'static str> {
    let header = domains.function(owner, work)?;
    if binder != header.original.binder {
        return Err("source public relocation captured a foreign actual original binder");
    }
    let index = (header.regions.late.start as usize)
        .checked_add(slot as usize)
        .ok_or("source public relocation index overflow")?;
    let slot = *arena
        .relocations
        .get(index)
        .ok_or("source public relocation has no exact original slot")?;
    Ok(Region::Bound {
        binder: header.callable.binder,
        slot,
    })
}
