//! Actual source selectors pay and retain their crate-owned transaction.
use super::*;
use crate::resolve::{
    expansion::Report,
    unit::{FileId, NodeKey},
};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn source(repeats: usize) -> TestResult<Report> {
    let mut text = crate::test_source::fixture("pa31/tests/spec/source-preludes/builtin-sized.rs")?;
    for _ in 0..repeats {
        text.push_str("\n#[prelude_import] use crate::prelude::*;\n");
    }
    Ok(crate::test_source::source_provider(
        &text,
        "authored_records",
    )?)
}

fn earlier(report: &Report) -> TestResult<(NodeKey, usize, Context)> {
    let selected = report
        .resolution
        .introductions
        .source_prelude()
        .ok_or("selected owner")?;
    let unit = report.package.unit(FileId(0));
    let (owner, effect) = unit
        .configured
        .owners
        .iter()
        .find_map(|row| {
            if row.node == selected.owner.node {
                return None;
            }
            unit.retained_indexed_attributes(row.attributes.clone())
                .find(|(_, attribute, forest)| {
                    attribute.name(forest, unit.source.text()) == Some("prelude_import")
                })
                .map(|(effect, _, _)| {
                    (
                        NodeKey {
                            file: FileId(0),
                            node: row.node,
                        },
                        effect,
                    )
                })
        })
        .ok_or("actual earlier selector")?;
    Ok((owner, effect, selected.context))
}

#[test]
fn source_replacement_keeps_one_paid_crate_entry_and_the_last_owner() -> TestResult {
    let report = source(2)?;
    let registry = report
        .resolution
        .introductions
        .item_scopes
        .as_deref()
        .and_then(|sites| sites.prelude.as_deref())
        .ok_or("source registry")?;
    assert_eq!(registry.entries.len(), 1);
    let entry = registry.entries.get(&CrateSlot::HOST).ok_or("host entry")?;
    assert!(entry.pending.is_none());
    let selected = entry.selected.as_ref().ok_or("last selector")?;
    let last = report
        .package
        .unit(FileId(0))
        .source
        .text()
        .rfind("#[prelude_import]")
        .ok_or("actual final selector")?;
    assert_eq!(selected.attribute.span.start as usize, last);
    assert!(
        report
            .resolution
            .source_prelude_scope(&report.package)?
            .is_some()
    );
    // This is a real SourceCore provider with no guests, through body typing,
    // rather than the legacy default-Context bootstrap fixture.
    crate::pipeline::analyze_source(&report)?;
    assert_eq!(size_of::<crate::resolve::model::Resolution>(), 2032);
    assert_eq!(size_of::<crate::resolve::package::Package>(), 256);
    assert_eq!(size_of::<crate::sema::model::NodeFacts>(), 80);
    Ok(())
}

#[test]
fn failed_replacement_payment_leaves_the_previous_selection_unchanged() -> TestResult {
    let mut report = source(1)?;
    let expected = report
        .resolution
        .introductions
        .source_prelude()
        .ok_or("selected owner")?
        .selected;
    let (earlier, effect, context) = earlier(&report)?;
    let payer = report.package.macro_context()?.ok_or("source payer")?;
    let remaining = payer
        .contracts()
        .work_limit()
        .checked_sub(payer.contracts().usage().0)
        .and_then(|left| left.checked_sub(1))
        .ok_or("remaining source allowance")?;
    payer.contracts().charge(remaining, 0)?;
    for _ in 0..2 {
        assert!(
            crate::resolve::prelude_scope::claim(
                &report.package,
                &mut report.resolution,
                earlier,
                effect,
                context
            )
            .is_err()
        );
        assert_eq!(
            report
                .resolution
                .introductions
                .source_prelude()
                .ok_or("prior selector")?
                .selected,
            expected
        );
        let state = report
            .resolution
            .introductions
            .item_scopes
            .as_deref()
            .and_then(|sites| sites.prelude.as_deref())
            .ok_or("source registry")?;
        assert!(state.pending(CrateSlot::HOST).is_none());
    }
    Ok(())
}

