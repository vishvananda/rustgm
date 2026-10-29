//! Inert strings do not become source identities, callable proof or wire roles.
use super::*;
use crate::{resolve::model::DefKind, test_source};
use std::fmt::Write as _;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
fn fixture(name: &str) -> TestResult<String> {
    Ok(test_source::fixture(&format!(
        "pa31/tests/spec/inert-diagnostics/{name}.rs"
    ))?)
}
fn provider(source: &str) -> TestResult<crate::resolve::expansion::Report> {
    Ok(test_source::source_provider(source, "inert_diagnostic")?)
}
#[test]
fn actual_extra_names_keep_targets_exact_strings_and_known_role_boundaries() -> TestResult {
    for name in ["targets", "exact-strings"] {
        let report = provider(&fixture(name)?)?;
        observe_column(&report, name)?;
    }
    for name in [
        "duplicate-items",
        "duplicate-owner",
        "invalid-target",
        "invalid-form",
        "unknown-lang",
        "unknown-builtin",
    ] {
        assert!(
            provider(&fixture(name)?).is_err(),
            "accepted invalid {name}"
        );
    }
    let prefix = fixture("exact-strings")?;
    for suffix in [
        "unsafe extern \"Rust\"{#[rustc_diagnostic_item=\"foreign_fn\"]fn foreign();}",
        "#[rustc_diagnostic_item=\"trait_impl\"]impl Clone for u8{fn clone(&self)->Self{*self}}",
        "#[rustc_diagnostic_item=\"neg\"]fn wrong_known(){}",
    ] {
        assert!(
            provider(&(prefix.clone() + suffix)).is_err(),
            "invalid target/known owner: {suffix}"
        );
    }
    let missing = fixture("targets")?.replace("rustc_attrs,", "");
    assert!(provider(&missing).is_err());
    let base = fixture("exact-strings")?;
    let (base, _) = base
        .split_once("#[rustc_diagnostic_item=\"\"]")
        .ok_or("base")?;
    let executable = provider(
        &(base.to_owned() + "#[rustc_diagnostic_item=\"inert_callable\"]pub fn named()->isize{7}"),
    )?;
    let analysis = crate::pipeline::analyze_source(&executable)?;
    assert!(analysis.facts.functions.iter().any(|function| {
        executable
            .package
            .names
            .get(executable.resolution.definitions[function.definition.0 as usize].name)
            == "named"
    }));
    let report = provider(&fixture("targets")?)?;
    assert!(
        report
            .resolution
            .definitions
            .iter()
            .any(|d| d.kind == DefKind::Trait)
    );
    eprintln!(
        "inert diagnostics Reservations={} Claim={} Entry={} Column={} Names={} Body={} Rvalue={} Package={}",
        size_of::<super::super::reservations::Reservations>(),
        size_of::<super::super::reservations::Claim>(),
        size_of::<Entry>(),
        size_of::<Column>(),
        size_of::<Names>(),
        size_of::<crate::body::ir::Body>(),
        size_of::<crate::body::ir::Rvalue>(),
        size_of::<crate::resolve::package::Package>()
    );
    // Appended source range-member roles extend this fixed cold table.
    assert_eq!(size_of::<super::super::reservations::Reservations>(), 2288);
    assert_eq!(size_of::<super::super::reservations::Claim>(), 2);
    assert_eq!(size_of::<crate::body::ir::Body>(), 456);
    assert_eq!(size_of::<crate::body::ir::Rvalue>(), 20);
    Ok(())
}
#[test]
fn sparse_scaling_atomic_names_and_exhausted_payments_keep_existing_data() -> TestResult {
    let seed = fixture("exact-strings")?;
    let (base, _) = seed
        .split_once("#[rustc_diagnostic_item=\"\"]")
        .ok_or("base")?;
    let mut previous = 0;
    for count in [0, 8, 32, 128] {
        let mut input = base.to_owned();
        for index in 0..count {
            write!(
                input,
                "#[rustc_diagnostic_item=\"extra_{index}\"]fn f{index}(){{}}"
            )?;
        }
        let report = provider(&input)?;
        let reserved = report
            .resolution
            .source_bindings
            .as_deref()
            .ok_or("source")?;
        assert_eq!(
            reserved
                .extra_diagnostics
                .as_deref()
                .map_or(0, |c| c.rows.len()),
            count
        );
        assert_eq!(reserved.extra_diagnostics.is_none(), count == 0);
        if count != 0 {
            assert!(reserved.steps > previous);
        }
        eprintln!(
            "inert count={count} steps={} retained={}",
            reserved.steps,
            reserved
                .extra_diagnostics
                .as_deref()
                .map_or(0, |c| size_of::<Column>() + retained::vector(&c.rows))
        );
        assert!(reserved.steps < count.saturating_mul(300).saturating_add(2048));
        let mut reverse = base.to_owned();
        for index in (0..count).rev() {
            write!(
                reverse,
                "#[rustc_diagnostic_item=\"extra_{index}\"]fn f{index}(){{}}"
            )?;
        }
        let opposite = provider(&reverse)?;
        let opposite_rows = opposite
            .resolution
            .source_bindings
            .as_deref()
            .ok_or("opposite source")?
            .extra_diagnostics
            .as_deref();
        let mut left: Vec<_> = reserved
            .extra_diagnostics
            .as_deref()
            .into_iter()
            .flat_map(|c| &c.rows)
            .map(|row| report.package.names.get(row.name))
            .collect();
        let mut right: Vec<_> = opposite_rows
            .into_iter()
            .flat_map(|c| &c.rows)
            .map(|row| opposite.package.names.get(row.name))
            .collect();
        left.sort_unstable();
        right.sort_unstable();
        assert_eq!(left, right);
        previous = reserved.steps;
    }
    let ordinary = crate::test_source::resolve("fn ordinary(){}", true)?;
    assert!(ordinary.resolution.source_bindings.is_none());
    let mut names = Names::default();
    let mut draft = Draft::default();
    draft.budget.work.steps = crates::MAX_COMPONENTS;
    assert!(draft.seal(&mut names).is_err());
    assert_eq!(draft.budget.work.steps, crates::MAX_COMPONENTS + 1);
    assert!(draft.pending.is_empty());
    assert!(draft.seal(&mut names).is_err());
    Ok(())
}

