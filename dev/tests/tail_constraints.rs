//! Unsizing eligibility never pays unrelated mutability, lifetime or type debts.
use std::{path::Path, process::Command};

const CASES: &[(&str, bool, &str)] = &[
    (
        "by-value-unsizing",
        false,
        "struct S<T:?Sized>{tail:T}fn f(){let _s:S<[u8]>=S{tail:[3,5]};}",
    ),
    (
        "projected-multiple-types",
        true,
        "trait A<P>{type Out:?Sized;}impl A<u8> for u8{type Out=[u16;3];}impl A<u16> for u16{type Out=[u16];}struct S<P,T:A<P>>{tail:T::Out}fn f(s:&S<u8,u8>)->&S<u16,u16>{s}",
    ),
    (
        "projected-type-and-const",
        true,
        "trait A<const N:usize>{type Out:?Sized;}impl A<2> for u8{type Out=[u16;3];}impl A<7> for u16{type Out=[u16];}struct S<T:A<N>,const N:usize>{tail:T::Out}fn f(s:&S<u8,2>)->&S<u16,7>{s}",
    ),
    (
        "projected-const-prefix",
        false,
        "trait A<const N:usize>{type Out:?Sized;}impl A<2> for u8{type Out=[u16;3];}impl A<7> for u16{type Out=[u16];}struct S<T:A<N>,const N:usize>{prefix:[u8;N],tail:T::Out}fn f(s:&S<u8,2>)->&S<u16,7>{s}",
    ),
    (
        "projected-shared-invariance",
        false,
        "trait A{type Out:?Sized;}impl<'a>A for &'a u8{type Out=[&'a u8;2];}impl<'a>A for &'a u16{type Out=[&'a u8];}struct S<T:A>{tail:T::Out}fn f<'a,'b:'a>(s:&'a S<&'b u8>)->&'a S<&'a u16>{s}",
    ),
    (
        "projected-shared-lifetime-control",
        true,
        "trait A{type Out:?Sized;}impl<'a>A for &'a u8{type Out=[&'a u8;2];}impl<'a>A for &'a u16{type Out=[&'a u8];}struct S<T:A>{tail:T::Out}fn f<'a>(s:&'a S<&'a u8>)->&'a S<&'a u16>{s}",
    ),
    (
        "projected-mutable-lifetime-control",
        true,
        "trait A{type Out:?Sized;}impl<'a>A for &'a u8{type Out=[&'a u8;2];}impl<'a>A for &'a u16{type Out=[&'a u8];}struct S<T:A>{tail:T::Out}fn f<'a>(s:&'a mut S<&'a u8>)->&'a mut S<&'a u16>{s}",
    ),
    (
        "projected-invariance",
        false,
        "trait A{type Out:?Sized;}impl<'a>A for &'a u8{type Out=[&'a u8;2];}impl<'a>A for &'a u16{type Out=[&'a u8];}struct S<T:A>{tail:T::Out}fn f<'a,'b:'a>(s:&'a mut S<&'b u8>)->&'a mut S<&'a u16>{s}",
    ),
    (
        "projected-elements",
        false,
        "trait A{type Out:?Sized;}impl A for [u8;2]{type Out=[u16;3];}impl A for [u8]{type Out=[u8];}struct S<T:A+?Sized>{tail:T::Out}fn f(s:&S<[u8;2]>)->&S<[u8]>{s}",
    ),
    (
        "projected-sized-field",
        false,
        "trait A{type Out:?Sized;}impl A for [u8;2]{type Out=u8;}impl A for [u8]{type Out=u8;}struct S<T:A+?Sized>{tail:T::Out}fn f(s:&S<[u8;2]>)->&S<[u8]>{s}",
    ),
    (
        "projected-marker-change",
        true,
        "trait A{type Out:?Sized;}impl A for u8{type Out=[u8;2];}impl A for u16{type Out=[u8];}struct S<T:A>{tail:T::Out}fn f(s:&S<u8>)->&S<u16>{s}",
    ),
    (
        "projected-prefix-use",
        false,
        "trait A{type Out:?Sized;}impl A for [u8;2]{type Out=[u8;2];}impl A for [u8]{type Out=[u8];}struct S<T:A+?Sized>{prefix:*const T,tail:T::Out}fn f(s:&S<[u8;2]>)->&S<[u8]>{s}",
    ),
    (
        "struct-deref-unsize",
        false,
        "struct S<T:?Sized>(T);fn f<'a>(s:&'a &'a S<[u8;2]>)->&'a S<[u8]>{s}",
    ),
    (
        "array-deref-unsize",
        false,
        "fn f<'a>(s:&'a &'a [u8;2])->&'a [u8]{s}",
    ),
    (
        "prefix-control",
        true,
        "struct S<T:?Sized>{prefix:*const u8,tail:T}fn f(s:&S<[u8;2]>)->&S<[u8]>{s}",
    ),
    (
        "prefix-use",
        false,
        "struct S<T:?Sized>{prefix:*const T,tail:T}fn f(s:&S<[u8;2]>)->&S<[u8]>{s}",
    ),
    (
        "nested-control",
        true,
        "struct S<P,T:?Sized>{prefix:P,tail:T}struct W<T:?Sized>{tail:S<u8,T>}fn f(s:&W<[u8;2]>)->&W<[u8]>{s}",
    ),
    (
        "nested-prefix",
        false,
        "struct S<P,T:?Sized>{prefix:*const P,tail:T}struct W<T:?Sized>{tail:S<T,T>}fn f(s:&W<[u8;2]>)->&W<[u8]>{s}",
    ),
    (
        "element-control",
        true,
        "struct S<T:?Sized>(T);fn f(s:&S<[u8;2]>)->&S<[u8]>{s}",
    ),
    (
        "wrong-element",
        false,
        "struct S<T:?Sized>(T);fn f(s:&S<[u8;2]>)->&S<[u16]>{s}",
    ),
    (
        "weaken",
        true,
        "struct S<T:?Sized>(T);fn f(s:&mut S<[u8;2]>)->&S<[u8]>{s}",
    ),
    (
        "strengthen",
        false,
        "struct S<T:?Sized>(T);fn f(s:&S<[u8;2]>)->&mut S<[u8]>{s}",
    ),
    (
        "same-lifetime",
        true,
        "struct S<T:?Sized>(T);fn f<'a>(s:&'a S<[u8;2]>)->&'a S<[u8]>{s}",
    ),
    (
        "escape",
        false,
        "struct S<T:?Sized>(T);fn f(s:&S<[u8;2]>)->&'static S<[u8]>{s}",
    ),
    (
        "direct-layer",
        true,
        "struct S<T:?Sized>(T);fn f(s:&S<[u8;2]>)->&S<[u8]>{s}",
    ),
    (
        "nested-layer",
        false,
        "struct S<T:?Sized>(T);fn f<'a>(s:&'a &'a S<[u8;2]>)->&'a &'a S<[u8]>{s}",
    ),
    ("tuple-reference", true, "fn f(s:&(u8,[u8]))->&(u8,[u8]){s}"),
    ("tuple-unsize", false, "fn f(s:&(u8,[u8;2]))->&(u8,[u8]){s}"),
    (
        "covariance",
        true,
        "struct S<T:?Sized>(T);fn f<'a,'b:'a>(s:&'a S<[&'b u8;2]>)->&'a S<[&'a u8]>{s}",
    ),
    (
        "invariance",
        false,
        "struct S<T:?Sized>(T);fn f<'a,'b:'a>(s:&'a mut S<[&'b u8;2]>)->&'a mut S<[&'a u8]>{s}",
    ),
];

