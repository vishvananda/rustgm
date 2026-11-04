//! Real bodies consume exact lexical const exemptions without ambient permission.
use super::*;
type TestResult = Result<(), Box<dyn std::error::Error>>;
fn fixture(name: &str) -> Result<String, std::io::Error> {
    crate::test_source::fixture(&format!("pa31/tests/spec/const-fn-allowances/{name}.rs"))
}
fn provider(source: &str) -> Result<crate::resolve::expansion::Report, String> {
    crate::test_source::source_provider(source, "const_allowances")
}
#[test]
fn actual_source_allowances_preserve_targets_features_and_const_call_opponents() -> TestResult {
    for (name, accepted) in [
        ("allowed", true),
        ("missing", false),
        ("wrong", false),
        ("requested-without-allow", false),
        ("combined", true),
        ("empty", true),
        ("unknown", true),
        ("no-staged", true),
        ("nonconst", false),
        ("struct", false),
        ("module", false),
        ("foreign", false),
        ("missing-feature", false),
        ("word", false),
        ("equals", false),
        ("nested-list", false),
        ("name-value", false),
        ("string-name", false),
        ("bool-name", false),
        ("sibling", false),
        ("nested-does-not-inherit", false),
        ("runtime", true),
        ("inline-const", true),
        ("unsafe-denied", false),
        ("nonconst-callee", false),
        ("const-method", true),
        ("ordinary-method", false),
        ("trait-body", true),
        ("trait-required", false),
        ("cfg-removed", true),
        ("owned-macro-gate", true),
        ("captured-macro-gate", false),
        ("intrinsic-granted", true),
        ("intrinsic-missing-request", false),
        ("intrinsic-missing-grant", false),
    ] {
        let source = fixture(name)?;
        let result = provider(&source).and_then(|report| {
            crate::pipeline::analyze_source(&report)
                .map(|_| ())
                .map_err(|e| e.to_string())
        });
        assert_eq!(result.is_ok(), accepted, "{name}: {result:?}");
    }
    Ok(())
}

#[path = "fn_allowance_transport.rs"]
mod transport;
#[test]
fn combined_lists_are_exact_lexical_data_and_failed_queries_remain_unpaid() -> TestResult {
    let report = provider(&fixture("combined")?)?;
    let inventory = report
        .package
        .library_policy()
        .and_then(Library::inventory)
        .ok_or("inventory")?;
    let grants = inventory
        .consts
        .as_deref()
        .and_then(|rows| rows.grants.as_deref())
        .ok_or("grants")?;
    let [row] = grants.rows.as_slice() else {
        return Err("one real function row".into());
    };
    assert_eq!(
        row.features
            .iter()
            .map(AsRef::as_ref)
            .collect::<Vec<&str>>(),
        ["door", "other"]
    );
    assert_eq!(row.sites.len(), 2);
    let feature = FeatureName::new("door")?;
    let mut steps = 0;
    assert!(inventory.const_body_allows(row.definition, &feature, &mut steps)?);
    assert!(
        !report
            .package
            .library_policy()
            .ok_or("library")?
            .requested_feature(&feature, &mut steps)?
    );
    let mut exhausted = crate::support::crates::MAX_COMPONENTS;
    assert!(
        inventory
            .const_body_allows(row.definition, &feature, &mut exhausted)
            .is_err()
    );
    assert!(
        inventory
            .const_body_allows(row.definition, &feature, &mut exhausted)
            .is_err()
    );
    assert_eq!(size_of::<crate::resolve::stability::Inventory>(), 120);
    assert_eq!(size_of::<ConstPolicies>(), 56);
    assert_eq!(size_of::<crate::body::ir::Body>(), 456);
    Ok(())
}
#[test]
fn sparse_function_grants_scale_with_source_and_preserve_ordinary_absence() -> TestResult {
    use std::fmt::Write as _;
    let seed = fixture("empty")?;
    let (head, _) = seed
        .split_once("#[rustc_allow_const_fn_unstable")
        .ok_or("head")?;
    let mut previous = None;
    for width in [8_usize, 32, 128] {
        let mut source = head.to_owned();
        for index in 0..width {
            write!(
                source,
                "#[rustc_allow_const_fn_unstable(door)]const fn f{index}()->u8{{7}}"
            )?;
        }
        let report = provider(&source)?;
        let inventory = report
            .package
            .library_policy()
            .and_then(Library::inventory)
            .ok_or("inventory")?;
        let grants = inventory
            .consts
            .as_deref()
            .and_then(|rows| rows.grants.as_deref())
            .ok_or("grants")?;
        assert_eq!(grants.rows.len(), width);
        assert!(
            grants
                .rows
                .iter()
                .all(|row| row.features.len() == 1 && row.sites.len() == 1)
        );
        let work = inventory.steps();
        if let Some(before) = previous {
            assert!(work < 6 * before);
        }
        previous = Some(work);
        eprintln!(
            "const grants width={width} work={work} bytes={}",
            grants.storage()
        );
    }
    let report = crate::test_source::resolve("fn ordinary(){}", true)?;
    assert!(report.package.library_policy().is_none());
    Ok(())
}
