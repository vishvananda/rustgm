//! Source auto syntax remains typed and cannot turn an ordinary trait into Freeze.
use super::{TestResult, destructibility::BASE, report};
use rustgm::{sema, support::bindings::LangItem};
const FREEZE: &str = r#"#[lang="freeze"]pub unsafe auto trait Freeze{}"#;
fn prefix() -> String {
    BASE.replace(
        "const_trait_impl)]",
        "const_trait_impl,auto_traits,negative_impls,freeze_impls)]",
    )
}
#[test]
fn actual_source_freeze_header_retains_auto_identity_and_weak_identifier() -> TestResult {
    for declaration in [
        FREEZE,
        r#"macro_rules! declare{()=>{#[lang="freeze"]pub unsafe auto trait Freeze{}}}declare!();#[cfg(any())]#[lang="freeze"]pub unsafe auto trait Disabled{}"#,
    ] {
        let source = [prefix(), declaration.into(), "fn auto()->u8{7}".into()].concat();
        let report = report(&source)?;
        let facts = sema::declarations::analyze(&report)?;
        let actual = report
            .resolution
            .source_bindings
            .as_deref()
            .ok_or("reservations")?
            .language(LangItem::Freeze)
            .ok_or("Freeze")?;
        let owner = facts
            .associated
            .owner(actual.definition)
            .and_then(|id| facts.associated.get(id))
            .ok_or("owner")?;
        assert!(matches!(owner.kind, sema::associated::OwnerKind::Trait {
            definition, unsafe_: true, auto_: true, constant: false
        } if definition == actual.definition));
    }
    Ok(())
}
#[test]
fn source_auto_header_profiles_and_feature_permissions_are_distinct() -> TestResult {
    for declaration in [
        r#"#[lang="freeze"]pub unsafe trait Freeze{}"#,
        r#"#[lang="freeze"]pub auto trait Freeze{}"#,
        r"pub unsafe auto trait Freeze{}",
        r#"#[lang="freeze"]pub unsafe auto trait Freeze{fn f();}"#,
    ] {
        let source = prefix() + declaration;
        let report = report(&source)?;
        assert!(
            sema::declarations::analyze(&report).is_err(),
            "{declaration}"
        );
    }
    let missing_feature = BASE.to_owned() + FREEZE;
    assert!(report(&missing_feature).is_err());
    let source = [
        prefix().replace(",freeze_impls", ""),
        FREEZE.into(),
        "struct N;unsafe impl Freeze for N{}".into(),
    ]
    .concat();
    let report = report(&source)?;
    let error = sema::declarations::analyze(&report)
        .err()
        .ok_or("missing freeze_impls accepted")?
        .to_string();
    assert!(error.contains("freeze_impls"), "{error}");
    Ok(())
}

#[test]
fn source_freeze_original_cases_retain_header_selection_and_body_proofs() -> TestResult {
    let mut failures = Vec::new();
    for &(name, source, accepted, _) in super::freeze_cases::CASES {
        let source = super::fixture(source)?;
        let result = super::lower(&source);
        if result.is_ok() != accepted {
            failures.push(format!("{name}: {:?}", result.err()));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

pub(super) const AUTOMATIC_CLEANUP: &str = "automatic-cleanup.rs";
#[test]
fn actual_freeze_and_glue_original_cleanup_passes_checked_lowering() -> TestResult {
    super::lower(&super::fixture(AUTOMATIC_CLEANUP)?)?;
    Ok(())
}
