//! Numbering opponents start from the unchanged full family's actual collector.
use super::{HeaderIndex, Work};
use crate::{
    resolve::expansion::Report,
    sema::model::Declarations,
    support::crates::{generics::source as s, owner_catalog::BodyRef},
};
use std::fmt::Write as _;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
#[path = "ranked_domains.rs"]
mod allocation_tests;

fn collect(extra: u32) -> Result<(Report, Declarations, s::SourceRows)> {
    let mut source = crate::test_source::fixture(
        "pa31/tests/spec/source-header-transport/semantic-provider-qualified.rs",
    )?;
    for index in 0..extra {
        writeln!(source, "\nfn unused{index}<T>(value:T)->T{{value}}")?;
    }
    inspect(&source)
}

fn inspect(source: &str) -> Result<(Report, Declarations, s::SourceRows)> {
    let report = crate::test_source::source_provider(source, "numbered_family")?;
    let identity = report
        .package
        .source_core()
        .ok_or("actual provider")?
        .provider
        .clone();
    let analysis = crate::pipeline::analyze_source(&report)?;
    let mut facts = analysis.facts;
    let rows = crate::lower::instances::inspect_source_transport_with_numbering(
        &report,
        &mut facts,
        &identity,
        analysis.prepared,
    )?
    .rows;
    Ok((report, facts, rows))
}

fn provided(rows: &s::SourceRows) -> Result<usize> {
    rows.body_units
        .iter()
        .position(|unit| {
            unit.aliases
                .as_ref()
                .is_some_and(|aliases| aliases.len() >= 2)
        })
        .ok_or_else(|| "actual provided alias ledger with two universals".into())
}

#[test]
fn numbered_family_preserves_actual_allocation_log_and_alias_membership() -> Result {
    let (report, facts, rows) = collect(0)?;
    assert_eq!(rows.body_units.len(), rows.originals.len());
    assert_eq!(rows.originals.len(), 3);
    let mut work = Work::default();
    let index = HeaderIndex::new(&rows, 0, &mut work)?;
    let numbered = index.numbering(&mut work)?;
    let mut normalizations = 0;
    let mut proof_suffixes = 0;
    for original in &rows.originals {
        let unit = numbered.unit(original.body, &mut work)?;
        let name = rows.declarations[original.declaration.0 as usize]
            .name
            .as_deref()
            .ok_or("actual name")?;
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
            .ok_or("actual function body")?;
        let source = function.body.ok_or("actual source")?;
        assert!(facts.body_regions.frontier(source)? as usize <= unit.allocation_order.len() + 1);
        let header = facts
            .function_header(function.definition)?
            .ok_or("actual header")?;
        assert_eq!(unit.universals, header.universals);
        let (owned, suffixes) = allocation_roles(
            &facts,
            source,
            unit,
            original,
            &rows.body_inputs[original.body.0 as usize],
        )?;
        normalizations += owned;
        proof_suffixes += suffixes;
        match (
            unit.aliases.as_ref(),
            header
                .environment
                .and_then(|environment| environment.body_aliases()),
        ) {
            (Some(slots), Some(aliases)) => {
                assert_eq!(slots.len(), aliases.len());
                for (&slot, &alias) in slots.iter().zip(aliases.ordered()) {
                    let ordinal = unit
                        .allocation_order
                        .iter()
                        .position(|&actual| actual == slot)
                        .ok_or("actual slot")?;
                    assert_eq!(alias.0 as usize, ordinal + 1);
                    assert!(matches!(
                        facts
                            .body_regions
                            .get(source, alias)
                            .ok_or("actual allocation")?
                            .origin,
                        crate::sema::model::BodyRegionOrigin::UniversalAlias
                    ));
                }
            }
            (None, None) => {}
            _ => return Err("actual alias absence changed during collection".into()),
        }
    }
    assert!(rows.body_units[provided(&rows)?].universals >= 2);
    assert!(
        normalizations > 0,
        "full family must preserve actual owned normalization allocations"
    );
    assert!(
        proof_suffixes > 0,
        "actual normalization domain must extend the semantic log"
    );
    assert_eq!(size_of::<crate::body::ir::Body>(), 448);
    assert_eq!(size_of::<crate::body::regions::Contract>(), 184);
    println!(
        "SourceRows={} Columns={} HeaderIndex={} NumberingUnits={} BodyUnit={} Body=448 Contract=184",
        size_of::<s::SourceRows>(),
        size_of::<s::Columns>(),
        size_of::<HeaderIndex<'_>>(),
        size_of::<s::NumberingUnits<'_>>(),
        size_of::<s::BodyUnit>()
    );
    Ok(())
}

