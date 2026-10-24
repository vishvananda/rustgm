//! Ordinary Clone headers remain ordinary at actual const call sites.
use super::*;

#[test]
fn ordinary_clone_role_retains_actual_nonconst_owner_and_required_method() -> Result {
    let source = crate::test_source::fixture(
        "pa31/tests/spec/source-functions-transport/minimal-provider.rs",
    )?;
    let report = crate::test_source::source_provider(&source, "ordinary_clone_role")?;
    let facts = crate::sema::declarations::analyze(&report)?;
    let catalog = facts.source_bindings.as_deref().ok_or("catalog")?;
    let declaration = catalog.header_role(LangItem::Clone)?.ok_or("Clone")?;
    let header = Trait::source(&report, &facts, declaration)?;
    assert!(!header.flags.constant);
    header.profile(
        &facts.types,
        Flags {
            constant: false,
            ..flags()
        },
        0,
    )?;
    let sized = catalog.header_role(LangItem::Sized)?.ok_or("Sized")?;
    assert!(header.requires(
        &facts.types,
        sized.definition,
        Some(sized.definition),
        Constness::Ordinary
    ));
    assert!(!header.requires(&facts.types, declaration.definition, None, Constness::Const));
    let method = catalog.header_role(LangItem::CloneFn)?.ok_or("clone_fn")?;
    assert_eq!(method.parent, Some(declaration.definition));
    assert!(!facts.is_const_function(method.definition));
    assert!(
        facts
            .function_header(method.definition)?
            .ok_or("method")?
            .body
            .is_none()
    );
    Ok(())
}

#[test]
fn ordinary_clone_const_call_rejects_at_actual_use_after_role_admission() -> Result {
    let source = crate::test_source::fixture(
        "pa31/tests/spec/source-functions-transport/ordinary-clone-concrete-const-use.rs",
    )?;
    let base = crate::test_source::fixture(
        "pa31/tests/spec/source-functions-transport/ordinary-clone-concrete-provider.rs",
    )?;
    assert!(source.starts_with(&base));
    let report = crate::test_source::source_provider(&source, "ordinary_clone_const_use")?;
    let result = checked_roots(&report);
    let error = result
        .err()
        .ok_or("ordinary Clone call gained const eligibility")?;
    assert_eq!(error.file.source(), Some(crate::resolve::unit::FileId(0)));
    assert!(error.span.start as usize >= base.len(), "{error:?}");
    assert!(error.message.contains("const"), "{error:?}");
    assert!(
        !error.message.contains("pinned provider profile"),
        "{error:?}"
    );
    Ok(())
}

#[test]
fn ordinary_clone_complete_concrete_provider_reaches_checked_roots() -> Result {
    let source = crate::test_source::fixture(
        "pa31/tests/spec/source-functions-transport/ordinary-clone-concrete-provider.rs",
    )?;
    let report = crate::test_source::source_provider(&source, "ordinary_clone_freeze")?;
    let program = checked_roots(&report)?;
    assert!(
        program
            .symbols
            .iter()
            .filter(|symbol| symbol.body.is_some())
            .count()
            >= 3
    );
    Ok(())
}

fn checked_roots(
    report: &Report,
) -> std::result::Result<crate::lowir::program::Program, crate::resolve::package::Error> {
    let analysis = crate::pipeline::analyze_source(report)?;
    let mut facts = analysis.facts;
    let definitions: Vec<_> = facts
        .functions
        .iter()
        .filter(|function| function.body.is_some())
        .map(|function| function.definition)
        .collect();
    crate::lower::instances::build_roots_prepared(
        report,
        &mut facts,
        crate::lower::Roots {
            entry: None,
            exports: &definitions,
        },
        analysis.prepared,
    )
}