#[test]
fn structural_unsizing_requires_independent_argument_and_reference_proofs()
-> Result<(), Box<dyn std::error::Error>> {
    let folder = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../obj/pa22-source/constraints-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    for &(name, accepted, source) in CASES {
        let input = folder.join(format!("{name}.rs"));
        std::fs::write(&input, format!("{source}fn main(){{}}"))?;
        let oracle = Command::new("rustc")
            .args([
                "+1.98.0",
                "--edition=2024",
                "--crate-name=probe",
                "--emit=obj",
            ])
            .arg(&input)
            .arg("-o")
            .arg(input.with_extension("o"))
            .output()?;
        std::fs::write(input.with_extension("rustc.stderr"), &oracle.stderr)?;
        assert_eq!(
            oracle.status.code(),
            Some(i32::from(!accepted)),
            "{name}: {}",
            String::from_utf8_lossy(&oracle.stderr)
        );
        let candidate = Command::new(env!("CARGO_BIN_EXE_rustgm"))
            .arg("--emit-lowir")
            .arg(&input)
            .output()?;
        std::fs::write(input.with_extension("candidate.stderr"), &candidate.stderr)?;
        std::fs::write(input.with_extension("lowir"), &candidate.stdout)?;
        assert_eq!(
            candidate.status.code(),
            Some(i32::from(!accepted)),
            "{name}: {}",
            String::from_utf8_lossy(&candidate.stderr)
        );
        assert_eq!(candidate.stderr.is_empty(), accepted, "{name}");
    }
    Ok(())
}
