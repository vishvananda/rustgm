//! Selected initializer readiness is independent of declaration order and instance identity.
use super::{native, object_outcome, semantic};
use std::fmt::Write as _;

const CASES: &[(&str, &str, bool)] = &[
    (
        "later-closed-override",
        "struct X<const N:usize>([u8;N]);fn main(){let x:X<{<u8 as A>::C}>=X([7;3]);check(x.0[2]==7);}trait A{const C:usize;const D:usize=3;}impl A for u8{const C:usize=Self::D;}",
        true,
    ),
    (
        "later-generic-override",
        "struct X<const N:usize>([u8;N]);struct S<T>(T);fn main(){let x:X<{<S<u8> as A>::C}>=X([7;3]);check(x.0[2]==7);}trait A{const C:usize;const D:usize=3;}impl<T>A for S<T>{const C:usize=Self::D;}",
        true,
    ),
    (
        "qualified-dependency",
        "struct X<const N:usize>([u8;N]);fn main(){let x:X<{<u8 as A>::C}>=X([7;3]);check(x.0[2]==7);}trait A{const C:usize;}trait B{const D:usize=3;}impl B for u16{}impl A for u8{const C:usize=<u16 as B>::D;}",
        true,
    ),
    (
        "inherent-precedence",
        "struct X<const N:usize>([u8;N]);struct S;fn main(){let x:X<{<S as A>::C}>=X([7;5]);check(x.0[4]==7);}trait A{const C:usize;const D:usize=3;}impl A for S{const C:usize=Self::D;}impl S{const D:usize=5;}",
        true,
    ),
    (
        "nested-argument",
        "struct X<const N:usize>([u8;N]);fn main(){let x:X<{<u8 as A>::C}>=X([7;3]);check(x.0[2]==7);}trait A{const C:usize;}trait B<const N:usize>{const D:usize=N;}impl<const N:usize>B<N>for u8{}impl A for u8{const C:usize=<u8 as B<{<u16 as A>::C}>>::D;}impl A for u16{const C:usize=3;}",
        true,
    ),
    (
        "closed-cycle",
        "struct X<const N:usize>([u8;N]);fn main(){let _x:X<{<u8 as A>::C}>=X([7;3]);}trait A{const C:usize;const D:usize;}impl A for u8{const C:usize=Self::D;const D:usize=Self::C;}",
        false,
    ),
    (
        "generic-cycle",
        "struct X<const N:usize>([u8;N]);struct S<T>(T);fn main(){let _x:X<{<S<u8> as A>::C}>=X([7;3]);}trait A{const C:usize;const D:usize;}impl<T>A for S<T>{const C:usize=Self::D;const D:usize=Self::C;}",
        false,
    ),
    (
        "nested-cycle",
        "struct X<const N:usize>([u8;N]);fn main(){let _x:X<{<u8 as A>::C}>=X([7;3]);}trait A{const C:usize;}trait B<const N:usize>{const D:usize=N;}impl<const N:usize>B<N>for u8{}impl A for u8{const C:usize=<u8 as B<{<u8 as A>::C}>>::D;}",
        false,
    ),
    (
        "unselected-fault",
        "struct X<const N:usize>([u8;N]);fn main(){let x:X<{<u8 as A>::C}>=X([7;3]);check(x.0[2]==7);}trait A{const C:usize;}impl A for u8{const C:usize=3;}impl A for u16{const C:usize=1/0;}",
        true,
    ),
    (
        "missing-bound",
        "struct X<const N:usize>([u8;N]);struct S<T>(T);struct U;fn main(){let _x:X<{<S<U> as A>::C}>=X([7;3]);}trait A{const C:usize;}impl<T:Copy>A for S<T>{const C:usize=3;}",
        false,
    ),
    (
        "qualified-inherent",
        "struct X<const N:usize>([u8;N]);struct S;fn main(){let x:X<{<S>::C}>=X([7;3]);check(x.0[2]==7);}impl S{const C:usize=3;}",
        true,
    ),
    (
        "named-inherent",
        "struct X<const N:usize>([u8;N]);struct S;fn main(){let x:X<{S::C}>=X([7;3]);check(x.0[2]==7);}impl S{const C:usize=3;}",
        true,
    ),
    (
        "inapplicable-inherent",
        "struct X<const N:usize>([u8;N]);struct S<T>(T);fn main(){let x:X<{S::<u8>::C}>=X([7;3]);check(x.0[2]==7);}impl S<u16>{const C:usize=9;}trait A{const C:usize=3;}impl A for S<u8>{}",
        true,
    ),
    (
        "private-inherent",
        "struct X<const N:usize>([u8;N]);mod m{pub struct S;impl S{const C:usize=3;}}fn main(){let _x:X<{m::S::C}>=X([7;3]);}",
        false,
    ),
    (
        "ambiguous-nominal",
        "struct X<const N:usize>([u8;N]);struct S;fn main(){let _x:X<{S::C}>=X([7;3]);}trait A{const C:usize=3;}trait B{const C:usize=3;}impl A for S{}impl B for S{}",
        false,
    ),
    (
        "instance-values",
        "struct X<const N:usize>([u8;N]);struct S<const N:usize>;fn main(){let x:X<{<S<3> as A>::C}>=X([7;3]);let y:X<{<S<5> as A>::C}>=X([8;5]);check(x.0[2]==7);check(y.0[4]==8);}trait A{const C:usize;const D:usize;}impl<const N:usize>A for S<N>{const C:usize=Self::D;const D:usize=N;}",
        true,
    ),
    (
        "overridden-default-before",
        "struct X<const N:usize>([u8;N]);fn main(){let x:X<{<u8 as A>::C}>=X([7;3]);check(x.0[2]==7);}trait A{const C:usize;const D:usize=<u16 as B<{<u8 as A>::C}>>::VALUE;}trait B<const N:usize>{const VALUE:usize=N;}impl<const N:usize>B<N>for u16{}impl A for u8{const C:usize=Self::D;const D:usize=3;}",
        true,
    ),
    (
        "overridden-default-after",
        "struct X<const N:usize>([u8;N]);trait A{const C:usize;const D:usize=<u16 as B<{<u8 as A>::C}>>::VALUE;}trait B<const N:usize>{const VALUE:usize=N;}impl<const N:usize>B<N>for u16{}impl A for u8{const C:usize=Self::D;const D:usize=3;}fn main(){let x:X<{<u8 as A>::C}>=X([7;3]);check(x.0[2]==7);}",
        true,
    ),
    (
        "selected-default-cycle",
        "struct X<const N:usize>([u8;N]);fn main(){let x:X<{<u8 as A>::C}>=X([7;3]);check(x.0[2]==7);}trait A{const C:usize;const D:usize=<u16 as B<{<u8 as A>::C}>>::VALUE;}trait B<const N:usize>{const VALUE:usize=N;}impl<const N:usize>B<N>for u16{}impl A for u8{const C:usize=Self::D;}",
        false,
    ),
    (
        "unselected-nonconst-call",
        "struct X<const N:usize>([u8;N]);fn main(){let x:X<{<u8 as A>::C}>=X([7;3]);check(x.0[2]==7);}fn runtime()->usize{3}trait A{const C:usize;const D:usize=runtime();}impl A for u8{const C:usize=Self::D;const D:usize=3;}",
        false,
    ),
];

