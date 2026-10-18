//! Resumable path applicability preserves ambiguity, bounds and caller lifetime proofs.
use super::{native, object_outcome, semantic};
use std::fmt::Write as _;
const CASES: &[(&str, &str, bool)] = &[
    (
        "trait-fallback",
        r"
trait B{fn value()->usize;}struct X<const N:usize>;impl B for X<{S::<u8>::C}>{fn value()->usize{3}}struct S<T>(T);impl S<u16>{const C:usize=9;}trait A{const C:usize=3;}impl A for S<u8>{}fn main(){check(<X<3>as B>::value()==3);}
",
        true,
    ),
    (
        "trait-only-shorthand",
        r"
trait B{fn value()->usize;}struct X<const N:usize>;impl B for X<{S::C}>{fn value()->usize{3}}struct S;trait A{const C:usize=3;}impl A for S{}fn main(){check(<X<3>as B>::value()==3);}
",
        true,
    ),
    (
        "bounded-inherent-with-trait-name",
        r"
trait B{fn value()->usize;}struct X<const N:usize>;impl B for X<{S::<u8>::C}>{fn value()->usize{3}}struct S<T>(T);trait P{}impl<T:P>S<T>{const C:usize=3;}trait Unused{const C:usize=9;}impl P for u8{}fn main(){check(<X<3>as B>::value()==3);}
",
        true,
    ),
    (
        "disjoint-type-header-dependency",
        r"
trait B{fn value()->usize;}struct X<const N:usize>;impl B for X<{S::<u8,3>::C}>{fn value()->usize{3}}struct S<T,const N:usize>(T);impl S<u16,{S::<u8,3>::C}>{const C:usize=9;}impl S<u8,3>{const C:usize=3;}fn main(){check(<X<3>as B>::value()==3);}
",
        false,
    ),
    (
        "two-traits-one-compatible",
        r"
trait B{fn value()->usize;}struct X<const N:usize>;impl B for X<{S::C}>{fn value()->usize{3}}struct S;struct U;trait A{const C:usize=3;}trait D{const C:usize=9;}impl A for S{}impl D for U{}fn main(){check(<X<3>as B>::value()==3);}
",
        true,
    ),
    (
        "two-traits-ambiguous",
        r"
trait B{fn value()->usize;}struct X<const N:usize>;impl B for X<{S::C}>{fn value()->usize{3}}struct S;trait A{const C:usize=3;}trait D{const C:usize=3;}impl A for S{}impl D for S{}fn main(){check(<X<3>as B>::value()==3);}
",
        false,
    ),
    (
        "missing-trait-impl",
        r"
trait B{fn value()->usize;}struct X<const N:usize>;impl B for X<{S::C}>{fn value()->usize{3}}struct S;trait A{const C:usize=3;}fn main(){check(<X<3>as B>::value()==3);}
",
        false,
    ),
    (
        "missing-inherent-bound-with-trait-name",
        r"
trait B{fn value()->usize;}struct X<const N:usize>;impl B for X<{S::<u8>::C}>{fn value()->usize{3}}struct S<T>(T);trait P{}impl<T:P>S<T>{const C:usize=3;}trait Unused{const C:usize=9;}fn main(){check(<X<3>as B>::value()==3);}
",
        false,
    ),
    (
        "interleaved-trait-lookup",
        r"
trait B{fn value()->usize;}struct X<const N:usize>;impl B for X<{X::<3>::C}>{fn value()->usize{3}}trait P<'a>{}trait Q<'a>{const C:usize;}trait A<'a>{const C:usize;}impl<'a,T:P<'a>>A<'a>for T{const C:usize=3;}impl<'a>P<'a>for X<{<u16 as Q<'static>>::C}>{}impl<'a>Q<'a>for u16{const C:usize=3;}fn main(){check(<X<3>as B>::value()==3);}
",
        true,
    ),
    (
        "bounded-reference-owner",
        r"
trait B{fn value()->usize;}struct X<const N:usize>;impl B for X<{S::<&'static u8>::C}>{fn value()->usize{3}}struct S<T>(T);trait P<'a>{}impl<'a,T:P<'a>>S<&'a T>{const C:usize=3;}trait Unused{const C:usize=9;}impl<'a>P<'a>for u8{}fn main(){check(<X<3>as B>::value()==3);}
",
        true,
    ),
    (
        "static-owner",
        r"
trait B{fn value()->usize;}struct X<const N:usize>;impl B for X<{S::<'static>::C}>{fn value()->usize{3}}struct S<'a>(&'a u8);impl<'a:'static>S<'a>{const C:usize=3;}trait Unused{const C:usize=9;}fn main(){check(<X<3>as B>::value()==3);}
",
        true,
    ),
    (
        "static-owner-qualified",
        r"
trait B{fn value()->usize;}struct X<const N:usize>;impl B for X<{<S<'static>>::C}>{fn value()->usize{3}}struct S<'a>(&'a u8);impl<'a:'static>S<'a>{const C:usize=3;}trait Unused{const C:usize=9;}fn main(){check(<X<3>as B>::value()==3);}
",
        true,
    ),
    (
        "static-owner-caller-debt",
        r"
trait B{fn value()->usize;}struct X<const N:usize>;impl B for X<{S::<'static>::C}>{fn value()->usize{3}}struct S<'a>(&'a u8);impl<'a:'static>S<'a>{const C:usize=3;}trait Unused{const C:usize=9;}fn bad<'a>(_:&'a u8)->usize{S::<'a>::C}fn main(){check(<X<3>as B>::value()==3);}
",
        false,
    ),
    (
        "static-owner-caller-debt-qualified",
        r"
trait B{fn value()->usize;}struct X<const N:usize>;impl B for X<{<S<'static>>::C}>{fn value()->usize{3}}struct S<'a>(&'a u8);impl<'a:'static>S<'a>{const C:usize=3;}trait Unused{const C:usize=9;}fn bad<'a>(_:&'a u8)->usize{<S<'a>>::C}fn main(){check(<X<3>as B>::value()==3);}
",
        false,
    ),
    (
        "caller-bound-isolation",
        r"
trait B{fn value()->usize;}struct X<const N:usize>;impl B for X<{S::<u8>::C}>{fn value()->usize{3}}struct S<T>(T);trait P{}impl<T:P>S<T>{const C:usize=3;}trait Unused{const C:usize=9;}impl P for u8{}fn good<T:P>()->usize{S::<T>::C}fn bad<T>()->usize{S::<T>::C}fn main(){check(<X<3>as B>::value()==3);}
",
        false,
    ),
    (
        "caller-bound",
        r"
trait B{fn value()->usize;}struct X<const N:usize>;impl B for X<{S::<u8>::C}>{fn value()->usize{3}}struct S<T>(T);trait P{}impl<T:P>S<T>{const C:usize=3;}trait Unused{const C:usize=9;}impl P for u8{}fn good<T:P>()->usize{S::<T>::C}fn main(){check(<X<3>as B>::value()==3);check(good::<u8>()==3);}
",
        true,
    ),
    (
        "trait-caller-lifetime",
        r"
trait B{fn value()->usize;}struct X<const N:usize>;impl B for X<{S::<'static>::C}>{fn value()->usize{3}}struct S<'a>(&'a u8);trait A<'a>{const C:usize;}impl<'a>A<'a>for S<'a>{const C:usize=3;}fn read<'a>(_:&'a u8)->usize{S::<'a>::C}fn main(){check(<X<3>as B>::value()==3);let x=1u8;check(read(&x)==3);}
",
        true,
    ),
    (
        "trait-caller-lifetime-qualified",
        r"
trait B{fn value()->usize;}struct X<const N:usize>;impl B for X<{<S<'static>>::C}>{fn value()->usize{3}}struct S<'a>(&'a u8);trait A<'a>{const C:usize;}impl<'a>A<'a>for S<'a>{const C:usize=3;}fn read<'a>(_:&'a u8)->usize{<S<'a>>::C}fn main(){check(<X<3>as B>::value()==3);let x=1u8;check(read(&x)==3);}
",
        true,
    ),
];
#[test]
fn declaration_time_lookup_matches_independent_source_outcomes()
-> Result<(), Box<dyn std::error::Error>> {
    for &(name, body, accepted) in CASES {
        let source = format!("{}{body}", native::CHECK);
        object_outcome(&format!("lookup-readiness-{name}"), &source, accepted)?;
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
    let mut source = "trait B{fn value()->usize;}struct X<const N:usize>;impl B for X<{S::C}>{fn value()->usize{3}}struct S;trait A{const C:usize;}impl<T:P0>A for T{const C:usize=3;}".to_owned();
    for i in 0..depth {
        write!(source, "trait P{i}{{}}impl<T:P{}>P{i} for T{{}}", i + 1)?;
    }
    write!(
        source,
        "trait P{depth}{{}}impl P{depth} for S{{}}fn main(){{check(<X<3>as B>::value()==3);}}"
    )?;
    Ok(source)
}
#[test]
fn recursive_lookup_conditions_preserve_one_candidate_probe()
-> Result<(), Box<dyn std::error::Error>> {
    for depth in [8, 32, 96] {
        let source = format!("{}{}", native::CHECK, chain(depth)?);
        object_outcome(&format!("lookup-chain-{depth}"), &source, true)?;
        let (_, facts) = semantic::analyze(&source)?;
        println!(
            "lookup-chain-{depth}: probes={} steps={}",
            facts.methods.selection_probes, facts.methods.selection_steps
        );
        assert_eq!(facts.methods.selection_probes, 1);
        assert!(facts.methods.selection_steps < 512 * (depth + 1));
    }
    Ok(())
}
#[test]
#[ignore = "requires the qualified CPPGM LowIR-to-native backend"]
fn declaration_time_lookup_preserves_native_values() -> Result<(), Box<dyn std::error::Error>> {
    let chain = chain(32)?;
    let mut cases: Vec<_> = CASES
        .iter()
        .filter(|(_, _, accepted)| *accepted)
        .map(|&(name, source, _)| (name, source, false))
        .collect();
    cases.push(("chain-32", &chain, false));
    native::check_cases("lookup-readiness", &cases)
}
