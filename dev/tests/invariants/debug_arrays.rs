//! Real source Debug arrays retain actual field order, coercions and paid output.
use super::*;
use crate::syntax::ast::{Kind, TypeFlavor};
fn prefix() -> TestResult<String> {
    let declarations =
        test_source::fixture("pa31/tests/spec/source-location/panic-debug-declarations.rs")?;
    let (debug, _) = declarations
        .split_once("    #[derive(Copy, Clone)]")
        .ok_or("source Debug declarations")?;
    let foundation = test_source::fixture(PROVIDER)?;
    let (header, _) = foundation
        .split_once("pub mod location")
        .ok_or("source foundation declarations")?;
    Ok(header.to_owned() + debug + "}use crate::fmt::Debug;")
}
fn fixture(name: &str) -> TestResult<String> {
    Ok(test_source::fixture(&format!(
        "pa31/tests/spec/debug-field-arrays/{name}.rs"
    ))?)
}
fn compact_generated(report: &super::super::super::Report, input: usize) -> String {
    report.package.units[0].source.text()[input..]
        .split_whitespace()
        .collect()
}
fn receipts(report: &super::super::super::Report) -> TestResult {
    let entries = &report
        .attribute_effects
        .as_deref()
        .and_then(|e| e.derives.as_deref())
        .ok_or("actual derives")?
        .entries;
    for entry in entries {
        report.expanded_derive(entry.owner, entry.effect)?;
        assert_eq!(report.invocations[entry.invocation].parser_calls, 1);
    }
    Ok(())
}
#[test]
fn original_function_pointer_and_five_six_seven_field_recipes_keep_real_arrays() -> TestResult {
    for shape in ["record", "tuple", "enum"] {
        for count in [5, 6, 7] {
            let input = prefix()? + &fixture(&format!("{shape}-{count}"))?;
            let report = expanded_provider(&input, "debug_cutoff")?;
            receipts(&report)?;
            let generated = compact_generated(&report, input.len());
            if count == 5 {
                assert!(generated.contains("field5_finish"));
                assert!(!generated.contains("letvalues:"));
            } else {
                assert!(generated.contains("letvalues:&[&dyncrate::fmt::Debug]=&["));
                assert!(
                    generated.contains("debug_struct_fields_finish")
                        || generated.contains("debug_tuple_fields_finish")
                );
                assert!(!generated.contains(&format!("field{count}_finish")));
                let ast = &report.package.units[0].configured.ast;
                assert!(ast.nodes.iter().any(|n| n.span.start as usize >= input.len()
                    && matches!(n.kind, Kind::Array(values) if ast.list(values).len() == count)));
                assert!(
                    ast.nodes
                        .iter()
                        .any(|n| n.span.start as usize >= input.len()
                            && matches!(n.kind, Kind::TraitType { opaque: false, .. }))
                );
            }
        }
    }
    let input = prefix()? + &fixture("upstream-fn-ptr")?;
    let report = expanded_provider(&input, "original_fn_ptr_debug")?;
    receipts(&report)?;
    let generated = compact_generated(&report, input.len());
    for field in [
        "unsafety",
        "abi",
        "inputs",
        "output",
        "variadic",
        "is_splatted",
        "splatted_index",
    ] {
        assert!(generated.contains(&format!("\"{field}\"")));
    }
    assert!(generated.contains("debug_struct_fields_finish(f,\"FnPtr\",names,values)"));
    assert!(generated.contains("&&self.splatted_index"));
    Ok(())
}
#[test]
fn unsized_final_indirection_labels_and_array_type_survive_source_provenance() -> TestResult {
    let tail_input = fixture("unsized-tail")?;
    let (tail, _) = tail_input
        .split_once("fn require_unsized")
        .ok_or("original authored DST declaration")?;
    let input = prefix()? + &fixture("source-payloads")? + &fixture("labels")? + tail;
    let mut report = expanded_provider(&input, "debug_labels")?;
    receipts(&report)?;
    let generated = compact_generated(&report, input.len());
    assert!(generated.contains("letnames:&'static_=&["));
    assert!(generated.contains("&&self.g"));
    assert!(generated.contains("&&self.tail"));
    assert!(generated.contains("&__derive_field_6"));
    assert!(generated.contains("debug_struct_fields_finish"));
    assert!(generated.contains("debug_tuple_fields_finish"));
    let unit = &report.package.units[0];
    let ast = &unit.configured.ast;
    let literals = ast
        .literals
        .iter()
        .map(|value| unit.literals.view(unit.source.text(), *value))
        .collect::<Result<Vec<_>, _>>()?;
    for name in [b"type".as_slice(), "café".as_bytes(), "δ".as_bytes()] {
        assert!(literals.iter().any(
            |value| matches!(value, crate::literal::Value::String(bytes) if bytes.as_ref() == name)
        ));
    }
    let Some(index) = ast.nodes.iter().find_map(|node| match node.kind {
        Kind::Let { ty: Some(ty), .. }
            if node.span.start as usize >= input.len()
                && matches!(
                    ast.node(ty).kind,
                    Kind::Type {
                        flavor: TypeFlavor::Ref,
                        ..
                    }
                ) =>
        {
            Some(ty)
        }
        _ => None,
    }) else {
        return Err("actual typed generated local".into());
    };
    assert!(matches!(ast.node(index).kind, Kind::Type { .. }));
    let entry = report
        .attribute_effects
        .as_deref()
        .and_then(|e| e.derives.as_deref())
        .ok_or("derive owner")?
        .entries
        .last()
        .ok_or("actual labels derive")?;
    let (owner, effect, member) = (entry.owner, entry.effect, entry.contexts.start);
    let definition = report.invocations[member].definition;
    let Body::Builtin(actual) = report.definitions[definition].body else {
        return Err("actual source Debug member".into());
    };
    report.definitions[definition].body = Body::Builtin(Builtin::Copy);
    assert!(report.expanded_derive(owner, effect).is_err());
    report.definitions[definition].body = Body::Builtin(actual);
    report.expanded_derive(owner, effect)?;
    Ok(())
}
#[test]
fn source_array_references_and_dynamic_fields_reach_checked_native_roots() -> TestResult {
    let input =
        test_source::fixture("pa31/tests/spec/debug-field-arrays-runtime/source-observer.rs")?;
    let report = test_source::source_provider(&input, "debug_array_calls")?;
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
        .join("../obj/parallel/debug-field-arrays-integration-v1/candidate/native");
    std::fs::create_dir_all(&out)?;
    std::fs::write(out.join("provider.rs"), input)?;
    crate::lowir::writer::write(
        &program,
        &mut std::fs::File::create(out.join("provider.lowir"))?,
    )?;
    Ok(())
}
#[test]
fn array_field_work_scales_and_warm_receipts_retain_no_new_storage() -> TestResult {
    use std::fmt::Write as _;
    let mut previous = None;
    let mut measured = Vec::new();
    for count in [8, 32, 128] {
        let mut input = prefix()?;
        input.push_str("#[derive(Debug)]struct Many{");
        for index in 0..count {
            write!(&mut input, "f{index}:u8,")?;
        }
        input.push('}');
        let report = expanded_provider(&input, "debug_array_growth")?;
        let payer = &report.package.source_core().ok_or("payer")?.macros;
        let before = payer.usage();
        receipts(&report)?;
        let after = payer.usage();
        assert!(after.0 > before.0);
        assert_eq!(after.1, before.1);
        if let Some((work, bytes)) = previous {
            assert!(after.0 <= work * 6, "fields={count}: {after:?}");
            assert!(after.1 <= bytes * 6, "fields={count}: {after:?}");
        }
        previous = Some((after.0, after.1));
        measured.push(after);
        eprintln!("Debug array fields={count}: {after:?}");
    }
    // Subtract the fixed source foundation so it cannot conceal quadratic work.
    assert!(measured[2].0 - measured[1].0 <= (measured[1].0 - measured[0].0) * 6);
    assert!(measured[2].1 - measured[1].1 <= (measured[1].1 - measured[0].1) * 6);
    let input =
        prefix()? + "#[derive(Debug,Missing)]struct Rejected{a:u8,b:u8,c:u8,d:u8,e:u8,f:u8}";
    super::super::super::role_declarations::with_engine(&input, true, |engine| {
        let original = engine.loader.package.units[0].source.text().len();
        assert!(engine.run().is_err());
        assert_eq!(engine.loader.package.units[0].source.text().len(), original);
        assert!(
            engine.loader.package.units[0]
                .configured
                .effects
                .iter()
                .all(|effect| effect.attribute.effect != Effect::ExpandedDerive)
        );
        Ok(())
    })?;
    assert_eq!(size_of::<crate::syntax::ast::Node>(), 36);
    assert_eq!(size_of::<crate::body::ir::Body>(), 456);
    Ok(())
}
