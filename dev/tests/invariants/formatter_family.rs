//! Exact formatter fragments exercise authenticated declarations and generated ASTs.
use super::*;
fn family_prefix() -> String {
    let mut source = String::from(
        "#![feature(no_core,lang_items,rustc_attrs,decl_macro,staged_api)]#![no_core]#![unstable(feature=\"formatter_observer\",issue=\"none\")]",
    );
    for role in [Builtin::Derive, Builtin::DeriveConst]
        .into_iter()
        .chain(crate::expand::builtin::derive_output::MEMBERS)
    {
        source.push_str("#[rustc_builtin_macro]pub macro ");
        source.push_str(role.name());
        source.push_str("($item:item){}");
    }
    source
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
        let source = family_prefix() + &original;
        let report = expanded_provider(&source, name)?;
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
            expanded_provider(&(family_prefix() + declaration), "bad_family").is_err(),
            "{declaration}"
        );
    }
    let source = family_prefix() + "#[derive(Copy)]#[derive_const(Clone)]struct Prior;";
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
    Ok(())
}
