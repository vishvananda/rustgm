//! Genuine private named templates retain full formals without publishing proofs.
use super::*;
use crate::unpaid_cases::{Result, actual, extended};
use std::{fmt::Write as _, sync::Arc};

fn named(inventory: &crate::support::crates::Dependencies, name: &str) -> Result<DeclarationRef> {
    let rows = inventory.artifacts()[0]
        .source_descriptor()
        .ok_or("actual descriptor")?
        .rows();
    rows.functions
        .iter()
        .find(|row| {
            rows.declarations[row.declaration.0 as usize]
                .name
                .as_deref()
                == Some(name)
        })
        .map(|row| row.declaration)
        .ok_or_else(|| "actual named function".into())
}
fn ready(progress: Progress) -> Result<Box<Session>> {
    let (session, status) = progress;
    if status != Demand::Ready(()) {
        return Err("actual callable unexpectedly blocked".into());
    }
    Ok(session)
}
fn start(
    source: &str,
    name: &str,
    work: &mut ReservationWork,
) -> Result<(Box<Session>, DeclarationRef)> {
    let (report, facts, inventory) = extended(source)?;
    let owner = named(&inventory, name)?;
    let session = Session::prepare(report, facts, inventory, 0, work)?.boxed(work)?;
    Ok((session, owner))
}
fn prepared(session: &Session) -> Result<&Prepared> {
    session
        .recipes
        .as_ref()
        .and_then(|state| state.callables.as_deref())
        .ok_or_else(|| "actual private callable owner".into())
}
#[test]
fn private_callable_templates_retain_full_family_and_unused_formals() -> Result {
    let (report, facts, inventory) = actual(true)?;
    let rows = inventory.artifacts()[0]
        .source_descriptor()
        .ok_or("actual descriptor")?
        .rows();
    assert_eq!(
        (rows.functions.len(), rows.originals.len(), rows.roles.len()),
        (4, 3, 9)
    );
    let mut work = ReservationWork::default();
    let mut session =
        Session::prepare(report, facts, Arc::clone(&inventory), 0, &mut work)?.boxed(&mut work)?;
    let old_functions = session.facts.functions.len();
    let old = session.old_definitions;
    for header in &rows.functions {
        session = ready(session.callable_template(header.declaration, &mut work)?)?;
        let table = prepared(&session)?;
        let item = ItemId(
            session
                .inputs
                .recipe_view(&inventory, 0, &mut work)?
                .definition(header.declaration, &mut work)?
                .0,
        );
        let record = table
            .ready
            .iter()
            .flatten()
            .find(|r| r.item == item)
            .ok_or("real readiness")?;
        assert_eq!(record.header, *header);
        assert_eq!(
            table.templates.get(item).ok_or("real template")?.signature,
            record.pair.callable
        );
        let expected = usize::from(
            header.parent.is_some_and(|parent| {
                matches!(rows.header(parent), Ok(Some(s::HeaderRef::Trait(_))))
            }) || rows.declarations[header.declaration.0 as usize]
                .name
                .as_deref()
                == Some("identity"),
        );
        assert_eq!(
            table
                .templates
                .generic_parameters(item)
                .ok_or("full formals")?
                .len(),
            expected
        );
        if rows.declarations[header.declaration.0 as usize]
            .name
            .as_deref()
            == Some("clone_from")
        {
            assert_eq!(
                header.constness,
                crate::support::constness::Constness::Ordinary
            );
            let s::FunctionBody::Original(body) = header.body else {
                return Err("real provided body".into());
            };
            assert_eq!(
                rows.originals[body.0 as usize].context,
                s::CallContext::ConstDefinition
            );
        }
    }
    assert_eq!(session.facts.functions.len(), old_functions);
    assert!(session.facts.definitions[old..].iter().all(Option::is_none));
    session.require(&mut work)?;
    assert!(
        inventory
            .source_provider_index(inventory.artifacts()[0].identity())
            .is_err()
    );
    let (session, owner) = start(
        "\nfn unused<T>(x:u8)->u8{x}\nfn plain(x:u8)->u8{x}",
        "unused",
        &mut work,
    )?;
    let session = ready(session.callable_template(owner, &mut work)?)?;
    let record = prepared(&session)?
        .ready
        .iter()
        .flatten()
        .next()
        .ok_or("unused formal record")?;
    assert_eq!(record.formals, 1);
    assert_eq!(
        session
            .facts
            .types
            .contains_parameters(record.pair.callable),
        Some(false)
    );
    let plain = named(&session.inventory, "plain")?;
    let session = ready(session.callable_template(plain, &mut work)?)?;
    assert!(
        prepared(&session)?
            .ready
            .iter()
            .flatten()
            .any(|r| r.formals == 0)
    );
    Ok(())
}
#[test]
fn private_callable_templates_reject_changed_provenance_and_incomplete_formals() -> Result {
    for opponent in 0..7 {
        let mut work = ReservationWork::default();
        let (session, owner) = start("\nfn unused<T>(x:u8)->u8{x}", "unused", &mut work)?;
        let mut session = ready(session.callable_template(owner, &mut work)?)?;
        match opponent {
            0 => {
                let (_, _, other) = actual(true)?;
                session.inventory = other;
            }
            1 => {
                session.facts.types = Store::default();
            }
            2 => {
                session.provider_index = 1;
            }
            3 => {
                let record = session
                    .recipes
                    .as_mut()
                    .and_then(|s| s.callables.as_mut())
                    .and_then(|p| p.ready.iter_mut().flatten().next())
                    .ok_or("actual record")?;
                record.item = ItemId(u32::MAX);
            }
            4 => {
                session
                    .recipes
                    .as_mut()
                    .and_then(|s| s.callables.as_mut())
                    .ok_or("actual prepared owner")?
                    .templates = Templates::default();
            }
            5 => {
                let other = named(&session.inventory, "identity")?;
                session = ready(session.callable_template(other, &mut work)?)?;
                let table = session
                    .recipes
                    .as_mut()
                    .and_then(|s| s.callables.as_mut())
                    .ok_or("actual private template owner")?;
                let wrong = *table
                    .ready
                    .iter()
                    .flatten()
                    .find(|r| r.header.declaration == other)
                    .ok_or("other actual signature")?;
                let selected = table
                    .ready
                    .iter_mut()
                    .flatten()
                    .find(|r| r.header.declaration == owner)
                    .ok_or("selected actual signature")?;
                selected.pair = wrong.pair;
            }
            _ => {
                let state = session.recipes.as_mut().ok_or("actual recipe state")?;
                let table = state.callables.as_mut().ok_or("actual table")?;
                for row in &mut table.ready {
                    *row = None;
                }
                table.scopes[owner.0 as usize] = None;
            }
        }
        let before = work.steps;
        assert!(session.callable_template(owner, &mut work).is_err());
        assert!(work.steps > before);
    }
    Ok(())
}
fn nested(depth: usize) -> String {
    let mut ty = "u8".to_owned();
    for _ in 0..depth {
        ty = format!("fn({ty})->u8");
    }
    ty
}
#[test]
fn private_callable_templates_bound_selected_work_and_warm_storage() -> Result {
    let mut deep_warm = None;
    for depth in [1, 4, 8] {
        let source = format!("\nfn nested<T>(x:{})->u8{{0u8}}", nested(depth));
        let mut work = ReservationWork::default();
        let (session, owner) = start(&source, "nested", &mut work)?;
        let mut session = ready(session.callable_template(owner, &mut work)?)?;
        let bytes = prepared(&session)?.retained_bytes();
        let counters = session
            .recipes
            .as_ref()
            .ok_or("actual memo")?
            .cursor
            .counters();
        let extent = session.facts.types.interned_extent();
        assert!(counters.expanded <= 6 * depth + 16);
        let before = work.steps;
        let budget = work.retained;
        for _ in 0..1000 {
            session = ready(session.callable_template(owner, &mut work)?)?;
        }
        assert_eq!(work.retained, budget);
        assert!(work.steps - before < 100_000);
        let cost = work.steps - before;
        if let Some(previous) = deep_warm {
            assert_eq!(cost, previous);
        } else {
            deep_warm = Some(cost);
        }
        assert_eq!(session.facts.types.interned_extent(), extent);
        assert_eq!(
            session
                .recipes
                .as_ref()
                .ok_or("actual memo")?
                .cursor
                .counters(),
            counters
        );
        assert_eq!(prepared(&session)?.retained_bytes(), bytes);
        eprintln!(
            "private callable depth={depth} expanded={} edges={} warm1000={cost} retained={bytes}",
            counters.expanded, counters.edges
        );
    }
    let mut warm = None;
    for unrelated in [1, 8, 32] {
        let mut source = String::new();
        for index in 0..unrelated {
            writeln!(source, "\nfn unrelated{index}<T>(x:T)->T{{x}}")?;
        }
        let mut work = ReservationWork::default();
        let (session, owner) = start(&source, "identity", &mut work)?;
        let mut session = ready(session.callable_template(owner, &mut work)?)?;
        let before = work.steps;
        let retained = work.retained;
        for _ in 0..1000 {
            session = ready(session.callable_template(owner, &mut work)?)?;
        }
        let delta = (work.steps - before, work.retained - retained);
        if let Some(previous) = warm {
            assert_eq!(delta, previous);
        } else {
            warm = Some(delta);
        }
    }
    for width in [1, 8, 32] {
        let mut source = String::from("\nfn wide<");
        for index in 0..width {
            if index != 0 {
                source.push(',');
            }
            write!(source, "T{index}")?;
        }
        source.push_str(">(x:u8)->u8{x}");
        let mut work = ReservationWork::default();
        let (session, owner) = start(&source, "wide", &mut work)?;
        let session = ready(session.callable_template(owner, &mut work)?)?;
        let record = prepared(&session)?
            .ready
            .iter()
            .flatten()
            .next()
            .ok_or("actual wide record")?;
        assert_eq!(record.formals, width);
        assert!(work.peak >= work.retained);
        eprintln!(
            "private callable width={width} retained={} peak={} steps={}",
            prepared(&session)?.retained_bytes(),
            work.peak,
            work.steps
        );
    }
    Ok(())
}
#[test]
fn private_callable_templates_keep_failed_attempt_and_peak_costs() -> Result {
    registration_boundary()?;
    for storage in [false, true] {
        let mut work = ReservationWork::default();
        let (session, owner) = start("\nfn unused<T>(x:u8)->u8{x}", "unused", &mut work)?;
        let session = ready(session.signature_pair(owner, &mut work)?)?;
        if storage {
            work.retained = crate::support::crates::MAX_BYTES;
        } else {
            work.steps = crate::support::crates::MAX_COMPONENTS;
        }
        assert!(session.callable_template(owner, &mut work).is_err());
        if storage {
            assert!(work.peak > crate::support::crates::MAX_BYTES);
        } else {
            assert!(work.steps > crate::support::crates::MAX_COMPONENTS);
        }
    }
    assert!(size_of::<Ready>() > size_of::<SignaturePair>());
    assert!(size_of::<Prepared>() >= size_of::<Templates>());
    eprintln!(
        "private callable layout: ready={} owner={} pair={}",
        size_of::<Ready>(),
        size_of::<Prepared>(),
        size_of::<SignaturePair>()
    );
    let mut work = ReservationWork::default();
    let (session, owner) = start(
        "\nfn declared<'a>(x:&'a u8)->&'a u8{x}",
        "declared",
        &mut work,
    )?;
    let (mut session, status) = session.callable_template(owner, &mut work)?;
    assert!(matches!(status, Demand::Blocked(_)));
    assert!(
        session
            .recipes
            .as_ref()
            .is_some_and(|s| s.callables.is_none())
    );
    let bytes = work.retained;
    for _ in 0..1000 {
        let (next, repeated) = session.callable_template(owner, &mut work)?;
        assert_eq!(repeated, status);
        session = next;
    }
    assert_eq!(work.retained, bytes);
    Ok(())
}