#[test]
fn a_staged_selector_keeps_the_old_scope_until_the_real_lookup_commits() -> TestResult {
    let mut report = source(1)?;
    let previous = report
        .resolution
        .introductions
        .source_prelude()
        .ok_or("selected owner")?
        .owner;
    let (owner, effect, context) = earlier(&report)?;
    crate::resolve::prelude_scope::claim(
        &report.package,
        &mut report.resolution,
        owner,
        effect,
        context,
    )?;
    assert_eq!(
        report
            .resolution
            .introductions
            .source_prelude()
            .ok_or("prior selection")?
            .owner,
        previous
    );
    assert!(
        report
            .resolution
            .source_prelude_scope(&report.package)
            .is_err(),
        "unfinished phase accepted"
    );
    let pending = report
        .resolution
        .introductions
        .item_scopes
        .as_deref()
        .and_then(|sites| sites.prelude.as_deref())
        .and_then(|state| state.pending(CrateSlot::HOST))
        .ok_or("staged owner")?;
    let crate::syntax::item::ItemKind::Use(tree) = pending.item.kind else {
        return Err("actual selector use".into());
    };
    let mut state = crate::resolve::index::State::default();
    state.facts = report.resolution;
    assert!(
        state
            .bind(&mut report.package)
            .select_source_prelude(context, tree)?
    );
    assert_eq!(
        state
            .facts
            .introductions
            .source_prelude()
            .ok_or("committed selection")?
            .owner,
        owner
    );
    assert!(state.facts.source_prelude_scope(&report.package)?.is_some());
    Ok(())
}

#[test]
fn a_foreign_crate_key_cannot_rebind_an_authentic_source_selector() -> TestResult {
    let mut report = source(0)?;
    let state = report
        .resolution
        .introductions
        .item_scopes
        .as_deref_mut()
        .and_then(|sites| sites.prelude.as_deref_mut())
        .ok_or("source registry")?;
    let entry = state.entries.remove(&CrateSlot::HOST).ok_or("host entry")?;
    state.entries.insert(CrateSlot(1), entry);
    assert!(
        report
            .resolution
            .source_prelude_scope(&report.package)
            .is_err()
    );
    let state = report
        .resolution
        .introductions
        .item_scopes
        .as_deref_mut()
        .and_then(|sites| sites.prelude.as_deref_mut())
        .ok_or("source registry")?;
    let entry = state.entries.remove(&CrateSlot(1)).ok_or("foreign entry")?;
    state.entries.insert(CrateSlot::HOST, entry);
    assert!(
        report
            .resolution
            .source_prelude_scope(&report.package)?
            .is_some()
    );
    Ok(())
}

#[test]
fn repeated_source_selectors_pay_linear_work_and_constant_warm_lookup() -> TestResult {
    let mut observations = Vec::new();
    for repeats in [8usize, 32, 128] {
        let report = source(repeats)?;
        let payer = report.package.macro_context()?.ok_or("source payer")?;
        let usage = payer.contracts().usage();
        let registry = report
            .resolution
            .introductions
            .item_scopes
            .as_deref()
            .and_then(|sites| sites.prelude.as_deref())
            .ok_or("source registry")?;
        assert_eq!(registry.entries.len(), 1);
        let selected = report.resolution.selected_source_prelude(&report.package)?;
        let before = payer.contracts().usage();
        for _ in 0..repeats {
            assert_eq!(
                report.resolution.selected_source_prelude(&report.package)?,
                selected
            );
        }
        let after = payer.contracts().usage();
        assert_eq!(after.0 - before.0, repeats);
        assert_eq!(after.1, before.1);
        observations.push(usage);
    }
    for pair in observations.windows(2) {
        assert!(pair[1].0 <= pair[0].0 * 6 + 4096, "{observations:?}");
        assert!(pair[1].1 <= pair[0].1 * 6 + 4096, "{observations:?}");
    }
    Ok(())
}
