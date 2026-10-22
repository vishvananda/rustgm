//! Independent unused-declaration coherence controls and indexed-work invariants.
use crate::qualify as rustc;
use crate::semantic;
use std::fmt::Write;

const HEADERS: &[(&str, &str, bool)] = &[
    (
        "duplicate-plain",
        "struct S;impl S{fn f(){}}impl S{fn f(){}}fn main(){}",
        false,
    ),
    (
        "different-names",
        "struct S;impl S{fn f(){}}impl S{fn g(){}}fn main(){}",
        true,
    ),
    (
        "different-owners",
        "struct S;struct R;impl S{fn f(){}}impl R{fn f(){}}fn main(){}",
        true,
    ),
    (
        "duplicate-generics",
        "struct S<T>(T);impl<T>S<T>{fn f(){}}impl<U>S<U>{fn f(){}}fn main(){}",
        false,
    ),
    (
        "generic-specific",
        "struct S<T>(T);impl<T>S<T>{fn f(){}}impl S<u8>{fn f(){}}fn main(){}",
        false,
    ),
    (
        "different-types",
        "struct S<T>(T);impl S<u8>{fn f(){}}impl S<u16>{fn f(){}}fn main(){}",
        true,
    ),
    (
        "different-consts",
        "struct S<const N:usize>([u8;N]);impl S<1>{fn f(){}}impl S<2>{fn f(){}}fn main(){}",
        true,
    ),
    (
        "generic-const-specific",
        "struct S<const N:usize>([u8;N]);impl<const N:usize>S<N>{fn f(){}}impl S<2>{fn f(){}}fn main(){}",
        false,
    ),
    (
        "repeated-type-disjoint",
        "struct S<T,U>(T,U);impl<T>S<T,T>{fn f(){}}impl S<u8,u16>{fn f(){}}fn main(){}",
        true,
    ),
    (
        "repeated-type-overlap",
        "struct S<T,U>(T,U);impl<T>S<T,T>{fn f(){}}impl S<u8,u8>{fn f(){}}fn main(){}",
        false,
    ),
    (
        "repeated-const-disjoint",
        "struct S<const N:usize,const M:usize>([u8;N],[u8;M]);impl<const N:usize>S<N,N>{fn f(){}}impl S<1,2>{fn f(){}}fn main(){}",
        true,
    ),
    (
        "repeated-const-overlap",
        "struct S<const N:usize,const M:usize>([u8;N],[u8;M]);impl<const N:usize>S<N,N>{fn f(){}}impl S<2,2>{fn f(){}}fn main(){}",
        false,
    ),
    (
        "occurs-disjoint",
        "struct S<T,U>(T,U);impl<T>S<T,T>{fn f(){}}impl<U>S<U,[U;1]>{fn f(){}}fn main(){}",
        true,
    ),
    (
        "nested-overlap",
        "struct S<T>(T);impl<T>S<T>{fn f(){}}impl<U>S<S<U>>{fn f(){}}fn main(){}",
        false,
    ),
    (
        "sized-str-disjoint",
        "struct S<T:?Sized>(*const T);impl<T>S<T>{fn f(){}}impl S<str>{fn f(){}}fn main(){}",
        true,
    ),
    (
        "copy-local-disjoint",
        "struct S<T>(T);struct N(u8);impl<T:Copy>S<T>{fn f(){}}impl S<N>{fn f(){}}fn main(){}",
        true,
    ),
    (
        "copy-clone-overlap",
        "struct S<T>(T);impl<T:Copy>S<T>{fn f(){}}impl<T:Clone>S<T>{fn f(){}}fn main(){}",
        false,
    ),
    (
        "copy-mutable-overlap",
        "struct S<T>(T);impl<T:Copy>S<T>{fn f(){}}impl<'a,T>S<&'a mut T>{fn f(){}}fn main(){}",
        false,
    ),
    (
        "copy-tuple-overlap",
        "struct S<T>(T);impl<T:Copy>S<T>{fn f(){}}impl<'a,T,U>S<(T,&'a mut U)>{fn f(){}}fn main(){}",
        false,
    ),
    (
        "copy-derived-overlap",
        "struct S<T>(T);#[derive(Clone,Copy)]struct D<T>(T);impl<T:Copy>S<T>{fn f(){}}impl<U>S<D<U>>{fn f(){}}fn main(){}",
        false,
    ),
    (
        "copy-nonderived-disjoint",
        "struct S<T>(T);struct D<T>(T);impl<T:Copy>S<T>{fn f(){}}impl<U>S<D<U>>{fn f(){}}fn main(){}",
        true,
    ),
    (
        "method-bound-ignored",
        "struct S<T>(T);struct N(u8);impl<T>S<T>{fn f()where T:Copy{}}impl S<N>{fn f(){}}fn main(){}",
        false,
    ),
    (
        "receiver-ignored",
        "struct S;impl S{fn f(&self){}}impl S{fn f(&mut self){}}fn main(){}",
        false,
    ),
    (
        "privacy-ignored",
        "struct S;impl S{pub fn f(){}}mod m{impl super::S{fn f(){}}}fn main(){}",
        false,
    ),
    (
        "lifetime-ignored",
        "struct S<'a>(&'a u8);impl<'a>S<'a>{fn f(){}}impl S<'static>{fn f(){}}fn main(){}",
        false,
    ),
    (
        "fn-binder-overlap",
        "struct S<T>(T);impl S<fn(&u8)>{fn f(){}}impl S<fn(&'static u8)>{fn f(){}}fn main(){}",
        false,
    ),
];

