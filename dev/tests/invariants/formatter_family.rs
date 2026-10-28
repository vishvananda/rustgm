//! Exact formatter fragments exercise authenticated declarations and generated ASTs.
use super::*;
fn family_prefix() -> TestResult<String> {
    Ok(String::from(
        "#![feature(no_core,lang_items,rustc_attrs,decl_macro,staged_api,const_trait_impl,allow_internal_unstable)]#![no_core]#![unstable(feature=\"formatter_observer\",issue=\"none\")]",
    ) + &test_source::fixture("pa31/tests/spec/formatter-derives/source-macros.rs")?)
}
#[test]
fn exact_formatter_fragments_expand_all_members_once_with_actual_source_receipts() -> TestResult {
    for name in [
        "alignment",
        "sign",
        "debug-as-hex",
        "formatting-options",
        "error",
        "ordering",
    ] {
        let original =
            test_source::fixture(&format!("pa31/tests/spec/formatter-derives/{name}.rs"))?;
        let source = family_prefix()? + &original;
        let report = expanded_provider(&source, &name.replace('-', "_"))
            .map_err(|error| format!("{name}: {error}"))?;
        let entries = &report
            .attribute_effects
            .as_deref()
            .and_then(|effects| effects.derives.as_deref())
            .ok_or("actual formatter transactions")?
            .entries;
        assert_eq!(
            entries.len(),
            if name == "ordering" { 2 } else { 1 },
            "{name}"
        );
        for entry in entries {
            report.expanded_derive(entry.owner, entry.effect)?;
            assert_eq!(
                report.invocations[entry.invocation].parser_calls, 1,
                "{name}"
            );
        }
        for captured in &report.package.units[0].configured.ast.attributes {
            if captured
                .attribute
                .name(&captured.forest, report.package.units[0].source.text())
                == Some("coverage")
            {
                let name = captured
                    .forest
                    .tree(captured.attribute.path.start)
                    .ok_or("actual generated coverage name")?
                    .span;
                assert!(report.macro_allowances().allows_at(
                    FileId(0),
                    name,
                    &crate::resolve::bindings::FeatureName::new("coverage_attribute")?
                )?);
            }
        }
        let output = &report.package.units[0].source.text()[source.len()..];
        assert!(output.contains("crate::cmp::"), "{name}");
        if name == "ordering" {
            assert!(output.contains("const unsafe impl"));
            assert!(output.contains("crate::intrinsics::discriminant_value"));
            assert!(output.contains("crate::hash::Hash::hash"));
        }
    }
    eprintln!(
        "formatter Reservations={} Transaction={} MemberContext={} Value={} Rvalue={} Package={} Report={}",
        size_of::<crate::resolve::bindings::Reservations>(),
        size_of::<Transaction>(),
        size_of::<crate::expand::builtin::derive_output::MemberContext>(),
        size_of::<crate::body::ir::Value>(),
        size_of::<crate::body::ir::Rvalue>(),
        size_of::<crate::resolve::package::Package>(),
        size_of::<super::super::super::Report>()
    );
    Ok(())
}
#[test]
fn failed_family_members_and_forged_prior_copy_do_not_hide_obligations() -> TestResult {
    for declaration in [
        "#[derive(Copy,Eq,Missing)]struct Missing;",
        "#[derive(Copy,Eq,Eq)]struct Duplicate;",
        "#[derive(Default)]enum MissingDefault{First,Second}",
    ] {
        assert!(
            expanded_provider(&(family_prefix()? + declaration), "bad_family").is_err(),
            "{declaration}"
        );
    }
    let source = family_prefix()? + "#[derive(Copy)]#[derive_const(Clone)]struct Prior;";
    let report = expanded_provider(&source, "prior_copy")?;
    let entries = &report
        .attribute_effects
        .as_deref()
        .and_then(|effects| effects.derives.as_deref())
        .ok_or("actual prior transactions")?
        .entries;
    assert_eq!(entries.len(), 2);
    for entry in entries {
        report.expanded_derive(entry.owner, entry.effect)?;
    }
    super::super::super::role_declarations::with_engine(&source, true, |engine| {
        engine.run()?;
        let transactions = engine
            .attribute_effects
            .as_deref()
            .and_then(|effects| effects.derives.as_deref())
            .ok_or("actual completed transactions")?;
        let outer = transactions.entries[1].outer;
        assert!(engine.completed_copy(outer)?);
        engine
            .attribute_effects
            .as_deref_mut()
            .and_then(|effects| effects.derives.as_deref_mut())
            .ok_or("actual mutable receipt")?
            .entries[0]
            .contexts = 0..0;
        assert!(!engine.completed_copy(outer)?);
        Ok(())
    })?;
    forged_grants()?;
    Ok(())
}

