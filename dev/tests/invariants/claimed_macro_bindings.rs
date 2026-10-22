//! Source builtin permission belongs to configured declarations in both name paths.
use crate::{
    expand::{builtin::Builtin, config::Config},
    resolve::{
        self,
        bindings::Context,
        expansion::Body,
        load::{InputFile, Sources},
        model::DefKind,
    },
    support::{bindings::Mode, crates::CrateIdentity, source::SourceFile},
};
use std::path::Path;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const HEADER: &str = "#![feature(no_core,lang_items,rustc_attrs)]#![no_core]#[lang=\"pointee_sized\"]trait PointeeSized{}#[lang=\"meta_sized\"]trait MetaSized:PointeeSized{}#[lang=\"sized\"]trait Sized:MetaSized{}";
struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _: &Path) -> std::result::Result<Option<InputFile>, String> {
        Ok(None)
    }
}
fn context() -> Result<Context> {
    let identity = CrateIdentity::new("minicore", "builtin_v1")?;
    Ok(Context {
        crate_name: Some("minicore".into()),
        current: Some(identity.clone()),
        mode: Mode::Source { provider: identity },
        ..Default::default()
    })
}
fn input(source: &str) -> Result<InputFile> {
    Ok(InputFile {
        identity: "source.rs".into(),
        path: "source.rs".into(),
        source: SourceFile::new(source.as_bytes().to_vec())?,
    })
}
fn names(source: &str) -> Result<resolve::model::Resolution> {
    let mut package = resolve::load::load_with_context(
        input(source)?,
        &mut NoFiles,
        &Config::default(),
        &context()?,
    )?;
    Ok(resolve::analyze(&mut package)?)
}
fn expanded(source: &str) -> Result<resolve::expansion::Report> {
    Ok(resolve::analyze_expanded(
        resolve::expansion::analyze_with_context(
            input(source)?,
            &mut NoFiles,
            &Config::default(),
            &context()?,
        )?,
    )?)
}
#[test]
fn source_builtin_definitions_retain_real_identity_and_normal_shadowing() -> Result {
    for attribute in [
        "#[rustc_builtin_macro(line)]",
        "#[rustc_builtin_macro(line,)]",
        "#[rustc_builtin_macro(r#line,attributes())]",
        "#[cfg_attr(all(),cfg_attr(all(),rustc_builtin_macro(line)))]",
    ] {
        let source = format!(
            "{HEADER}{attribute}macro_rules!location{{(unused)=>{{99u32}}}}const N:u32=location!();"
        );
        let facts = names(&source)?;
        assert!(
            facts
                .definitions
                .iter()
                .any(|d| d.kind == DefKind::Macro && d.node.is_some())
        );
        let report = expanded(&source)?;
        assert!(report.resolution.macro_prelude.is_empty());
        let invocation = report.invocations.first().ok_or("invocation")?;
        let definition = &report.definitions[invocation.definition];
        assert!(matches!(definition.body, Body::Builtin(Builtin::Line)));
        let node = definition.node.ok_or("source macro node")?;
        assert!(
            report
                .resolution
                .definitions
                .iter()
                .any(|d| d.kind == DefKind::Macro && d.node == Some(node))
        );
    }
    let source = format!(
        "{HEADER}#[rustc_builtin_macro]macro_rules!line{{()=>{{99u32}}}}mod inner{{macro_rules!line{{()=>{{7u32}}}}const N:u32=line!();}}const N:u32=line!();"
    );
    let report = expanded(&source)?;
    assert_eq!(report.invocations.len(), 2);
    assert!(matches!(
        report.definitions[report.invocations[0].definition].body,
        Body::Declarative(_)
    ));
    assert!(matches!(
        report.definitions[report.invocations[1].definition].body,
        Body::Builtin(Builtin::Line)
    ));
    Ok(())
}
#[test]
fn source_builtin_claims_fail_closed_in_both_pipelines() {
    for tail in [
        "#[rustc_builtin_macro]fn line(){}",
        "#[rustc_builtin_macro]struct line;",
        "#[rustc_builtin_macro()]macro_rules!line{()=>{99u32}}",
        "#[rustc_builtin_macro(\"line\")]macro_rules!line{()=>{99u32}}",
        "#[rustc_builtin_macro(core::line)]macro_rules!line{()=>{99u32}}",
        "#[rustc_builtin_macro(line,attributes(a))]macro_rules!line{()=>{99u32}}",
        "#[rustc_builtin_macro(line,attributes(),other)]macro_rules!line{()=>{99u32}}",
        "#[rustc_builtin_macro]#[rustc_builtin_macro]macro_rules!line{()=>{99u32}}",
        "#[rustc_builtin_macro(unknown_builtin)]macro_rules!line{()=>{99u32}}",
        "#[rustc_builtin_macro]macro_rules!asm{()=>{99u32}}",
    ] {
        let source = format!("{HEADER}{tail}");
        assert!(names(&source).is_err(), "names: {tail}");
        assert!(expanded(&source).is_err(), "expanded: {tail}");
    }
    let source = format!("{HEADER}#[rustc_builtin_macro]macro_rules!line{{()=>{{99u32}}}}")
        .replace(",rustc_attrs", "");
    assert!(names(&source).is_err());
    assert!(expanded(&source).is_err());
    assert!(
        crate::test_source::resolve("#[rustc_builtin_macro]macro_rules!line{()=>{99u32}}", true)
            .is_err()
    );
}
#[test]
fn disabled_generated_and_stub_contracts_keep_their_own_admission() -> Result {
    let disabled = format!(
        "{HEADER}#[cfg(any())]#[rustc_builtin_macro(unknown)]fn wrong(){{}}#[cfg_attr(any(),rustc_builtin_macro)]macro_rules!line{{()=>{{99u32}}}}const N:u32=line!();"
    );
    names(&disabled)?;
    let report = expanded(&disabled)?;
    assert!(matches!(
        report.definitions[report.invocations[0].definition].body,
        Body::Declarative(_)
    ));
    for body in ["", "nonsense", "($a:invalid)=>{99u32}"] {
        assert!(
            expanded(&format!(
                "{HEADER}#[rustc_builtin_macro]macro_rules!line{{{body}}}"
            ))
            .is_err()
        );
    }
    let source = format!(
        "{HEADER}macro_rules!make{{()=>{{#[rustc_builtin_macro(line)]macro_rules!location{{()=>{{99u32}}}}}}}}make!();const N:u32=location!();"
    );
    let report = expanded(&source)?;
    assert!(matches!(
        report.definitions[report.invocations[1].definition].body,
        Body::Builtin(Builtin::Line)
    ));
    Ok(())
}
