//! Tuple and record constructors call each field's actual source `Default`.
use super::*;
#[path = "elected_defaults.rs"]
mod enum_defaults;
const FIXTURES: &str = "pa31/tests/spec/tuple-default/";
fn fixture(name: &str) -> TestResult<String> {
    Ok(test_source::fixture(&format!("{FIXTURES}{name}.rs"))?)
}

#[test]
fn original_saturating_keeps_all_eight_members_and_real_tuple_default_production() -> TestResult {
    let input = fixture("saturating-expansion")?;
    let report = expanded_provider(&input, "saturating_eight")?;
    let entries = &report
        .attribute_effects
        .as_deref()
        .and_then(|effects| effects.derives.as_deref())
        .ok_or("actual derive list")?
        .entries;
    assert_eq!(entries.len(), 1);
    let entry = &entries[0];
    report.expanded_derive(entry.owner, entry.effect)?;
    assert_eq!(entry.contexts.len(), 8);
    for role in [
        Builtin::PartialEq,
        Builtin::Eq,
        Builtin::PartialOrd,
        Builtin::Ord,
        Builtin::Clone,
        Builtin::Copy,
        Builtin::Default,
        Builtin::Hash,
    ] {
        assert!(
            entry.contexts.clone().any(|index| matches!(
            report.definitions[report.invocations[index].definition].body,
            Body::Builtin(actual) if actual == role)),
            "{role:?}"
        );
    }
    let output = &report.package.units[0].source.text()[input.len()..];
    assert!(output.contains("crate::default::Default::default(),"));
    let unit = &report.package.units[entry.owner.file.0 as usize];
    let ast = &unit.configured.ast;
    let generated = report.invocations[entry.invocation].generated;
    assert!(ast.nodes.iter().any(|node| {
        let crate::syntax::ast::Kind::Call { callee, arguments } = node.kind else {
            return false;
        };
        let callee = ast.nodes[callee.get() as usize];
        let crate::syntax::ast::Kind::Name(name, _) = callee.kind else {
            return false;
        };
        if arguments.end - arguments.start != 1 {
            return false;
        }
        let field = ast.node(ast.list(arguments)[0]);
        let crate::syntax::ast::Kind::Call { callee, .. } = field.kind else {
            return false;
        };
        let path = ast.node(callee).span;
        node.span.start >= generated.start
            && node.span.end <= generated.end
            && &unit.source.text()[name.start as usize..name.end as usize] == "Self"
            && &unit.source.text()[path.start as usize..path.end as usize]
                == "crate::default::Default::default"
    }));
    assert!(!report.names_resolved);
    Ok(())
}

#[test]
fn generic_noncopy_tuple_and_record_defaults_reach_actual_ordered_native_bodies() -> TestResult {
    let input = fixture("generic-source")?;
    let report = test_source::source_provider(&input, "tuple_defaults")?;
    let entries = &report
        .attribute_effects
        .as_deref()
        .and_then(|effects| effects.derives.as_deref())
        .ok_or("actual default lists")?
        .entries;
    assert_eq!(entries.len(), 5);
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
            .any(|symbol| symbol.name == "observed" && symbol.body.is_some())
    );
    let out = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/tuple-default-v1/candidate/native");
    std::fs::create_dir_all(&out)?;
    std::fs::write(out.join("provider.rs"), input)?;
    crate::lowir::writer::write(
        &program,
        &mut std::fs::File::create(out.join("provider.lowir"))?,
    )?;
    Ok(())
}

#[test]
fn missing_field_defaults_and_wrong_values_fail_without_granting_field_proof() -> TestResult {
    for (name, expected) in [
        ("missing-default", "trait obligation"),
        ("wrong-field-type", "type"),
    ] {
        let result =
            test_source::source_provider(&fixture(name)?, "default_opponent").and_then(|report| {
                crate::pipeline::analyze_source(&report)
                    .map(|_| ())
                    .map_err(|error| error.to_string())
            });
        let error = result.err().ok_or_else(|| format!("accepted {name}"))?;
        assert!(error.contains(expected), "{name}: {error}");
        assert!(!error.contains("budget"), "{name}: {error}");
        eprintln!("field Default opponent {name}: {error}");
    }
    Ok(())
}

#[test]
fn tuple_default_work_scales_with_real_fields_and_keeps_dense_records_unchanged() -> TestResult {
    use std::fmt::Write as _;
    let input = fixture("generic-source")?;
    let mut previous = None;
    for count in [8, 32, 128] {
        let mut source = input.clone();
        source.push_str("#[derive(Default)]struct Many<T>(");
        for _ in 0..count {
            write!(&mut source, "T,")?;
        }
        source.push_str(");");
        let report = expanded_provider(&source, "many_default_fields")?;
        let payer = &report
            .package
            .source_core()
            .ok_or("actual source payer")?
            .macros;
        let before = payer.usage();
        let entries = &report
            .attribute_effects
            .as_deref()
            .and_then(|effects| effects.derives.as_deref())
            .ok_or("actual default lists")?
            .entries;
        for entry in entries {
            report.expanded_derive(entry.owner, entry.effect)?;
        }
        let after = payer.usage();
        assert!(after.0 > before.0);
        assert_eq!(after.1, before.1);
        if let Some((steps, bytes)) = previous {
            assert!(after.0 <= steps * 6, "fields={count}: {after:?}");
            assert!(after.1 <= bytes * 6, "fields={count}: {after:?}");
        }
        previous = Some((after.0, after.1));
        eprintln!("tuple Default fields={count} usage={after:?}");
    }
    assert_eq!(size_of::<crate::body::ir::Body>(), 456);
    assert_eq!(size_of::<crate::syntax::ast::Node>(), 36);
    Ok(())
}
