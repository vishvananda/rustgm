//! Declaration-time constants: their body publishes its types only after the
//! declaration requirements of every nominal in them are complete, so an
//! array length may name an associated constant of a type declared after it.
//! Each such constant is typed in its own lexical module.
type TestResult = Result<(), Box<dyn std::error::Error>>;

/// A minimal core with the callable family a closure needs; the generic
/// `Wrap` makes nominal requirements live.
const HEAD: &str = r#"#![feature(no_core, lang_items, unboxed_closures, rustc_attrs, const_trait_impl)]
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
impl<T: PointeeSized> LegacyReceiver for &mut T {}
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
#[lang = "tuple_trait"]
#[rustc_deny_explicit_impl]
pub trait Tuple {}
#[lang = "fn_once"]
pub const trait FnOnce<Args: Tuple> {
    #[lang = "fn_once_output"]
    type Output;
    extern "rust-call" fn call_once(self, args: Args) -> Self::Output;
}
#[lang = "fn_mut"]
pub const trait FnMut<Args: Tuple>: FnOnce<Args> {
    extern "rust-call" fn call_mut(&mut self, args: Args) -> Self::Output;
}
#[lang = "fn"]
pub const trait Fn<Args: Tuple>: [const] FnMut<Args> {
    extern "rust-call" fn call(&self, args: Args) -> Self::Output;
}
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

/// Each constant is typed in its own lexical module, also where its position
/// has no binder: a nominal's field, a body, a const item's initializer, a
/// local item or a closure. Outcomes as for `CASES`; the rejected controls
/// are rustc's E0624 and type privacy errors.
const SITES: [(&str, &str, Option<&str>); 9] = [
    (
        "nested-field",
        "pub mod m {
    struct P(u8);
    impl P {
        const N: usize = 2;
    }
    pub struct S([u8; P::N]);
}
",
        None,
    ),
    (
        "nested-body",
        "pub mod m {
    struct P(u8);
    impl P {
        const N: usize = 2;
    }
    pub fn f() {
        let _a = [0u8; P::N];
        let _b: [u8; crate::size::<P>()] = [0];
    }
}
",
        None,
    ),
    (
        "nested-const",
        "pub mod m {
    struct P(u8);
    impl P {
        const N: usize = 2;
    }
    pub const C: [u8; 2] = {
        let a = [7u8; P::N];
        a
    };
}
",
        None,
    ),
    (
        "nested-local-item",
        "pub mod m {
    struct P(u8);
    impl P {
        const N: usize = 2;
    }
    pub fn f() {
        struct Local([u8; P::N]);
        let _l = Local([0; P::N]);
    }
}
",
        None,
    ),
    (
        "nested-closure",
        "pub mod m {
    struct P(u8);
    impl P {
        const N: usize = 2;
    }
    pub fn f() {
        let _c = || [0u8; P::N];
    }
}
",
        None,
    ),
    (
        "root-private-assoc",
        "pub mod m {
    pub struct Q(u8);
    impl Q {
        const N: usize = 2;
    }
}
pub fn f() {
    let _a = [0u8; m::Q::N];
}
",
        Some(PRIVATE_MEMBER),
    ),
    (
        "sibling-private-type",
        "pub mod a {
    pub(super) mod b {
        pub(super) struct P(u8);
    }
    pub type A = b::P;
}
pub mod c {
    pub fn f() {
        let _a = [0u8; crate::size::<crate::a::A>()];
    }
}
",
        Some("semantic type or associated item is private"),
    ),
    (
        "sibling-visible-field",
        "pub mod a {
    pub struct Q(u8);
    impl Q {
        pub(super) const N: usize = 2;
    }
}
pub mod c {
    pub struct S([u8; crate::a::Q::N]);
}
",
        None,
    ),
    (
        "sibling-private-field",
        "pub mod a {
    pub struct Q(u8);
    impl Q {
        const N: usize = 2;
    }
}
pub mod c {
    pub struct S([u8; crate::a::Q::N]);
}
",
        Some(PRIVATE_MEMBER),
    ),
];
const PRIVATE_MEMBER: &str = "inherent method is private";

#[test]
fn constants_are_typed_in_their_own_lexical_module() -> TestResult {
    outcomes(&SITES, "constant_sites")
}
