//! Independently qualified declarations exercise early size queries through real source.
#[path = "helpers/qualify.rs"]
mod rustc;
#[path = "support/semantic.rs"]
mod semantic;
use std::fmt::Write as _;

#[test]
fn forward_and_recursive_projections_wait_for_size_evidence()
-> Result<(), Box<dyn std::error::Error>> {
    for (name, source) in [
        (
            "forward",
            r"
            trait A { type Out; }
            impl<T> A for T { type Out = u8; }
            type X = <S as A>::Out;
            struct S { x: u16 }
            fn f(_: X) {}
        ",
        ),
        (
            "mutual",
            r"
            trait A { type Out: ?Sized; }
            impl<T> A for T { type Out = u8; }
            struct S { x: <U as A>::Out }
            struct U { x: <S as A>::Out }
        ",
        ),
        (
            "self",
            r"
            trait A { type Out: ?Sized; }
            impl<T> A for T { type Out = u8; }
            struct S { x: <S as A>::Out }
        ",
        ),
        (
            "function",
            r"
            trait A { type Out; }
            impl<T> A for T { type Out = fn() -> T; }
            struct S { x: <S as A>::Out }
        ",
        ),
        (
            "pointer",
            r"
            trait A { type Out; }
            impl<T> A for T { type Out = *const T; }
            struct S { x: <S as A>::Out }
        ",
        ),
    ] {
        rustc::qualify(&format!("size-ready-{name}"), source, true)?;
        semantic::analyze(source)?;
    }
    Ok(())
}

#[test]
fn source_field_dependencies_keep_long_forward_chains_valid()
-> Result<(), Box<dyn std::error::Error>> {
    for count in [16, 64] {
        let mut source = String::from(
            r"
            trait A { type Out; }
            impl<T> A for T { type Out = u8; }
            type X = <S0 as A>::Out;
            fn f(_: X) {}
        ",
        );
        for index in 0..count - 1 {
            writeln!(source, "struct S{index} {{ x: S{} }}", index + 1)?;
        }
        writeln!(source, "struct S{} {{ x: u8 }}", count - 1)?;
        rustc::qualify(&format!("size-forward-{count}"), &source, true)?;
        semantic::analyze(&source)?;
    }
    Ok(())
}

const TAIL_CASES: &[(&str, bool, &str)] = &[
    (
        "concrete",
        true,
        r"
            struct S { tag: u16, tail: str }
            struct U { tag: u8, tail: S }
        ",
    ),
    (
        "generic",
        true,
        r"
            struct S<T: ?Sized> { tag: u16, tail: T }
            struct U { tag: u8, tail: S<str> }
            struct V { tag: u8, tail: S<u32> }
        ",
    ),
    (
        "projected",
        true,
        r"
            trait A { type Out: ?Sized; }
            impl A for u8 { type Out = str; }
            struct S { tail: <u8 as A>::Out }
        ",
    ),
    (
        "packed",
        true,
        r"
            #[repr(packed)]
            struct S { tag: u16, tail: [u8] }
            #[repr(packed)]
            struct U<T: Copy> { tag: u16, tail: [T] }
        ",
    ),
    (
        "transparent",
        true,
        r"
            struct Z;
            #[repr(transparent)]
            struct S { z: Z, tail: str }
        ",
    ),
    (
        "prefix",
        false,
        r"
            struct S<T: ?Sized> { tail: T }
            struct U { bad: S<str>, tail: u8 }
        ",
    ),
    (
        "enum",
        false,
        r"
            enum E { Bad(str) }
        ",
    ),
    (
        "array",
        false,
        r"
            struct S { tail: str }
            struct U { bad: [S; 0] }
        ",
    ),
    (
        "packed-unknown",
        false,
        r"
            #[repr(packed)]
            struct S<T: ?Sized> { tail: T }
        ",
    ),
    (
        "packed-drop",
        false,
        r"
            struct D;
            impl Drop for D { fn drop(&mut self) {} }
            #[repr(packed)]
            struct S { tail: [D] }
        ",
    ),
    (
        "transparent-two",
        false,
        r"
            #[repr(transparent)]
            struct S { tag: u8, tail: str }
        ",
    ),
    (
        "by-value",
        false,
        r"
            struct S { tail: str }
            fn f(_: S) {}
        ",
    ),
    (
        "disjoint-sized-blanket",
        true,
        r"
            struct S { tail: str }
            trait A {}
            impl<T> A for T {}
            impl A for S {}
        ",
    ),
    (
        "disjoint-explicit-sized",
        true,
        r"
            struct S { tail: str }
            trait A {}
            impl<T: ?Sized> A for T where T: Sized {}
            impl A for S {}
        ",
    ),
    (
        "disjoint-instantiated-tail",
        true,
        r"
            struct S<T: ?Sized> { tail: T }
            trait A {}
            impl<T> A for T {}
            impl A for S<[u8]> {}
        ",
    ),
    (
        "overlapping-relaxed-blanket",
        false,
        r"
            struct S { tail: str }
            trait A {}
            impl<T: ?Sized> A for T {}
            impl A for S {}
        ",
    ),
    (
        "overlapping-sized-tail",
        false,
        r"
            struct S { tail: u8 }
            trait A {}
            impl<T> A for T {}
            impl A for S {}
        ",
    ),
    (
        "overlapping-projected-tail",
        false,
        r"
            trait Value { type Out: ?Sized; }
            impl Value for u8 { type Out = u8; }
            struct S<T: Value> { tail: T::Out }
            trait A {}
            impl<T> A for T {}
            impl A for S<u8> {}
        ",
    ),
];

