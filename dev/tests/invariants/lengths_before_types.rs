//! Declaration-time constants: their body publishes its types only after the
//! declaration requirements of every nominal in them are complete, so an
//! array length may name an associated constant of a type declared after it.
type TestResult = Result<(), Box<dyn std::error::Error>>;

/// A minimal core; the generic `Wrap` makes nominal requirements live.
const HEAD: &str = r#"#![feature(no_core, lang_items)]
#![no_core]
#![allow(dead_code, unused_variables, internal_features)]
#[lang = "pointee_sized"]
pub trait PointeeSized {}
#[lang = "meta_sized"]
pub trait MetaSized: PointeeSized {}
#[lang = "sized"]
pub trait Sized: MetaSized {}
#[lang = "legacy_receiver"]
pub trait LegacyReceiver: PointeeSized {}
impl<T: PointeeSized> LegacyReceiver for &T {}
#[lang = "clone"]
pub trait Clone: Sized {
    #[lang = "clone_fn"]
    fn clone(&self) -> Self;
}
#[lang = "copy"]
pub trait Copy: Clone {}
impl Clone for u8 {
    fn clone(&self) -> Self {
        *self
    }
}
impl Copy for u8 {}
pub const fn size<T>() -> usize {
    1
}
pub struct Wrap<T>(T);
"#;

/// Outcomes declared before either compiler ran and checked against rustc
/// 1.98.0: `None` accepts, otherwise the first error this compiler reports
/// for the control rustc also rejects (E0277).
const CASES: [(&str, &str, Option<&str>); 3] = [
    (
        "length-before-type",
        "pub struct Aligned([u8; Group::WIDTH]);
pub struct Group(u8);
impl Group {
    pub const WIDTH: usize = 2;
}
",
        None,
    ),
    (
        "later-bound-met",
        "pub trait Shape {}
impl Shape for u8 {}
pub struct Aligned([u8; Bounded::<u8>::WIDTH]);
pub struct Bounded<T: Shape>(T);
impl<T: Shape> Bounded<T> {
    pub const WIDTH: usize = 2;
}
",
        None,
    ),
    (
        "later-bound-unmet",
        "pub trait Shape {}
pub struct Aligned([u8; Bounded::<u8>::WIDTH]);
pub struct Bounded<T: Shape>(T);
impl<T: Shape> Bounded<T> {
    pub const WIDTH: usize = 2;
}
",
        Some(UNMET),
    ),
];
const UNMET: &str =
    "trait obligation requires an unambiguous matching assumption or implementation";

#[test]
fn a_length_waits_for_a_later_nominals_declaration_requirements() -> TestResult {
    outcomes(&CASES, "later_nominal")
}

/// Analyze each case after `HEAD` as its own crate and compare its outcome.
fn outcomes(cases: &[(&str, &str, Option<&str>)], tag: &str) -> TestResult {
    for &(name, case, expected) in cases {
        let source = format!("{HEAD}{case}");
        let report = crate::test_source::source_provider(&source, tag)?;
        match (expected, crate::pipeline::analyze_source(&report)) {
            (None, Ok(_)) => {}
            (Some(message), Err(error)) => assert_eq!(error.message, message, "{name}"),
            (None, Err(error)) => return Err(format!("{name}: rejected: {error}").into()),
            (Some(message), Ok(_)) => return Err(format!("{name}: accepted, not {message}").into()),
        }
    }
    Ok(())
}
