//! Exact template relocation is distinct from each Call's actual late opening.
use super::{
    Context, DomainIndex, Work, g,
    nodes::{Arena, Key, MappedSlot, Region, Rewrite},
    s,
};
use crate::support::crates::owner_catalog::DeclarationRef;

pub(super) fn relocations(
    domains: &DomainIndex<'_, '_>,
    arena: &mut Arena,
    work: &mut Work,
) -> Result<(), &'static str> {
    let rows = domains.headers.rows();
    if domains.declaration_policies {
        return function_slot_map(domains, arena, work);
    }
    arena.reserve(
        rows.region_slots.len().saturating_mul(size_of::<u32>()),
        0,
        work,
    )?;
    work.charge(rows.region_slots.len())?;
    arena.relocations = vec![0; rows.region_slots.len()];
    for header in &rows.functions {
        work.charge(1)?;
        // Version thirteen: an inherent method's captured slots lead both of
        // its frames unchanged; the late table relocates only its own slots.
        let captures = header.regions.captures;
        for (public, slot) in header
            .regions
            .late
            .slice(&rows.region_slots)?
            .iter()
            .enumerate()
        {
            work.charge(1)?;
            let canonical = rows.slots[slot.0 as usize]
                .index
                .checked_sub(captures)
                .ok_or("source public relocation names a captured slot")?;
            let index = (header.regions.late.start as usize)
                .checked_add(canonical as usize)
                .ok_or("source public relocation index overflow")?;
            *arena
                .relocations
                .get_mut(index)
                .ok_or("source public relocation exceeds its exact slot map")? =
                u32::try_from(public)
                    .ok()
                    .and_then(|public| public.checked_add(captures))
                    .ok_or("source public relocation slot overflow")?;
        }
    }
    Ok(())
}

fn function_slot_map(
    domains: &DomainIndex<'_, '_>,
    arena: &mut Arena,
    work: &mut Work,
) -> Result<(), &'static str> {
    let rows = domains.headers.rows();
    work.charge(rows.functions.len().saturating_add(rows.region_slots.len()))?;
    let mut total_own = 0usize;
    for header in &rows.functions {
        let own = header
            .regions
            .total
            .checked_sub(header.regions.captures)
            .ok_or("source function captures exceed total regions")?;
        total_own = total_own
            .checked_add(own as usize)
            .ok_or("source function region slot map overflow")?;
    }
    arena.reserve(
        rows.functions
            .len()
            .saturating_mul(size_of::<u32>())
            .saturating_add(total_own.saturating_mul(size_of::<MappedSlot>())),
        0,
        work,
    )?;
    work.charge(total_own)?;
    arena.fn_slot_base = Vec::with_capacity(rows.functions.len());
    arena.slot_map = vec![MappedSlot::Unused; total_own];
    let mut cursor = 0usize;
    for header in &rows.functions {
        arena
            .fn_slot_base
            .push(u32::try_from(cursor).map_err(|_| "source function slot map base overflow")?);
        let captures = header.regions.captures;
        let own = (header.regions.total - captures) as usize;
        for (offset, slot) in header
            .regions
            .early
            .slice(&rows.region_slots)?
            .iter()
            .enumerate()
        {
            work.charge(1)?;
            let canonical = rows.slots[slot.0 as usize]
                .index
                .checked_sub(captures)
                .ok_or("source early slot names a captured slot")?;
            let arg = u32::try_from(offset)
                .ok()
                .and_then(|offset| offset.checked_add(captures))
                .ok_or("source early argument index overflow")?;
            arena.slot_map[cursor + canonical as usize] = MappedSlot::Early(arg);
        }
        for (offset, slot) in header
            .regions
            .late
            .slice(&rows.region_slots)?
            .iter()
            .enumerate()
        {
            work.charge(1)?;
            let canonical = rows.slots[slot.0 as usize]
                .index
                .checked_sub(captures)
                .ok_or("source late slot names a captured slot")?;
            let late = u32::try_from(offset).map_err(|_| "source late slot index overflow")?;
            arena.slot_map[cursor + canonical as usize] = MappedSlot::Late(late);
        }
        cursor += own;
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
    // Function recipes (including those nested inside closures or applied
    // arguments) retain formation ownership even with an empty lifetime domain.
    let (closed, function) = match rewrite {
        Rewrite::Application(application) => (
            domains.types.closed_application(application, work)?,
            domains.types.application_has_function(application, work)?,
        ),
        _ => (
            domains.types.closed_root(root, work)?,
            domains.types.has_function(root, work)?,
        ),
    };
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
    if !matches!(input.rewrite, Rewrite::Application(_)) {
        return Ok(None);
    }
    let s::Argument::Type(root) = applied_argument(domains, input.rewrite, slot, work)? else {
        return Err("source signature application omitted its complete actual type argument");
    };
    key(domains, root, Rewrite::Original, input.caller, work).map(Some)
}

