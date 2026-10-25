//! Real attribute-role declarations retain headers but never execute placeholders.
use super::*;
use crate::expand::builtin::{Builtin, BuiltinKind};

fn source(name: &str) -> TestResult<String> {
    Ok(crate::test_source::fixture(&format!(
        "pa31/tests/spec/source-attribute-builtins/{name}.rs"
    ))?)
}

#[test]
fn real_attribute_role_width_and_imports_prepare_each_actual_declaration_once() -> TestResult {
    for category in ["declarations", "imports"] {
        for width in [8_usize, 32, 128] {
            let probe = Probe::start()?;
            let report = crate::test_source::source_provider(
                &source(&format!("{category}-{width}"))?,
                "attribute_declarations",
            )?;
            let trace = probe.finish()?;
            let expected = if category == "declarations" { width } else { 1 };
            assert_eq!(
                trace
                    .events
                    .iter()
                    .filter(|event| matches!(event, Event::Compile { .. }))
                    .count(),
                expected
            );
            assert_eq!(
                trace
                    .events
                    .iter()
                    .filter(|event| matches!(event, Event::Published { .. }))
                    .count(),
                expected
            );
            assert!(
                !trace
                    .events
                    .iter()
                    .any(|event| matches!(event, Event::Selected(_) | Event::Denied(_)))
            );
            let mut actual = 0;
            for definition in &report.definitions {
                if let crate::resolve::expansion::Body::Builtin(role) = definition.body {
                    assert_eq!(role.kind(), BuiltinKind::Attribute);
                    assert!(!role.executable());
                    let key = definition.node.ok_or("real attribute declaration")?;
                    let header = crate::resolve::bindings::panic_contracts::published(
                        &report.package,
                        key,
                        definition.header.as_ref(),
                        1,
                    )?;
                    assert_eq!(header.builtin, Some(role));
                    assert_ne!(header.source_owner(), key);
                    actual += 1;
                }
            }
            assert_eq!(actual, expected);
            assert!(report.invocations.is_empty());
            assert_eq!(report.captures.len(), 0);
            assert_eq!(report.package.marks.context_count(), 1);
            if category == "imports" {
                assert_eq!(report.resolution.imports.len(), width);
            }
            let (work, payload, _) = report.package.source_core().ok_or("source")?.macros.usage();
            assert!(work < 4096 + 1024 * width);
            assert!(payload < 131_072 + 16_384 * width);
            eprintln!(
                "attribute {category} width={width} declarations={actual} work={work} payload={payload}"
            );
        }
    }
    Ok(())
}

#[test]
fn actual_alias_generated_and_configured_attribute_uses_stop_before_execution_work() -> TestResult {
    for name in [
        "direct-attribute-empty",
        "qualified-attribute-empty",
        "alias-attribute-empty",
        "generated-captured-path",
        "generated-captured-metadata",
        "generated-literal-attribute",
        "configured-active-attribute",
    ] {
        let probe = Probe::start()?;
        let error = crate::test_source::source_provider(&source(name)?, "attribute_denial")
            .err()
            .ok_or("attribute placeholder executed")?;
        let trace = probe.finish()?;
        assert!(
            error.contains("builtin attribute execution is not implemented"),
            "{name}: {error}"
        );
        let selected: Vec<_> = trace
            .events
            .iter()
            .filter_map(|event| {
                if let Event::Selected(value) = event {
                    Some(*value)
                } else {
                    None
                }
            })
            .collect();
        let denied: Vec<_> = trace
            .events
            .iter()
            .filter_map(|event| {
                if let Event::Denied(value) = event {
                    Some(*value)
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(selected.len(), 1, "{name}");
        assert_eq!(
            selected, denied,
            "attribute execution consumed work in {name}"
        );
        assert!(selected[0].owner.is_some());
        eprintln!("attribute denial {name}: {:?}", selected[0]);
    }
    Ok(())
}

#[test]
fn declaration_kind_and_macro_kind_remain_separate_for_actual_bang_calls() -> TestResult {
    for name in [
        "attribute-as-bang",
        "alias-attribute-as-bang",
        "scoped-bang-as-attribute",
    ] {
        let error = crate::test_source::source_provider(&source(name)?, "attribute_kind")
            .err()
            .ok_or("wrong macro kind executed")?;
        assert!(
            error.contains(if name == "scoped-bang-as-attribute" {
                "unresolved attribute macro"
            } else {
                "unresolved macro"
            }),
            "{name}: {error}"
        );
    }
    assert_eq!(Builtin::ConcatBytes as usize, 20);
    assert_eq!(Builtin::AutodiffForward as usize, 21);
    assert_eq!(Builtin::AutodiffReverse as usize, 22);
    assert_eq!(Builtin::Test as usize, 23);
    Ok(())
}