fn observe_column(report: &crate::resolve::expansion::Report, name: &str) -> TestResult {
    let reserved = report
        .resolution
        .source_bindings
        .as_deref()
        .ok_or("source reservations")?;
    let column = reserved
        .extra_diagnostics
        .as_deref()
        .ok_or("actual inert rows")?;
    assert_eq!(column.rows.len(), if name == "targets" { 16 } else { 4 });
    for entry in &column.rows {
        let unit = report.package.unit(entry.owner.file);
        assert!(entry.attribute.end as usize <= unit.source.text().len());
        assert!(
            !matches!(unit.configured.ast.node(entry.owner.node).kind,Kind::Item(i)
            if matches!(unit.configured.ast.items[i as usize].kind,ItemKind::Static{..}|ItemKind::ForeignType))
        );
    }
    if name == "exact-strings" {
        let values: Vec<_> = column
            .rows
            .iter()
            .map(|row| report.package.names.get(row.name))
            .collect();
        assert!(values.contains(&""));
        assert!(values.contains(&"nul\0name"));
        assert!(values.contains(&"é"));
        assert!(values.contains(&"e\u{301}"));
        assert_ne!(column.rows[2].name, column.rows[3].name);
    }
    assert!(
        reserved
            .language(crate::support::bindings::LangItem::Neg)
            .is_none()
    );
    let foreign = provider(&fixture(name)?)?;
    assert!(!std::ptr::eq(
        reserved,
        foreign
            .resolution
            .source_bindings
            .as_deref()
            .ok_or("foreign source")?
    ));
    Ok(())
}
