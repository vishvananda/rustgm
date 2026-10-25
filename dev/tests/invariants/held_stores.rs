//! The actual type store moves with the report; reserved None cells grant no proof.
use super::*;
use crate::unpaid_cases::{Result, actual};

#[test]
fn unpaid_session_moves_actual_store_and_only_reserves_empty_semantic_cells() -> Result {
    let (report, mut facts, inventory) = actual(true)?;
    let arena = facts.types.arena_identity();
    let types = facts.types.len();
    let definitions = facts.definitions.len();
    let functions = facts.functions.len();
    let linkage = facts
        .crates
        .as_deref()
        .and_then(crate::sema::crates::Linkage::original_work);
    let mut work = ReservationWork::default();
    let session = Session::prepare(report, facts, inventory, 0, &mut work)?;
    assert!(session.facts.types.has_identity(&arena));
    assert_eq!(session.facts.types.len(), types);
    assert_eq!(session.facts.functions.len(), functions);
    assert_eq!(session.old_definitions, definitions);
    assert!(
        session.facts.definitions[definitions..]
            .iter()
            .all(Option::is_none)
    );
    assert_eq!(
        session
            .facts
            .crates
            .as_deref()
            .and_then(crate::sema::crates::Linkage::original_work),
        linkage
    );
    session.require(&mut work)?;
    Ok(())
}

#[test]
fn unpaid_session_rejects_replaced_arena_and_invented_semantic_payment() -> Result {
    let (report, facts, inventory) = actual(true)?;
    let (other_report, other_facts, other_inventory) = actual(true)?;
    let mut work = ReservationWork::default();
    let mut first = Session::prepare(report, facts, inventory, 0, &mut work)?;
    let mut second = Session::prepare(other_report, other_facts, other_inventory, 0, &mut work)?;
    first.facts.definitions[first.old_definitions] = Some(crate::types::TypeId::UNIT);
    assert!(first.require(&mut work).is_err());
    first.facts.definitions[first.old_definitions] = None;
    std::mem::swap(&mut first.facts.types, &mut second.facts.types);
    assert!(first.require(&mut work).is_err());
    Ok(())
}
