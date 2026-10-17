//! Qualified constant arguments retain selected owners and their dependency preparation.
use super::{native, object_outcome, semantic};

const CASES: &[(&str, &str, bool)] = &[
    (
        "qualified-local",
        "trait A{const C:usize=3;}impl A for u8{}struct X<const N:usize>([u8;N]);fn main(){let x:X<{<u8 as A>::C}>=X([7;3]);check(x.0[2]==7);}",
        true,
    ),
    (
        "qualified-override",
        "trait A{const C:usize=3;}impl A for u8{const C:usize=5;}struct X<const N:usize>([u8;N]);fn main(){let x:X<{<u8 as A>::C}>=X([7;5]);check(x.0[4]==7);}",
        true,
    ),
    (
        "qualified-function",
        "trait A{const C:usize=3;}impl A for u8{}fn f<const N:usize>()->usize{N}fn main(){check(f::<{<u8 as A>::C}>()==3);}",
        true,
    ),
    (
        "trait-const-argument",
        "trait A<const N:usize>{const C:usize=N;}impl<const N:usize>A<N>for u8{}struct X<const N:usize>([u8;N]);fn main(){let x:X<{<u8 as A<3>>::C}>=X([7;3]);check(x.0[2]==7);}",
        true,
    ),
    (
        "trait-type-argument",
        "trait A<T>{const C:usize=3;}impl A<u16>for u8{}struct X<const N:usize>([u8;N]);fn main(){let x:X<{<u8 as A<u16>>::C}>=X([7;3]);check(x.0[2]==7);}",
        true,
    ),
    (
        "generic-owner",
        "trait A{const C:usize=3;}struct S<T>(T);impl<T:Copy>A for S<T>{}struct X<const N:usize>([u8;N]);fn main(){let x:X<{<S<u8>as A>::C}>=X([7;3]);check(x.0[2]==7);}",
        true,
    ),
    (
        "transitive-value",
        "trait A{const C:usize=3;const D:usize=Self::C+1;}impl A for u8{}struct X<const N:usize>([u8;N]);fn main(){let x:X<{<u8 as A>::D}>=X([7;4]);check(x.0[3]==7);}",
        true,
    ),
    (
        "nested-argument",
        "trait A<const N:usize>{const C:usize=N;}trait B{const C:usize=3;}impl<const N:usize>A<N>for u8{}impl B for u16{}struct X<const N:usize>([u8;N]);fn main(){let x:X<{<u8 as A<{<u16 as B>::C}>>::C}>=X([7;3]);check(x.0[2]==7);}",
        true,
    ),
    (
        "missing-impl",
        "trait A{const C:usize=3;}struct X<const N:usize>([u8;N]);fn main(){let _x:X<{<u8 as A>::C}>=X([7;3]);}",
        false,
    ),
    (
        "missing-bound",
        "trait A{const C:usize=3;}struct S<T>(T);struct U;impl<T:Copy>A for S<T>{}struct X<const N:usize>([u8;N]);fn main(){let _x:X<{<S<U>as A>::C}>=X([7;3]);}",
        false,
    ),
    (
        "captured-type",
        "trait A{const C:usize=3;}struct S<T>(T);impl<T>A for S<T>{}struct X<const N:usize>([u8;N]);fn f<T>(){let _x:X<{<S<T>as A>::C}>=X([7;3]);}fn main(){}",
        false,
    ),
    (
        "later-declared-default",
        "struct X<const N:usize>([u8;N]);fn main(){let x:X<{<u8 as A>::D}>=X([7;4]);check(x.0[3]==7);}trait A{const C:usize=3;const D:usize=Self::C+1;}impl A for u8{}",
        true,
    ),
    (
        "primitive-shorthand",
        "trait A{const C:usize=3;}impl A for u8{}struct X<const N:usize>([u8;N]);fn main(){let x:X<{u8::C}>=X([7;3]);check(x.0[2]==7);}",
        true,
    ),
    (
        "ambiguous-shorthand",
        "trait A{const C:usize=3;}trait B{const C:usize=3;}impl A for u8{}impl B for u8{}struct X<const N:usize>([u8;N]);fn main(){let _x:X<{u8::C}>=X([7;3]);}",
        false,
    ),
    (
        "trait-without-self",
        "trait A{const C:usize=3;}impl A for u8{}struct X<const N:usize>([u8;N]);fn main(){let _x:X<{A::C}>=X([7;3]);}",
        false,
    ),
    (
        "explicit-projected-value-type",
        "trait A{type Out;const C:<Self as A>::Out;}impl A for u8{type Out=usize;const C:usize=3;}struct X<const N:usize>([u8;N]);fn main(){let x:X<{<u8 as A>::C}>=X([7;3]);check(x.0[2]==7);}",
        true,
    ),
    (
        "static-trait-argument",
        "trait A<'a>{const C:usize=3;}impl<'a>A<'a>for u8{}struct X<const N:usize>([u8;N]);fn main(){let x:X<{<u8 as A<'static>>::C}>=X([7;3]);check(x.0[2]==7);}",
        true,
    ),
    (
        "captured-lifetime",
        "trait A<'a>{const C:usize=3;}impl<'a>A<'a>for u8{}struct X<const N:usize>([u8;N]);fn f<'a>(){let _x:X<{<u8 as A<'a>>::C}>=X([7;3]);}fn main(){}",
        false,
    ),
];

#[test]
fn selected_arguments_match_qualified_outcomes() -> Result<(), Box<dyn std::error::Error>> {
    for &(name, body, accepted) in CASES {
        let source = format!("{}{body}", native::CHECK);
        object_outcome(&format!("argument-{name}"), &source, accepted)?;
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

#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn selected_arguments_preserve_native_values() -> Result<(), Box<dyn std::error::Error>> {
    let cases = CASES
        .iter()
        .filter(|(_, _, accepted)| *accepted)
        .map(|&(name, source, _)| (name, source, false))
        .collect::<Vec<_>>();
    native::check_cases("associated-arguments", &cases)
}
