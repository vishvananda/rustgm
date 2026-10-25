//! Only real producer units supply private allocation records; none is a proof.
use super::*;
use crate::unpaid_cases::{Result, actual, extended};
use std::{fmt::Write as _, sync::Arc};

fn named(inventory: &crate::support::crates::Dependencies, name: &str) -> Result<DeclarationRef> {
    let rows = inventory.artifacts()[0]
        .source_descriptor()
        .ok_or("actual rows")?
        .rows();
    rows.functions
        .iter()
        .find(|header| {
            rows.declarations[header.declaration.0 as usize]
                .name
                .as_deref()
                == Some(name)
                && matches!(header.body, s::FunctionBody::Original(_))
        })
        .map(|header| header.declaration)
        .ok_or_else(|| "actual named original".into())
}
fn ready(progress: Progress) -> Result<Box<Session>> {
    let (session, status) = progress;
    if status != Demand::Ready(()) {
        return Err("actual recording unexpectedly blocked".into());
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
    Ok((
        Session::prepare(report, facts, inventory, 0, work)?.boxed(work)?,
        owner,
    ))
}
fn prepared(session: &Session) -> Result<&super::super::Prepared> {
    session
        .recipes
        .as_ref()
        .and_then(|state| state.callables.as_deref())
        .ok_or_else(|| "actual template owner".into())
}
fn record(session: &Session, owner: DeclarationRef) -> Result<(&Ready, &RecordedDomain)> {
    let table = prepared(session)?;
    let expected = table
        .ready
        .iter()
        .flatten()
        .find(|ready| ready.header.declaration == owner)
        .ok_or("actual recorded header")?;
    let row = table
        .recordings
        .as_ref()
        .ok_or("actual cold numbering owner")?
        .entries[expected.pair.function_row as usize]
        .as_deref()
        .ok_or("actual recorded product")?;
    Ok((expected, row))
}
#[test]
fn private_numbering_records_actual_units_without_binding_environments() -> Result {
    full_family()?;
    foreign_contexts()?;
    failed_prepayment()?;
    growth()?;
    Ok(())
}
fn full_family() -> Result {
    let (report, facts, inventory) = actual(true)?;
    let family = Arc::clone(&inventory);
    let rows = family.artifacts()[0]
        .source_descriptor()
        .ok_or("actual descriptor")?
        .rows();
    let source_environments = facts.generics.environments.len();
    let source_bodies = facts.generics.body_environments.len();
    let mut work = ReservationWork::default();
    let mut session = Session::prepare(report, facts, inventory, 0, &mut work)?.boxed(&mut work)?;
    let mut normalization = 0;
    let mut aliases = 0;
    for header in &rows.functions {
        let s::FunctionBody::Original(body) = header.body else {
            continue;
        };
        session = ready(session.record_numbering(header.declaration, &mut work)?)?;
        let (expected, actual) = record(&session, header.declaration)?;
        let unit = &rows.body_units[body.0 as usize];
        assert_eq!(actual.log.ordered.len(), unit.allocation_order.len());
        let view = session
            .inputs
            .recipe_view(&session.inventory, 0, &mut work)?;
        for (ordinal, (&slot, allocation)) in unit
            .allocation_order
            .iter()
            .zip(&actual.log.ordered)
            .enumerate()
        {
            assert_eq!(allocation.slot, slot);
            assert_eq!(allocation.id.0 as usize, ordinal + 1);
            assert_eq!(
                actual.region(&view, &session.facts.types, expected, slot, &mut work)?,
                allocation.id
            );
            assert_eq!(
                allocation.site,
                view.body_site(
                    body,
                    rows.originals[body.0 as usize].variables[slot as usize].site,
                    &mut work
                )?
            );
            if matches!(
                rows.originals[body.0 as usize].variables[slot as usize].allocation,
                s::RegionAllocation::Normalization(_)
            ) {
                normalization += 1;
            }
        }
        assert_eq!(
            actual.aliases.as_ref().map(Vec::len),
            unit.aliases.as_ref().map(Vec::len)
        );
        if let (Some(mapped), Some(slots)) = (&actual.aliases, &unit.aliases) {
            assert_eq!(mapped.len(), unit.universals as usize);
            for (&alias, &slot) in mapped.iter().zip(slots) {
                assert_eq!(Some(alias), actual.log.by_slot[slot as usize]);
            }
            aliases += mapped.len();
        }
        assert!(
            actual
                .region(&view, &session.facts.types, expected, u32::MAX, &mut work)
                .is_err()
        );
    }
    assert!(normalization > 0);
    assert!(aliases >= 2);
    let original = rows
        .originals
        .iter()
        .find(|original| {
            original.variables.iter().any(|variable| {
                matches!(variable.allocation, s::RegionAllocation::Normalization(_))
            })
        })
        .ok_or("actual normalization domain")?;
    let other = rows
        .originals
        .iter()
        .find(|candidate| candidate.body != original.body)
        .ok_or("actual other body")?;
    let (_, normalized) = record(&session, original.declaration)?;
    let (foreign, _) = record(&session, other.declaration)?;
    let slot = original
        .variables
        .iter()
        .position(|variable| matches!(variable.allocation, s::RegionAllocation::Normalization(_)))
        .ok_or("actual normalization slot")?;
    let view = session
        .inputs
        .recipe_view(&session.inventory, 0, &mut work)?;
    assert!(
        normalized
            .region(
                &view,
                &session.facts.types,
                foreign,
                u32::try_from(slot)?,
                &mut work
            )
            .is_err()
    );
    assert_eq!(
        session.facts.generics.environments.len(),
        source_environments
    );
    assert_eq!(
        session.facts.generics.body_environments.len(),
        source_bodies
    );
    assert!(prepared(&session)?.environments.is_none());
    assert!(
        session.facts.definitions[session.old_definitions..]
            .iter()
            .all(Option::is_none)
    );
    let deferred = named(&session.inventory, "clone_from")?;
    let (session, status) = session.defining_environment(deferred, &mut work)?;
    assert!(matches!(
        status,
        Demand::Blocked(
            crate::sema::crates::typed_headers::recipe_cursor::Dependency::OwnedBodyDomain(_)
        )
    ));
    assert!(prepared(&session)?.environments.is_none());
    let (_, _, ordinary) = actual(false)?;
    let ordinary = ordinary.artifacts()[0]
        .source_descriptor()
        .ok_or("actual old Functions")?
        .rows();
    let body = ordinary
        .originals
        .first()
        .ok_or("actual old original")?
        .body;
    let mut failed = g::Work::default();
    assert!(
        ordinary
            .allocation_ordinals(body, usize::MAX, 0, &mut failed)
            .is_err()
    );
    assert!(failed.steps > 0);
    Ok(())
}
const TWINS: &str = "\nfn left(value:&u8)->&u8{value}fn right(value:&u8)->&u8{value}";
fn foreign_contexts() -> Result {
    let mut work = ReservationWork::default();
    let (session, left) = start(TWINS, "left", &mut work)?;
    let mut session = ready(session.record_numbering(left, &mut work)?)?;
    let right = named(&session.inventory, "right")?;
    session = ready(session.record_numbering(right, &mut work)?)?;
    let (left_header, left_domain) = record(&session, left)?;
    let (right_header, right_domain) = record(&session, right)?;
    assert_eq!(left_header.pair.original, right_header.pair.original);
    assert_eq!(left_header.pair.callable, right_header.pair.callable);
    assert_eq!(
        left_domain.log.ordered.first().ok_or("left allocation")?.id,
        right_domain
            .log
            .ordered
            .first()
            .ok_or("right allocation")?
            .id
    );
    let view = session
        .inputs
        .recipe_view(&session.inventory, 0, &mut work)?;
    let slot = left_domain.log.ordered[0].slot;
    assert!(
        left_domain
            .region(&view, &session.facts.types, right_header, slot, &mut work)
            .is_err()
    );
    let foreign_types = Store::default();
    assert!(
        left_domain
            .region(&view, &foreign_types, left_header, slot, &mut work)
            .is_err()
    );
    for opponent in 0..3 {
        let mut work = ReservationWork::default();
        let (session, owner) = start(TWINS, "left", &mut work)?;
        let mut session = ready(session.record_numbering(owner, &mut work)?)?;
        match opponent {
            0 => session.facts.types = Store::default(),
            1 => {
                let (_, _, foreign) = actual(true)?;
                session.inventory = foreign;
            }
            _ => {
                session
                    .recipes
                    .as_mut()
                    .and_then(|state| state.callables.as_mut())
                    .ok_or("actual table")?
                    .scopes[owner.0 as usize] = None
            }
        }
        let before = work.steps;
        assert!(session.record_numbering(owner, &mut work).is_err());
        assert!(work.steps > before);
    }
    Ok(())
}
fn failed_prepayment() -> Result {
    let mut work = ReservationWork::default();
    let (session, owner) = start(TWINS, "left", &mut work)?;
    let session = ready(session.callable_template(owner, &mut work)?)?;
    let table = prepared(&session)?;
    let expected = table
        .ready
        .iter()
        .flatten()
        .find(|ready| ready.header.declaration == owner)
        .ok_or("actual ready header")?;
    let view = session
        .inputs
        .recipe_view(&session.inventory, 0, &mut work)?;
    let stage = DomainStage::new(&view, &session.facts.types, expected, &mut work)?;
    let extent = session.facts.types.interned_extent();
    work.retained = crate::support::crates::MAX_BYTES;
    let before = work.steps;
    assert!(stage.record(&view, &mut work).is_err());
    assert!(work.steps > before);
    assert!(work.peak > crate::support::crates::MAX_BYTES);
    assert_eq!(session.facts.types.interned_extent(), extent);
    assert!(prepared(&session)?.recordings.is_none());
    Ok(())
}
fn chain(depth: usize) -> Result<String> {
    let mut source = String::from("\nfn selected<T>(input:u8){");
    let mut previous = "input".to_owned();
    for index in 0..depth {
        writeln!(source, "let reference{index}=&{previous};")?;
        previous = format!("reference{index}");
    }
    source.push('}');
    Ok(source)
}
fn growth() -> Result {
    eprintln!(
        "recording layouts: stage={} domain={} seal={} allocation={} log={} owner={} prepared={}",
        size_of::<DomainStage>(),
        size_of::<RecordedDomain>(),
        size_of::<Seal>(),
        size_of::<Allocation>(),
        size_of::<AllocationLog>(),
        size_of::<Owner>(),
        size_of::<super::super::Prepared>()
    );
    depth_growth()?;
    unrelated_growth()?;
    width_growth()?;
    Ok(())
}
fn depth_growth() -> Result {
    let mut previous = None;
    for depth in [8, 32, 128] {
        let mut work = ReservationWork::default();
        let (session, owner) = start(&chain(depth)?, "selected", &mut work)?;
        let session = ready(session.callable_template(owner, &mut work)?)?;
        let before = work.steps;
        let session = ready(session.record_numbering(owner, &mut work)?)?;
        let cold = work.steps - before;
        let (_, domain) = record(&session, owner)?;
        let n = domain.log.ordered.len();
        if let Some((prior_n, prior_work)) = previous {
            assert!(cold <= prior_work * n / prior_n + 128);
        }
        previous = Some((n, cold));
        eprintln!(
            "recording depth={depth} variables={n} cold={cold} bytes={} record_cap={} slot_cap={} alias_cap={} peak={}",
            domain.retained_bytes(),
            domain.log.ordered.capacity(),
            domain.log.by_slot.capacity(),
            domain.aliases.as_ref().map_or(0, Vec::capacity),
            work.peak
        );
        warm(session, owner, &mut work)?;
    }
    Ok(())
}
fn unrelated_growth() -> Result {
    let mut common = None;
    for unrelated in [1, 8, 32] {
        let mut source = chain(8)?;
        for index in 0..unrelated {
            writeln!(source, "fn unrelated{index}(value:u8)->u8{{value}}")?;
        }
        let mut work = ReservationWork::default();
        let (session, owner) = start(&source, "selected", &mut work)?;
        let session = ready(session.record_numbering(owner, &mut work)?)?;
        let (session, cost) = warm(session, owner, &mut work)?;
        if let Some(previous) = common {
            assert_eq!(cost, previous);
        } else {
            common = Some(cost);
        }
        let owner = prepared(&session)?
            .recordings
            .as_ref()
            .ok_or("actual recording owner")?;
        assert_eq!(owner.entries.iter().flatten().count(), 1);
        eprintln!(
            "recording unrelated={unrelated} warm1000={cost} owner_bytes={} index_cap={}",
            owner.retained_bytes(),
            owner.entries.capacity()
        );
    }
    Ok(())
}
fn width_growth() -> Result {
    for width in [1, 8, 32] {
        let mut source = String::from("\nfn selected<T>(");
        for index in 0..width {
            if index != 0 {
                source.push(',');
            }
            write!(source, "p{index}:&u8")?;
        }
        source.push_str("){}");
        let mut work = ReservationWork::default();
        let (session, owner) = start(&source, "selected", &mut work)?;
        let session = ready(session.record_numbering(owner, &mut work)?)?;
        let (expected, domain) = record(&session, owner)?;
        assert_eq!(expected.header.regions.total as usize, width);
        eprintln!(
            "recording width={width} variables={} aliases={:?} bytes={} peak={}",
            domain.log.ordered.len(),
            domain.aliases.as_ref().map(Vec::len),
            domain.retained_bytes(),
            work.peak
        );
        warm(session, owner, &mut work)?;
    }
    Ok(())
}
fn warm(
    mut session: Box<Session>,
    owner: DeclarationRef,
    work: &mut ReservationWork,
) -> Result<(Box<Session>, usize)> {
    let before = work.steps;
    let bytes = work.retained;
    let extent = session.facts.types.interned_extent();
    let counters = session
        .recipes
        .as_ref()
        .ok_or("actual cursor")?
        .cursor
        .counters();
    for _ in 0..1000 {
        session = ready(session.record_numbering(owner, work)?)?;
    }
    assert_eq!(work.retained, bytes);
    assert_eq!(session.facts.types.interned_extent(), extent);
    assert_eq!(
        session
            .recipes
            .as_ref()
            .ok_or("actual cursor")?
            .cursor
            .counters(),
        counters
    );
    Ok((session, work.steps - before))
}
