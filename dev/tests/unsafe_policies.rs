//! Scoped lint levels neither grant unsafe permission nor disappear at item boundaries.
#[path = "support/semantic.rs"]
mod semantic;
#[path = "helpers/source_cases.rs"]
mod source;

const CASES: &[(&str, bool, &str)] = &[
    (
        "statement-operation-allow",
        true,
        "#[deny(unsafe_op_in_unsafe_fn)]unsafe fn f(p:*const u8){#[allow(unsafe_op_in_unsafe_fn)]let _=*p;}",
    ),
    (
        "statement-operation-deny",
        false,
        "unsafe fn f(p:*const u8){#[deny(unsafe_op_in_unsafe_fn)]let _=*p;}",
    ),
    (
        "statement-reference-allow",
        true,
        "static mut X:u8=0;fn f(){unsafe{#[allow(static_mut_refs)]let _=&X;}}",
    ),
    (
        "statement-reference-deny",
        false,
        "#![allow(static_mut_refs)]static mut X:u8=0;fn f(){unsafe{#[deny(static_mut_refs)]let _=&X;}}",
    ),
    (
        "operation-deny",
        false,
        "#[deny(unsafe_op_in_unsafe_fn)]unsafe fn f(p:*const u8)->u8{*p}",
    ),
    (
        "operation-forbid",
        false,
        "#[forbid(unsafe_op_in_unsafe_fn)]unsafe fn f(p:*const u8)->u8{*p}",
    ),
    (
        "operation-allow",
        true,
        "#[allow(unsafe_op_in_unsafe_fn)]unsafe fn f(p:*const u8)->u8{*p}",
    ),
    (
        "operation-warn",
        true,
        "#[warn(unsafe_op_in_unsafe_fn)]unsafe fn f(p:*const u8)->u8{*p}",
    ),
    (
        "operation-explicit",
        true,
        "#[deny(unsafe_op_in_unsafe_fn)]unsafe fn f(p:*const u8)->u8{unsafe{*p}}",
    ),
    (
        "allow-safe-function",
        false,
        "#[allow(unsafe_op_in_unsafe_fn)]fn f(p:*const u8)->u8{*p}",
    ),
    (
        "deny-call",
        false,
        "unsafe fn g(){}#[deny(unsafe_op_in_unsafe_fn)]unsafe fn f(){g();}",
    ),
    (
        "deny-static",
        false,
        "static mut X:u8=0;#[deny(unsafe_op_in_unsafe_fn)]unsafe fn f()->u8{X}",
    ),
    (
        "crate-deny",
        false,
        "#![deny(unsafe_op_in_unsafe_fn)]unsafe fn f(p:*const u8)->u8{*p}",
    ),
    (
        "crate-override",
        true,
        "#![deny(unsafe_op_in_unsafe_fn)]#[allow(unsafe_op_in_unsafe_fn)]unsafe fn f(p:*const u8)->u8{*p}",
    ),
    (
        "block-override",
        true,
        "#[deny(unsafe_op_in_unsafe_fn)]unsafe fn f(p:*const u8)->u8{#![allow(unsafe_op_in_unsafe_fn)]*p}",
    ),
    (
        "nested-item-inherit",
        false,
        "#[deny(unsafe_op_in_unsafe_fn)]fn f(){unsafe fn g(p:*const u8)->u8{*p}}",
    ),
    (
        "nested-item-override",
        true,
        "#[deny(unsafe_op_in_unsafe_fn)]fn f(){#[allow(unsafe_op_in_unsafe_fn)]unsafe fn g(p:*const u8)->u8{*p}}",
    ),
    (
        "inline-module-inherit",
        false,
        "#![deny(unsafe_op_in_unsafe_fn)]mod m{pub unsafe fn f(p:*const u8)->u8{*p}}",
    ),
    (
        "inline-module-override",
        true,
        "#![deny(unsafe_op_in_unsafe_fn)]#[allow(unsafe_op_in_unsafe_fn)]mod m{pub unsafe fn f(p:*const u8)->u8{*p}}",
    ),
    (
        "inline-inner-deny",
        false,
        "mod m{#![deny(unsafe_op_in_unsafe_fn)]pub unsafe fn f(p:*const u8)->u8{*p}}",
    ),
    (
        "forbid-allow",
        false,
        "#![forbid(unsafe_op_in_unsafe_fn)]#[allow(unsafe_op_in_unsafe_fn)]fn f(){}",
    ),
    (
        "forbid-warn",
        false,
        "#![forbid(unsafe_op_in_unsafe_fn)]#[warn(unsafe_op_in_unsafe_fn)]fn f(){}",
    ),
    (
        "forbid-deny",
        true,
        "#![forbid(unsafe_op_in_unsafe_fn)]#[deny(unsafe_op_in_unsafe_fn)]fn f(){}",
    ),
    (
        "forbid-same",
        true,
        "#![forbid(unsafe_op_in_unsafe_fn)]#[forbid(unsafe_op_in_unsafe_fn)]fn f(){}",
    ),
    (
        "forbid-deny-allow",
        false,
        "#![forbid(unsafe_op_in_unsafe_fn)]#[deny(unsafe_op_in_unsafe_fn)]mod m{#[allow(unsafe_op_in_unsafe_fn)]fn f(){}}",
    ),
    (
        "reference-allow",
        true,
        "static mut X:u8=0;#[allow(static_mut_refs)]fn f()->&'static u8{unsafe{&X}}",
    ),
    (
        "reference-warn",
        true,
        "static mut X:u8=0;#[warn(static_mut_refs)]fn f()->&'static u8{unsafe{&X}}",
    ),
    (
        "reference-safe",
        false,
        "static mut X:u8=0;#[allow(static_mut_refs)]fn f()->&'static u8{&X}",
    ),
    (
        "reference-deny-override",
        false,
        "#![allow(static_mut_refs)]static mut X:u8=0;#[deny(static_mut_refs)]fn f()->&'static u8{unsafe{&X}}",
    ),
    (
        "reference-forbid",
        false,
        "#![forbid(static_mut_refs)]#[allow(static_mut_refs)]fn f(){}",
    ),
    (
        "reference-binding-allow",
        true,
        "static mut X:u8=0;#[allow(static_mut_refs)]fn f(){unsafe{let ref r=X;let _=r;}}",
    ),
    (
        "reference-method-allow",
        true,
        "struct S;impl S{fn g(&self){}}static mut X:S=S;#[allow(static_mut_refs)]fn f(){unsafe{X.g();}}",
    ),
    (
        "configured-allow",
        true,
        "static mut X:u8=0;#[cfg_attr(all(),allow(static_mut_refs))]fn f()->&'static u8{unsafe{&X}}",
    ),
    (
        "configured-disabled",
        false,
        "static mut X:u8=0;#[cfg_attr(any(),allow(static_mut_refs))]fn f()->&'static u8{unsafe{&X}}",
    ),
    (
        "unknown-path",
        false,
        "static mut X:u8=0;#[allow(clippy::static_mut_refs)]fn f()->&'static u8{unsafe{&X}}",
    ),
];

