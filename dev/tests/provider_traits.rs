//! Source compiler roles retain actual declarations, qualified goals and original proofs.
//! Header cutouts are attributed to pinned Rust core marker/clone/deref/function sources.
use rustgm::{
    expand::config::Config,
    resolve::{
        self,
        bindings::Context,
        load::{InputFile, Sources},
    },
    sema,
    support::{bindings::Mode, crates::CrateIdentity, source::SourceFile},
};
use std::path::Path;
#[path = "provider_traits/cases.rs"]
mod cases;
#[path = "provider_traits/destructibility.rs"]
mod destructibility;
#[path = "provider_traits/forbidden_cleanup.rs"]
mod forbidden_cleanup;
#[path = "provider_traits/native.rs"]
mod native;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const PROVIDER: &str = r#"#![feature(unboxed_closures)]
#![feature(no_core,lang_items,rustc_attrs,const_trait_impl)]
#![no_core]
#![allow(internal_features,dead_code,unused_variables)]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[lang="legacy_receiver"]pub trait LegacyReceiver:PointeeSized{}
impl<T:PointeeSized> LegacyReceiver for &T{}
impl<T:PointeeSized> LegacyReceiver for &mut T{}
#[lang="clone"]pub const trait Clone:Sized{#[lang="clone_fn"]fn clone(&self)->Self;}
#[lang="copy"]pub trait Copy:Clone{}
impl Clone for u8{fn clone(&self)->Self{*self}}impl Copy for u8{}
#[lang="tuple_trait"]#[rustc_deny_explicit_impl]pub trait Tuple{}
#[lang="fn_once"]pub const trait FnOnce<Args:Tuple>{#[lang="fn_once_output"]type Output;extern "rust-call" fn call_once(self,args:Args)->Self::Output;}
#[lang="fn_mut"]pub const trait FnMut<Args:Tuple>:FnOnce<Args>{extern "rust-call" fn call_mut(&mut self,args:Args)->Self::Output;}
#[lang="fn"]pub const trait Fn<Args:Tuple>:[const]FnMut<Args>{extern "rust-call" fn call(&self,args:Args)->Self::Output;}

"#;
struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _: &Path) -> Result<Option<InputFile>, String> {
        Ok(None)
    }
}
fn report(source: &str) -> TestResult<resolve::expansion::Report> {
    let identity = CrateIdentity::new("minicore", "source_traits_v1")?;
    let context = Context {
        crate_name: Some("minicore".into()),
        current: Some(identity.clone()),
        mode: Mode::Source { provider: identity },
        ..Default::default()
    };
    Ok(resolve::analyze_expanded(
        resolve::expansion::analyze_with_context(
            InputFile {
                identity: "provider.rs".into(),
                path: "provider.rs".into(),
                source: SourceFile::new(source.as_bytes().to_vec())?,
            },
            &mut NoFiles,
            &Config::default(),
            &context,
        )?,
    )?)
}
#[test]
fn actual_source_role_headers_keep_real_definition_identity() -> TestResult {
    let report = report(PROVIDER)?;
    let facts = sema::declarations::analyze(&report)?;
    for role in [
        rustgm::resolve::model::BuiltinTrait::Copy,
        rustgm::resolve::model::BuiltinTrait::Clone,
        rustgm::resolve::model::BuiltinTrait::Sized,
    ] {
        assert!(report.resolution.builtin_traits.get(role).is_none());
    }
    for function in &facts.functions {
        assert_eq!(
            report.resolution.definitions[function.definition.0 as usize].node,
            Some(function.node)
        );
    }
    Ok(())
}
#[test]
fn source_roles_reject_wrong_owners_and_unsupported_header_profiles() -> TestResult {
    for (from, to) in [
        ("pub trait Copy:Clone", "pub trait Copy"),
        ("fn clone(&self)->Self", "fn clone(&self)->u8"),
        ("pub const trait Clone", "pub trait Clone"),
        (r#"#[lang="clone_fn"]"#, ""),
        (r#"#[lang="fn_once_output"]"#, ""),
        (
            r#"extern "rust-call" fn call_once"#,
            r#"extern "C" fn call_once"#,
        ),
    ] {
        let source = PROVIDER.replace(from, to);
        assert_ne!(source, PROVIDER, "control did not alter its exact header");
        let report = report(&source)?;
        assert!(sema::declarations::analyze(&report).is_err(), "{source}");
    }
    Ok(())
}

// These unchanged upstream-derived cutouts were independently object/native
// qualified before implementation (source-traits-v1 observations[-v2]).
const COPY_EXECUTION: &str = cases::GENERIC_COPY;
const CLONE_EXECUTION: &str = cases::SOURCE_CLONE_UFCS;
const CALL_EXECUTION: &str = cases::CALLABLE_FUNCTION_ITEM;
fn lower(source: &str) -> TestResult<rustgm::lowir::program::Program> {
    let report = report(source)?;
    let rustgm::pipeline::Analysis {
        mut facts,
        prepared,
    } = rustgm::pipeline::analyze_source(&report)?;
    Ok(rustgm::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        rustgm::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )?)
}
#[test]
fn source_copy_clone_and_callable_bodies_pass_ordinary_original_and_instance_proofs() -> TestResult
{
    for (name, source) in [
        ("copy", COPY_EXECUTION),
        ("clone", CLONE_EXECUTION),
        ("callable", CALL_EXECUTION),
    ] {
        let program = lower(source).map_err(|error| format!("{name}: {error}"))?;
        let mut output = Vec::new();
        rustgm::lowir::writer::write(&program, &mut output)?;
        assert!(!output.is_empty(), "{name}");
    }
    Ok(())
}

#[test]
fn source_copy_policy_is_demanded_and_never_inferred_from_primitive_shape() -> TestResult {
    for source in [cases::ABSENT_ROLE_LITERAL, cases::MISSING_IMPL_ONE_USE] {
        lower(source)?;
    }
    for source in [
        cases::ABSENT_ROLE_ONE_USE,
        cases::MISSING_IMPL_DOUBLE_USE,
        cases::COPY_IMPL_UNPAID_CLONE,
    ] {
        assert!(lower(source).is_err(), "{source}");
    }
    Ok(())
}

#[test]
fn source_conditional_calls_preserve_runtime_and_const_opposites() -> TestResult {
    lower(cases::UNUSED_CONDITIONAL_RUNTIME)?;
    for source in [
        cases::NONCONST_TRAIT_QUALIFIED,
        cases::NONCONST_TRAIT_CONST_IMPL,
        cases::UNPAID_CONST_DEFINING_BODY,
    ] {
        assert!(lower(source).is_err(), "{source}");
    }
    for source in [
        cases::UNUSED_CONDITIONAL_CONST_UNPAID,
        cases::UNUSED_CONDITIONAL_CONST_PAID,
    ] {
        let report = report(source)?;
        let error = rustgm::pipeline::analyze_source(&report)
            .err()
            .ok_or("unsupported const payment accepted")?
            .to_string();
        assert!(
            error.contains("conditional const-call debt payment is not implemented")
                || error.contains("trait obligation"),
            "{error}"
        );
    }
    Ok(())
}
