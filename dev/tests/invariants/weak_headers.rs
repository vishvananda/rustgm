//! Actual foreign-header completion rejects stale ordinary producer records.
use super::*;
use crate::{expand::config::Config, pipeline, support::diagnostics::LintCap, test_source};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn checked() -> Result<(Report, Declarations, Box<Catalog>)> {
    let input = test_source::fixture("pa31/tests/spec/weak-symbols/source-v2.rs")?;
    let mut config = Config::default();
    config.set_lint_cap(LintCap::Allow);
    let report = test_source::source_provider_config(&input, "weak_symbols", &config)?;
    let mut facts = pipeline::analyze_source(&report)?.facts;
    let catalog = facts
        .source_bindings
        .take()
        .ok_or("actual source catalog")?;
    Ok((report, facts, catalog))
}
#[test]
fn ordinary_foreign_completion_rejects_missing_registry_and_wrong_provider() -> Result {
    let (report, mut facts, catalog) = checked()?;
    for role in ROLES {
        assert_eq!(complete(&report, &facts, &catalog, role)?, Some(role));
    }
    let wrong =
        test_source::source_provider("#![feature(no_core)] #![no_core]", "another_provider")?;
    assert!(complete(&wrong, &facts, &catalog, LangItem::MemCpy).is_err());
    facts.foreign = None;
    assert!(complete(&report, &facts, &catalog, LangItem::MemCpy).is_err());
    Ok(())
}
#[test]
fn ordinary_foreign_completion_rejects_stale_owner_signature_and_safety() -> Result {
    let (mut report, mut facts, catalog) = checked()?;
    let declaration = catalog.language(LangItem::MemCpy).ok_or("actual memcpy")?;
    let definition = declaration.definition;
    let owner = report
        .resolution
        .foreign_owners
        .remove(&definition)
        .ok_or("actual foreign owner")?;
    assert!(complete(&report, &facts, &catalog, LangItem::MemCpy).is_err());
    report.resolution.foreign_owners.insert(definition, owner);
    let index = *facts
        .function_indices
        .get(&definition)
        .ok_or("actual function index")?;
    let previous = facts.functions[index].signature;
    facts.functions[index].signature = crate::types::TypeId::UNIT;
    assert!(complete(&report, &facts, &catalog, LangItem::MemCpy).is_err());
    facts.functions[index].signature = previous;
    let mut replacement = crate::sema::foreign::Registry::default();
    replacement.import_function(
        definition,
        previous,
        crate::syntax::item::Safety::Safe,
        "memcpy",
    )?;
    facts.foreign = Some(Box::new(replacement));
    assert!(complete(&report, &facts, &catalog, LangItem::MemCpy).is_err());
    Ok(())
}
