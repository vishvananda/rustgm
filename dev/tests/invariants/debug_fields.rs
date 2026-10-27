//! Source declarations expand before their real formatter bodies are demanded.
use super::*;
const DECLARATIONS: &str = "pa31/tests/spec/source-location/panic-debug-declarations.rs";
fn source() -> TestResult<String> {
    Ok(test_source::fixture(PROVIDER)? + &test_source::fixture(DECLARATIONS)?)
}

#[test]
fn panic_info_debug_and_argument_enum_keep_real_fields_and_once_parsed_receipts() -> TestResult {
    let source = source()?;
    let mut report = expanded_provider(&source, "panic_debug")?;
    assert!(!report.names_resolved);
    let transactions = report
        .attribute_effects
        .as_deref()
        .and_then(|effects| effects.derives.as_deref())
        .ok_or("derive transactions")?;
    assert_eq!(transactions.entries.len(), 3);
    for entry in &transactions.entries {
        report.expanded_derive(entry.owner, entry.effect)?;
        assert_eq!(report.invocations[entry.invocation].parser_calls, 1);
    }
    let unit = &report.package.units[0];
    let generated = &unit.source.text()[source.len()..];
    assert!(generated.contains("debug_struct_field4_finish"));
    assert!(generated.contains("&&self. force_no_backtrace"));
    for field in ["message", "location", "can_unwind", "force_no_backtrace"] {
        assert!(generated.contains(&format!("\"{field}\"")), "{field}");
    }
    assert!(
        unit.configured.ast.function_types.len() >= 2,
        "the actual enum formatter signature must survive opaque type copying"
    );
    let entry = &transactions.entries[2];
    let (owner, effect) = (entry.owner, entry.effect);
    let member = entry.contexts.start;
    let definition = report.invocations[member].definition;
    let Body::Builtin(original) = report.definitions[definition].body else {
        return Err("actual Debug source member".into());
    };
    report.definitions[definition].body = Body::Builtin(Builtin::Copy);
    assert!(report.expanded_derive(owner, effect).is_err());
    report.definitions[definition].body = Body::Builtin(original);
    report.expanded_derive(owner, effect)?;
    eprintln!(
        "debug Reservations={} Effects={} Transaction={} Report={} Body={} Definition={}",
        size_of::<crate::resolve::bindings::Reservations>(),
        size_of::<super::super::super::attribute_effects::Effects>(),
        size_of::<Transaction>(),
        size_of::<super::super::super::Report>(),
        size_of::<super::super::super::Body>(),
        size_of::<super::super::super::Definition>()
    );
    Ok(())
}

#[test]
fn debug_failed_member_preserves_list_and_unrelated_traits_can_retain_the_marker() -> TestResult {
    for members in ["Debug, Missing", "Debug, Debug"] {
        let source = source()?.replace("#[derive(Debug)]", &format!("#[derive({members})]"));
        super::super::super::role_declarations::with_engine(&source, true, |engine| {
            assert!(engine.run().is_err(), "{members}");
            let unit = &engine.loader.package.units[0];
            let effect = unit
                .configured
                .effects
                .iter()
                .find(|effect| {
                    let span = effect.attribute.attribute.span;
                    unit.source
                        .text()
                        .get(span.start as usize..span.end as usize)
                        .is_some_and(|text| text.contains(members))
                })
                .ok_or("actual failed derive list")?;
            assert_eq!(effect.attribute.effect, Effect::Keep);
            Ok(())
        })?;
    }
    let prefix = "#![feature(no_core,lang_items,rustc_attrs)]#![no_core]";
    let markers =
        "#[rustc_trivial_field_reads]trait First{}#[rustc_trivial_field_reads]trait Second{}";
    test_source::source_provider(&format!("{prefix}{markers}"), "separate_markers")?;
    for declaration in [
        "#[rustc_trivial_field_reads]#[rustc_trivial_field_reads]trait D{}",
        "#[rustc_trivial_field_reads(x)]trait D{}",
        "#[rustc_trivial_field_reads]fn d(){}",
        "#[rustc_diagnostic_item=\"Debug\"]struct D;",
    ] {
        assert!(
            test_source::source_provider(&format!("{prefix}{declaration}"), "bad_debug").is_err(),
            "{declaration}"
        );
    }
    Ok(())
}
