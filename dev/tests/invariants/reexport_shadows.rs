//! Real namespace, import-site, effective-export and early completion controls.
use crate::{pipeline, sema};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
fn fixture(name: &str) -> std::io::Result<String> {
    std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../pa40/tests/spec/hidden-glob-reexports")
            .join(format!("{name}.rs")),
    )
}
#[test]
fn completed_resolution_closes_hidden_expectations_without_body_demand() -> TestResult {
    for name in ["expect", "no-glob"] {
        let report = crate::test_source::resolve(&fixture(name)?, false)?;
        let Err(error) = sema::declarations::analyze(&report) else {
            return Err("accepted explicit lint without completed import provenance".into());
        };
        assert!(
            error
                .to_string()
                .contains("type checking requires completed name resolution")
        );
    }
    let legacy = crate::test_source::resolve("fn ordinary() {}", true)?;
    assert!(
        sema::declarations::analyze(&legacy)?
            .warnings()
            .next()
            .is_none()
    );
    for (name, expected) in [("expect", 0), ("no-glob", 1), ("macro-only", 1)] {
        let report = crate::test_source::resolve(&fixture(name)?, true)?;
        for _ in 0..2 {
            let facts = sema::declarations::analyze(&report)?;
            assert_eq!(facts.warnings().count(), expected, "{name}");
            assert!(facts.typed_function_bodies.is_empty());
        }
    }
    let report = crate::test_source::resolve(&fixture("early-declaration")?, true)?;
    let facts = sema::declarations::analyze(&report)?;
    assert!(facts.warnings().next().is_none());
    assert!(facts.typed_function_bodies.is_empty());
    let Err(error) = pipeline::analyze(&report, facts) else {
        return Err("accepted invalid lazy body".into());
    };
    assert!(error.warnings().is_empty());
    Ok(())
}

#[test]
fn completed_shadow_causes_authenticate_source_and_resolved_binding() -> TestResult {
    let mut report = crate::test_source::resolve(&fixture("warning")?, true)?;
    let producer = report.lint_producer().ok_or("source producer")?;
    let cause = producer
        .hidden_glob_events()
        .first()
        .ok_or("actual shadow event")?;
    let key = cause.key;
    let old = *report
        .resolution
        .bindings
        .get_key(key)
        .ok_or("private winner")?;
    report.resolution.bindings.insert_key(
        key,
        super::Binding {
            access: super::Access::Public,
            ..old
        },
    )?;
    let Err(error) = sema::declarations::analyze(&report) else {
        return Err("accepted changed winner".into());
    };
    assert!(
        error
            .to_string()
            .contains("hidden glob resolution identity changed")
    );
    let mut report = crate::test_source::resolve(&fixture("warning")?, true)?;
    let key = report
        .lint_producer()
        .ok_or("source producer")?
        .hidden_glob_events()[0]
        .owner
        .key;
    report.package.units[key.file.0 as usize]
        .configured
        .ast
        .nodes[key.node.get() as usize]
        .span
        .end += 1;
    let Err(error) = sema::declarations::analyze(&report) else {
        return Err("accepted changed source owner".into());
    };
    assert!(
        error
            .to_string()
            .contains("hidden glob source owner changed")
    );
    let mut report = crate::test_source::resolve(&fixture("warning")?, true)?;
    let counterpart = report
        .lint_producer()
        .ok_or("source producer")?
        .hidden_glob_events()[0]
        .counterpart;
    report.resolution.bindings.insert_key(
        counterpart.0,
        super::Binding {
            access: super::Access::Within(crate::resolve::package::ModuleId(0)),
            ..counterpart.1
        },
    )?;
    let Err(error) = sema::declarations::analyze(&report) else {
        return Err("accepted changed public counterpart".into());
    };
    assert!(
        error
            .to_string()
            .contains("hidden glob resolution identity changed")
    );
    Ok(())
}
