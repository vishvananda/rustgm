//! Explicit actual Drop uses reject during typing, before registration or execution.
use super::{
    TestResult,
    destructibility::{BASE, DROP},
    report,
};
use rustgm::sema;
pub(super) const NEGATIVES: &[(&str, &str)] = &[
    (
        "warm-lookalike-first",
        r"mod local{pub trait Drop{fn drop(&mut self);}impl Drop for u8{fn drop(&mut self){}}}fn bad<T:Drop>(){let good=<u8 as local::Drop>::drop;let bad=<T as Drop>::drop;}",
    ),
    ("generic-ufcs", r"fn bad<T:Drop>(x:&mut T){Drop::drop(x);}"),
    (
        "generic-qualified",
        r"fn bad<T:Drop>(x:&mut T){<T as Drop>::drop(x);}",
    ),
    ("generic-dot", r"fn bad<T:Drop>(x:&mut T){x.drop();}"),
    (
        "generic-item",
        r"fn bad<T:Drop>(){let f=<T as Drop>::drop;}",
    ),
    (
        "generic-pointer",
        r"fn bad<T:Drop>(){let f:fn(&mut T)=<T as Drop>::drop;}",
    ),
    (
        "trait-alias",
        r"use Drop as Cleanup;fn bad<T:Cleanup>(x:&mut T){Cleanup::drop(x);}",
    ),
    (
        "concrete-qualified",
        r"struct D;impl Drop for D{fn drop(&mut self){}}fn bad(x:&mut D){<D as Drop>::drop(x);}",
    ),
    (
        "concrete-dot",
        r"struct D;impl Drop for D{fn drop(&mut self){}}fn bad(x:&mut D){x.drop();}",
    ),
    (
        "concrete-item",
        r"struct D;impl Drop for D{fn drop(&mut self){}}fn bad(){let f=D::drop;}",
    ),
    (
        "concrete-pointer",
        r"struct D;impl Drop for D{fn drop(&mut self){}}fn bad(){let f:fn(&mut D)=D::drop;}",
    ),
    (
        "type-alias",
        r"struct D;impl Drop for D{fn drop(&mut self){}}type A=D;fn bad(x:&mut A){A::drop(x);}",
    ),
];
pub(super) const POSITIVES: &[(&str, &str)] = &[
    (
        "other-trait",
        r"mod local{pub trait Drop{fn drop(&mut self);}impl Drop for u8{fn drop(&mut self){*self=7;}}}fn ordinary(x:&mut u8){<u8 as local::Drop>::drop(x);}",
    ),
    (
        "other-trait-pointer",
        r"mod local{pub trait Drop{fn drop(&mut self);}impl Drop for u8{fn drop(&mut self){*self=7;}}}fn ordinary(){let f:fn(&mut u8)=<u8 as local::Drop>::drop;}",
    ),
    (
        "inherent",
        r"struct D;impl D{fn drop(&mut self){}}fn ordinary(x:&mut D){x.drop();}",
    ),
    (
        "inherent-pointer",
        r"struct D;impl D{fn drop(&mut self){}}fn ordinary(){let f:fn(&mut D)=D::drop;}",
    ),
    (
        "free-pointer",
        r"fn drop(x:&mut u8){*x=11;}fn ordinary(){let f:fn(&mut u8)=drop;}",
    ),
];
pub(super) fn source(tail: &str) -> String {
    [BASE, DROP, tail].concat()
}
#[test]
fn actual_drop_calls_and_values_reject_before_registration() -> TestResult {
    for (name, tail) in NEGATIVES {
        denied(name, &source(tail))?;
    }
    let renamed = [
        BASE,
        &DROP.replace("trait Drop", "trait Finalizer"),
        "fn bad<T:Finalizer>(x:&mut T){Finalizer::drop(x);}",
    ]
    .concat();
    denied("renamed-owner", &renamed)?;
    Ok(())
}
fn denied(name: &str, source: &str) -> TestResult {
    let report = report(source)?;
    let facts = sema::declarations::analyze(&report)?;
    let error = sema::bodies::analyze(&report, facts)
        .err()
        .ok_or_else(|| format!("explicit Drop use accepted: {name}"))?
        .to_string();
    assert!(
        error.contains("explicit destructor calls or function values are forbidden"),
        "{name}: {error}"
    );
    assert!(!error.contains("registration"), "{name}: {error}");
    Ok(())
}
#[test]
fn same_spelling_ordinary_members_remain_available_with_actual_drop_present() -> TestResult {
    for (name, tail) in POSITIVES.iter().copied().chain([
        ("member-native-actual-drop", MEMBER_NATIVE),
        ("free-native-actual-drop", FREE_NATIVE),
    ]) {
        let report = report(&source(tail))?;
        let facts = sema::declarations::analyze(&report)?;
        sema::bodies::analyze(&report, facts).map_err(|e| format!("{name}: {e}"))?;
    }
    Ok(())
}
pub(super) const MEMBER_NATIVE: &str = r#"mod local{pub trait Drop{fn drop(&mut self);}impl Drop for u8{fn drop(&mut self){*self=7;}}}#[unsafe(no_mangle)]pub extern "C" fn answer()->u8{let f:fn(&mut u8)=<u8 as local::Drop>::drop;let mut x=0u8;f(&mut x);x}"#;
pub(super) const FREE_NATIVE: &str = r#"fn drop(x:&mut u8){*x=11;}#[unsafe(no_mangle)]pub extern "C" fn answer()->u8{let f:fn(&mut u8)=drop;let mut x=0u8;f(&mut x);x}"#;
