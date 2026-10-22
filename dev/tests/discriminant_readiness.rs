//! Shape constants must query enum values before final layout publication.
use crate::qualify as rustc;
use crate::semantic;
use std::fmt::Write as _;

const ACCEPTED: &[(&str, &str)] = &[
    (
        "explicit",
        "enum E{A=7,B}type A<const N:usize>=[u8;N];fn f(){let _:A<{E::B as usize}>=[0;8];}",
    ),
    (
        "implicit",
        "enum E{A,B,C}type A<const N:usize>=[u8;N];fn f(){let _:A<{E::C as usize}>=[0;2];}",
    ),
    (
        "unsigned-limit",
        "#[repr(u8)]enum E{A=253,B,C}type A<const N:usize>=[u8;N];fn f(){let _:A<{E::C as usize}>=[0;255];}",
    ),
    (
        "signed",
        "#[repr(i8)]enum E{A=-3,B,C,D}type A<const N:usize>=[u8;N];fn f(){let _:A<{(E::B as isize+3) as usize}>=[0;1];}",
    ),
    (
        "truncated",
        "#[repr(u16)]enum E{A=257}type A<const N:usize>=[u8;N];fn f(){let _:A<{E::A as u8 as usize}>=[0;1];}",
    ),
    (
        "other-enum",
        "enum E{A=F::B as isize}enum F{A=6,B}type A<const N:usize>=[u8;N];fn f(){let _:A<{E::A as usize}>=[0;7];}",
    ),
    (
        "forward-variant",
        "enum E{A=E::B as isize+1,B=4}type A<const N:usize>=[u8;N];fn f(){let _:A<{E::A as usize}>=[0;5];}",
    ),
    (
        "named-constant",
        "const C:isize=7;enum E{A=C,B}type A<const N:usize>=[u8;N];fn f(){let _:A<{E::B as usize}>=[0;8];}",
    ),
    (
        "late-constant",
        "enum E{A=C,B}const C:isize=7;type A<const N:usize>=[u8;N];fn f(){let _:A<{E::B as usize}>=[0;8];}",
    ),
    (
        "mixed-variants",
        "struct S;struct T(u8);enum E{A=7,B}type A<const N:usize>=[u8;N];fn f(){let _:A<{E::B as usize}>=[0;8];}",
    ),
    (
        "reset-implicit",
        "enum E{A,B,C}enum F{A,B}type A<const N:usize>=[u8;N];fn f(){let _:A<{E::C as usize+F::B as usize}>=[0;3];}",
    ),
    (
        "repeated-demand",
        "enum E{A=7,B}type A<const N:usize>=[u8;N];fn f(){let _:A<{E::B as usize}>=[0;8];let _:A<{E::A as usize}>=[0;7];let _:A<{E::B as usize}>=[0;8];}",
    ),
];
const REJECTED: &[(&str, &str)] = &[
    (
        "wrong-explicit-zero",
        "enum E{A=7,B}type A<const N:usize>=[u8;N];fn f(){let _:A<{E::B as usize}>=[0;0];}",
    ),
    (
        "wrong-implicit-zero",
        "enum E{A,B,C}type A<const N:usize>=[u8;N];fn f(){let _:A<{E::C as usize}>=[0;0];}",
    ),
    (
        "variant-cycle",
        "enum E{A=E::B as isize,B=E::A as isize}type A<const N:usize>=[u8;N];fn f(){let _:A<{E::A as usize}>=[0;0];}",
    ),
    (
        "constant-cycle",
        "const C:isize=E::A as isize;enum E{A=C}type A<const N:usize>=[u8;N];fn f(){let _:A<{E::A as usize}>=[0;0];}",
    ),
    (
        "duplicate",
        "enum E{A=3,B=3}type A<const N:usize>=[u8;N];fn f(){let _:A<{E::B as usize}>=[0;3];}",
    ),
    (
        "undemanded-duplicate",
        "enum E{A=3,B=3}enum F{A=4}type A<const N:usize>=[u8;N];fn f(){let _:A<{F::A as usize}>=[0;4];}",
    ),
    (
        "overflow",
        "#[repr(u8)]enum E{A=255,B}type A<const N:usize>=[u8;N];fn f(){let _:A<{E::B as usize}>=[0;0];}",
    ),
    (
        "undemanded-overflow",
        "#[repr(u8)]enum E{A=255,B}enum F{A}type A<const N:usize>=[u8;N];fn f(){let _:A<{F::A as usize}>=[0;0];}",
    ),
];

#[test]
fn local_shape_constants_use_queried_discriminants() -> Result<(), String> {
    for &(name, source) in ACCEPTED {
        semantic::analyze(source).map_err(|e| format!("{name}: {e}"))?;
    }
    Ok(())
}
#[test]
fn provisional_zero_and_invalid_enums_cannot_pass() {
    for &(name, source) in REJECTED {
        assert!(semantic::analyze(source).is_err(), "{name}: {source}");
    }
}
#[test]
fn enum_argument_outcomes_match_independent_rustc() -> Result<(), Box<dyn std::error::Error>> {
    for (cases, accepted) in [(ACCEPTED, true), (REJECTED, false)] {
        for &(name, source) in cases {
            rustc::qualify(&format!("enum-argument-{name}"), source, accepted)?;
        }
    }
    Ok(())
}
#[test]
fn repeated_last_variant_queries_preserve_one_long_predecessor_chain()
-> Result<(), Box<dyn std::error::Error>> {
    let mut source = String::from("enum E{A=3,");
    for index in 0..2048 {
        write!(source, "V{index},")?;
    }
    source.push_str("}type A<const N:usize>=[u8;N];fn f(){");
    for _ in 0..2048 {
        source.push_str("let _:A<{E::V2047 as usize}>=[0;2051];");
    }
    source.push('}');
    semantic::analyze(&source)?;
    Ok(())
}