#[test]
fn unsized_tails_preserve_field_and_representation_requirements()
-> Result<(), Box<dyn std::error::Error>> {
    for &(name, accepted, source) in TAIL_CASES {
        rustc::qualify(&format!("size-tail-{name}"), source, accepted)?;
        let result = semantic::analyze(source);
        assert_eq!(result.is_ok(), accepted, "{name}: {:?}", result.err());
    }
    Ok(())
}

#[test]
fn early_associated_constants_recheck_nominal_size_after_fields_complete()
-> Result<(), Box<dyn std::error::Error>> {
    for (name, accepted, source) in [
        (
            "sized",
            true,
            r"
            struct Array<const N: usize>([u8; N]);
            struct S { a: Array<{ <S as A>::V }>, tail: u8 }
            trait A { const V: usize; }
            impl<T> A for T { const V: usize = 1; }
        ",
        ),
        (
            "unsized",
            false,
            r"
            struct Array<const N: usize>([u8; N]);
            struct S { a: Array<{ <S as A>::V }>, tail: str }
            trait A { const V: usize; }
            impl<T> A for T { const V: usize = 1; }
        ",
        ),
        (
            "relaxed",
            true,
            r"
            struct Array<const N: usize>([u8; N]);
            struct S { a: Array<{ <S as A>::V }>, tail: str }
            trait A { const V: usize; }
            impl<T: ?Sized> A for T { const V: usize = 1; }
        ",
        ),
        (
            "trait-argument",
            false,
            r"
            struct Array<const N: usize>([u8; N]);
            struct S { a: Array<{ <() as A<S>>::V }>, tail: str }
            trait A<T> { const V: usize; }
            impl<T> A<T> for () { const V: usize = 1; }
        ",
        ),
        (
            "trait-argument-relaxed",
            true,
            r"
            struct Array<const N: usize>([u8; N]);
            struct S { a: Array<{ <() as A<S>>::V }>, tail: str }
            trait A<T: ?Sized> { const V: usize; }
            impl<T: ?Sized> A<T> for () { const V: usize = 1; }
        ",
        ),
    ] {
        rustc::qualify(&format!("size-constant-{name}"), source, accepted)?;
        let result = semantic::analyze(source);
        assert_eq!(result.is_ok(), accepted, "{name}: {:?}", result.err());
    }
    Ok(())
}

