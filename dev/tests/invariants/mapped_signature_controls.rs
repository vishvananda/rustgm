//! Real reserved signature pairs use the one persistent canonical memo.
use super::*;
use crate::unpaid_cases::{Result, extended};
use std::fmt::Write as _;

fn named(view: &RecipeView<'_>, name: &str) -> Result<DeclarationRef> {
    let rows = view.rows()?;
    rows.functions
        .iter()
        .find(|header| {
            rows.declarations[header.declaration.0 as usize]
                .name
                .as_deref()
                == Some(name)
        })
        .map(|header| header.declaration)
        .ok_or_else(|| "actual named callable".into())
}
fn start(source: &str, work: &mut ReservationWork) -> Result<Box<Session>> {
    let (report, facts, inventory) = extended(source)?;
    Ok(Session::prepare(report, facts, inventory, 0, work)?.boxed(work)?)
}
fn ready(progress: Progress) -> Result<Box<Session>> {
    let (session, status) = progress;
    assert_eq!(status, Demand::Ready(()));
    Ok(session)
}
#[test]
fn shared_signature_reader_binds_actual_writer_owner_arena_and_ready_shapes() -> Result {
    let mut work = ReservationWork::default();
    let mut session = start(
        "fn selected<T>(x:u8)->u8{x}fn other<U>(x:bool)->bool{x}",
        &mut work,
    )?;
    let view = session
        .inputs
        .recipe_view(&session.inventory, 0, &mut work)?;
    let owner = named(&view, "selected")?;
    let other = named(&view, "other")?;
    let rows = view.rows()?;
    let Some(s::HeaderRef::Function(index)) = rows.header(owner)? else {
        return Err("actual function row".into());
    };
    let header = rows.functions[index as usize];
    let mut state = State::new(&view, &mut session.facts.types, &mut work)?;
    let reader = state
        .cursor
        .reader(&view, &session.facts.types, &mut work)?;
    assert!(
        matches!(reader.get(&view, &session.facts.types, header.original.result, &mut work)?, MemoLookup::Pending {owner: actual, root} if actual == view.root() && root == header.original.result)
    );
    let Demand::Ready(pair) = state.pair(view, &mut session.facts.types, owner, &mut work)? else {
        return Err("actual pair blocked".into());
    };
    let view = session
        .inputs
        .recipe_view(&session.inventory, 0, &mut work)?;
    state.require_mapping(&view, &session.facts.types, true, &mut work)?;
    state.require_signatures(&view, &session.facts.types, header, pair, &mut work)?;
    assert!(matches!(
        reader.get(
            &view,
            &session.facts.types,
            header.original.result,
            &mut work
        )?,
        MemoLookup::Ready(_)
    ));
    let Demand::Ready(other_pair) = state.pair(view, &mut session.facts.types, other, &mut work)?
    else {
        return Err("actual other pair blocked".into());
    };
    let view = session
        .inputs
        .recipe_view(&session.inventory, 0, &mut work)?;
    assert!(
        state
            .require_signatures(
                &view,
                &session.facts.types,
                header,
                SignaturePair {
                    callable: other_pair.callable,
                    ..pair
                },
                &mut work
            )
            .is_err()
    );
    reject_replaced_readers(&mut session, &reader, header.original.result, &mut work)?;
    Ok(())
}
fn reject_replaced_readers(
    session: &mut Session,
    reader: &MemoReader,
    root: g::TypeRef,
    work: &mut ReservationWork,
) -> Result {
    let view = session.inputs.recipe_view(&session.inventory, 0, work)?;
    let replacement = State::new(&view, &mut session.facts.types, work)?;
    assert!(
        replacement
            .cursor
            .require_reader(reader, &view, &session.facts.types, work)
            .is_err()
    );
    assert!(reader.get(&view, &Store::default(), root, work).is_err());
    let mut foreign = start("fn selected<T>(x:u8)->u8{x}", work)?;
    let foreign_view = foreign.inputs.recipe_view(&foreign.inventory, 0, work)?;
    assert!(
        reader
            .get(&foreign_view, &session.facts.types, root, work)
            .is_err()
    );
    // The genuine foreign owner also has a real independently initialized arena.
    let _ = State::new(&foreign_view, &mut foreign.facts.types, work)?;
    assert!(reader.get(&view, &foreign.facts.types, root, work).is_err());
    Ok(())
}
fn warm(width: usize, unrelated: usize) -> Result<usize> {
    let mut source = "fn selected<T>(".to_owned();
    for index in 0..width {
        if index != 0 {
            source.push(',');
        }
        write!(source, "x{index}:u8")?;
    }
    source.push_str(")->u8{x0}");
    for index in 0..unrelated {
        write!(source, "fn unrelated_{index}<T>(x:T)->T{{x}}")?;
    }
    let mut work = ReservationWork::default();
    let session = start(&source, &mut work)?;
    let view = session
        .inputs
        .recipe_view(&session.inventory, 0, &mut work)?;
    let owner = named(&view, "selected")?;
    let mut session = ready(session.callable_template(owner, &mut work)?)?;
    let before = (
        work.retained,
        work.peak,
        session.facts.types.interned_extent(),
    );
    let steps = work.steps;
    for _ in 0..1000 {
        session = ready(session.callable_template(owner, &mut work)?)?;
    }
    assert_eq!(
        before,
        (
            work.retained,
            work.peak,
            session.facts.types.interned_extent()
        )
    );
    let delta = work.steps - steps;
    let state = session.recipes.as_ref().ok_or("actual memo state")?;
    assert!(state.reader.is_some());
    eprintln!(
        "shared-mapping width={width} unrelated={unrelated} warm1000={delta} state={} cursor={} reader={} session={} retained={} peak={} memo_bytes={}",
        size_of::<State>(),
        size_of::<Cursor>(),
        size_of::<MemoReader>(),
        size_of::<Session>(),
        work.retained,
        work.peak,
        state.retained_bytes()
    );
    work.steps = crate::support::crates::MAX_COMPONENTS;
    assert!(session.callable_template(owner, &mut work).is_err());
    assert!(work.steps > crate::support::crates::MAX_COMPONENTS);
    Ok(delta)
}
#[test]
fn shared_signature_mapping_bounds_warm_width_unrelated_and_failed_precharge() -> Result {
    let mut expected = None;
    for width in [1, 8, 32] {
        for unrelated in [1, 8, 32] {
            let actual = warm(width, unrelated)?;
            if let Some(expected) = expected {
                assert_eq!(actual, expected);
            }
            expected = Some(actual);
        }
    }
    let mut work = ReservationWork::default();
    let session = start("fn selected<T>(x:u8)->u8{x}", &mut work)?;
    let view = session
        .inputs
        .recipe_view(&session.inventory, 0, &mut work)?;
    let owner = named(&view, "selected")?;
    work.retained = crate::support::crates::MAX_BYTES;
    assert!(session.callable_template(owner, &mut work).is_err());
    assert!(work.peak > crate::support::crates::MAX_BYTES);
    Ok(())
}
