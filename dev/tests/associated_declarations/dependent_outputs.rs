//! Projection selection waits for exact headers and definitions without dropping caller debts.
use super::{native, object_outcome, semantic};
use std::fmt::Write as _;

const CASES: &[(&str, &str, bool)] = &[
    (
        "projected-value-type",
        r"trait B{fn value()->usize;}struct X<const N:usize>;trait A{type Out;const C:Self::Out;}impl B for X<{<u8 as A>::C}>{fn value()->usize{3}}impl A for u8{type Out=usize;const C:usize=3;}fn main(){check(<X<3>as B>::value()==3);}
",
        true,
    ),
    (
        "projected-impl-subject",
        r"trait B{fn value()->usize;}struct X<const N:usize>;trait Identity{type Out;}trait A{const C:usize;}impl B for X<{<u8 as A>::C}>{fn value()->usize{3}}impl A for <u16 as Identity>::Out{const C:usize=3;}impl Identity for u16{type Out=u8;}fn main(){check(<X<3>as B>::value()==3);}
",
        true,
    ),
    (
        "nested-projected-subject",
        r"trait B{fn value()->usize;}struct X<const N:usize>;trait Identity{type Out;}trait A{const C:usize;}impl B for X<{<u8 as A>::C}>{fn value()->usize{3}}impl A for <u16 as Identity>::Out{const C:usize=3;}impl Identity for u16{type Out=<u32 as Identity>::Out;}impl Identity for u32{type Out=u8;}fn main(){check(<X<3>as B>::value()==3);}
",
        true,
    ),
    (
        "missing-projection-impl",
        r"trait B{fn value()->usize;}struct X<const N:usize>;trait Identity{type Out;}trait A{const C:usize;}impl B for X<{<u8 as A>::C}>{fn value()->usize{3}}impl A for <u16 as Identity>::Out{const C:usize=3;}fn main(){check(<X<3>as B>::value()==3);}
",
        false,
    ),
    (
        "cyclic-projection-result",
        r"trait B{fn value()->usize;}struct X<const N:usize>;trait Identity{type Out;}trait A{const C:usize;}impl B for X<{<u8 as A>::C}>{fn value()->usize{3}}impl A for <u16 as Identity>::Out{const C:usize=3;}impl Identity for u16{type Out=<u16 as Identity>::Out;}fn main(){check(<X<3>as B>::value()==3);}
",
        false,
    ),
    (
        "projected-owner-overlap",
        r"trait B{fn value()->usize;}struct X<const N:usize>;trait Identity{type Out;}trait A{const C:usize;}impl B for X<{<u8 as A>::C}>{fn value()->usize{3}}impl A for <u16 as Identity>::Out{const C:usize=3;}impl A for u8{const C:usize=3;}impl Identity for u16{type Out=u8;}fn main(){check(<X<3>as B>::value()==3);}
",
        false,
    ),
    (
        "projected-static-subject",
        r"trait B{fn value()->usize;}struct X<const N:usize>;trait A<'a>{type Out;}trait Z{const N:usize;}impl B for X<{<&'static u8 as Z>::N}>{fn value()->usize{3}}impl Z for <&'static u8 as A<'static>>::Out{const N:usize=3;}impl<'a>A<'a>for &'a u8{type Out=&'a u8;}fn main(){check(<X<3>as B>::value()==3);}
",
        true,
    ),
    (
        "projected-caller-lifetime-debt",
        r"trait B{fn value()->usize;}struct X<const N:usize>;trait A<'a>{type Out;}trait Z{const N:usize;}impl B for X<{<&'static u8 as Z>::N}>{fn value()->usize{3}}impl Z for <&'static u8 as A<'static>>::Out{const N:usize=3;}impl<'a>A<'a>for &'a u8{type Out=&'a u8;}fn bad<'a>(_:&'a u8)->usize{<&'a u8 as Z>::N}fn main(){check(<X<3>as B>::value()==3);}
",
        false,
    ),
    (
        "generic-definition-subject",
        r"trait B{fn value()->usize;}struct X<const N:usize>;trait A{type Out;}trait Z{const C:usize;}struct S<T>(T);impl B for X<{<S<u8>as Z>::C}>{fn value()->usize{3}}impl Z for <u8 as A>::Out{const C:usize=3;}impl A for u8{type Out=S<u8>;}fn main(){check(<X<3>as B>::value()==3);}
",
        true,
    ),
    (
        "generic-owner-definition",
        r"trait B{fn value()->usize;}struct X<const N:usize>;trait A{type Out;}trait Z{const C:usize;}struct S<T>(T);impl B for X<{<S<u8>as Z>::C}>{fn value()->usize{3}}impl Z for <u8 as A>::Out{const C:usize=3;}impl<T>A for T{type Out=S<T>;}fn main(){check(<X<3>as B>::value()==3);}
",
        true,
    ),
    (
        "conditional-definition-subject",
        r"trait B{fn value()->usize;}struct X<const N:usize>;trait A{type Out;}trait P{}trait Z{const C:usize;}impl B for X<{<u8 as Z>::C}>{fn value()->usize{3}}impl Z for <u16 as A>::Out{const C:usize=3;}impl<T:P>A for T{type Out=u8;}impl P for u16{}fn main(){check(<X<3>as B>::value()==3);}
",
        true,
    ),
    (
        "missing-conditional-definition-bound",
        r"trait B{fn value()->usize;}struct X<const N:usize>;trait A{type Out;}trait P{}trait Z{const C:usize;}impl B for X<{<u8 as Z>::C}>{fn value()->usize{3}}impl Z for <u16 as A>::Out{const C:usize=3;}impl<T:P>A for T{type Out=u8;}fn main(){check(<X<3>as B>::value()==3);}
",
        false,
    ),
    (
        "unused-result-bound-debt",
        r"trait B{fn value()->usize;}struct X<const N:usize>;trait A{type Out:Copy;}struct Bad;impl B for X<{<u8 as Z>::C}>{fn value()->usize{3}}trait Z{const C:usize;}impl Z for <u16 as A>::Out{const C:usize=3;}impl A for u16{type Out=u8;}impl A for u32{type Out=Bad;}fn main(){check(<X<3>as B>::value()==3);}
",
        false,
    ),
    (
        "lifetime-erased-subject-proof",
        r"trait B{fn value()->usize;}struct X<const N:usize>;trait A<'a>{type Out;}trait Z{const C:usize;}impl B for X<{<u8 as Z>::C}>{fn value()->usize{3}}impl Z for <&'static u16 as A<'static>>::Out{const C:usize=3;}impl<'a> A<'a> for &'a u16{type Out=u8;}fn main(){check(<X<3>as B>::value()==3);}
",
        true,
    ),
    (
        "invalid-unused-result-lifetime",
        r"trait B{fn value()->usize;}struct X<const N:usize>;trait A<'a>{type Out;}trait Z{const C:usize;}impl B for X<{<u8 as Z>::C}>{fn value()->usize{3}}impl Z for <&'static u16 as A<'static>>::Out{const C:usize=3;}impl<'a> A<'a> for &'a u16{type Out=u8;}fn bad<'a>(x:&'a u8)->&'static u8{x}fn main(){check(<X<3>as B>::value()==3);}
",
        false,
    ),
    (
        "projected-member-result-bound",
        r"trait B{fn value()->usize;}struct X<const N:usize>;trait A{type Out:P;const C:Self::Out;}trait P{}impl P for usize{}impl B for X<{<u8 as A>::C}>{fn value()->usize{3}}impl A for u8{type Out=usize;const C:usize=3;}fn main(){check(<X<3>as B>::value()==3);}
",
        true,
    ),
];
#[test]
fn projected_headers_match_independent_source_outcomes() -> Result<(), Box<dyn std::error::Error>> {
    for &(name, body, accepted) in CASES {
        let source = format!("{}{body}", native::CHECK);
        object_outcome(&format!("projected-header-{name}"), &source, accepted)?;
        let result = semantic::analyze(&source).and_then(|(mut report, mut facts)| {
            let entry =
                rustgm::sema::entry::select(&mut report, &facts).map_err(|e| e.to_string())?;
            rustgm::lower::instances::build(&report, &mut facts, entry)
                .map(|_| ())
                .map_err(|e| e.to_string())
        });
        assert_eq!(result.is_ok(), accepted, "{name}: {result:?}");
    }
    Ok(())
}
fn chain(depth: usize) -> Result<String, std::fmt::Error> {
    let mut source = "trait B{fn value()->usize;}struct X<const N:usize>;trait A{type Out;const C:Self::Out;}impl B for X<{<u8 as A>::C}>{fn value()->usize{3}}impl<T:P0>A for T{type Out=usize;const C:usize=3;}".to_owned();
    for i in 0..depth {
        write!(source, "trait P{i}{{}}impl<T:P{}>P{i} for T{{}}", i + 1)?;
    }
    write!(
        source,
        "trait P{depth}{{}}impl P{depth} for u8{{}}fn main(){{check(<X<3>as B>::value()==3);}}"
    )?;
    Ok(source)
}
#[test]
fn nested_projection_header_conditions_reach_complete_lowering()
-> Result<(), Box<dyn std::error::Error>> {
    for depth in [8, 32, 96] {
        let source = format!("{}{}", native::CHECK, chain(depth)?);
        object_outcome(&format!("projected-header-chain-{depth}"), &source, true)?;
        let (mut report, mut facts) = semantic::analyze(&source)?;
        let entry = rustgm::sema::entry::select(&mut report, &facts)?;
        rustgm::lower::instances::build(&report, &mut facts, entry)?;
    }
    Ok(())
}
#[test]
#[ignore = "requires the qualified CPPGM LowIR-to-native backend"]
fn projected_headers_preserve_native_values() -> Result<(), Box<dyn std::error::Error>> {
    let chain = chain(32)?;
    let mut cases: Vec<_> = CASES
        .iter()
        .filter(|(_, _, accepted)| *accepted)
        .map(|&(name, source, _)| (name, source, false))
        .collect();
    cases.push(("chain-32", &chain, false));
    native::check_cases("projected-headers", &cases)
}
