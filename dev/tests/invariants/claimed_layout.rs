//! The actual allocation nominal claim supplies identity, never allocation proof.
use crate::{resolve::expansion::Report, support::bindings::LangItem};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn source(name: &str) -> Result<String> {
    Ok(crate::test_source::fixture(&format!(
        "pa31/tests/spec/alloc-layout-identity/{name}.rs"
    ))?)
}
fn report(name: &str) -> Result<Report> {
    Ok(crate::test_source::source_provider(
        &source(name)?,
        "alloc_layout_claim",
    )?)
}
#[test]
fn actual_struct_targets_retain_unconstrained_generics_and_pending_semantic_permission() -> Result {
    assert_eq!(LangItem::ALL.get(61), Some(&LangItem::AllocLayout));
    assert_eq!(LangItem::SliceLen.index(), 60);
    assert_eq!(LangItem::AllocLayout.index(), 61);
    assert_eq!(
        LangItem::from_name("alloc_layout"),
        Some(LangItem::AllocLayout)
    );
    for name in ["struct", "generic"] {
        let report = report(name)?;
        let claimed = report
            .resolution
            .source_bindings
            .as_deref()
            .ok_or("actual reservations")?
            .language(LangItem::AllocLayout)
            .ok_or("actual struct role")?;
        let actual = report.package.unit(claimed.origin.file);
        assert!(actual.configured.active[claimed.origin.node.get() as usize]);
        let facts = crate::sema::declarations::analyze(&report)?;
        let catalog = facts.source_bindings.as_deref().ok_or("actual catalog")?;
        assert_eq!(catalog.language(LangItem::AllocLayout), Some(claimed));
        assert!(catalog.validated_language(LangItem::AllocLayout).is_err());
    }
    let report = report("ordinary")?;
    assert!(
        report
            .resolution
            .source_bindings
            .as_deref()
            .ok_or("source")?
            .language(LangItem::AllocLayout)
            .is_none()
    );
    eprintln!(
        "allocation claim Reservations={} Catalog={} Declaration={} OptionDeclaration={} Body={} Rvalue={} SourceCore={}",
        size_of::<crate::resolve::bindings::Reservations>(),
        size_of::<crate::sema::compiler_items::Catalog>(),
        size_of::<crate::resolve::bindings::Declaration>(),
        size_of::<Option<crate::resolve::bindings::Declaration>>(),
        size_of::<crate::body::ir::Body>(),
        size_of::<crate::body::ir::Rvalue>(),
        size_of::<crate::resolve::bindings::SourceCore>()
    );
    Ok(())
}
#[test]
fn wrong_targets_duplicates_and_unrequested_source_claims_fail_before_publication() -> Result {
    for name in ["enum", "trait", "function", "alias", "duplicate"] {
        assert!(report(name).is_err(), "{name}");
    }
    let missing = source("struct")?.replace("no_core,lang_items", "no_core");
    let error = crate::test_source::source_provider(&missing, "alloc_layout_missing")
        .err()
        .ok_or("unrequested lang_items accepted")?;
    assert!(error.contains("lang requires"), "{error}");
    assert!(crate::test_source::resolve(&source("struct")?, true).is_err());
    Ok(())
}