fn allocation_roles(
    facts: &Declarations,
    source: crate::resolve::unit::NodeKey,
    unit: &s::BodyUnit,
    original: &s::Original,
    input: &s::BodyInputs,
) -> Result<(usize, usize)> {
    let mut normalizations = 0;
    let mut proof_suffixes = 0;
    for (ordinal, &slot) in unit.allocation_order.iter().enumerate() {
        let actual = facts
            .body_regions
            .get(source, crate::types::RegionId(u32::try_from(ordinal + 1)?));
        if let Some(actual) = actual {
            assert_eq!(actual.body, source);
        }
        match original.variables[slot as usize].allocation {
            s::RegionAllocation::Inference { .. } => assert!(!matches!(
                actual
                    .ok_or("inference allocation must occur in actual source log")?
                    .origin,
                crate::sema::model::BodyRegionOrigin::QuantifiedProof
            )),
            s::RegionAllocation::Normalization(normalization) => {
                normalizations += 1;
                if let Some(actual) = actual {
                    assert!(matches!(
                        actual.origin,
                        crate::sema::model::BodyRegionOrigin::QuantifiedProof
                    ));
                } else {
                    proof_suffixes += 1;
                }
                assert!(
                    input.normalizations[normalization as usize]
                        .owned
                        .iter()
                        .any(|range| range.start <= slot && slot < range.end)
                );
            }
        }
    }
    Ok((normalizations, proof_suffixes))
}

fn reject(rows: &s::SourceRows) -> Result {
    let mut work = Work::default();
    let index = HeaderIndex::new(rows, 0, &mut work)?;
    let before = work.steps;
    assert!(index.numbering(&mut work).is_err());
    assert!(work.steps > before);
    Ok(())
}

#[test]
fn numbered_units_reject_incomplete_permutations_headers_and_alias_inputs() -> Result {
    let (_, _, actual) = collect(0)?;
    let body = provided(&actual)?;
    for case in 0..13 {
        let mut rows = actual.clone();
        let unit = &mut rows.body_units[body];
        match case {
            0 => {
                rows.body_units.pop();
            }
            1 => {
                rows.body_units.swap(0, 1);
            }
            2 => unit.body = BodyRef(99),
            3 => unit.universals -= 1,
            4 => {
                unit.allocation_order.pop();
            }
            5 => unit.allocation_order[0] = unit.allocation_order[1],
            6 => unit.allocation_order[0] = u32::MAX,
            7 => {
                unit.aliases.as_mut().ok_or("aliases")?.pop();
            }
            8 => {
                let aliases = unit.aliases.as_mut().ok_or("aliases")?;
                aliases[0] = aliases[1];
            }
            9 => {
                let slot = unit.aliases.as_ref().ok_or("aliases")?[0];
                rows.originals[body].variables[slot as usize].allocation =
                    s::RegionAllocation::Inference {
                        needs_provenance: true,
                    };
            }
            10 => {
                let slot = unit.aliases.as_ref().ok_or("aliases")?[0];
                rows.body_inputs[body].obligations.retain(|input| !matches!(input.kind, s::InputKind::Variable {slot: actual, ..} if actual == slot));
            }
            11 => {
                unit.aliases.as_mut().ok_or("aliases")?.swap(0, 1);
            }
            12 => {
                let original = rows.originals[body].declaration;
                let header = rows
                    .functions
                    .iter()
                    .find(|function| function.declaration == original)
                    .ok_or("actual header")?;
                rows.body_inputs[body].frame = header.callable.binder;
            }
            _ => unreachable!(),
        }
        reject(&rows).map_err(|error| format!("case {case}: {error}"))?;
    }
    Ok(())
}