#[test]
fn unused_inherent_declarations_require_disjoint_headers() -> Result<(), Box<dyn std::error::Error>>
{
    for &(name, source, accepted) in HEADERS {
        check(name, source, accepted)?;
    }
    Ok(())
}

fn check(name: &str, source: &str, accepted: bool) -> Result<(), Box<dyn std::error::Error>> {
    rustc::qualify(&format!("inherent-overlap-{name}"), source, accepted)?;
    let result = semantic::analyze(source);
    if accepted {
        result.map_err(|error| format!("{name}: {error}"))?;
    } else {
        let error = result
            .err()
            .ok_or_else(|| format!("{name}: accepted duplicate methods"))?;
        assert!(
            error.contains("duplicate inherent method"),
            "{name}: {error}"
        );
    }
    Ok(())
}

#[test]
fn compatible_worlds_differ_from_body_copy_proofs() -> Result<(), Box<dyn std::error::Error>> {
    for (name, subject, copy, clone_, sized) in [
        ("str", "str", true, true, true),
        ("slice", "[u8]", true, true, true),
        ("mutable-scalar", "&'static mut u8", false, false, false),
        ("mutable-local", "&'static mut N", true, true, false),
        ("tuple-local", "(N,)", false, false, false),
        ("array-local", "[N;1]", false, false, false),
        (
            "tuple-local-mutable",
            "(N,&'static mut u8)",
            false,
            false,
            false,
        ),
        ("derived-local", "D<N>", true, true, false),
        ("derived-mutable", "D<&'static mut u8>", false, false, false),
        ("mutable-derived", "&'static mut D<u8>", true, true, false),
        ("pointer-local", "*const N", false, false, false),
    ] {
        for (bound, accepted) in [("Copy", copy), ("Clone", clone_), ("Sized", sized)] {
            let source = format!(
                "struct S<T:?Sized>(*const T);struct N;#[derive(Clone,Copy)]struct D<T>(T);impl<T:{bound}>S<T>{{fn f(){{}}}}impl S<{subject}>{{fn f(){{}}}}fn main(){{}}"
            );
            check(&format!("{bound}-{name}"), &source, accepted)?;
        }
    }
    Ok(())
}

#[test]
fn concrete_shapes_skip_pairs_and_generic_pairs_share_across_names()
-> Result<(), Box<dyn std::error::Error>> {
    let mut source = "struct S<const N:usize>;fn main(){}".to_owned();
    for index in 0..1024 {
        write!(source, "impl S<{index}>{{fn f(){{}}}}")?;
    }
    rustc::qualify("inherent-closed-shapes", &source, true)?;
    let (_, facts) = semantic::analyze(&source)?;
    assert_eq!(facts.methods.overlap_pairs, 0);
    assert_eq!(facts.methods.overlap_steps, 0);

    let mut source = "struct S<T>(T);struct N;fn main(){}".to_owned();
    for header in ["impl<T:Copy>S<T>", "impl S<N>"] {
        write!(source, "{header}{{")?;
        for index in 0..128 {
            write!(source, "fn f{index}(){{}}")?;
        }
        source.push('}');
    }
    rustc::qualify("inherent-shared-owner-probes", &source, true)?;
    let (_, facts) = semantic::analyze(&source)?;
    assert_eq!(facts.methods.overlap_pairs, 1);
    assert!(facts.methods.overlap_steps < 128);
    Ok(())
}