fn registration_boundary() -> Result {
    let mut source = String::from("\nfn budget_wide<");
    for index in 0..32 {
        if index != 0 {
            source.push(',');
        }
        write!(source, "T{index}")?;
    }
    source.push_str(">(x:u8)->u8{x}");
    let mut work = ReservationWork::default();
    let (session, owner) = start(&source, "budget_wide", &mut work)?;
    let seed = named(&session.inventory, "identity")?;
    let session = ready(session.callable_template(seed, &mut work)?)?;
    let mut session = ready(session.signature_pair(owner, &mut work)?)?;
    let before = work.retained;
    let view = session
        .inputs
        .recipe_view(&session.inventory, 0, &mut work)?;
    let state = session.recipes.as_mut().ok_or("actual recipe state")?;
    let table = state.callables.as_deref().ok_or("actual template table")?;
    // Measure the real selected collector once, then retire that diagnostic
    // scratch. The following owned entry must fail specifically at registration
    // growth, after the identical authentic formal collection fits.
    let formals = formal_path::collect(
        view,
        &table.scopes,
        &mut state.maps,
        &mut session.facts.types,
        owner,
        &mut work,
    )?;
    assert_eq!(formals.parameters.len(), 32);
    let scratch = work.retained - before;
    let growth = table
        .templates
        .registration_allocation_bound(formals.parameters.len(), formals.owners.len());
    assert!(growth > 0);
    let allowance = scratch
        .checked_add(growth)
        .and_then(|n| n.checked_sub(1))
        .ok_or("actual registration boundary")?;
    drop(formals);
    work.retained = crate::support::crates::MAX_BYTES - allowance;
    assert!(session.callable_template(owner, &mut work).is_err());
    assert_eq!(work.retained, crate::support::crates::MAX_BYTES + 1);
    assert_eq!(work.peak, work.retained);
    Ok(())
}
