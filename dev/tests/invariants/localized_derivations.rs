//! Local derives preserve the actual block owner and ordinary field obligations.
use super::*;
use crate::test_source;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const FIXTURES: &str = "pa31/tests/spec/block-derives/";

fn provider(name: &str) -> TestResult<crate::resolve::expansion::Report> {
    Ok(test_source::source_provider(
        &test_source::fixture(&format!("{FIXTURES}{name}.rs"))?,
        "local_derivations",
    )?)
}

#[test]
fn original_span_calls_real_field_default_clone_and_copy_in_its_block() -> TestResult {
    let report = provider("span-source")?;
    let entries = &report
        .attribute_effects
        .as_deref()
        .and_then(|effects| effects.derives.as_deref())
        .ok_or("derive transactions")?
        .entries;
    let local = entries
        .iter()
        .filter(|entry| entry.placement.in_block)
        .collect::<Vec<_>>();
    assert_eq!(local.len(), 1);
    let entry = local[0];
    assert_eq!(entry.contexts.len(), 3);
    assert!(entry.placement.parent.is_some());
    assert_ne!(entry.placement.scope, report.resolution.module_scopes[0]);
    for entry in entries {
        report.expanded_derive(entry.owner, entry.effect)?;
    }
    let crate::pipeline::Analysis {
        mut facts,
        prepared,
    } = crate::pipeline::analyze_source(&report)?;
    let program = crate::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        crate::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )?;
    assert!(
        program
            .symbols
            .iter()
            .any(|symbol| symbol.name == "span_observed" && symbol.body.is_some())
    );
    let out = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/block-derive-v1/candidate/native");
    std::fs::create_dir_all(&out)?;
    std::fs::write(
        out.join("provider.rs"),
        test_source::fixture(&format!("{FIXTURES}span-source.rs"))?,
    )?;
    crate::lowir::writer::write(
        &program,
        &mut std::fs::File::create(out.join("provider.lowir"))?,
    )?;
    Ok(())
}

#[test]
fn local_derive_scope_and_invocation_receipts_reject_relocation() -> TestResult {
    let mut report = provider("span-source")?;
    let entries = report
        .attribute_effects
        .as_deref_mut()
        .and_then(|effects| effects.derives.as_deref_mut())
        .ok_or("derive transactions")?;
    let index = entries
        .entries
        .iter()
        .position(|entry| entry.placement.in_block)
        .ok_or("local derive")?;
    let entry = &entries.entries[index];
    let (owner, effect, original) = (entry.owner, entry.effect, entry.placement);
    let mut opponents = [original; 5];
    opponents[0].scope = report.resolution.module_scopes[0];
    opponents[1].parent = None;
    opponents[2].scope_introduction = ExpansionId(u32::MAX);
    opponents[3].in_block = false;
    opponents[4].introduction = ExpansionId(u32::MAX);
    for changed in opponents {
        report
            .attribute_effects
            .as_deref_mut()
            .and_then(|effects| effects.derives.as_deref_mut())
            .ok_or("derive transactions")?
            .entries[index]
            .placement = changed;
        let before = report
            .package
            .source_core()
            .ok_or("source payer")?
            .macros
            .usage();
        assert!(report.expanded_derive(owner, effect).is_err());
        let rejected = report
            .package
            .source_core()
            .ok_or("source payer")?
            .macros
            .usage();
        assert!(rejected.0 > before.0);
        assert_eq!(rejected.1, before.1);
    }
    report
        .attribute_effects
        .as_deref_mut()
        .and_then(|effects| effects.derives.as_deref_mut())
        .ok_or("derive transactions")?
        .entries[index]
        .placement = original;
    let payer = &report.package.source_core().ok_or("source payer")?.macros;
    let before = payer.usage();
    report.expanded_derive(owner, effect)?;
    let first = payer.usage();
    report.expanded_derive(owner, effect)?;
    let warm = payer.usage();
    assert!(first.0 > before.0 && warm.0 > first.0);
    assert_eq!(first.1, before.1);
    assert_eq!(warm.1, first.1);
    Ok(())
}

#[test]
fn local_container_batch_work_tracks_actual_blocks_without_dense_growth() -> TestResult {
    use std::fmt::Write as _;
    let source = test_source::fixture(&format!("{FIXTURES}span-source.rs"))?;
    let prefix = source
        .split_once("unsafe extern \"C\" { fn event")
        .ok_or("source foundation")?
        .0;
    let mut previous = None;
    for count in [8, 32, 128] {
        let mut input = prefix.to_owned();
        for index in 0..count {
            write!(
                &mut input,
                "fn block_{index}(){{use crate::derive;use crate::marker::Copy;use crate::clone::Clone;use crate::default::Default;#[derive(Copy,Clone,Default)]struct Span{{start:usize,len:usize}}let _=Span::default();}}"
            )?;
        }
        let report = test_source::source_provider(&input, "local_container_work")?;
        let usage = report
            .package
            .source_core()
            .ok_or("source payer")?
            .macros
            .usage();
        if let Some((work, bytes)) = previous {
            assert!(usage.0 <= work * 6, "blocks={count}: {usage:?}");
            assert!(usage.1 <= bytes * 6, "blocks={count}: {usage:?}");
        }
        previous = Some((usage.0, usage.1));
        eprintln!("local blocks={count} source usage={usage:?}");
    }
    assert_eq!(size_of::<crate::syntax::ast::Node>(), 36);
    assert_eq!(size_of::<crate::resolve::expansion::Body>(), 96);
    eprintln!(
        "cold local derive Placement={} Transaction={}",
        size_of::<Placement>(),
        size_of::<super::super::Transaction>()
    );
    Ok(())
}

#[test]
fn local_names_nested_capture_and_missing_field_default_keep_real_failures() -> TestResult {
    for name in [
        "name-leak",
        "other-local-owner",
        "nested-generic-capture",
        "missing-default",
    ] {
        let result = provider(name).and_then(|report| {
            crate::pipeline::analyze_source(&report)
                .map(|_| ())
                .map_err(Into::into)
        });
        let error = result
            .err()
            .ok_or_else(|| format!("accepted {name}"))?
            .to_string();
        assert!(!error.contains("budget"), "{name}: {error}");
        assert!(
            !error.contains("block-owned source derives"),
            "{name}: {error}"
        );
        eprintln!("local derive opponent {name}: {error}");
    }
    Ok(())
}
