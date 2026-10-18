//! Inherent constants are discoverable while a trait implementation header is pending.
use super::{native, object_outcome, semantic};
use std::fmt::Write as _;

const CASES: &[(&str, &str, bool)] = &[
    (
        "named-later",
        r"
trait B{fn value()->usize;}struct X<const N:usize>;
impl B for X<{S::C}>{fn value()->usize{3}}struct S;impl S{const C:usize=3;}fn main(){check(<X<3>as B>::value()==3);}
",
        true,
    ),
    (
        "qualified-later",
        r"
trait B{fn value()->usize;}struct X<const N:usize>;
impl B for X<{<S>::C}>{fn value()->usize{3}}struct S;impl S{const C:usize=3;}fn main(){check(<X<3>as B>::value()==3);}
",
        true,
    ),
    (
        "generic-owner",
        r"
trait B{fn value()->usize;}struct X<const N:usize>;
impl B for X<{S::<u8>::C}>{fn value()->usize{3}}struct S<T>(T);impl<T>S<T>{const C:usize=3;}fn main(){check(<X<3>as B>::value()==3);}
",
        true,
    ),
    (
        "alias-value-without-type",
        r"
trait B{fn value()->usize;}struct X<const N:usize>;
impl B for X<{Alias::C}>{fn value()->usize{3}}struct S;type Alias<T=S>=T;impl Alias{const C:usize=3;}fn main(){check(<X<3>as B>::value()==3);}
",
        false,
    ),
    (
        "alias-default-owner",
        r"
trait B{fn value()->usize;}struct X<const N:usize>;
impl B for X<{<Alias>::C}>{fn value()->usize{3}}struct S;type Alias<T=S>=T;impl Alias{const C:usize=3;}fn main(){check(<X<3>as B>::value()==3);}
",
        true,
    ),
    (
        "header-trait-dependency",
        r"
trait B{fn value()->usize;}struct X<const N:usize>;
impl B for X<{S::<3>::C}>{fn value()->usize{3}}struct S<const N:usize>;trait A{const C:usize;}impl S<{<u8 as A>::C}>{const C:usize=3;}impl A for u8{const C:usize=3;}fn main(){check(<X<3>as B>::value()==3);}
",
        true,
    ),
    (
        "bound-forward",
        r"
trait B{fn value()->usize;}struct X<const N:usize>;
impl B for X<{S::<u8>::C}>{fn value()->usize{3}}struct S<T>(T);trait P{}impl<T:P>S<T>{const C:usize=3;}impl P for u8{}fn main(){check(<X<3>as B>::value()==3);}
",
        true,
    ),
    (
        "unrelated-owner-header",
        r"
trait B{fn value()->usize;}struct X<const N:usize>;
impl B for X<{S::C}>{fn value()->usize{3}}struct S;struct U<const N:usize>;impl U<{S::C}>{const C:usize=7;}impl S{const C:usize=3;}fn main(){check(<X<3>as B>::value()==3);}
",
        true,
    ),
    (
        "inherent-precedence",
        r"
trait B{fn value()->usize;}struct X<const N:usize>;
impl B for X<{S::C}>{fn value()->usize{3}}struct S;trait A{const C:usize=9;}impl A for S{}impl S{const C:usize=3;}fn main(){check(<X<3>as B>::value()==3);}
",
        true,
    ),
    (
        "nested-value-owner",
        r"
trait B{fn value()->usize;}struct X<const N:usize>;
impl B for X<{S::C}>{fn value()->usize{3}}struct S;struct U;impl S{const C:usize=U::C;}impl U{const C:usize=3;}fn main(){check(<X<3>as B>::value()==3);}
",
        true,
    ),
    (
        "disjoint-owner-arguments",
        r"
trait B{fn value()->usize;}struct X<const N:usize>;
impl B for X<{S::<u8>::C}>{fn value()->usize{3}}struct S<T>(T);impl S<u16>{const C:usize=9;}impl S<u8>{const C:usize=3;}fn main(){check(<X<3>as B>::value()==3);}
",
        true,
    ),
    (
        "private-member",
        r"
trait B{fn value()->usize;}struct X<const N:usize>;
impl B for X<{m::S::C}>{fn value()->usize{3}}mod m{pub struct S;impl S{const C:usize=3;}}fn main(){check(<X<3>as B>::value()==3);}
",
        false,
    ),
    (
        "overlapping-members",
        r"
trait B{fn value()->usize;}struct X<const N:usize>;
impl B for X<{S::<u8>::C}>{fn value()->usize{3}}struct S<T>(T);impl<T>S<T>{const C:usize=3;}impl S<u8>{const C:usize=3;}fn main(){check(<X<3>as B>::value()==3);}
",
        false,
    ),
    (
        "missing-owner-bound",
        r"
trait B{fn value()->usize;}struct X<const N:usize>;
impl B for X<{S::<u8>::C}>{fn value()->usize{3}}struct S<T>(T);trait P{}impl<T:P>S<T>{const C:usize=3;}fn main(){check(<X<3>as B>::value()==3);}
",
        false,
    ),
    (
        "cyclic-header",
        r"
trait B{fn value()->usize;}struct X<const N:usize>;
impl B for X<{S::<3>::C}>{fn value()->usize{3}}struct S<const N:usize>;impl S<{S::<3>::C}>{const C:usize=3;}fn main(){check(<X<3>as B>::value()==3);}
",
        false,
    ),
    (
        "unused-invalid-member",
        r"
trait B{fn value()->usize;}struct X<const N:usize>;
impl B for X<{S::C}>{fn value()->usize{3}}struct S;struct U;impl S{const C:usize=3;}impl U{const C:usize=true;}fn main(){check(<X<3>as B>::value()==3);}
",
        false,
    ),
];

#[test]
fn inherent_header_dependencies_preserve_qualified_outcomes()
-> Result<(), Box<dyn std::error::Error>> {
    for &(name, body, accepted) in CASES {
        let source = format!("{}{body}", native::CHECK);
        object_outcome(&format!("inherent-header-{name}"), &source, accepted)?;
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
    let mut source = "trait B{fn value()->usize;}struct X<const N:usize>;impl B for X<{S0::C}>{fn value()->usize{3}}".to_owned();
    for index in 0..depth {
        write!(source, "struct S{index};impl S{index}{{const C:usize=")?;
        if index + 1 == depth {
            source.push_str("3;");
        } else {
            write!(source, "S{}::C;", index + 1)?;
        }
        source.push('}');
    }
    source.push_str("fn main(){check(<X<3>as B>::value()==3);}");
    Ok(source)
}

#[test]
fn nested_inherent_values_share_completed_headers() -> Result<(), Box<dyn std::error::Error>> {
    for depth in [8, 32, 128] {
        let source = format!("{}{}", native::CHECK, chain(depth)?);
        object_outcome(&format!("inherent-header-chain-{depth}"), &source, true)?;
        let (_, facts) = semantic::analyze(&source)?;
        println!(
            "inherent-chain-{depth}: steps={} suspensions={}",
            facts.body_work.steps, facts.body_work.suspensions
        );
        assert!(facts.body_work.steps < 256 * (depth + 1));
    }
    Ok(())
}

#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn inherent_header_constants_preserve_native_values() -> Result<(), Box<dyn std::error::Error>> {
    let chain = chain(32)?;
    let mut cases: Vec<_> = CASES
        .iter()
        .filter(|(_, _, accepted)| *accepted)
        .map(|&(name, source, _)| (name, source, false))
        .collect();
    cases.push(("chain-32", &chain, false));
    native::check_cases("inherent-header-readiness", &cases)
}