fn forged_grants() -> TestResult {
    let source = family_prefix()?
        + &test_source::fixture("pa31/tests/spec/formatter-derives/error.rs")?
        + "#[coverage(off)]fn caller_owned(){}";
    let mut report = expanded_provider(&source, "forged_derive_grant")?;
    let unit = &report.package.units[0];
    let tokens: Vec<_> = unit
        .configured
        .ast
        .attributes
        .iter()
        .filter_map(|capture| {
            (capture.attribute.name(&capture.forest, unit.source.text()) == Some("coverage"))
                .then(|| {
                    capture
                        .forest
                        .tree(capture.attribute.path.start)
                        .map(|tree| tree.span)
                })
                .flatten()
        })
        .collect();
    let generated = *tokens
        .iter()
        .find(|span| span.start as usize >= source.len())
        .ok_or("actual generated coverage")?;
    let caller = *tokens
        .iter()
        .find(|span| (span.start as usize) < source.len())
        .ok_or("actual caller coverage")?;
    let feature = crate::resolve::bindings::FeatureName::new("coverage_attribute")?;
    assert!(
        report
            .macro_allowances()
            .allows_at(FileId(0), generated, &feature)?
    );
    assert!(
        !report
            .macro_allowances()
            .allows_at(FileId(0), caller, &feature)?
    );
    let member = report
        .invocations
        .iter()
        .position(|invocation| {
            matches!(
                report.definitions[invocation.definition].body,
                Body::Builtin(Builtin::Eq)
            )
        })
        .ok_or("actual Eq member")?;
    let old = report.invocations[member].parent;
    report.invocations[member].parent = ExpansionId(0);
    assert!(
        report
            .macro_allowances()
            .allows_at(FileId(0), generated, &feature)
            .is_err()
    );
    report.invocations[member].parent = old;
    let definition = report.invocations[member].definition;
    report.definitions[definition].body = Body::Builtin(Builtin::Copy);
    assert!(
        report
            .macro_allowances()
            .allows_at(FileId(0), generated, &feature)
            .is_err()
    );
    report.definitions[definition].body = Body::Builtin(Builtin::Eq);
    assert!(
        report
            .macro_allowances()
            .allows_at(FileId(0), generated, &feature)?
    );
    Ok(())
}