#[test]
fn projected_storage_sizes_use_the_instantiated_tail() -> Result<(), Box<dyn std::error::Error>> {
    let prefix = r"
        trait Value { type Out: ?Sized; }
        impl Value for u8 { type Out = u16; }
        impl Value for u16 { type Out = str; }
        struct S<T: Value> { tag: u8, tail: T::Out }
    ";
    for (name, accepted, body) in [
        ("field", true, "struct P { first: S<u8>, last: u8 }"),
        (
            "field-unsized",
            false,
            "struct P { first: S<u16>, last: u8 }",
        ),
        ("array", true, "struct P { values: [S<u8>; 2] }"),
        ("array-unsized", false, "struct P { values: [S<u16>; 0] }"),
        ("local", true, "fn f() { let value: S<u8>; }"),
        ("local-unsized", false, "fn f() { let value: S<u16>; }"),
        ("bound", true, "fn size<T>() {} fn f() { size::<S<u8>>(); }"),
        (
            "bound-unsized",
            false,
            "fn size<T>() {} fn f() { size::<S<u16>>(); }",
        ),
        (
            "assumed",
            true,
            "fn f<T: Value>() where T::Out: Sized { let _: [S<T>; 2]; }",
        ),
        ("relaxed", false, "fn f<T: Value>() { let _: [S<T>; 2]; }"),
        ("pointer-array", true, "fn f(_: *const [S<u8>; 2]) {}"),
        (
            "pointer-array-unsized",
            false,
            "fn f(_: *const [S<u16>; 0]) {}",
        ),
        ("function-array", true, "fn f(_: fn(&[S<u8>; 2])) {}"),
        (
            "function-array-unsized",
            false,
            "fn f(_: fn(&[S<u16>; 0])) {}",
        ),
        ("closed-bound", true, "fn f() where S<u8>: Sized {}"),
        (
            "closed-bound-unsized",
            false,
            "fn f() where S<u16>: Sized {}",
        ),
    ] {
        let source = format!("{prefix}{body}");
        rustc::qualify(&format!("size-projected-{name}"), &source, accepted)?;
        let result = semantic::analyze(&source);
        assert_eq!(result.is_ok(), accepted, "{name}: {:?}", result.err());
    }
    Ok(())
}

#[test]
fn projected_size_proofs_preserve_callable_and_clause_lifetimes()
-> Result<(), Box<dyn std::error::Error>> {
    let prefix = r"
        trait Value<'a> { type Out: ?Sized; }
        impl<'a> Value<'a> for u8 { type Out = &'a u8; }
        impl<'a> Value<'a> for u16 { type Out = str; }
        struct S<'a, T: Value<'a>> { tail: T::Out }
    ";
    for (name, accepted, body) in [
        (
            "callable",
            true,
            "fn f(_: for<'a> fn(&'a [S<'a, u8>; 2])) {}",
        ),
        (
            "callable-conditional",
            true,
            "fn f(_: for<'a> fn(&'a [S<'a, u16>; 0])) {}",
        ),
        (
            "nested",
            true,
            "fn f(_: fn(for<'a> fn(&'a [S<'a, u8>; 2]))) {}",
        ),
        (
            "nested-conditional",
            true,
            "fn f(_: fn(for<'a> fn(&'a [S<'a, u16>; 0]))) {}",
        ),
        ("clause", true, "fn f() where for<'a> S<'a, u8>: Sized {}"),
        (
            "clause-conditional",
            true,
            "fn f() where for<'a> S<'a, u16>: Sized {}",
        ),
        (
            "owner",
            true,
            "fn f<'a>(_: &'a u8, _: fn(&[S<'a, u8>; 2])) {}",
        ),
        (
            "owner-unsized",
            false,
            "fn f<'a>(_: &'a u8, _: fn(&[S<'a, u16>; 0])) {}",
        ),
        (
            "local-unsized",
            false,
            "fn f<'a>() { let value: [S<'a, u16>; 0]; }",
        ),
        (
            "clause-use",
            false,
            "fn f() where for<'a> S<'a, u16>: Sized {} fn g() { f(); }",
        ),
        (
            "closed-callable",
            false,
            "fn f(_: fn(&[S<'static, u16>; 0])) {}",
        ),
    ] {
        let source = format!("{prefix}{body}");
        rustc::qualify(&format!("size-scope-{name}"), &source, accepted)?;
        let result = semantic::analyze(&source);
        assert_eq!(result.is_ok(), accepted, "{name}: {:?}", result.err());
    }
    Ok(())
}