#[test]
fn numbered_units_keep_unpaid_absence_and_arbitrary_exact_permutation_distinct() -> Result {
    let (_, _, mut rows) = collect(0)?;
    let body = provided(&rows)?;
    rows.body_units[body].allocation_order.reverse();
    let order = rows.body_units[body].allocation_order.clone();
    let mut work = Work::default();
    let index = HeaderIndex::new(&rows, 0, &mut work)?;
    assert_eq!(
        index
            .numbering(&mut work)?
            .unit(BodyRef(u32::try_from(body)?), &mut work)?
            .allocation_order,
        order
    );
    // Only semantic binding can derive whether alias presence was required.
    rows.body_units[body].aliases = None;
    let zero = rows
        .body_units
        .iter()
        .position(|unit| unit.universals == 0)
        .ok_or("actual zero-universal body")?;
    rows.body_units[zero].aliases = Some(Vec::new());
    let index = HeaderIndex::new(&rows, 0, &mut work)?;
    let units = index.numbering(&mut work)?;
    assert!(
        units
            .unit(BodyRef(u32::try_from(body)?), &mut work)?
            .aliases
            .is_none()
    );
    assert_eq!(
        units
            .unit(BodyRef(u32::try_from(zero)?), &mut work)?
            .aliases,
        Some(Vec::new())
    );
    Ok(())
}

#[test]
fn numbered_units_charge_failed_attempts_and_borrow_without_growth() -> Result {
    for extra in [1, 8, 32] {
        let (_, _, rows) = collect(extra)?;
        assert_eq!(rows.body_units.len(), extra as usize + 3);
        let mut work = Work::default();
        let index = HeaderIndex::new(&rows, 0, &mut work)?;
        let units = index.numbering(&mut work)?;
        let peak = units.peak_bytes();
        let before = work.steps;
        let actual = units.unit(BodyRef(0), &mut work)?;
        for _ in 0..128 {
            assert!(std::ptr::eq(actual, units.unit(BodyRef(0), &mut work)?));
        }
        assert!(units.unit(BodyRef(u32::MAX), &mut work).is_err());
        assert_eq!(work.steps - before, 130);
        assert_eq!(units.peak_bytes(), peak);
        println!(
            "unrelated={extra} units={} preparation={} lookup=130 peak={peak}",
            rows.body_units.len(),
            before
        );
        let mut exhausted = Work {
            steps: crate::support::crates::MAX_COMPONENTS,
            ..Work::default()
        };
        assert!(index.numbering(&mut exhausted).is_err());
        assert!(exhausted.steps > crate::support::crates::MAX_COMPONENTS);
    }
    Ok(())
}

#[test]
fn numbered_inspection_cannot_publish_through_functions_profile() -> Result {
    let source = crate::test_source::fixture(
        "pa31/tests/spec/source-functions-transport/minimal-provider.rs",
    )?;
    let (report, _, rows) = inspect(&source)?;
    let identity = &report
        .package
        .source_core()
        .ok_or("actual plain provider")?
        .provider;
    assert!(!rows.body_units.is_empty());
    let mut ordinary = rows.clone();
    ordinary.body_units.clear();
    let baseline = crate::support::crates::generics::Payload::from_source(
        identity,
        ordinary,
        0,
        &mut Work::default(),
    )?;
    assert!(matches!(
        baseline.view(),
        crate::support::crates::generics::PayloadView::Source(_)
    ));
    assert_eq!(
        crate::support::crates::generics::Payload::from_source(
            identity,
            rows,
            0,
            &mut Work::default()
        ),
        Err("source numbering units require their wider explicit profile")
    );
    Ok(())
}
