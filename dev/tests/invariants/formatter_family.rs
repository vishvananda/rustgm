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
        let report =
            expanded_provider(&source, name).map_err(|error| format!("{name}: {error}"))?;
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
        "#[derive(Clone)]struct NoCopy;",
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