/// The application's actual argument at a type or const formal's position.
pub(super) fn applied_argument(
    domains: &DomainIndex<'_, '_>,
    rewrite: Rewrite,
    slot: s::SlotRef,
    work: &mut Work,
) -> Result<s::Argument, &'static str> {
    let Rewrite::Application(application) = rewrite else {
        return Err("source applied formal has no actual application");
    };
    work.charge(1)?;
    let application = domains
        .headers
        .rows()
        .function_applications
        .get(application.0 as usize)
        .ok_or("source signature application is unknown")?;
    let target = domains.target(application.declaration.declaration, work)?;
    let position = domains
        .headers
        .scopes()
        .type_argument_position(target.scope(), slot, work)?;
    let arguments = application
        .arguments
        .slice(&domains.headers.rows().arguments)?;
    // A lifetime-generic enum constructor's region arguments lead its types.
    let regions = arguments
        .iter()
        .take_while(|argument| matches!(argument, s::Argument::Region(_)))
        .count();
    arguments
        .get(regions.saturating_add(position as usize))
        .copied()
        .ok_or("source signature application omitted its complete actual type argument")
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
            s::Region::Bound { slot, depth, .. } => {
                let binder = domains
                    .binders
                    .target(reference, work)?
                    .ok_or("source signature bound region has no actual ancestor")?;
                if matches!(
                    rows.binders[binder.0 as usize].kind,
                    s::BinderKind::DynamicType | s::BinderKind::CallableType
                ) {
                    return Ok(Region::ObjectBound { depth, slot });
                }
                match rewrite {
                    Rewrite::Nominal { .. } => {
                        return Err(
                            "source nominal field captured an unsupported declaration binder",
                        );
                    }
                    Rewrite::Original => Region::Bound { binder, slot },
                    Rewrite::Public(owner) => {
                        public_region(domains, arena, owner, binder, slot, work)?
                    }
                    Rewrite::Application(application) => {
                        return application_region(
                            domains,
                            arena,
                            application,
                            (binder, depth, slot),
                            work,
                        );
                    }
                }
            }
        },
    )
}

fn application_region(
    domains: &DomainIndex<'_, '_>,
    arena: &Arena,
    application: s::FunctionApplicationRef,
    (binder, depth, slot): (g::BinderRef, u16, u32),
    work: &mut Work,
) -> Result<Region, &'static str> {
    let rows = domains.headers.rows();
    if domains.declaration_policies && rows.binders[binder.0 as usize].kind == s::BinderKind::Clause
    {
        return Ok(Region::ObjectBound { depth, slot });
    }
    let target = rows
        .function_applications
        .get(application.0 as usize)
        .ok_or("source signature application is unknown")?;
    let Some(s::HeaderRef::Function(fn_index)) = domains
        .headers
        .header(target.declaration.declaration, work)?
    else {
        return Err("source application target has no actual function header");
    };
    let header = &rows.functions[fn_index as usize];
    let parent_binder = if binder == header.callable.binder
        || (domains.declaration_policies && binder == header.original.binder)
    {
        false
    } else if domains.declaration_policies
        && let Some(parent) = header.parent
        && domains.binders.declaration_frame(parent, work)? == Some(binder)
    {
        true
    } else {
        return Err("source callable template captured another actual binder");
    };
    let captures = header.regions.captures;
    if slot < captures {
        return applied_region_argument(domains, arena, target, slot, work);
    }
    if parent_binder {
        return Err("source callable parent binder exceeded its captured regions");
    }
    if !domains.declaration_policies {
        return Ok(Region::Late {
            binder: header.callable.binder,
            slot: slot - captures,
        });
    }
    let base = *arena
        .fn_slot_base
        .get(fn_index as usize)
        .ok_or("source function slot map base is absent")? as usize;
    let mapped = arena
        .slot_map
        .get(base.saturating_add((slot - captures) as usize))
        .copied()
        .ok_or("source callable slot is outside its mapped region table")?;
    match mapped {
        MappedSlot::Early(arg) => applied_region_argument(domains, arena, target, arg, work),
        MappedSlot::Late(late) => Ok(Region::Late {
            binder: header.callable.binder,
            slot: late,
        }),
        MappedSlot::Unused => Err("source callable signature used an unmapped lifetime slot"),
    }
}

fn applied_region_argument(
    domains: &DomainIndex<'_, '_>,
    arena: &Arena,
    target: &s::FunctionApplication,
    index: u32,
    work: &mut Work,
) -> Result<Region, &'static str> {
    let rows = domains.headers.rows();
    let Some(&s::Argument::Region(argument)) =
        target.arguments.slice(&rows.arguments)?.get(index as usize)
    else {
        return Err("source captured method application omitted its region argument");
    };
    region(domains, arena, argument, Rewrite::Original, work)
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
    if domains.declaration_policies {
        if slot >= header.regions.total {
            return Err("source public relocation has no exact original slot");
        }
        return Ok(Region::Bound {
            binder: header.callable.binder,
            slot,
        });
    }
    let Some(own) = slot.checked_sub(header.regions.captures) else {
        return Ok(Region::Bound {
            binder: header.callable.binder,
            slot,
        });
    };
    let index = (header.regions.late.start as usize)
        .checked_add(own as usize)
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