#[test]
fn scoped_unsafe_lints() -> Result<(), Box<dyn std::error::Error>> {
    source::check("unsafe-policies", CASES)
}

#[test]
fn policies_allocate_only_when_relevant_and_walk_additively()
-> Result<(), Box<dyn std::error::Error>> {
    use std::fmt::Write;
    for attrs in [false, true] {
        for functions in [8, 32, 128] {
            for depth in [1, 8, 32] {
                let mut text = String::new();
                for index in 0..functions {
                    if attrs {
                        text.push_str("#[deny(unsafe_op_in_unsafe_fn)]");
                    }
                    write!(text, "unsafe fn f{index}(p:*const u8)->u8{{")?;
                    for _ in 0..depth {
                        text.push_str("unsafe{");
                    }
                    text.push_str("*p");
                    for _ in 0..=depth {
                        text.push('}');
                    }
                }
                let (report, facts) = semantic::analyze(&text)?;
                let work = facts.safety_work();
                if attrs {
                    let nodes: usize = report
                        .package
                        .units
                        .iter()
                        .map(|u| u.configured.ast.nodes.len())
                        .sum();
                    assert_eq!(work.attributes, functions);
                    assert_eq!(work.entries, functions);
                    assert_eq!(work.retained_nodes, nodes);
                    assert!(work.nodes <= 2 * nodes + report.package.modules.len());
                    assert!(work.edges <= 2 * nodes);
                } else {
                    assert_eq!(work.attributes, 0);
                    assert_eq!(work.retained_nodes, 0);
                    assert_eq!(work.nodes, 0);
                    assert_eq!(work.edges, 0);
                }
            }
        }
    }
    Ok(())
}
