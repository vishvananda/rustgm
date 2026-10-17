//! Early shape values require checked member contracts before method conformance.
use super::{native, object_outcome, semantic};

const CASES: &[(&str, &str, bool)] = &[
    (
        "v1-signature",
        "trait A{const C:usize=3;}impl A for u8{}struct X<const N:usize>([u8;N]);fn f()->X<{<u8 as A>::C}>{X([7;3])}fn main(){check(f().0[2]==7);}",
        true,
    ),
    (
        "v1-signature-override",
        "trait A{const C:usize=3;}impl A for u8{const C:usize=3;}struct X<const N:usize>([u8;N]);fn f()->X<{<u8 as A>::C}>{X([7;3])}fn main(){check(f().0[2]==7);}",
        true,
    ),
    (
        "v1-signature-late-impl",
        "trait A{const C:usize=3;}struct X<const N:usize>([u8;N]);fn f()->X<{<u8 as A>::C}>{X([7;3])}fn main(){check(f().0[2]==7);}\n\nimpl A for u8{}",
        true,
    ),
    (
        "v1-signature-wrong-type",
        "trait A{const C:usize=3;}impl A for u8{const C:u8=3;}struct X<const N:usize>([u8;N]);fn f()->X<{<u8 as A>::C}>{X([7;3])}fn main(){check(f().0[2]==7);}",
        false,
    ),
    (
        "v1-signature-missing-method",
        "trait A{fn required();const C:usize=3;}impl A for u8{}struct X<const N:usize>([u8;N]);fn f()->X<{<u8 as A>::C}>{X([7;3])}fn main(){check(f().0[2]==7);}",
        false,
    ),
    (
        "v1-signature-overlap",
        "trait A{const C:usize=3;}impl A for u8{}struct X<const N:usize>([u8;N]);fn f()->X<{<u8 as A>::C}>{X([7;3])}fn main(){check(f().0[2]==7);}\n\nimpl A for u8{const C:usize=3;}",
        false,
    ),
    (
        "v1-default-argument",
        "trait A{const C:usize=3;}impl A for u8{}struct X<const N:usize={<u8 as A>::C}>([u8;N]);fn main(){let x:X=X([7;3]);check(x.0[2]==7);}",
        true,
    ),
    (
        "v1-default-argument-override",
        "trait A{const C:usize=3;}impl A for u8{const C:usize=3;}struct X<const N:usize={<u8 as A>::C}>([u8;N]);fn main(){let x:X=X([7;3]);check(x.0[2]==7);}",
        true,
    ),
    (
        "v1-default-argument-late-impl",
        "trait A{const C:usize=3;}struct X<const N:usize={<u8 as A>::C}>([u8;N]);fn main(){let x:X=X([7;3]);check(x.0[2]==7);}\n\nimpl A for u8{}",
        true,
    ),
    (
        "v1-default-argument-wrong-type",
        "trait A{const C:usize=3;}impl A for u8{const C:u8=3;}struct X<const N:usize={<u8 as A>::C}>([u8;N]);fn main(){let x:X=X([7;3]);check(x.0[2]==7);}",
        false,
    ),
    (
        "v1-default-argument-missing-method",
        "trait A{fn required();const C:usize=3;}impl A for u8{}struct X<const N:usize={<u8 as A>::C}>([u8;N]);fn main(){let x:X=X([7;3]);check(x.0[2]==7);}",
        false,
    ),
    (
        "v1-default-argument-overlap",
        "trait A{const C:usize=3;}impl A for u8{}struct X<const N:usize={<u8 as A>::C}>([u8;N]);fn main(){let x:X=X([7;3]);check(x.0[2]==7);}\n\nimpl A for u8{const C:usize=3;}",
        false,
    ),
    (
        "v3-constant-type",
        "trait A{const C:usize=3;} impl A for u8{} struct V<const N:usize>([u8;N]); trait B{const V:V<{<u8 as A>::C}>;} impl B for u16{const V:V<3>=V([7;3]);} fn main(){check(<u16 as B>::V.0[2]==7);}",
        true,
    ),
    (
        "v3-method-signature",
        "trait A{const C:usize=3;} impl A for u8{} struct V<const N:usize>([u8;N]); trait B{fn f()->V<{<u8 as A>::C}>;} impl B for u16{fn f()->V<3>{V([7;3])}} fn main(){check(<u16 as B>::f().0[2]==7);}",
        true,
    ),
    (
        "v3-same-impl-method",
        "struct V<const N:usize>([u8;N]); trait A{const C:usize=3;fn f()->V<{<u8 as A>::C}>;} impl A for u8{fn f()->V<3>{V([7;3])}} fn main(){check(<u8 as A>::f().0[2]==7);}",
        true,
    ),
    (
        "v3-default-override-breaks-cycle",
        "struct V<const N:usize>([u8;N]); trait A{const C:usize=<u16 as B>::C;} trait B{const C:usize=<u8 as A>::C;} impl A for u8{const C:usize=3;} impl B for u16{} fn f()->V<{<u16 as B>::C}>{V([7;3])} fn main(){check(f().0[2]==7);}",
        true,
    ),
    (
        "v3-selected-value-cycle",
        "struct V<const N:usize>([u8;N]); trait A{const C:usize=<u16 as B>::C;} trait B{const C:usize=<u8 as A>::C;} impl A for u8{} impl B for u16{} fn f()->V<{<u16 as B>::C}>{V([7;3])} fn main(){check(f().0[2]==7);}",
        false,
    ),
    (
        "v3-unused-invalid-method",
        "trait A{const C:usize=3;} impl A for u8{} struct V<const N:usize>([u8;N]); trait B{fn f()->V<{<u8 as A>::C}>;} impl B for u16{fn f()->V<4>{V([7;4])}} fn main(){}",
        false,
    ),
    (
        "v3-unused-invalid-constant",
        "trait A{const C:usize=3;} impl A for u8{} struct V<const N:usize>([u8;N]); trait B{const V:V<{<u8 as A>::C}>;} impl B for u16{const V:V<4>=V([7;4]);} fn main(){}",
        false,
    ),
    (
        "v6-generic-owner-chain",
        "struct X<const N:usize>([u8;N]); trait A{const C:usize;const D:usize;}struct S<const N:usize>;impl<const N:usize>A for S<N>{const C:usize=Self::D;const D:usize=N;}fn f()->X<{<S<3>as A>::C}>{X([7;3])}fn main(){check(f().0[2]==7);}",
        true,
    ),
    (
        "v6-own-associated-type-default",
        "struct X<const N:usize>([u8;N]); trait A{type Out;const C:usize=3;}impl A for u8{type Out=X<{<u8 as A>::C}>;}fn main(){let x:<u8 as A>::Out=X([7;3]);check(x.0[2]==7);}",
        true,
    ),
    (
        "v6-own-associated-type-override",
        "struct X<const N:usize>([u8;N]); trait A{type Out;const C:usize;}impl A for u8{type Out=X<{<u8 as A>::C}>;const C:usize=3;}fn main(){let x:<u8 as A>::Out=X([7;3]);check(x.0[2]==7);}",
        true,
    ),
    (
        "v6-projected-type-with-dependent-other",
        "struct X<const N:usize>([u8;N]); trait A{type Out;type Other;const C:Self::Out;}impl A for u8{type Out=usize;type Other=X<{<u8 as A>::C}>;const C:usize=3;}fn main(){let x:<u8 as A>::Other=X([7;3]);check(x.0[2]==7);}",
        true,
    ),
    (
        "v6-generic-bound",
        "struct X<const N:usize>([u8;N]); trait A{const C:usize;}struct S<T>(T);impl<T:Copy>A for S<T>{const C:usize=3;}fn f()->X<{<S<u8>as A>::C}>{X([7;3])}fn main(){check(f().0[2]==7);}",
        true,
    ),
    (
        "v6-generic-missing-bound",
        "struct X<const N:usize>([u8;N]); trait A{const C:usize;}struct S<T>(T);struct U;impl<T:Copy>A for S<T>{const C:usize=3;}fn f()->X<{<S<U>as A>::C}>{X([7;3])}fn main(){}",
        false,
    ),
    (
        "v6-projected-value",
        "struct X<const N:usize>([u8;N]); trait A{type Out;const C:Self::Out;}impl A for u8{type Out=usize;const C:usize=3;}fn f()->X<{<u8 as A>::C}>{X([7;3])}fn main(){check(f().0[2]==7);}",
        true,
    ),
    (
        "v6-static-trait-argument",
        "struct X<const N:usize>([u8;N]);trait A<'a>{const C:usize=3;}impl<'a>A<'a>for u8{}fn f()->X<{<u8 as A<'static>>::C}>{X([7;3])}fn main(){check(f().0[2]==7);}",
        true,
    ),
    (
        "v6-two-owners-one-expression",
        "struct X<const N:usize>([u8;N]); trait A{const C:usize;}impl A for u8{const C:usize=3;}impl A for u16{const C:usize=4;}fn f()->X<{<u8 as A>::C+<u16 as A>::C}>{X([7;7])}fn main(){check(f().0[6]==7);}",
        true,
    ),
    (
        "v6-repeated-owner-one-expression",
        "struct X<const N:usize>([u8;N]); trait A{const C:usize=3;}impl A for u8{}fn f()->X<{<u8 as A>::C+<u8 as A>::C}>{X([7;6])}fn main(){check(f().0[5]==7);}",
        true,
    ),
    (
        "v6-boolean-cast",
        "struct X<const N:usize>([u8;N]); trait A{const C:bool=true;}impl A for u8{}fn f()->X<{<u8 as A>::C as usize}>{X([7;1])}fn main(){check(f().0[0]==7);}",
        true,
    ),
    (
        "v6-unused-invalid-method",
        "struct X<const N:usize>([u8;N]); trait A{const C:usize=3;fn g()->u8;}impl A for u8{fn g()->u16{7}}fn f()->X<{<u8 as A>::C}>{X([7;3])}fn main(){}",
        false,
    ),
    (
        "v6-unused-invalid-type",
        "struct X<const N:usize>([u8;N]); trait A{type Out:Copy;const C:usize=3;}struct U;impl A for u8{type Out=U;}fn f()->X<{<u8 as A>::C}>{X([7;3])}fn main(){}",
        false,
    ),
    (
        "v6-unused-invalid-initializer",
        "struct X<const N:usize>([u8;N]); trait A{const C:usize=3;const D:u8;}impl A for u8{const D:u8=true;}fn f()->X<{<u8 as A>::C}>{X([7;3])}fn main(){}",
        false,
    ),
];
#[test]
fn early_constants_preserve_declared_language_outcomes() -> Result<(), Box<dyn std::error::Error>> {
    for &(name, body, accepted) in CASES {
        let source = format!("{}{body}", native::CHECK);
        object_outcome(&format!("early-values-{name}"), &source, accepted)?;
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
fn early_constants_preserve_native_values() -> Result<(), Box<dyn std::error::Error>> {
    let cases = CASES
        .iter()
        .filter(|(_, _, accepted)| *accepted)
        .map(|&(name, source, _)| (name, source, false))
        .collect::<Vec<_>>();
    native::check_cases("early-values", &cases)
}
