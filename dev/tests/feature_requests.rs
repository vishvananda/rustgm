//! Source gate declarations remain distinct from configured semantic uses.
use rustgm::{
    expand::config::Config,
    resolve::{
        self,
        bindings::Context,
        expansion,
        load::{self, InputFile, Sources},
        package::Package,
    },
    support::{bindings::Mode, crates::CrateIdentity, source::SourceFile},
};
use std::path::Path;

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const HEADER: &str = "#![feature(no_core,lang_items,rustc_attrs)]#![no_core]#[lang=\"pointee_sized\"]trait PointeeSized{}#[lang=\"meta_sized\"]trait MetaSized:PointeeSized{}#[lang=\"sized\"]trait Sized:MetaSized{}";

#[derive(Default)]
struct Files<'a>(&'a [(&'a str, &'a str)]);
impl Sources for Files<'_> {
    fn read(&mut self, path: &Path) -> std::result::Result<Option<InputFile>, String> {
        self.0
            .iter()
            .find(|(name, _)| path == Path::new(name))
            .map(|(_, text)| input(path, text))
            .transpose()
    }
}
fn input(path: &Path, source: &str) -> std::result::Result<InputFile, String> {
    Ok(InputFile {
        identity: path.into(),
        path: path.into(),
        source: SourceFile::new(source.as_bytes().to_vec())?,
    })
}
fn context() -> Result<Context> {
    let identity = CrateIdentity::new("minicore", "features_v1")?;
    Ok(Context {
        crate_name: Some("minicore".into()),
        current: Some(identity.clone()),
        mode: Mode::Source { provider: identity },
        ..Default::default()
    })
}
fn package(source: &str, files: &mut Files<'_>, expanded: bool) -> Result<Package> {
    let input = input(Path::new("lib.rs"), source)?;
    if expanded {
        Ok(expansion::analyze_with_context(input, files, &Config::default(), &context()?)?.package)
    } else {
        Ok(load::load_with_context(
            input,
            files,
            &Config::default(),
            &context()?,
        )?)
    }
}

#[test]
fn declared_source_feature_requests_preserve_actual_use_contracts() {
    // These declarations reuse the unchanged, independently qualified v1 corpus.
    // They test the published loading/expansion contracts, not completed core.
    for (request, tail, accepted) in [
        ("asm_experimental_arch", "", true),
        ("never_type", "type N=!;", true),
        ("", "type N=!;", false),
        ("", "fn f()->!{loop{}}", true),
        ("", "#[cfg(any())]type N=!;", true),
        ("", "mod forged{#![feature(never_type)]type N=!;}", false),
        ("", "mod inert{#![feature(never_type)]}", true),
        (
            "never_type",
            "mod inert{#![feature(never_type)]type N=!;}",
            true,
        ),
        (
            "stmt_expr_attributes",
            "const N:u8=#[allow(unused)]1u8;",
            true,
        ),
        ("", "const N:u8=#[allow(unused)]1u8;", false),
        ("", "#[cfg(any())]const N:u8=#[allow(unused)]1u8;", true),
        ("", "#[allow(unused)]const N:u8=1u8;", true),
    ] {
        let source = format!("#![feature({request})]{HEADER}{tail}");
        for expanded in [false, true] {
            let result = package(&source, &mut Files::default(), expanded);
            assert_eq!(
                result.is_ok(),
                accepted,
                "expanded={expanded}: {source}: {:?}",
                result.err()
            );
        }
    }
}

#[test]
fn unknown_source_requests_are_retained_then_rejected_by_completed_inventory() -> Result {
    let source = format!("#![feature(unknown_source_gate)]{HEADER}");
    let mut loaded = package(&source, &mut Files::default(), false)?;
    assert!(
        loaded
            .library_policy()
            .ok_or("missing request owner")?
            .inventory()
            .is_none()
    );
    let error = resolve::analyze(&mut loaded)
        .err()
        .ok_or("unknown request accepted")?;
    assert!(
        error
            .to_string()
            .contains("undefined library feature request"),
        "{error}"
    );
    assert!(
        loaded
            .library_policy()
            .ok_or("lost request owner")?
            .inventory()
            .is_none()
    );

    let expanded = expansion::analyze_with_context(
        input(Path::new("lib.rs"), &source)?,
        &mut Files::default(),
        &Config::default(),
        &context()?,
    )?;
    assert!(
        expanded
            .package
            .library_policy()
            .ok_or("missing expanded request owner")?
            .inventory()
            .is_none()
    );
    let error = resolve::analyze_expanded(expanded)
        .err()
        .ok_or("unknown expanded request accepted")?;
    assert!(
        error
            .to_string()
            .contains("undefined library feature request"),
        "{error}"
    );
    Ok(())
}

