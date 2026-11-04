//! Spelling sharing preserves actual policy ownership and strict inventory bounds.
use super::*;
use crate::{
    pipeline,
    resolve::stability::{Library, PolicyKind},
    test_source,
};
use std::fmt::Write as _;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn source(name: &str) -> TestResult<String> {
    Ok(test_source::fixture(&format!(
        "pa31/tests/spec/stability-symbols/{name}.rs"
    ))?)
}
fn repeated(count: usize, unique: bool) -> TestResult<String> {
    let mut text = source("repeated")?;
    for index in 0..count {
        let feature = if unique {
            format!("unique_{index}_{}", "x".repeat(60))
        } else {
            format!("shared_{}", "x".repeat(121))
        };
        writeln!(
            text,
            "#[stable(feature=\"{feature}\",since=\"1.0.0\")]pub fn f{index}()->u8{{7}}"
        )?;
    }
    Ok(text)
}

#[test]
fn decoded_duplicate_spellings_share_storage_without_merging_source_policy_owners() -> TestResult {
    let report = test_source::resolve(&source("escaped-duplicate")?, true)?;
    let inventory = report
        .package
        .library_policy()
        .and_then(Library::inventory)
        .ok_or("inventory")?;
    let rows: Vec<_> = inventory
        .policies
        .iter()
        .filter(|p| p.feature.as_ref() == "shared_api")
        .collect();
    assert_eq!(rows.len(), 2);
    assert!(Rc::ptr_eq(&rows[0].feature, &rows[1].feature));
    assert_ne!(rows[0].origin, rows[1].origin);
    pipeline::analyze_source(&report)?;
    let report = test_source::resolve(&source("const-and-default")?, true)?;
    let inventory = report
        .package
        .library_policy()
        .and_then(Library::inventory)
        .ok_or("inventory")?;
    let shared: Vec<_> = inventory
        .policies
        .iter()
        .filter(|p| p.feature.as_ref() == "shared_api")
        .collect();
    assert_eq!(shared.len(), 4);
    assert!(
        shared
            .iter()
            .all(|p| Rc::ptr_eq(&p.feature, &shared[0].feature))
    );
    assert_eq!(
        shared
            .iter()
            .filter(|p| p.kind == PolicyKind::Const)
            .count(),
        1
    );
    let default = inventory
        .policies
        .iter()
        .find(|p| p.kind == PolicyKind::DefaultBody)
        .ok_or("default")?;
    assert_eq!(default.feature.as_ref(), "next_api");
    assert!(!Rc::ptr_eq(&default.feature, &shared[0].feature));
    pipeline::analyze_source(&report)?;
    Ok(())
}

#[test]
fn feature_conflicts_remain_deferred_until_all_source_attributes_validate() -> TestResult {
    for (name, expected) in [
        ("late-conflict", "conflicting library feature definitions"),
        (
            "invalid-before-conflict",
            "unstable library policy requires issue",
        ),
    ] {
        let error = test_source::resolve(&source(name)?, true)
            .err()
            .ok_or_else(|| format!("invalid {name} accepted"))?;
        assert!(error.contains(expected), "{name}: {error}");
    }
    Ok(())
}

#[test]
fn shared_and_distinct_feature_work_scales_with_actual_canonical_storage() -> TestResult {
    for unique in [false, true] {
        let mut previous = None;
        for count in [16, 64, 256] {
            let report = test_source::resolve(&repeated(count, unique)?, true)?;
            let inventory = report
                .package
                .library_policy()
                .and_then(Library::inventory)
                .ok_or("inventory")?;
            let steps = inventory.steps();
            if !unique {
                assert!(steps < count * 192 + 10000, "{count}: {steps}");
            }
            if let Some(prior) = previous {
                assert!(steps < prior * 6, "{unique} {count}: {steps}");
            }
            previous = Some(steps);
            pipeline::analyze_source(&report)?;
        }
    }
    Ok(())
}

#[test]
fn large_repeated_feature_crate_stays_inside_the_original_inventory_limits() -> TestResult {
    let text = repeated(4096, false)?;
    let report = test_source::resolve(&text, true)?;
    let inventory = report
        .package
        .library_policy()
        .and_then(Library::inventory)
        .ok_or("inventory")?;
    assert_eq!(inventory.policies.len(), 4097);
    assert!(inventory.steps() < 1_000_000);
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/stability-symbols-v1");
    std::fs::create_dir_all(&folder)?;
    std::fs::write(folder.join("repeated-4096.rs"), text)?;
    std::fs::write(folder.join("large-work.txt"), inventory.steps().to_string())?;
    pipeline::analyze_source(&report)?;
    assert_eq!(size_of::<super::super::Policy>(), 56);
    assert_eq!(size_of::<super::super::Inventory>(), 112);
    assert_eq!(size_of::<crate::syntax::ast::Node>(), 36);
    assert_eq!(size_of::<crate::body::ir::Body>(), 456);
    Ok(())
}

#[test]
fn failed_lookup_and_capacity_payment_publish_no_canonical_feature() -> TestResult {
    let mut exhausted = Draft {
        steps: 1_000_000,
        ..Draft::default()
    };
    assert!(exhausted.feature("unpaid", FileId(0), 0).is_err());
    assert!(exhausted.features.definitions.is_empty());
    let mut draft = Draft::default();
    let (name, first) = draft.feature("unpublished", FileId(0), 0)?;
    assert_eq!(name.as_ref(), "unpublished");
    assert!(first.is_none() && draft.features.definitions.is_empty());
    draft.bytes = 64 * 1024 * 1024;
    assert!(draft.reserve_feature(first).is_err());
    assert!(draft.features.definitions.is_empty() && draft.features.conflict.is_none());
    assert!(draft.policies.is_empty());
    Ok(())
}
