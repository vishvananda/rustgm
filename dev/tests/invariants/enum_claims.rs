//! Source declaration claims retain actual enum/member owners without execution.
use super::*;
use crate::resolve::model::{DefKind, Namespace};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const FOUNDATION: &str = r#"#![feature(no_core,lang_items,rustc_attrs)]#![no_core]
#[lang="pointee_sized"]trait PointeeSized{}
#[lang="meta_sized"]trait MetaSized:PointeeSized{}
#[lang="sized"]trait Sized:MetaSized{}"#;

fn report(tail: &str) -> TestResult<crate::resolve::expansion::Report> {
    Ok(crate::test_source::source_provider(
        &(FOUNDATION.to_owned() + tail),
        "enum_claims",
    )?)
}

#[test]
fn renamed_enum_variants_keep_actual_parent_identity_and_source_nodes() -> TestResult {
    for (source, diagnostic, variants) in [
        (
            r#"#[rustc_diagnostic_item="Result"]pub enum Outcome<T,E>{
                #[lang="Ok"]Good(T),#[lang="Err"]Bad(E)}"#,
            DiagnosticItem::Result,
            [LangItem::ResultOk, LangItem::ResultErr],
        ),
        (
            r#"#[rustc_diagnostic_item="Option"]#[lang="Option"]
                pub enum Maybe<T>{#[lang="Some"]Present(T),#[lang="None"]Absent}"#,
            DiagnosticItem::Option,
            [LangItem::OptionSome, LangItem::OptionNone],
        ),
    ] {
        let report = report(source)?;
        let reserved = report
            .resolution
            .source_bindings
            .as_deref()
            .ok_or("actual enum reservations")?;
        let owner = reserved.diagnostics[diagnostic.index()].ok_or("diagnostic owner")?;
        let parent = &report.resolution.definitions[owner.definition.0 as usize];
        assert_eq!(parent.kind, DefKind::Enum);
        assert_eq!(parent.node, Some(owner.origin));
        for role in variants {
            let variant = reserved.language(role).ok_or("actual variant role")?;
            let child = &report.resolution.definitions[variant.definition.0 as usize];
            assert_eq!(child.kind, DefKind::Variant);
            assert_eq!(child.namespace, Namespace::Value);
            assert_eq!(parent.members, Some(child.scope));
            assert_eq!(child.node, Some(variant.origin));
        }
    }
    report(
        r#"#[lang="Ordering"]enum Order{Less,Equal,Greater}
        #[rustc_diagnostic_item="Alignment"]enum Align{Left,Right,Center}"#,
    )?;
    Ok(())
}

#[test]
fn option_language_owner_does_not_require_a_diagnostic_marker() -> TestResult {
    for diagnostic in ["", "#[rustc_diagnostic_item=\"Option\"]enum Hint{Value}"] {
        let report = report(&format!(
            "{diagnostic}#[lang=\"Option\"]enum Maybe<T>{{#[lang=\"None\"]Absent,#[lang=\"Some\"]Present(T)}}"
        ))?;
        let reserved = report
            .resolution
            .source_bindings
            .as_deref()
            .ok_or("claims")?;
        let owner = reserved
            .language(LangItem::Option)
            .ok_or("language owner")?;
        for role in [LangItem::OptionNone, LangItem::OptionSome] {
            let variant = reserved.language(role).ok_or("variant")?;
            assert_eq!(
                report.resolution.definitions[owner.definition.0 as usize].members,
                Some(report.resolution.definitions[variant.definition.0 as usize].scope)
            );
        }
    }
    assert!(
        report("#[lang=\"Option\"]enum Actual<T>{Value(T)}enum Other{#[lang=\"None\"]Absent}")
            .is_err()
    );
    Ok(())
}

#[test]
fn wrong_enum_kinds_or_detached_variants_cannot_acquire_claims() {
    for source in [
        r#"#[rustc_diagnostic_item="Result"]trait Wrong{}"#,
        r#"#[rustc_diagnostic_item="Result"]struct Wrong;"#,
        r#"#[lang="Ok"]fn wrong(){}"#,
        r#"enum Orphan{#[lang="Ok"]Good}"#,
        r#"#[rustc_diagnostic_item="Result"]enum Actual{Good}
            enum Other{#[lang="Err"]Bad}"#,
        r#"#[rustc_diagnostic_item="Option"]struct Wrong;"#,
        r#"#[lang="Option"]trait Wrong{}"#,
        r#"#[lang="Some"]fn wrong(){}"#,
        r#"enum Orphan{#[lang="None"]Absent}"#,
        r#"#[rustc_diagnostic_item="Option"]enum Actual{Absent}
            enum Other{#[lang="Some"]Present}"#,
    ] {
        assert!(report(source).is_err(), "{source}");
    }
}

#[test]
fn diagnostic_methods_require_their_actual_declared_trait_owner() -> TestResult {
    report(
        r#"#[rustc_diagnostic_item="Formatter"]struct Formatter;
        #[rustc_diagnostic_item="Display"]trait Display{fn fmt(&self);}
        #[rustc_diagnostic_item="FmtWrite"]trait Write{fn write_str(&mut self,s:&str);}
        #[rustc_diagnostic_item="Default"]trait Default{
            #[rustc_diagnostic_item="default_fn"]fn default()->Self;}
        #[lang="eq"]#[rustc_diagnostic_item="PartialEq"]trait PartialEq{
            #[rustc_diagnostic_item="cmp_partialeq_eq"]fn eq(&self,other:&Self)->bool;}
        #[lang="partial_ord"]#[rustc_diagnostic_item="PartialOrd"]trait PartialOrd{}"#,
    )?;
    for source in [
        r#"#[rustc_diagnostic_item="Formatter"]trait Wrong{}"#,
        r#"#[rustc_diagnostic_item="Display"]struct Wrong;"#,
        r#"#[rustc_diagnostic_item="FmtWrite"]fn wrong(){}"#,
        r#"#[rustc_diagnostic_item="default_fn"]fn wrong(){}"#,
        r#"#[rustc_diagnostic_item="Default"]trait Default{}
            trait Other{#[rustc_diagnostic_item="default_fn"]fn wrong();}"#,
        r#"#[rustc_diagnostic_item="PartialEq"]trait MissingActualLanguage{}"#,
        r#"#[rustc_diagnostic_item="PartialOrd"]trait MissingActualLanguage{}"#,
    ] {
        assert!(report(source).is_err(), "{source}");
    }
    Ok(())
}
