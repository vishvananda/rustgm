//! Discriminants select associated values after complete implementation validation.
use super::{native, object_outcome, semantic};

const CASES: &[(&str, &str, bool)] = &[
    (
        "enclosing-type",
        "struct S<T>(T);impl<T>S<T>{const C:u8=3;}#[repr(u8)]enum E<T>{A(T)=S::<T>::C}fn main(){}",
        false,
    ),
    (
        "enclosing-const",
        "struct S<const N:u8>;impl<const N:u8>S<N>{const C:u8=3;}#[repr(u8)]enum E<const N:u8>{A=S::<N>::C}fn main(){}",
        false,
    ),
    (
        "unused-enum-parameter",
        "struct S<T>(T);impl<T>S<T>{const C:u8=3;}#[repr(u8)]enum E<T>{A(T)=S::<u8>::C}fn main(){let x=E::A(7u8);match x{E::A(v)=>check(v==7)}}",
        true,
    ),
    (
        "default",
        "trait A{const C:isize=3;}impl A for u8{}enum E{A=<u8 as A>::C,B}fn main(){check(E::A as isize==3);check(E::B as isize==4);}",
        true,
    ),
    (
        "override",
        "trait A{const C:isize=3;}impl A for u8{const C:isize=7;}enum E{A=<u8 as A>::C,B}fn main(){check(E::A as isize==7);check(E::B as isize==8);}",
        true,
    ),
    (
        "shorthand",
        "trait A{const C:isize=3;}impl A for u8{}enum E{A=u8::C,B}fn main(){check(E::B as isize==4);}",
        true,
    ),
    (
        "inherent-generic",
        "struct S<T>(T);impl<T>S<T>{const C:isize=3;}enum E{A=S::<u8>::C,B}fn main(){check(E::B as isize==4);}",
        true,
    ),
    (
        "inherent-const",
        "struct S<const N:isize>;impl<const N:isize>S<N>{const C:isize=N;}enum E{A=S::<3>::C,B=S::<5>::C,C}fn main(){check(E::A as isize==3);check(E::B as isize==5);check(E::C as isize==6);}",
        true,
    ),
    (
        "trait-generic",
        "trait A{const C:isize=3;}struct S<T>(T);impl<T:Copy>A for S<T>{const C:isize=7;}enum E{A=<S<u8>as A>::C,B}fn main(){check(E::B as isize==8);}",
        true,
    ),
    (
        "transitive-default",
        "trait A{const C:isize=3;const D:isize=Self::C+1;}impl A for u8{const C:isize=5;}enum E{A=<u8 as A>::D,B}fn main(){check(E::A as isize==6);check(E::B as isize==7);}",
        true,
    ),
    (
        "separate-owners",
        "trait A{const C:isize=3;}impl A for u8{}impl A for u16{const C:isize=5;}enum E{A=<u8 as A>::C,B=<u16 as A>::C,C}fn main(){check(E::A as isize==3);check(E::B as isize==5);check(E::C as isize==6);}",
        true,
    ),
    (
        "repeat-use",
        "trait A{const C:isize=3;}impl A for u8{}enum E{A=<u8 as A>::C,B=<u8 as A>::C+1}enum F{A=<u8 as A>::C,B}fn main(){check(E::B as isize==4);check(F::B as isize==4);}",
        true,
    ),
    (
        "static-lifetimes",
        "trait A<'a>{const C:isize=3;}struct S<'a>(&'a u8);impl<'a:'static>A<'a>for S<'a>{}enum E{A=<S<'static>as A<'static>>::C,B}fn main(){check(E::B as isize==4);}",
        true,
    ),
    (
        "reference-value",
        "trait A{const C:&'static isize=&3;}impl A for u8{}enum E{A=*<u8 as A>::C,B}fn main(){check(E::B as isize==4);}",
        true,
    ),
    (
        "skipped-fault",
        "trait A{const C:isize=if true{3}else{1/0};}impl A for u8{}enum E{A=<u8 as A>::C,B}fn main(){check(E::B as isize==4);}",
        true,
    ),
    (
        "duplicate",
        "trait A{const C:isize=3;}impl A for u8{}enum E{A=<u8 as A>::C,B=3}fn main(){}",
        false,
    ),
    (
        "missing",
        "trait A{const C:isize=3;}enum E{A=<u8 as A>::C}fn main(){}",
        false,
    ),
    (
        "bound-failure",
        "trait A{const C:isize=3;}struct S<T>(T);struct U;impl<T:Copy>A for S<T>{}enum E{A=<S<U>as A>::C}fn main(){}",
        false,
    ),
    (
        "wrong-type",
        "trait A{const C:u8=3;}impl A for u8{}enum E{A=<u8 as A>::C}fn main(){}",
        false,
    ),
    (
        "overflow",
        "trait A{const C:u8=255;}impl A for u8{}#[repr(u8)]enum E{A=<u8 as A>::C,B}fn main(){}",
        false,
    ),
    (
        "cycle",
        "trait A{const C:isize=Self::D;const D:isize=Self::C;}impl A for u8{}enum E{A=<u8 as A>::C}fn main(){}",
        false,
    ),
    (
        "unused-invalid-impl",
        "trait A{const C:isize=3;}trait B{fn f();}impl A for u8{}impl B for u8{}enum E{A=<u8 as A>::C}fn main(){}",
        false,
    ),
    (
        "enum-dependency",
        "trait A{const C:isize=3;}impl A for u8{}enum E{A=<u8 as A>::C}struct S<const N:isize>;impl<const N:isize>S<N>{const C:isize=E::A as isize+N;}enum F{A=S::<5>::C,B}fn main(){check(F::A as isize==8);check(F::B as isize==9);}",
        true,
    ),
];

#[test]
fn selected_discriminants_match_qualified_outcomes() -> Result<(), Box<dyn std::error::Error>> {
    for &(name, body, accepted) in CASES {
        let source = format!("{}{body}", native::CHECK);
        object_outcome(&format!("discriminant-{name}"), &source, accepted)?;
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
fn selected_discriminants_preserve_native_values() -> Result<(), Box<dyn std::error::Error>> {
    let cases = CASES
        .iter()
        .filter(|(_, _, accepted)| *accepted)
        .map(|&(name, source, _)| (name, source, false))
        .collect::<Vec<_>>();
    native::check_cases("associated-discriminants", &cases)
}
