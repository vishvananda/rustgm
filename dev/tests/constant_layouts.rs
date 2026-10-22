//! Constant layout demands must publish final enum tags before caching byte layouts.
use crate::qualify as rustc;
use crate::semantic;
use rustgm::{abi::layout::Shape, types::Primitive};
use std::fmt::Write as _;

#[test]
fn aligned_borrows_preserve_lazy_values_and_pointer_recursion() -> Result<(), String> {
    for &(name, source) in ACCEPTED {
        semantic::analyze(source).map_err(|e| format!("{name}: {e}"))?;
    }
    Ok(())
}

#[test]
fn packed_borrows_require_all_tags_even_in_dead_branches() {
    for &(name, source) in REJECTED {
        assert!(semantic::analyze(source).is_err(), "{name}: {source}");
    }
}

#[test]
fn constant_layout_outcomes_match_independent_rustc() -> Result<(), Box<dyn std::error::Error>> {
    for (cases, accepted) in [(ACCEPTED, true), (REJECTED, false)] {
        for &(name, source) in cases {
            rustc::qualify(&format!("constant-layout-{name}"), source, accepted)?;
        }
    }
    Ok(())
}

#[test]
fn cached_enum_tags_and_variants_are_final() -> Result<(), String> {
    for (name, expected_tag, expected_bits) in [
        ("c-packed8-large", Primitive::U64, 4_294_967_296_u128),
        (
            "negative-aligned",
            Primitive::I64,
            u128::from(u64::MAX) - 4_294_967_295,
        ),
    ] {
        let source = ACCEPTED
            .iter()
            .find(|c| c.0 == name)
            .ok_or("missing layout case")?
            .1;
        let (report, facts) = semantic::analyze(source)?;
        let index = report
            .resolution
            .definitions
            .iter()
            .position(|d| report.package.names.get(d.name) == "E")
            .ok_or("missing enum")?;
        let ty = facts.definitions[index].ok_or("missing enum type")?;
        let layout = facts.layouts.get(ty).ok_or("missing enum layout")?;
        assert_eq!((layout.size, layout.align), (8, 8));
        let Shape::Aggregate { tag: Some(tag), .. } = layout.shape else {
            return Err("missing cached enum tag".into());
        };
        assert_eq!(tag.ty, expected_tag);
        assert_eq!(
            facts.layouts.variants(layout)[1].discriminant,
            expected_bits
        );
    }
    Ok(())
}

#[test]
fn repeated_borrows_share_completed_layouts() -> Result<(), Box<dyn std::error::Error>> {
    let mut source = String::from("#[repr(C)]#[derive(Copy,Clone)]enum E{A=0,B=4294967296}");
    source.push_str("#[repr(packed(8))]struct S{e:E}type A<const N:usize>=[u8;N];fn f(){");
    for _ in 0..1024 {
        write!(source, "let _:A<{{(&S{{e:E::B}}.e,0).1}}>=[];")?;
    }
    source.push('}');
    let (_, facts) = semantic::analyze(&source)?;
    assert!(
        facts.layouts.steps < 64,
        "{} layout steps",
        facts.layouts.steps
    );
    Ok(())
}