#[test]
fn selected_initializers_match_qualified_outcomes() -> Result<(), Box<dyn std::error::Error>> {
    for &(name, body, accepted) in CASES {
        let source = format!("{}{body}", native::CHECK);
        object_outcome(&format!("readiness-{name}"), &source, accepted)?;
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
fn selected_initializers_preserve_native_values() -> Result<(), Box<dyn std::error::Error>> {
    let chain = dependency_chain(32, false)?;
    let mut cases = CASES
        .iter()
        .filter(|(_, _, accepted)| *accepted)
        .map(|&(name, source, _)| (name, source, false))
        .collect::<Vec<_>>();
    cases.push(("long-chain", &chain, false));
    native::check_cases("initializer-readiness", &cases)
}

fn dependency_chain(length: usize, early: bool) -> Result<String, std::fmt::Error> {
    let mut source = String::from(
        "trait A{const C:usize;}struct X<const N:usize>([u8;N]);fn main(){let x:X<{<S0 as A>::C}>=X([7;3]);check(x.0[2]==7);}",
    );
    if early {
        source = String::from(
            "trait A{const C:usize;}struct X<const N:usize>([u8;N]);fn f()->X<{<S0 as A>::C}>{X([7;3])}fn main(){check(f().0[2]==7);}",
        );
    }
    for index in 0..length {
        write!(
            source,
            "struct S{index};impl A for S{index}{{const C:usize="
        )?;
        if index + 1 == length {
            source.push_str("3;");
        } else {
            write!(source, "<S{} as A>::C;", index + 1)?;
        }
        source.push('}');
    }
    Ok(source)
}

#[test]
fn selected_dependency_chains_resume_without_recursive_declaration_draining()
-> Result<(), Box<dyn std::error::Error>> {
    for length in [32, 96] {
        for early in [false, true] {
            let source = format!("{}{}", native::CHECK, dependency_chain(length, early)?);
            object_outcome(&format!("readiness-chain-{length}-{early}"), &source, true)?;
            let (mut report, mut facts) = semantic::analyze(&source)?;
            let entry = rustgm::sema::entry::select(&mut report, &facts)?;
            rustgm::lower::instances::build(&report, &mut facts, entry)?;
        }
    }
    Ok(())
}
