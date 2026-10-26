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
    let mut port = Reserved::new(view, &state.maps, &session.facts.types, &mut work)?;
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
fn reference_chain(depth: usize) -> Result<String> {
    // Flat actual expressions keep parser depth bounded while semantic reference
    // types reach the declared depth. Unused T invokes the real scheme producer.
    let mut source = "\npub fn cursor_chain<T>(input:u8){".to_owned();
    let mut previous = "input".to_owned();
    for index in 0..depth {
        writeln!(source, "let cursor_ref{index}=&{previous};")?;
        previous = format!("cursor_ref{index}");
    }
    source.push('}');
    Ok(source)
}
fn reference_frontier(depth: usize, repeats: usize) -> Result {
    let (report, facts, inventory) = extended(&reference_chain(depth)?)?;
    let owner = named(&inventory, "cursor_chain")?;
    let rows = inventory.artifacts()[0]
        .source_descriptor()
        .ok_or("actual descriptor")?
        .rows();
    let Some(s::HeaderRef::Function(index)) = rows.header(owner)? else {
        return Err("actual reference-chain function".into());
    };
    let s::FunctionBody::Original(body) = rows.functions[index as usize].body else {
        return Err("actual reference-chain body".into());
    };
    let original = &rows.originals[body.0 as usize];
    assert_eq!(original.declaration, owner);
    assert_eq!(
        rows.body_inputs[body.0 as usize]
            .obligations
            .iter()
            .filter(|input| matches!(input.kind, s::InputKind::LocalBorrow(_)))
            .count(),
        depth
    );
    let name = format!("cursor_ref{}", depth - 1);
    let local = original
        .locals
        .iter()
        .find(|local| {
            local.definition.is_some_and(|declaration| {
                rows.declarations[declaration.0 as usize].name.as_deref() == Some(name.as_str())
            })
        })
        .ok_or("actual deepest reference local")?;
    let declaration = local.definition.ok_or("actual local declaration")?;
    let declared = &rows.declarations[declaration.0 as usize];
    assert_eq!(declared.definition.parent, Some(owner));
    assert_eq!(
        declared.local.ok_or("actual local body association")?.body,
        body
    );
    let probe = local.ty;
    let mut work = ReservationWork::default();
    let mut session = prepared(Arc::clone(&inventory), report, facts, owner, &mut work)?;
    let extent = session.facts.types.interned_extent();
    let view = session
        .inputs
        .recipe_view(&session.inventory, 0, &mut work)?;
    view.declaration(declaration, &mut work)?;
    let expected = Demand::Blocked(Dependency::OwnedBodyDomain(view.body(body, &mut work)?));
    let state = session.recipes.as_mut().ok_or("actual memo")?;
    let Pair::Ready(pair) = state.pairs[index as usize] else {
        return Err("actual completed reference-chain signature pair".into());
    };
    let before = state.cursor.counters();
    let bytes = state.retained_bytes();
    let mut port = Reserved::new(view, &state.maps, &session.facts.types, &mut work)?;
    assert_eq!(
        state
            .cursor
            .demand(&mut port, &mut session.facts.types, probe, &mut work)?,
        expected
    );
    let stopped = state.cursor.counters();
    assert_eq!(stopped.expanded - before.expanded, depth);
    assert_eq!(stopped.edges - before.edges, depth);
    let attempted = work.steps;
    for _ in 0..repeats {
        assert_eq!(
            state
                .cursor
                .demand(&mut port, &mut session.facts.types, probe, &mut work)?,
            expected
        );
    }
    assert_eq!(work.steps - attempted, repeats);
    assert_eq!(state.cursor.counters(), stopped);
    assert_eq!(state.retained_bytes(), bytes);
    assert_eq!(session.facts.types.interned_extent(), extent);
    let unused = rows
        .slots
        .iter()
        .position(|slot| slot.owner == owner && matches!(slot.kind, s::ParameterKind::Type { .. }))
        .ok_or("actual unused declared type formal")?;
    assert_eq!(state.maps.slot(unused)?, None);
    session = ready(session.signature_pair(owner, &mut work)?)?;
    let state = session.recipes.as_ref().ok_or("actual retained memo")?;
    assert!(matches!(state.pairs[index as usize], Pair::Ready(actual) if actual == pair));
    assert_eq!(state.cursor.counters(), stopped);
    assert_eq!(state.retained_bytes(), bytes);
    assert_eq!(session.facts.types.interned_extent(), extent);
    Ok(())
}
#[test]
fn header_signature_cursor_bounds_shared_depth_and_unrelated_originals() -> Result {
    // Anonymous nested fn-pointer/alias transport remains unsupported. These
    // genuine body roots test the same declared depths without granting a body
    // region mapping or claiming deep ready signature support.
    for (depth, repeats) in [(8, 1), (32, 1), (128, 1), (32, 8), (32, 32)] {
        reference_frontier(depth, repeats)?;
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
    provenance_opponents()?;
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

#[derive(Default)]
struct PlaceBudget {
    work: ReservationWork,
    emitted: usize,
}
impl crate::borrow::regions::PlaceBoundsSink for PlaceBudget {
    fn charge(&mut self, count: usize) -> std::result::Result<(), &'static str> {
        self.work.charge(count).map_err(|_| "place work exhausted")
    }
    fn scratch(&mut self, growth: usize) -> std::result::Result<(), &'static str> {
        self.work
            .reserve(growth)
            .map_err(|_| "place scratch exhausted")
    }
    fn obligation(
        &mut self,
        _: crate::body::regions::Obligation,
    ) -> std::result::Result<(), &'static str> {
        self.charge(1)?;
        self.work
            .reserve(4 * size_of::<crate::body::regions::Obligation>())
            .map_err(|_| "place output exhausted")?;
        self.emitted += 1;
        Ok(())
    }
}
fn place_budget(report: &Report, facts: &Declarations) -> Result {
    let templates = crate::body::templates::build(report, facts)?;
    let body = &templates
        .iter()
        .find(|template| match template.body.owner {
            crate::body::ir::Owner::Function { definition, .. } => {
                report
                    .package
                    .names
                    .get(report.resolution.definitions[definition.0 as usize].name)
                    == "cursor_chain"
            }
            _ => false,
        })
        .ok_or("actual reference-chain CFG")?
        .body;
    let mut ordinary = PlaceBudget::default();
    crate::borrow::regions::visit_place_bounds(&facts.types, body, &mut ordinary)?;
    assert_eq!(ordinary.emitted, 2);
    let mut failed_work = PlaceBudget::default();
    failed_work.work.steps = crate::support::crates::MAX_COMPONENTS;
    assert!(
        crate::borrow::regions::visit_place_bounds(&facts.types, body, &mut failed_work).is_err()
    );
    assert!(failed_work.work.steps > crate::support::crates::MAX_COMPONENTS);
    assert_eq!(failed_work.emitted, 0);
    let mut failed_storage = PlaceBudget::default();
    failed_storage.work.retained = crate::support::crates::MAX_BYTES;
    assert!(
        crate::borrow::regions::visit_place_bounds(&facts.types, body, &mut failed_storage)
            .is_err()
    );
    assert!(failed_storage.work.peak > crate::support::crates::MAX_BYTES);
    assert_eq!(failed_storage.emitted, 0);
    Ok(())
}
fn provenance_opponents() -> Result {
    crate::unpaid_cases::extended_with(&reference_chain(2)?, |report, facts, rows, interface| {
        place_budget(report, facts)?;
        let header = rows
            .functions
            .iter()
            .find(|header| {
                rows.declarations[header.declaration.0 as usize]
                    .name
                    .as_deref()
                    == Some("cursor_chain")
            })
            .ok_or("actual reference-chain header")?;
        let s::FunctionBody::Original(body) = header.body else {
            return Err("actual reference-chain original".into());
        };
        let inputs = &rows.body_inputs[body.0 as usize];
        let borrowed = inputs
            .obligations
            .iter()
            .enumerate()
            .filter_map(|(index, input)| {
                matches!(input.kind, s::InputKind::LocalBorrow(_)).then_some(index)
            })
            .collect::<Vec<_>>();
        assert_eq!(borrowed.len(), 2);
        for replaced in [false, true] {
            let mut wrong = rows.clone();
            let obligations = &mut wrong.body_inputs[body.0 as usize].obligations;
            if replaced {
                obligations[borrowed[0]].kind = obligations[borrowed[1]].kind;
            } else {
                obligations.remove(borrowed[0]);
            }
            let error = g::Payload::from_source_provided(
                interface.identity(),
                wrong,
                interface.base_retained_bytes(),
                &mut g::Work::default(),
            )
            .err()
            .ok_or("missing actual local borrow was accepted")?;
            assert_eq!(
                error,
                "source local borrow omitted its exact function-storage upper-bound input"
            );
        }
        Ok(())
    })?;
    Ok(())
}