const ACCEPTED: &[(&str, &str)] = &[
    (
        "c-packed8-large",
        "#[repr(C)]#[derive(Copy,Clone)]enum E{A=0,B=4294967296}#[repr(packed(8))]struct S{e:E}type A<const N:usize>=[u8;N];fn f(){let _:A<{*(&S{e:E::B}.e) as u8 as usize}>=[0;0];}",
    ),
    (
        "c-packed4-small",
        "#[repr(C)]#[derive(Copy,Clone)]enum E{A=0,B=256}#[repr(packed(4))]struct S{e:E}type A<const N:usize>=[u8;N];fn f(){let _:A<{*(&S{e:E::B}.e) as u8 as usize}>=[0;0];}",
    ),
    (
        "nested-array-aligned",
        "#[repr(C)]#[derive(Copy,Clone)]enum E{A=0,B=4294967296}#[repr(packed(8))]struct S{e:[E;1]}type A<const N:usize>=[u8;N];fn f(){let _:A<{(&S{e:[E::B;1]}.e)[0] as u8 as usize}>=[0;0];}",
    ),
    (
        "zero-array-aligned",
        "#[repr(C)]#[derive(Copy,Clone)]enum E{A=0,B=4294967296}#[repr(packed(8))]struct S{e:[E;0]}type A<const N:usize>=[u8;N];fn f(){let _:A<{(&S{e:[]}.e,0).1}>=[0;0];}",
    ),
    (
        "nested-tuple-aligned",
        "#[repr(C)]#[derive(Copy,Clone)]enum E{A=0,B=4294967296}#[repr(packed(8))]struct S{e:(E,)}type A<const N:usize>=[u8;N];fn f(){let _:A<{(&S{e:(E::B,)}.e).0 as u8 as usize}>=[0;0];}",
    ),
    (
        "nested-struct-aligned",
        "#[repr(C)]#[derive(Copy,Clone)]enum E{A=0,B=4294967296}#[derive(Copy,Clone)]struct W{e:E}#[repr(packed(8))]struct S{w:W}type A<const N:usize>=[u8;N];fn f(){let _:A<{(&S{w:W{e:E::B}}.w).e as u8 as usize}>=[0;0];}",
    ),
    (
        "unread-tag-aligned",
        "#[repr(C)]#[derive(Copy,Clone)]enum E{A=0,B=4294967296}#[repr(packed(8))]struct S{e:E}type A<const N:usize>=[u8;N];fn f(){let _:A<{(&S{e:E::A}.e,0).1}>=[0;0];}",
    ),
    (
        "other-enum",
        "#[repr(C)]#[derive(Copy,Clone)]enum E{A=0,B=*(&S{e:F::B}.e) as isize+1}#[repr(C)]#[derive(Copy,Clone)]enum F{A=0,B=256}#[repr(packed(4))]struct S{e:F}",
    ),
    (
        "dead-arithmetic",
        "#[repr(C)]#[derive(Copy,Clone)]enum E{A=0,B=4294967296}#[repr(packed(8))]struct S{e:E}type A<const N:usize>=[u8;N];fn f(){let _:A<{if true{0}else{(*(&S{e:E::B}.e) as u8 as usize)/(E::A as usize)}}>=[0;0];}",
    ),
    (
        "pointer-recursion",
        "#[derive(Copy,Clone)]struct R{next:&'static R,n:usize}static V:R=R{next:&V,n:3};#[repr(packed(8))]struct S{r:R}type A<const N:usize>=[u8;N];fn f(){let _:A<{(&S{r:V}.r).n}>=[0;3];}",
    ),
    (
        "negative-aligned",
        "#[repr(C)]#[derive(Copy,Clone)]enum E{A=0,B=-4294967296}#[repr(packed(8))]struct S{e:E}type A<const N:usize>=[u8;N];fn f(){let _:A<{*(&S{e:E::B}.e) as u8 as usize}>=[0;0];}",
    ),
];
const REJECTED: &[(&str, &str)] = &[
    (
        "c-packed4-large",
        "#[repr(C)]#[derive(Copy,Clone)]enum E{A=0,B=4294967296}#[repr(packed(4))]struct S{e:E}type A<const N:usize>=[u8;N];fn f(){let _:A<{*(&S{e:E::B}.e) as u8 as usize}>=[0;0];}",
    ),
    (
        "c-packed4-negative-large",
        "#[repr(C)]#[derive(Copy,Clone)]enum E{A=0,B=-4294967296}#[repr(packed(4))]struct S{e:E}type A<const N:usize>=[u8;N];fn f(){let _:A<{*(&S{e:E::B}.e) as u8 as usize}>=[0;0];}",
    ),
    (
        "u64-packed4",
        "#[repr(u64)]#[derive(Copy,Clone)]enum E{A=0,B=256}#[repr(packed(4))]struct S{e:E}type A<const N:usize>=[u8;N];fn f(){let _:A<{*(&S{e:E::B}.e) as u8 as usize}>=[0;0];}",
    ),
    (
        "c-packed4-large-forward",
        "#[repr(C)]#[derive(Copy,Clone)]enum E{A=0,B=OTHER}#[repr(packed(4))]struct S{e:E}type A<const N:usize>=[u8;N];fn f(){let _:A<{*(&S{e:E::B}.e) as u8 as usize}>=[0;0];}const OTHER:isize=4294967296;",
    ),
    (
        "nested-array",
        "#[repr(C)]#[derive(Copy,Clone)]enum E{A=0,B=4294967296}#[repr(packed(4))]struct S{e:[E;1]}type A<const N:usize>=[u8;N];fn f(){let _:A<{(&S{e:[E::B;1]}.e)[0] as u8 as usize}>=[0;0];}",
    ),
    (
        "zero-array",
        "#[repr(C)]#[derive(Copy,Clone)]enum E{A=0,B=4294967296}#[repr(packed(4))]struct S{e:[E;0]}type A<const N:usize>=[u8;N];fn f(){let _:A<{(&S{e:[]}.e,0).1}>=[0;0];}",
    ),
    (
        "nested-tuple",
        "#[repr(C)]#[derive(Copy,Clone)]enum E{A=0,B=4294967296}#[repr(packed(4))]struct S{e:(E,)}type A<const N:usize>=[u8;N];fn f(){let _:A<{(&S{e:(E::B,)}.e).0 as u8 as usize}>=[0;0];}",
    ),
    (
        "nested-struct",
        "#[repr(C)]#[derive(Copy,Clone)]enum E{A=0,B=4294967296}#[derive(Copy,Clone)]struct W{e:E}#[repr(packed(4))]struct S{w:W}type A<const N:usize>=[u8;N];fn f(){let _:A<{(&S{w:W{e:E::B}}.w).e as u8 as usize}>=[0;0];}",
    ),
    (
        "unread-tag",
        "#[repr(C)]#[derive(Copy,Clone)]enum E{A=0,B=4294967296}#[repr(packed(4))]struct S{e:E}type A<const N:usize>=[u8;N];fn f(){let _:A<{(&S{e:E::A}.e,0).1}>=[0;0];}",
    ),
    (
        "dead-branch",
        "#[repr(C)]#[derive(Copy,Clone)]enum E{A=0,B=4294967296}#[repr(packed(4))]struct S{e:E}type A<const N:usize>=[u8;N];fn f(){let _:A<{if true{0}else{*(&S{e:E::B}.e) as u8 as usize}}>=[0;0];}",
    ),
    (
        "dead-short-circuit",
        "#[repr(C)]#[derive(Copy,Clone)]enum E{A=0,B=4294967296}#[repr(packed(4))]struct S{e:E}type A<const N:bool>=[u8;0];fn f(){let _:A<{true || (*(&S{e:E::B}.e) as u8==0)}>=[];}",
    ),
    (
        "explicit-self-borrow",
        "#[repr(u8)]#[derive(Copy,Clone)]enum E{A=0,B=*(&S{e:E::A}.e) as u8+1}#[repr(packed)]struct S{e:E}",
    ),
    (
        "explicit-cross-borrow",
        "#[repr(u8)]#[derive(Copy,Clone)]enum E{A=0,B=*(&S{e:F::A}.e) as u8+1}#[repr(u8)]#[derive(Copy,Clone)]enum F{A=0,B=*(&T{e:E::A}.e) as u8+2}#[repr(packed)]struct S{e:F}#[repr(packed)]struct T{e:E}",
    ),
    (
        "explicit-self-late",
        "#[repr(u8)]#[derive(Copy,Clone)]enum E{A=0,B=*(&S{e:E::A}.e) as u8+1,C=255}#[repr(packed)]struct S{e:E}",
    ),
    (
        "c-self-borrow",
        "#[repr(C)]#[derive(Copy,Clone)]enum E{A=0,B=*(&S{e:E::A}.e) as isize+1}#[repr(packed(8))]struct S{e:E}",
    ),
    (
        "unused-duplicate",
        "#[repr(C)]#[derive(Copy,Clone)]enum E{A=0,B=0}#[repr(packed(8))]struct S{e:E}type A<const N:usize>=[u8;N];fn f(){let _:A<{(&S{e:E::A}.e,0).1}>=[];}",
    ),
    (
        "unused-overflow",
        "#[repr(u8)]#[derive(Copy,Clone)]enum E{A=0,B=256}#[repr(packed(8))]struct S{e:E}type A<const N:usize>=[u8;N];fn f(){let _:A<{(&S{e:E::A}.e,0).1}>=[];}",
    ),
];