#[test]
fn original_generic_payload_declarations_keep_each_member_and_checked_transaction() -> TestResult {
    for name in ["result", "option", "control-flow"] {
        let declaration = test_source::fixture(&format!(
            "pa31/tests/spec/formatter-derives/generic-payload/{name}-source.rs"
        ))?;
        let source = family_prefix()? + &declaration;
        let report = expanded_provider(&source, "generic_payload")?;
        let entries = &report
            .attribute_effects
            .as_deref()
            .and_then(|owner| owner.derives.as_deref())
            .ok_or("actual generic transactions")?
            .entries;
        assert_eq!(entries.len(), 2, "{name}");
        for entry in entries {
            report.expanded_derive(entry.owner, entry.effect)?;
            assert_eq!(report.invocations[entry.invocation].parser_calls, 1);
        }
        let generated = &report.package.units[0].source.text()[source.len()..];
        assert!(generated.contains("match"), "{name}");
        assert!(
            generated.contains("crate::fmt::Formatter::")
                && generated.contains("debug_tuple_field1_finish"),
            "{name}"
        );
        if name == "result" {
            assert!(generated.contains("[const] crate::cmp::Ord"), "{name}");
        }
    }
    Ok(())
}
#[test]
fn generic_clone_source_calls_real_noncopy_fields_and_does_not_invent_copy_bounds() -> TestResult {
    let source =
        test_source::fixture("pa31/tests/spec/formatter-derives/generic-payload/clone-source.rs")?;
    let report = test_source::source_provider(&source, "generic_clone")?;
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
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/pa31/panic-handler-exit-v2/generic-clone-native");
    std::fs::create_dir_all(&folder)?;
    std::fs::write(folder.join("provider.rs"), &source)?;
    crate::lowir::writer::write(
        &program,
        &mut std::fs::File::create(folder.join("provider.lowir"))?,
    )?;
    let missing = source.replace(
        "impl Clone for Leaf { fn clone(&self)->Self { unsafe {cloned(self.0)}; Self(self.0) } }",
        "",
    );
    assert_ne!(missing, source);
    let report = test_source::source_provider(&missing, "missing_clone")?;
    assert!(crate::pipeline::analyze_source(&report).is_err());
    let moved = source
        .replace(
            "pub mod generic_user {",
            "pub mod generic_user {use crate::marker::Copy;",
        )
        .replace(
            "#[derive(Clone)] struct Wrapper<T>",
            "#[derive(Clone,Copy)] struct Wrapper<T>",
        )
        .replace(
            "let b=a.clone();",
            "let moved=a;let again=a;let b=again.clone();",
        );
    assert_ne!(moved, source);
    let report = test_source::source_provider(&moved, "missing_copy")?;
    let crate::pipeline::Analysis {
        mut facts,
        prepared,
    } = crate::pipeline::analyze_source(&report)?;
    // Moves and their actual conditional Copy goals are checked by ordinary
    // borrow/instance preparation, after successful declaration/body typing.
    assert!(
        crate::lower::instances::build_roots_prepared(
            &report,
            &mut facts,
            crate::lower::Roots {
                entry: None,
                exports: &[]
            },
            prepared,
        )
        .is_err()
    );
    Ok(())
}

#[test]
fn generic_variant_work_scales_with_actual_emitted_members_and_storage() -> TestResult {
    use std::fmt::Write as _;
    let mut prior = None;
    for count in [8, 32, 128] {
        let mut input = family_prefix()?;
        input.push_str("#[derive(Clone)]enum Group<T>{");
        for n in 0..count {
            write!(&mut input, "V{n}(T),")?;
        }
        input.push('}');
        let report = expanded_provider(&input, "generic_work")?;
        let source = report.package.source_core().ok_or("source macro budget")?;
        let usage = source.macros.usage();
        if let Some((work, bytes)) = prior {
            assert!(usage.0 <= work * 6);
            assert!(usage.1 <= bytes * 6);
        }
        prior = Some((usage.0, usage.1));
        eprintln!(
            "generic derive variants={count} usage={usage:?} layouts={:?}",
            crate::expand::builtin::derive_output::transient_layouts()
        );
    }
    for declaration in [
        "#[derive(Clone)]struct Bounded<T:Clone>(T);",
        "#[derive(Clone)]struct Array<T,const N:usize>([T;N]);",
        "#[derive(Clone)]struct Where<T>(T)where T:Clone;",
    ] {
        assert!(expanded_provider(&(family_prefix()? + declaration), "wider_generic").is_err());
    }
    // Closed paired Clone remains separate from the generic fieldwise recipe.
    expanded_provider(
        &(family_prefix()? + "#[derive(Clone)]struct WithoutCopy;"),
        "fieldwise_unit",
    )?;
    Ok(())
}
