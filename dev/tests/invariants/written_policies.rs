//! Written policies are positively checked; ordinary provided methods stay overridable.
use super::*;

#[test]
fn plain_override_executes_but_written_modifiers_and_tracking_stay_closed() -> Result {
    let (_, artifact) = provider(
        "\npub trait Overridable{fn value(value:u8)->u8{value}}impl Overridable for u8{fn value(value:u8)->u8{value}}",
    )?;
    let bytes = crates::encode(&artifact)?;
    client(
        crates::decode(&bytes)?,
        "#![feature(no_core)]#![no_core]fn main(){let _=<u8 as base::Overridable>::value(7u8);}",
        |_, _, _| (),
    )?;
    let source = crate::test_source::fixture(
        "pa31/tests/spec/source-header-transport/semantic-provider-qualified.rs",
    )?;
    for (feature, extra, expected) in [
        (
            "final_associated_functions",
            "pub trait FinalTrait{final fn value(value:u8)->u8{value}}",
            "source final trait method contract transport is not implemented",
        ),
        (
            "specialization",
            "pub trait DefaultTrait{fn value(value:u8)->u8;}impl DefaultTrait for u8{default fn value(value:u8)->u8{value}}",
            "source defaultness contract transport is not implemented",
        ),
        (
            "no_core",
            "#[track_caller]pub fn tracked(value:u8)->u8{value}",
            "source caller tracking policy transport is not implemented",
        ),
    ] {
        let input = format!("#![feature({feature})]\n{source}\n{extra}");
        let Err(error) = module_packets::exported_profile(
            &input,
            SourcePublication::ProvidedContracts,
            "provided_policy_opponent",
        ) else {
            return Err("unsupported written policy published".into());
        };
        assert!(error.to_string().contains(expected), "{error}");
    }
    Ok(())
}

#[test]
fn decoded_advisory_columns_preserve_present_absent_and_optional_reason() -> Result {
    let (_, artifact) = provider(
        "\n#[must_use=\"retain returned value\"]pub fn marked<T>(value:T)->T{value}pub fn unmarked<T>(value:T)->T{value}",
    )?;
    let bytes = crates::encode(&artifact)?;
    let decoded = crates::decode(&bytes)?;
    client(
        decoded,
        "#![feature(no_core)]#![no_core]fn main(){let _=base::unmarked(7u8);}",
        |report, facts, _| -> Result {
            let imported = report
                .resolution
                .crates
                .as_deref()
                .ok_or("actual imported catalog")?;
            let mut names = 0;
            for definition in imported.generic_definitions() {
                let definition = definition?;
                let actual = &report.resolution.definitions[definition.0 as usize];
                let name = report.package.names.get(actual.name);
                if !matches!(name, "marked" | "unmarked") {
                    continue;
                }
                let contract = facts.must_use_contract(report, definition)?;
                if name == "marked" {
                    let contract = contract.ok_or("transported actual must_use")?;
                    assert_eq!(contract.reason, Some("retain returned value"));
                    assert!(contract.origin.imported().is_some());
                } else {
                    assert!(contract.is_none());
                }
                names += 1;
            }
            assert_eq!(names, 2);
            Ok(())
        },
    )??;
    Ok(())
}
