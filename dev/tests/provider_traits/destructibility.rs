//! Actual Drop headers do not publish cleanup; ordinary Destruct keeps its identity.
use super::{TestResult, lower, report};
use rustgm::{sema, support::bindings::LangItem};
pub(super) const BASE: &str = r#"#![feature(no_core,lang_items,rustc_attrs,const_trait_impl)]
#![no_core]
#![allow(internal_features,dead_code,unused_variables)]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[lang="legacy_receiver"]pub trait LegacyReceiver:PointeeSized{}
impl<T:PointeeSized>LegacyReceiver for &T{}
impl<T:PointeeSized>LegacyReceiver for &mut T{}
#[lang="clone"]pub const trait Clone:Sized{#[lang="clone_fn"]fn clone(&self)->Self;}
#[lang="copy"]pub trait Copy:Clone{}
impl Clone for u8{fn clone(&self)->Self{*self}}impl Copy for u8{}
"#;
pub(super) const DROP: &str = r#"#[lang="drop"]pub const trait Drop{fn drop(&mut self);}
"#;
pub(super) const DESTRUCT: &str = r#"#[lang="destruct"]#[rustc_deny_explicit_impl]pub const trait Destruct:PointeeSized{}
"#;
pub(super) const ORDINARY: &str = r#"fn seven<T:Destruct>()->u8{7}#[unsafe(no_mangle)]pub extern "C" fn answer()->u8{seven::<u8>()}
"#;
pub(super) fn ordinary_source() -> String {
    [BASE, DESTRUCT, ORDINARY].concat()
}
#[test]
fn source_destruction_headers_retain_real_declarations_without_execution_permission() -> TestResult
{
    let report = report(&[BASE, DROP, DESTRUCT].concat())?;
    sema::declarations::analyze(&report)?;
    let reservations = report
        .resolution
        .source_bindings
        .as_deref()
        .ok_or("reservations")?;
    for role in [LangItem::Drop, LangItem::Destruct] {
        let declaration = reservations.language(role).ok_or("role")?;
        assert_eq!(
            report.resolution.definitions[declaration.definition.0 as usize].node,
            Some(declaration.origin)
        );
    }
    Ok(())
}
#[test]
fn source_destruction_headers_reject_forged_and_malformed_contracts() -> TestResult {
    for tail in [
        r#"#[lang="drop"]pub struct Drop;"#,
        r#"#[lang="drop"]pub const trait Drop{fn drop(self);}"#,
        r#"#[lang="drop"]pub const trait Drop{fn drop(&mut self)->u8;}"#,
        r#"#[lang="destruct"]pub const trait Destruct:PointeeSized{}"#,
    ] {
        let report = report(&[BASE, tail].concat())?;
        assert!(sema::declarations::analyze(&report).is_err(), "{tail}");
    }
    Ok(())
}
#[test]
fn ordinary_destruct_is_not_const_or_a_lookalike_implementation() -> TestResult {
    lower(&ordinary_source())?;
    let lookalike =
        ordinary_source().replace(r#"#[lang="destruct"]#[rustc_deny_explicit_impl]"#, "");
    assert!(lower(&lookalike).is_err());
    let explicit = [BASE, DESTRUCT, "impl Destruct for u8{}"].concat();
    let report = report(&explicit)?;
    assert!(sema::declarations::analyze(&report).is_err());
    Ok(())
}
#[test]
fn actual_drop_without_registration_cannot_publish_even_literal_bodies() -> TestResult {
    let source = [BASE, DROP, "fn one()->u8{7}"].concat();
    let report = report(&source)?;
    sema::declarations::analyze(&report)?;
    let error = lower(&source)
        .err()
        .ok_or("unprepared Drop published")?
        .to_string();
    assert!(
        error.contains("source destructor registration is not implemented"),
        "{error}"
    );
    lower(&[BASE, "trait Drop{}fn one()->u8{7}"].concat())?;
    Ok(())
}
