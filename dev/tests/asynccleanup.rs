//! Actual async destructor ownership retains ordinary trait and body proof.
use crate::resolve as test_source;
use rustgm::{pipeline, types::Kind};
type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn async_source_roles_and_nominal_coverage_match_frozen_outcomes() -> TestResult {
    let mut count = 0;
    let mut failures = Vec::new();
    let expectations = test_source::fixture("pa43/tests/spec/async-destructors/causes.tsv")?;
    let mut causes = std::collections::HashMap::new();
    for line in expectations.lines() {
        let (path, rest) = line.split_once('\t').ok_or("cause path column")?;
        let (_, cause) = rest.split_once('\t').ok_or("candidate cause column")?;
        causes.insert(path, cause);
    }
    for manifest in [
        "pa31/tests/spec/async-drop-source/cases.tsv",
        "pa43/tests/spec/async-destructors/cases.tsv",
    ] {
        for line in test_source::fixture(manifest)?.lines() {
            let (path, status) = line.split_once('\t').ok_or("two case columns")?;
            let source = test_source::fixture(path)?;
            let result = (|| {
                let report = test_source::source_provider(&source, "async_cleanup")?;
                Ok::<_, Box<dyn std::error::Error>>(pipeline::analyze_source(&report)?)
            })();
            match (status, result) {
                ("0", Ok(_)) => {}
                ("0", Err(error)) => failures.push(format!("{path}: {error}")),
                ("1", Ok(_)) => failures.push(format!("accepted {path}")),
                ("1", Err(error)) => {
                    let message = error.to_string();
                    if let Some(cause) = causes.get(path)
                        && !message.contains(cause)
                    {
                        failures.push(format!("{path}: unrelated rejection: {message}"));
                    }
                    if [
                        "not implemented",
                        "exhausted",
                        "unavailable",
                        "missing registered",
                        "unsupported active source-core feature",
                    ]
                    .iter()
                    .any(|fence| message.contains(fence))
                    {
                        failures.push(format!("{path}: {message}"));
                    }
                }
                _ => return Err("invalid predicted outcome".into()),
            }
            count += 1;
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    assert_eq!(count, 35);
    Ok(())
}

#[test]
fn independent_async_companions_preserve_real_authored_bodies() -> TestResult {
    let manifest = test_source::fixture("pa43/tests/spec/async-destructors/oracle.tsv")?;
    let mut count = 0;
    for line in manifest.lines() {
        let (path, _) = line.split_once('\t').ok_or("two oracle columns")?;
        let name = std::path::Path::new(path)
            .file_name()
            .ok_or("fixture name")?
            .to_str()
            .ok_or("fixture UTF-8")?;
        let source = test_source::fixture(&format!("pa43/tests/spec/async-destructors/{name}"))?;
        let (_, body) = source
            .split_once(
                "#[lang=\"async_drop\"]pub trait Actual{async fn drop(self:Pin<&mut Self>);}\n",
            )
            .ok_or("actual source trait prefix")?;
        let oracle = test_source::fixture(path)?;
        let (_, oracle_body) = oracle
            .split_once("use core::pin::Pin;use core::future::AsyncDrop as Actual;\n")
            .ok_or("actual standard-core import prefix")?;
        assert_eq!(body.as_bytes(), oracle_body.as_bytes(), "{path}");
        count += 1;
    }
    assert_eq!(count, 9);
    Ok(())
}

#[test]
fn async_family_reuses_nominal_coverage_without_sync_method_registration() -> TestResult {
    let source = test_source::fixture("pa43/tests/spec/async-destructors/actual-generic.rs")?;
    let report = test_source::source_provider(&source, "async_family")?;
    let analysis = pipeline::analyze_source(&report)?;
    let facts = analysis.facts;
    assert_eq!(facts.destructors.implementations().len(), 1);
    let entries = facts
        .destructors
        .async_implementations()
        .collect::<Vec<_>>();
    let [entry] = entries.as_slice() else {
        return Err("one actual async implementation".into());
    };
    assert!(facts.destructors.self_type(entry.self_definition).is_none());
    assert!(facts.destructors.item(entry.item).is_some());
    let owner = facts
        .associated
        .owner(entry.self_definition)
        .ok_or("actual associated owner")?;
    let methods = facts
        .associated
        .owner_members(owner)
        .ok_or("actual members")?;
    let [method] = methods else {
        return Err("one actual async method".into());
    };
    assert!(facts.destructors.method(*method).is_none());
    let schema = facts
        .associated
        .schema(entry.self_definition)
        .ok_or("actual schema")?;
    assert!(
        matches!(facts.types.kind(schema.self_type),Some(Kind::Adt(item,_)) if item==entry.item)
    );
    assert!(
        facts
            .types
            .arguments(entry.coverage.arguments)
            .is_some_and(|args| args.len() == 1)
    );
    assert!(facts.opaques.body_results.contains_key(method));
    Ok(())
}

#[test]
fn ordinary_source_drop_retains_real_cleanup_body_and_native_lowering() -> TestResult {
    let source =
        test_source::fixture("pa31/tests/spec/async-drop-source/ordinary-drop-runtime.rs")?;
    let report = test_source::source_provider(&source, "ordinary_drop")?;
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = pipeline::analyze_source(&report)?;
    assert_eq!(facts.destructors.implementations().len(), 1);
    let program = rustgm::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        rustgm::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )?;
    for name in ["scalar", "borrowed", "owned"] {
        assert!(
            program
                .symbols
                .iter()
                .any(|s| s.name == name && s.body.is_some())
        );
    }
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/async-drop/source-v1/native");
    std::fs::create_dir_all(&folder)?;
    rustgm::lowir::writer::write(
        &program,
        &mut std::fs::File::create(folder.join("source.lowir"))?,
    )?;
    Ok(())
}
