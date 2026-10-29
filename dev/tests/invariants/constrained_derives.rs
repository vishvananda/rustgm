//! Parsed generic constraints survive source-owned derive headers and real bodies.
use super::*;
use crate::syntax::{ast::Kind, item::GenericFlavor};
const FIXTURES: &str = "pa31/tests/spec/bounded-derive/";
fn fixture(name: &str) -> TestResult<String> {
    Ok(test_source::fixture(&format!("{FIXTURES}{name}.rs"))?)
}
#[test]
fn original_manually_drop_preserves_four_members_and_relaxed_bound() -> TestResult {
    let input = fixture("manually-drop-expansion")?;
    let report = expanded_provider(&input, "original_manually_drop")?;
    let entries = &report
        .attribute_effects
        .as_deref()
        .and_then(|e| e.derives.as_deref())
        .ok_or("actual derives")?
        .entries;
    assert_eq!(entries.len(), 1);
    let entry = &entries[0];
    report.expanded_derive(entry.owner, entry.effect)?;
    assert_eq!(entry.contexts.len(), 4);
    for role in [
        Builtin::Copy,
        Builtin::Clone,
        Builtin::Debug,
        Builtin::Default,
    ] {
        assert!(entry.contexts.clone().any(|index| matches!(report.definitions[report.invocations[index].definition].body, Body::Builtin(actual) if actual == role)));
    }
    let unit = &report.package.units[0];
    let ast = &unit.configured.ast;
    let mut count = 0;
    for node in &ast.nodes {
        if (node.span.start as usize) < input.len() {
            continue;
        }
        let Kind::Item(index) = node.kind else {
            continue;
        };
        let item = ast.items[index as usize];
        if !matches!(item.kind, crate::syntax::item::ItemKind::Impl { .. }) {
            continue;
        }
        let parameters = ast.list(item.generics);
        assert_eq!(parameters.len(), 1);
        let Kind::Generic(index) = ast.node(parameters[0]).kind else {
            return Err("actual formal".into());
        };
        let parameter = ast.generics[index as usize];
        assert_eq!(parameter.flavor, GenericFlavor::Type);
        assert!(parameter.default.is_none());
        let bounds = ast.list(parameter.bounds);
        assert_eq!(bounds.len(), 2);
        assert!(matches!(
            ast.node(bounds[0]).kind,
            Kind::Bound { optional: true, .. }
        ));
        count += 1;
    }
    assert_eq!(count, 4);
    assert!(!report.names_resolved);
    Ok(())
}
#[test]
fn bounded_type_const_defaults_and_where_calls_reach_ordered_native_bodies() -> TestResult {
    let input = fixture("generic-source")?;
    let report = test_source::source_provider(&input, "bounded_derive")?;
    check_actual_headers(&report, input.len())?;
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
        .join("../obj/parallel/bounded-derive-v1/candidate/native");
    std::fs::create_dir_all(&out)?;
    std::fs::write(out.join("provider.rs"), input)?;
    crate::lowir::writer::write(
        &program,
        &mut std::fs::File::create(out.join("provider.lowir"))?,
    )?;
    Ok(())
}
fn check_actual_headers(report: &super::super::super::Report, original: usize) -> TestResult {
    use crate::syntax::item::ItemKind;
    let unit = &report.package.units[0];
    let ast = &unit.configured.ast;
    let mut count = 0;
    for node in &ast.nodes {
        let Kind::Item(index) = node.kind else {
            continue;
        };
        let item = ast.items[index as usize];
        let ItemKind::Impl { ty, .. } = item.kind else {
            continue;
        };
        if (node.span.start as usize) < original || ast.list(item.generics).is_empty() {
            continue;
        }
        let application = if let Kind::Path { segments, .. } = ast.node(ty).kind {
            let [segment] = ast.list(segments) else {
                return Err("actual nominal application path".into());
            };
            *segment
        } else {
            ty
        };
        let Kind::Applied {
            base, arguments, ..
        } = ast.node(application).kind
        else {
            return Err("actual generic application".into());
        };
        let name = ast.node(base).span;
        let name = &unit.source.text()[name.start as usize..name.end as usize];
        assert_eq!(ast.list(arguments).len(), ast.list(item.generics).len());
        for &parameter in ast.list(item.generics) {
            let Kind::Generic(index) = ast.node(parameter).kind else {
                return Err("actual impl formal".into());
            };
            assert!(ast.generics[index as usize].default.is_none());
        }
        assert_eq!(
            ast.list(item.where_clause).len(),
            usize::from(name != "Unsized"),
            "{name}"
        );
        let first = ast.node(ast.list(item.generics)[0]);
        let Kind::Generic(index) = first.kind else {
            return Err("first actual formal".into());
        };
        let parameter = ast.generics[index as usize];
        let expected = match name {
            "Bounded" | "ConstHolder" | "Unsized" => 2,
            "Borrowed" => 1,
            _ => return Err(format!("unexpected generic derive {name}").into()),
        };
        assert_eq!(ast.list(parameter.bounds).len(), expected, "{name}");
        count += 1;
    }
    assert_eq!(count, 4);
    Ok(())
}
#[test]
fn authored_constraints_and_field_defaults_are_independent_requirements() -> TestResult {
    for name in [
        "missing-owner-bound",
        "missing-where-bound",
        "missing-default",
        "wrong-const-type",
    ] {
        let error = test_source::source_provider(&fixture(name)?, "bound_opponent")
            .and_then(|report| {
                crate::pipeline::analyze_source(&report)
                    .map(|_| ())
                    .map_err(|e| e.to_string())
            })
            .err()
            .ok_or_else(|| format!("accepted {name}"))?;
        assert!(!error.contains("wider profile"), "{name}: {error}");
        assert!(!error.contains("budget"), "{name}: {error}");
        eprintln!("derive constraint {name}: {error}");
    }
    let input = fixture("generic-source")?;
    let higher = input + "#[derive(Default)]struct Higher<T>where T:for<'a> Stamp{value:T}";
    let error = expanded_provider(&higher, "higher_required")
        .err()
        .ok_or("accepted higher-ranked predicate")?
        .to_string();
    assert!(
        error.contains("higher-ranked source derive bounds require a wider profile"),
        "{error}"
    );
    Ok(())
}
#[test]
fn header_work_scales_with_actual_predicates_and_warm_receipts() -> TestResult {
    let input = fixture("generic-source")?;
    let mut previous = None;
    for count in [8, 32, 128] {
        let mut source = input.clone();
        source.push_str("#[derive(Default)]struct Many<T:Stamp=First> where ");
        for _ in 0..count {
            source.push_str("T:Extra,");
        }
        source.push_str("{value:T}");
        let report = expanded_provider(&source, "many_constraints")?;
        let payer = &report.package.source_core().ok_or("actual payer")?.macros;
        let before = payer.usage();
        let entries = &report
            .attribute_effects
            .as_deref()
            .and_then(|e| e.derives.as_deref())
            .ok_or("actual derives")?
            .entries;
        for entry in entries {
            report.expanded_derive(entry.owner, entry.effect)?;
        }
        let after = payer.usage();
        assert!(after.0 > before.0);
        assert_eq!(after.1, before.1);
        if let Some((work, bytes)) = previous {
            assert!(after.0 <= work * 6, "predicates={count}: {after:?}");
            assert!(after.1 <= bytes * 6, "predicates={count}: {after:?}");
        }
        previous = Some((after.0, after.1));
        eprintln!("derive predicates={count}: {after:?}");
    }
    assert_eq!(size_of::<crate::syntax::ast::Node>(), 36);
    assert_eq!(size_of::<crate::body::ir::Body>(), 456);
    Ok(())
}
