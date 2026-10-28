//! Stable cold words retain independent header data without changing const rules.
use crate::test_source;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn fixture(name: &str) -> Result<String> {
    Ok(test_source::fixture(&format!(
        "pa26/tests/spec/attributes/cold/{name}.rs"
    ))?)
}
#[test]
fn cold_words_preserve_inline_and_const_semantics_with_explicit_transport_debt() -> Result {
    let report = test_source::resolve(&fixture("valid")?, true)?;
    let facts = crate::pipeline::analyze_source(&report)?.facts;
    let (id, _) = report
        .resolution
        .definitions
        .iter()
        .enumerate()
        .find(|(_, row)| report.package.names.get(row.name) == "ANSWER")
        .ok_or("actual const ANSWER")?;
    assert!(
        crate::sema::values::Reader::new(&facts)
            .definition(crate::resolve::model::DefId(u32::try_from(id)?))
            .is_some()
    );
    for name in ["malformed", "unsafe-wrapper", "nonconst"] {
        let result: Result<crate::pipeline::Analysis> = test_source::resolve(&fixture(name)?, true)
            .map_err(Into::into)
            .and_then(|report| crate::pipeline::analyze_source(&report).map_err(Into::into));
        assert!(result.is_err(), "{name}");
    }
    for name in ["required-prototype", "required-foreign"] {
        let report = test_source::resolve(&fixture(name)?, true)?;
        let error = crate::pipeline::analyze_source(&report)
            .err()
            .ok_or("wider cold target silently accepted")?;
        assert!(
            error
                .to_string()
                .contains("cold function-body profile is not implemented"),
            "{error}"
        );
    }
    let source =
        test_source::fixture("pa31/tests/spec/source-functions-transport/minimal-provider.rs")?
            .replace(
                "pub fn identity<T>",
                "#[cold]#[inline(never)]pub fn identity<T>",
            );
    let report = test_source::source_provider(&source, "cold_source")?;
    let facts = crate::pipeline::analyze_source(&report)?.facts;
    let (index, _) = report
        .resolution
        .definitions
        .iter()
        .enumerate()
        .find(|(_, row)| report.package.names.get(row.name) == "identity")
        .ok_or("actual source identity")?;
    let hint = facts
        .inline_hint_for_definition(&report, crate::resolve::model::DefId(u32::try_from(index)?))?
        .ok_or("independent inline hint")?;
    assert_eq!(hint.hint, super::Hint::Never);
    for profile in [
        crate::lower::instances::SourcePublication::LegacyFunctions,
        crate::lower::instances::SourcePublication::FunctionContracts,
    ] {
        let error = crate::module_packets::exported_profile(&source, profile, "cold_publication")
            .err()
            .ok_or("cold data dropped")?;
        assert!(
            error
                .to_string()
                .contains("cold advisory marker transport is not implemented"),
            "{error}"
        );
    }
    Ok(())
}
