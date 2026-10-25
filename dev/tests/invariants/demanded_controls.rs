//! The shared demand cursor consumes real owned headers, never placeholder shapes.
use super::*;
use crate::unpaid_cases::{Result, actual, extended};
use std::fmt::Write as _;

fn named(inventory: &Dependencies, name: &str) -> Result<DeclarationRef> {
    let rows = inventory.artifacts()[0]
        .source_descriptor()
        .ok_or("actual descriptor")?
        .rows();
    rows.functions
        .iter()
        .find(|function| {
            rows.declarations[function.declaration.0 as usize]
                .name
                .as_deref()
                == Some(name)
        })
        .map(|function| function.declaration)
        .ok_or_else(|| "actual named function".into())
}
fn ready(progress: Progress) -> Result<Box<Session>> {
    let (owner, status) = progress;
    if status != Demand::Ready(()) {
        return Err("actual header signature unexpectedly blocked".into());
    }
    Ok(owner)
}
fn prepared(
    inventory: Arc<Dependencies>,
    report: Report,
    facts: Declarations,
    owner: DeclarationRef,
    work: &mut ReservationWork,
) -> Result<Box<Session>> {
    ready(Session::select_signature(
        report, facts, inventory, 0, owner, work,
    )?)
}
#[test]
fn header_signature_cursor_uses_real_full_family_ports() -> Result {
    let (report, facts, inventory) = actual(true)?;
    let source_functions = facts.functions.len();
    let old_cells = facts.definitions.len();
    let rows = inventory.artifacts()[0]
        .source_descriptor()
        .ok_or("actual descriptor")?
        .rows();
    let owners = rows
        .functions
        .iter()
        .map(|row| row.declaration)
        .collect::<Vec<_>>();
    assert_eq!(owners.len(), 4);
    assert_eq!(rows.originals.len(), 3);
    assert_eq!(rows.roles.len(), 9);
    let mut work = ReservationWork::default();
    let mut session = prepared(Arc::clone(&inventory), report, facts, owners[0], &mut work)?;
    for &owner in &owners {
        session = ready(session.signature_pair(owner, &mut work)?)?;
    }
    let state = session.recipes.as_ref().ok_or("actual recipe state")?;
    let mut bodies = 0;
    for pair in &state.pairs {
        let Pair::Ready(pair) = pair else {
            return Err("actual header pair omitted".into());
        };
        let raw = &rows.functions[pair.function_row as usize];
        let name = rows.declarations[raw.declaration.0 as usize]
            .name
            .as_deref();
        if let s::FunctionBody::Original(body) = raw.body {
            if name == Some("clone_from") {
                assert_eq!(
                    raw.constness,
                    crate::support::constness::Constness::Ordinary
                );
                assert_eq!(
                    rows.originals[body.0 as usize].context,
                    s::CallContext::ConstDefinition
                );
            } else if name == Some("clone") {
                assert_eq!(
                    rows.originals[body.0 as usize].context,
                    s::CallContext::Runtime
                );
            } else if name == Some("identity") {
                assert_eq!(raw.constness, crate::support::constness::Constness::Const);
            }
        }
        let Some(crate::types::Kind::Function { bound_regions, .. }) =
            session.facts.types.kind(pair.original)
        else {
            return Err("actual original signature".into());
        };
        assert_eq!(
            bound_regions,
            rows.binders[raw.original.binder.0 as usize].count
        );
        let Some(crate::types::Kind::Function { bound_regions, .. }) =
            session.facts.types.kind(pair.callable)
        else {
            return Err("actual public signature".into());
        };
        assert_eq!(
            bound_regions,
            rows.binders[raw.callable.binder.0 as usize].count
        );
        bodies += usize::from(pair.body.is_some());
    }
    assert_eq!(bodies, 3);
    assert_eq!(session.facts.functions.len(), source_functions);
    assert!(
        session.facts.definitions[old_cells..]
            .iter()
            .all(Option::is_none)
    );
    session.require(&mut work)?;
    assert!(
        inventory
            .source_provider_index(inventory.artifacts()[0].identity())
            .is_err()
    );
    Ok(())
}
#[test]
fn header_signature_cursor_rejects_foreign_owner_arena_and_purpose() -> Result {
    let (report, facts, inventory) = actual(true)?;
    let owner = named(&inventory, "clone_from")?;
    let rows = inventory.artifacts()[0]
        .source_descriptor()
        .ok_or("actual descriptor")?
        .rows();
    let other = rows
        .functions
        .iter()
        .find(|row| row.declaration != owner)
        .ok_or("other real function")?;
    let selected = rows
        .functions
        .iter()
        .find(|row| row.declaration == owner)
        .ok_or("actual selected function")?;
    assert!(
        signature_shapes::check(
            rows,
            Some((owner, s::BinderKind::OriginalFunction)),
            other.original
        )
        .is_err()
    );
    assert!(
        signature_shapes::check(
            rows,
            Some((owner, s::BinderKind::OriginalFunction)),
            selected.callable
        )
        .is_err()
    );
    let mut work = ReservationWork::default();
    let mut first = prepared(inventory, report, facts, owner, &mut work)?;
    let (report, facts, inventory) = actual(true)?;
    let mut second = prepared(inventory, report, facts, owner, &mut work)?;
    let view = second.inputs.recipe_view(&second.inventory, 0, &mut work)?;
    assert!(
        first
            .recipes
            .as_mut()
            .ok_or("actual first memo")?
            .pair(view, &mut first.facts.types, owner, &mut work)
            .is_err()
    );
    let view = first.inputs.recipe_view(&first.inventory, 0, &mut work)?;
    assert!(
        first
            .recipes
            .as_mut()
            .ok_or("actual first memo")?
            .pair(view, &mut second.facts.types, owner, &mut work)
            .is_err()
    );
    std::mem::swap(&mut first.recipes, &mut second.recipes);
    assert!(first.signature_pair(owner, &mut work).is_err());
    let (report, facts, inventory) = actual(true)?;
    assert!(
        Session::select_signature(
            report,
            facts,
            inventory,
            0,
            DeclarationRef(u32::MAX),
            &mut work
        )
        .is_err()
    );
    Ok(())
}
fn unavailable_root(rows: s::ValidatedView<'_>, owner: DeclarationRef) -> Result<g::TypeRef> {
    let Some(s::HeaderRef::Function(index)) = rows.header(owner)? else {
        return Err("actual function".into());
    };
    let s::FunctionBody::Original(body) = rows.functions[index as usize].body else {
        return Err("actual provided body".into());
    };
    let slot = rows.types.iter().position(|row| matches!(row, s::Type::FunctionItem(_) | s::Type::ApplicationSignature(_)))
        .or_else(|| rows.types.iter().position(|row| matches!(row, s::Type::Reference { region, .. }
            if matches!(rows.regions[region.0 as usize], s::Region::Variable { body: actual, .. } if actual == body))))
        .ok_or("actual retained body recipe requiring a later producer")?;
    Ok(g::TypeRef(u32::try_from(slot)?))
}
fn dependency(
    rows: s::ValidatedView<'_>,
    probe: g::TypeRef,
    view: &RecipeView<'_>,
    work: &mut ReservationWork,
) -> Result<Dependency> {
    match rows.types[probe.0 as usize] {
        s::Type::FunctionItem(application) | s::Type::ApplicationSignature(application) => {
            let row = rows.function_applications[application.0 as usize];
            Ok(Dependency::CallableTemplate(
                view.declaration(row.declaration.declaration, work)?,
            ))
        }
        s::Type::Reference { region, .. } => match rows.regions[region.0 as usize] {
            s::Region::Variable { body, .. } => {
                Ok(Dependency::OwnedBodyDomain(view.body(body, work)?))
            }
            _ => Err("actual unavailable body domain".into()),
        },
        _ => Err("actual unavailable recipe".into()),
    }
}
#[test]
fn header_signature_cursor_reuses_memo_and_leaves_undemanded_body_nodes() -> Result {
    let (report, facts, inventory) = actual(true)?;
    let owner = named(&inventory, "clone_from")?;
    let rows = inventory.artifacts()[0]
        .source_descriptor()
        .ok_or("actual descriptor")?
        .rows();
    let probe = unavailable_root(rows, owner)?;
    let mut work = ReservationWork::default();
    let mut session = prepared(Arc::clone(&inventory), report, facts, owner, &mut work)?;
    let state = session.recipes.as_ref().ok_or("actual memo")?;
    let counters = state.cursor.counters();
    let bytes = state.retained_bytes();
    let types = session.facts.types.interned_extent();
    let before = work.steps;
    session = ready(session.signature_pair(owner, &mut work)?)?;
    let cost = work.steps - before;
    let before = work.steps;
    for _ in 0..1000 {
        session = ready(session.signature_pair(owner, &mut work)?)?;
    }
    assert_eq!(work.steps - before, cost * 1000);
    let view = session
        .inputs
        .recipe_view(&session.inventory, 0, &mut work)?;
    let state = session.recipes.as_mut().ok_or("actual memo")?;
    assert_eq!(state.cursor.counters(), counters);
    assert_eq!(state.retained_bytes(), bytes);
    assert_eq!(session.facts.types.interned_extent(), types);
    let expected = Demand::Blocked(dependency(rows, probe, &view, &mut work)?);
    let mut port = Reserved::new(view, &mut state.maps)?;
    assert_eq!(
        state
            .cursor
            .demand(&mut port, &mut session.facts.types, probe, &mut work)?,
        expected
    );
    assert_eq!(session.facts.types.interned_extent(), types);
    let stopped = state.cursor.counters();
    for _ in 0..32 {
        assert_eq!(
            state
                .cursor
                .demand(&mut port, &mut session.facts.types, probe, &mut work)?,
            expected
        );
    }
    assert_eq!(state.cursor.counters(), stopped);
    Ok(())
}
fn chain(depth: usize, repeats: usize) -> Result<String> {
    let mut ty = "u8".to_owned();
    for _ in 0..depth {
        ty = format!("fn({ty},u8)->u8");
    }
    let mut source = format!("\npub fn cursor_chain(value:{ty}");
    for index in 0..repeats {
        write!(source, ",other{index}:{ty}")?;
    }
    write!(source, ")->{ty}{{value}}")?;
    Ok(source)
}
#[test]
fn header_signature_cursor_bounds_shared_depth_and_unrelated_originals() -> Result {
    for (depth, repeats) in [(8, 1), (32, 1), (128, 1), (32, 8), (32, 32)] {
        let (report, facts, inventory) = extended(&chain(depth, repeats)?)?;
        let owner = named(&inventory, "cursor_chain")?;
        let mut work = ReservationWork::default();
        let session = prepared(inventory, report, facts, owner, &mut work)?;
        let counters = session
            .recipes
            .as_ref()
            .ok_or("actual memo")?
            .cursor
            .counters();
        assert_eq!(counters.expanded, depth + 1);
        assert_eq!(counters.edges, depth * 3);
    }
    let mut warm = None;
    for unrelated in [1, 8, 32] {
        let mut source = String::new();
        for index in 0..unrelated {
            writeln!(source, "\nfn cursor_unused{index}<T>(x:T)->T{{x}}")?;
        }
        let (report, facts, inventory) = extended(&source)?;
        let owner = named(&inventory, "clone_from")?;
        let mut work = ReservationWork::default();
        let mut session = prepared(inventory, report, facts, owner, &mut work)?;
        let expanded = session
            .recipes
            .as_ref()
            .ok_or("actual memo")?
            .cursor
            .counters()
            .expanded;
        let before = work.steps;
        for _ in 0..1000 {
            session = ready(session.signature_pair(owner, &mut work)?)?;
        }
        let cost = (expanded, work.steps - before);
        if let Some(previous) = warm {
            assert_eq!(cost, previous);
        } else {
            warm = Some(cost);
        }
    }
    Ok(())
}
#[test]
fn header_signature_cursor_failure_drops_owned_session_and_keeps_charges() -> Result {
    let (report, facts, inventory) = actual(true)?;
    let owner = named(&inventory, "identity")?;
    let old = facts.types.len();
    let mut work = ReservationWork::default();
    let session = prepared(inventory, report, facts, owner, &mut work)?;
    assert!(session.facts.types.len() > old);
    let arena = session.arena.clone();
    work.steps = crate::support::crates::MAX_COMPONENTS;
    assert!(session.signature_pair(owner, &mut work).is_err());
    assert!(work.steps > crate::support::crates::MAX_COMPONENTS);
    let (report, facts, inventory) = actual(true)?;
    assert!(!facts.types.has_identity(&arena));
    let mut exhausted = ReservationWork {
        retained: crate::support::crates::MAX_BYTES,
        ..ReservationWork::default()
    };
    assert!(Session::select_signature(report, facts, inventory, 0, owner, &mut exhausted).is_err());
    assert!(exhausted.peak > crate::support::crates::MAX_BYTES);
    Ok(())
}
