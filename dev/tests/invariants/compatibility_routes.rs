//! Actual immutable catalog joins, paid path growth and borrowed repeated lookup.
use super::*;
use crate::{lower::instances::SourcePublication, module_packets, source_packets, test_source};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
type LegacyImports = (
    std::rc::Rc<crate::resolve::body_origin::Catalog>,
    std::sync::Arc<crate::support::crates::Dependencies>,
    crate::support::hash::HashMap<DefId, Option<Declaration>>,
    crate::support::hash::HashMap<super::super::BindingKey, super::super::Binding>,
    crate::support::hash::HashSet<String>,
    Vec<PathEntry>,
    std::cell::Cell<super::super::Budget>,
);
fn artifact() -> TestResult<crate::support::crates::Interface> {
    module_packets::exported_profile(
        &test_source::fixture("pa31/tests/spec/source-module-compatibility/provider.rs")?,
        SourcePublication::ModuleCompatibility,
        "compatibility_routes",
    )
}
fn report() -> TestResult<Report> {
    let (report, _, _) = source_packets::client_analysis(
        artifact()?,
        &test_source::fixture("pa31/tests/spec/source-module-compatibility/marked-warning.rs")?,
    )?;
    Ok(report)
}
fn imports(report: &Report) -> TestResult<&Imports> {
    report
        .resolution
        .crates
        .as_deref()
        .and_then(|i| i.availability.as_deref())
        .ok_or_else(|| "actual imported availability".into())
}
fn imports_mut(report: &mut Report) -> TestResult<&mut Imports> {
    report
        .resolution
        .crates
        .as_deref_mut()
        .and_then(|i| i.availability.as_deref_mut())
        .ok_or_else(|| "actual imported availability".into())
}
#[test]
fn path_groups_reject_foreign_changed_terminal_binding_ranges_and_failed_payers() -> TestResult {
    let mut report = report()?;
    let index = imports(&report)?
        .paths
        .iter()
        .position(|p| matches!(p.kind, PathKind::Body(_)))
        .ok_or("actual body path")?;
    let site = imports(&report)?.paths[index].site;
    let terminal = imports(&report)?.paths[index].terminal;
    let entry = imports(&report)?.paths[index].entries.start;
    imports_mut(&mut report)?.paths[index].terminal = DefId(u32::MAX);
    assert!(visit_warnings(&report, Some((site, terminal)), |_| Ok(())).is_err());
    imports_mut(&mut report)?.paths[index].terminal = terminal;
    let saved = imports(&report)?.entries[entry].selected;
    imports_mut(&mut report)?.entries[entry].selected.ambiguous = !saved.ambiguous;
    assert!(visit_warnings(&report, Some((site, terminal)), |_| Ok(())).is_err());
    imports_mut(&mut report)?.entries[entry].selected = saved;
    let range = imports(&report)?.paths[index].entries.clone();
    imports_mut(&mut report)?.paths[index].entries = usize::MAX..usize::MAX;
    assert!(visit_warnings(&report, Some((site, terminal)), |_| Ok(())).is_err());
    imports_mut(&mut report)?.paths[index].entries = range;
    let mut other = self::report()?;
    std::mem::swap(
        &mut report
            .resolution
            .crates
            .as_deref_mut()
            .ok_or("imports")?
            .availability,
        &mut other
            .resolution
            .crates
            .as_deref_mut()
            .ok_or("other imports")?
            .availability,
    );
    assert!(visit_warnings(&report, Some((site, terminal)), |_| Ok(())).is_err());
    std::mem::swap(
        &mut report
            .resolution
            .crates
            .as_deref_mut()
            .ok_or("imports")?
            .availability,
        &mut other
            .resolution
            .crates
            .as_deref_mut()
            .ok_or("other imports")?
            .availability,
    );
    let mut events = 0;
    visit_warnings(&report, Some((site, terminal)), |_| {
        events += 1;
        Ok(())
    })?;
    assert_eq!(events, 1);
    let imports = imports(&report)?;
    let before = imports.budget.get();
    assert!(imports.charge(0, usize::MAX).is_err());
    let first = imports.budget.get();
    assert!(visit_warnings(&report, Some((site, terminal)), |_| Ok(())).is_err());
    assert!(imports.budget.get().steps > first.steps);
    assert!(first.bytes > before.bytes);
    Ok(())
}
#[test]
fn repeated_groups_share_real_event_dedup_and_warm_lookup_has_no_retained_growth() -> TestResult {
    let mut report = report()?;
    let index = imports(&report)?
        .paths
        .iter()
        .position(|p| matches!(p.kind, PathKind::Body(_)))
        .ok_or("actual path")?;
    let path = &imports(&report)?.paths[index];
    let site = path.site;
    let terminal = path.terminal;
    let duplicate = PathUse {
        site,
        terminal,
        entries: path.entries.clone(),
        kind: path.kind,
    };
    let actual = imports_mut(&mut report)?;
    let growth =
        retained::Growth::vector::<PathUse>(actual.paths.len(), actual.paths.capacity(), 1);
    actual.charge(growth.work, growth.bytes)?;
    actual.paths.insert(index, duplicate);
    let facts = crate::pipeline::analyze_source(&report)?.facts;
    let before = imports(&report)?.budget.get();
    for _ in 0..128 {
        visit_warnings(&report, Some((site, terminal)), |_| Ok(()))?;
    }
    let after = imports(&report)?.budget.get();
    assert_eq!(before.bytes, after.bytes);
    assert_eq!(
        facts
            .warnings()
            .filter(|w| w.code == crate::support::diagnostics::WarningCode::Deprecated)
            .count(),
        1
    );
    assert!(after.steps > before.steps);
    assert_eq!(size_of::<Imports>(), size_of::<LegacyImports>() + 32);
    println!(
        "compatibility Imports={} oldowner={} PathUse={} Entry={} warm128={} retainedgrowth={}",
        size_of::<Imports>(),
        size_of::<LegacyImports>(),
        size_of::<PathUse>(),
        size_of::<PathEntry>(),
        after.steps - before.steps,
        after.bytes - before.bytes
    );
    Ok(())
}
#[test]
fn real_path_width_and_duplicate_feature_imports_keep_touched_cost_bounded() -> TestResult {
    let bytes = crate::support::crates::encode(&artifact()?)?;
    let mut previous = None;
    for width in [8, 32, 128] {
        let mut source = String::from("#![feature(no_core)]#![no_core]extern crate base;");
        for n in 0..width {
            use std::fmt::Write;
            write!(&mut source, "pub fn f{n}()->u8{{base::gated::marked()}}")?;
        }
        let (report, _, _) =
            source_packets::client_analysis(crate::support::crates::decode(&bytes)?, &source)?;
        let actual = imports(&report)?;
        let before = actual.budget.get();
        let paths = actual.paths.len();
        for path in &actual.paths {
            if !matches!(path.kind, PathKind::Import) {
                visit_warnings(&report, Some((path.site, path.terminal)), |_| Ok(()))?;
            }
        }
        let after = actual.budget.get();
        let work = after.steps - before.steps;
        assert_eq!(before.bytes, after.bytes);
        if let Some((old_paths, old_work)) = previous {
            assert_eq!(paths, old_paths * 4);
            assert!(work >= old_work * 4 && work < old_work * 6);
        }
        previous = Some((paths, work));
        println!(
            "compatibility width={width} paths={paths} work={work} bytes={}",
            after.bytes
        );
    }
    let mut report = report()?;
    let actual = imports_mut(&mut report)?;
    let policy = actual
        .inventory
        .artifacts()
        .iter()
        .find_map(|a| {
            a.source_descriptor()
                .and_then(|d| d.availability().ok())
                .and_then(|r| r.policies.first())
        })
        .ok_or("actual policy")?
        .clone();
    let before = actual.budget.get();
    let capacity = actual.features.capacity();
    for _ in 0..128 {
        actual.features(std::slice::from_ref(&policy))?;
    }
    assert_eq!(actual.features.capacity(), capacity);
    assert_eq!(actual.budget.get().bytes, before.bytes);
    let plain = crate::test_source::resolve("fn plain(){}", true)?;
    let ordinary = crate::pipeline::analyze_source(&plain)?;
    assert!(ordinary.facts.warnings().next().is_none());
    assert!(
        plain
            .resolution
            .crates
            .as_deref()
            .is_none_or(|i| i.availability.is_none())
    );
    assert_eq!(size_of::<crate::syntax::ast::Node>(), 36);
    assert_eq!(size_of::<crate::body::ir::Body>(), 456);
    Ok(())
}
