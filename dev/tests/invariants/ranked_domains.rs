//! Inverse order joins real aliases; it never authenticates imported execution.
use super::{BodyRef, HeaderIndex, Result, Work, collect, provided};

#[test]
fn inverse_ordinals_join_actual_aliases_and_normalization_domain() -> Result {
    let (report, facts, rows) = collect(0)?;
    let reference = provided(&rows)?;
    let raw = &rows.originals[reference];
    let name = rows.declarations[raw.declaration.0 as usize]
        .name
        .as_deref()
        .ok_or("actual provided name")?;
    let function = facts
        .functions
        .iter()
        .find(|function| {
            function.body.is_some()
                && report
                    .package
                    .names
                    .get(report.resolution.definitions[function.definition.0 as usize].name)
                    == name
        })
        .ok_or("actual provided function")?;
    let header = facts
        .function_header(function.definition)?
        .ok_or("actual provided header")?;
    let actual = header
        .environment
        .and_then(|environment| environment.body_aliases())
        .ok_or("actual default aliases")?;
    let mut work = Work::default();
    let index = HeaderIndex::new(&rows, 0, &mut work)?;
    let units = index.numbering(&mut work)?;
    let body = BodyRef(u32::try_from(reference)?);
    let inverse = units.allocation_ordinals(body, 0, &mut work)?;
    assert_eq!(actual.len(), rows.body_units[reference].universals as usize);
    for (universal, alias) in actual.ordered().iter().enumerate() {
        assert_eq!(
            inverse.alias(u32::try_from(universal)?, &mut work)?,
            Some(alias.0.checked_sub(1).ok_or("actual owned alias")?)
        );
    }
    for original in &rows.originals {
        let inverse = units.allocation_ordinals(original.body, 0, &mut work)?;
        let unit = units.unit(original.body, &mut work)?;
        if original.variables.is_empty() {
            assert_eq!(
                inverse.retained_bytes(),
                size_of::<crate::support::crates::generics::source::AllocationOrdinals<'_>>()
            );
        }
        for (ordinal, &slot) in unit.allocation_order.iter().enumerate() {
            assert_eq!(inverse.ordinal(slot, &mut work)?, u32::try_from(ordinal)?);
        }
    }
    assert!(
        rows.originals.iter().any(
            |original| original.variables.iter().any(|variable| matches!(
                variable.allocation,
                super::s::RegionAllocation::Normalization(_)
            ))
        )
    );
    Ok(())
}

#[test]
fn inverse_ordinals_charge_failed_queries_and_preallocation() -> Result {
    let (_, _, mut rows) = collect(0)?;
    let reference = provided(&rows)?;
    rows.body_units[reference].aliases = None;
    let mut work = Work::default();
    let index = HeaderIndex::new(&rows, 0, &mut work)?;
    let units = index.numbering(&mut work)?;
    let body = BodyRef(u32::try_from(reference)?);
    let before = work.steps;
    assert!(
        units
            .allocation_ordinals(body, crate::support::crates::MAX_BYTES, &mut work)
            .is_err()
    );
    assert_eq!(
        work.steps - before,
        rows.originals[reference].variables.len() + 2
    );
    let before = work.steps;
    assert!(
        units
            .allocation_ordinals(BodyRef(u32::MAX), 0, &mut work)
            .is_err()
    );
    assert_eq!(work.steps - before, 1);
    let inverse = units.allocation_ordinals(body, 0, &mut work)?;
    assert_eq!(inverse.alias(0, &mut work)?, None);
    let before = work.steps;
    assert!(inverse.ordinal(u32::MAX, &mut work).is_err());
    assert!(
        inverse
            .alias(rows.body_units[reference].universals, &mut work)
            .is_err()
    );
    assert_eq!(work.steps - before, 2);
    let mut exhausted = Work {
        steps: crate::support::crates::MAX_COMPONENTS,
        ..Work::default()
    };
    assert!(units.allocation_ordinals(body, 0, &mut exhausted).is_err());
    assert!(exhausted.steps > crate::support::crates::MAX_COMPONENTS);
    Ok(())
}

#[test]
fn inverse_ordinals_reuse_has_fixed_work_with_unrelated_originals() -> Result {
    for extra in [1, 8, 32] {
        let (_, _, rows) = collect(extra)?;
        let reference = provided(&rows)?;
        let mut work = Work::default();
        let index = HeaderIndex::new(&rows, 0, &mut work)?;
        let units = index.numbering(&mut work)?;
        let inverse =
            units.allocation_ordinals(BodyRef(u32::try_from(reference)?), 0, &mut work)?;
        let bytes = inverse.retained_bytes();
        let peak = units.peak_bytes();
        let before = work.steps;
        let slot = rows.body_units[reference].allocation_order[0];
        for _ in 0..1000 {
            assert_eq!(inverse.ordinal(slot, &mut work)?, 0);
        }
        assert_eq!(work.steps - before, 1000);
        assert_eq!(inverse.retained_bytes(), bytes);
        assert_eq!(units.peak_bytes(), peak);
        assert_eq!(
            bytes,
            size_of::<crate::support::crates::generics::source::AllocationOrdinals<'_>>()
                + 4 * rows.originals[reference].variables.len()
        );
    }
    Ok(())
}