#[test]
fn requests_follow_root_configuration_and_never_authorize_assembly() -> Result {
    for request in [
        "#![cfg_attr(all(),feature(asm_experimental_arch))]",
        "#![cfg_attr(all(),cfg_attr(all(),feature(never_type)))]",
        "#![cfg_attr(any(),feature(unknown_source_gate))]",
    ] {
        for expanded in [false, true] {
            package(
                &format!("{request}{HEADER}"),
                &mut Files::default(),
                expanded,
            )?;
        }
    }
    for tail in [
        "#[rustc_builtin_macro]macro_rules!asm{()=>{}}fn f(){unsafe{asm!(\"nop\");}}",
        "#[rustc_builtin_macro]macro_rules!global_asm{()=>{}}",
    ] {
        let source = format!("#![feature(asm_experimental_arch)]{HEADER}{tail}");
        assert!(package(&source, &mut Files::default(), true).is_err());
        let mut loaded = package(&source, &mut Files::default(), false)?;
        assert!(resolve::analyze(&mut loaded).is_err());
    }
    let ordinary = format!(
        "#![feature(asm_experimental_arch)]{HEADER}macro_rules!asm{{()=>{{1u8}}}}const N:u8=asm!();"
    );
    package(&ordinary, &mut Files::default(), true)?;
    Ok(())
}

#[test]
fn installed_source_uses_keep_parent_roles_and_diagnostic_ownership() -> Result {
    let include = "#[rustc_builtin_macro]macro_rules!include{($($t:tt)*)=>{}}";
    for tail in [
        "include!(\"items.rs\");",
        "macro_rules!ty{()=>{!}}type N=ty!();",
    ] {
        let source = format!("{HEADER}{include}{tail}");
        let result = package(&source, &mut Files(&[("items.rs", "type N=!;")]), true);
        let error = result.err().ok_or("missing never-type feature error")?;
        assert!(error.to_string().contains("never_type"), "{error}");
        let source = format!("#![feature(never_type)]{source}");
        package(&source, &mut Files(&[("items.rs", "type N=!;")]), true)?;
    }
    let source = format!("{HEADER}{include}include!(\"function.rs\");");
    package(
        &source,
        &mut Files(&[("function.rs", "fn f()->!{loop{}}")]),
        true,
    )?;
    let source = format!("{HEADER}mod child;");
    let result = package(&source, &mut Files(&[("child.rs", "type N=!;")]), false);
    let error = result.err().ok_or("missing module feature error")?;
    assert!(error.to_string().contains("file 1:"), "{error}");
    Ok(())
}

#[test]
fn declared_additional_never_positions_and_expression_boundaries() {
    // New opposing hypotheses. Independently qualify before claiming support.
    for (tail, accepted) in [
        ("type F=fn()->!;", true),
        ("fn f()->[!;0]{loop{}}", false),
        ("trait Call<R>{}trait T:Call<fn()->!>{}", true),
        ("fn f(){#[allow(unused)]let x=1u8;}", true),
        ("fn f(){#[allow(unused)]1u8;}", true),
        ("fn f()->u8{#[allow(unused)]1u8}", true),
        ("const N:u8={#![allow(unused)]1u8};", false),
        ("fn f(){loop{#![allow(unused)]break;}}", true),
        ("fn f(){while false{#![allow(unused)]break;}}", true),
        ("fn f(){'l:loop{#![allow(unused)]break 'l;}}", true),
        ("fn f(){let _=loop{#![allow(unused)]break;};}", false),
        ("fn f(){let _=while false{#![allow(unused)]break;};}", false),
        ("fn f(){if true{#![allow(unused)]}}", false),
        ("const N:[u8;#[allow(unused)]1]=[0];", false),
        ("fn f(){let _=(#[allow(unused)]1u8);}", false),
    ] {
        let source = format!("{HEADER}{tail}");
        for expanded in [false, true] {
            let result = package(&source, &mut Files::default(), expanded);
            assert_eq!(
                result.is_ok(),
                accepted,
                "expanded={expanded}: {tail}: {:?}",
                result.err()
            );
        }
    }
    let source =
        format!("#![feature(stmt_expr_attributes)]{HEADER}fn f(){{if true{{#![allow(unused)]}}}}");
    assert!(package(&source, &mut Files::default(), false).is_err());
}
