//! Declaration-time const arguments wait for exactly the compatible implementation headers.
use super::{native, object_outcome, semantic};
use std::fmt::Write as _;

const CASES: &[(&str, &str, bool)] = &[
    (
        "trait-before",
        r"
trait A{const C:usize=3;}
trait B{fn value()->usize;}
struct X<const N:usize>;
impl A for u8{}
impl B for X<{<u8 as A>::C}>{fn value()->usize{3}}
fn main(){check(<X<3>as B>::value()==3);}",
        true,
    ),
    (
        "trait-after",
        r"
trait A{const C:usize=3;}
trait B{fn value()->usize;}
struct X<const N:usize>;
impl B for X<{<u8 as A>::C}>{fn value()->usize{3}}
impl A for u8{}
fn main(){check(<X<3>as B>::value()==3);}",
        true,
    ),
    (
        "trait-argument",
        r"
trait A{const C:usize=3;}
trait B<const N:usize>{fn value()->usize;}
impl B<{<u8 as A>::C}>for u16{fn value()->usize{3}}
impl A for u8{}
fn main(){check(<u16 as B<3>>::value()==3);}",
        true,
    ),
    (
        "nested-trait-arguments",
        r"
trait A<const N:usize>{const C:usize=N;}
trait C{const D:usize=3;}
trait B{fn value()->usize;}
struct X<const N:usize>;
impl B for X<{<u8 as A<{<u16 as C>::D}>>::C}>{fn value()->usize{3}}impl<const N:usize>A<N>for u8{}
impl C for u16{}
fn main(){check(<X<3>as B>::value()==3);}",
        true,
    ),
    (
        "overlap",
        r"
trait A{const C:usize=3;}
trait B{fn value()->usize;}
struct X<const N:usize>;
impl A for u8{}
impl B for X<{<u8 as A>::C}>{fn value()->usize{3}}
impl B for X<3>{fn value()->usize{4}}
fn main(){}",
        false,
    ),
    (
        "missing-impl",
        r"
trait A{const C:usize=3;}
trait B{fn value()->usize;}
struct X<const N:usize>;
impl B for X<{<u8 as A>::C}>{fn value()->usize{3}}
fn main(){}",
        false,
    ),
    (
        "invalid-unused-override",
        r"
trait A{const C:usize=3;}
trait B{fn value()->usize;}
struct X<const N:usize>;
impl A for u8{const C:usize=true;}
impl B for X<{<u8 as A>::C}>{fn value()->usize{3}}
fn main(){}",
        false,
    ),
    (
        "same-trait-disjoint-subject",
        r"
trait A{const C:usize;}
struct X<const N:usize>;
impl A for X<{<u8 as A>::C}>{const C:usize=4;}
impl A for u8{const C:usize=3;}
fn main(){check(<X<3>as A>::C==4);}",
        true,
    ),
    (
        "generic-bound-forward",
        r"
trait B{fn value()->usize;}
struct X<const N:usize>;
trait P{}
struct S<T>(T);
trait A{const C:usize;}
impl B for X<{<S<u8>as A>::C}>{fn value()->usize{3}}impl<T:P>A for S<T>{const C:usize=3;}
impl P for u8{}
fn main(){check(<X<3>as B>::value()==3);}",
        true,
    ),
    (
        "missing-generic-bound",
        r"
trait B{fn value()->usize;}
struct X<const N:usize>;
trait P{}
struct S<T>(T);
trait A{const C:usize;}
impl B for X<{<S<u8>as A>::C}>{fn value()->usize{3}}impl<T:P>A for S<T>{const C:usize=3;}
fn main(){check(<X<3>as B>::value()==3);}",
        false,
    ),
    (
        "overridden-default-dependency",
        r"
trait A{const C:usize=<u16 as B<3>>::C;}
trait B<const N:usize>{const C:usize;}
impl B<{<u8 as A>::C}>for u16{const C:usize=7;}
impl A for u8{const C:usize=3;}
fn main(){check(<u8 as A>::C==3&&<u16 as B<3>>::C==7);}",
        true,
    ),
    (
        "pending-overlap",
        r"
trait B{fn value()->usize;}
struct X<const N:usize>;
trait A{const C:usize;}
impl B for X<{<u8 as A>::C}>{fn value()->usize{3}}
impl A for u8{const C:usize=3;}
impl A for u8{const C:usize=4;}
fn main(){}",
        false,
    ),
    (
        "alias-default-target",
        r"
trait B{fn value()->usize;}
struct X<const N:usize>;
trait A{const C:usize;}type Alias<T=X<{<u8 as A>::C}>>=T;
impl B for Alias{fn value()->usize{3}}
impl A for u8{const C:usize=3;}
fn main(){check(<X<3>as B>::value()==3);}",
        true,
    ),
];

const INTERLEAVED: &str = "
trait B {fn value()->usize;}
struct X<const N:usize>;
trait R<'a> {const C:usize;}
trait P<'a> {}
trait A<'a> {const C:usize;}
impl B for X<{<X<3>as A<'static>>::C}> {fn value()->usize{3}}
impl<'a,T:P<'a>> A<'a> for T {const C:usize=3;}
impl<'a> P<'a> for X<{<u16 as R<'static>>::C}> {}
impl<'a> R<'a> for u16 {const C:usize=3;}
fn main(){check(<X<3>as B>::value()==3);}
";

fn outcome(name: &str, body: &str, accepted: bool) -> Result<(), Box<dyn std::error::Error>> {
    let source = format!("{}{body}", native::CHECK);
    object_outcome(&format!("impl-header-{name}"), &source, accepted)?;
    let result = semantic::analyze(&source).and_then(|(mut report, mut facts)| {
        let entry = rustgm::sema::entry::select(&mut report, &facts).map_err(|e| e.to_string())?;
        rustgm::lower::instances::build(&report, &mut facts, entry)
            .map(|_| ())
            .map_err(|e| e.to_string())
    });
    assert_eq!(result.is_ok(), accepted, "{name}: {result:?}");
    Ok(())
}

#[test]
fn implementation_header_dependencies_preserve_qualified_outcomes()
-> Result<(), Box<dyn std::error::Error>> {
    for &(name, source, accepted) in CASES {
        outcome(name, source, accepted)?;
    }
    outcome("interleaved-lifetimes", INTERLEAVED, true)
}

#[test]
fn header_preparation_can_allocate_foreign_lifetimes_during_a_parent_query()
-> Result<(), Box<dyn std::error::Error>> {
    let source = format!("{}{INTERLEAVED}", native::CHECK);
    let (_, facts) = semantic::analyze(&source)?;
    let mut gaps = 0;
    let mut owned = 0;
    for retained in &facts.quantified_proofs {
        let proof = &retained.proof;
        let mut count = 0;
        for id in proof.owned_regions().flatten() {
            assert_eq!(facts.body_regions[id as usize - 1].body, retained.body);
            count += 1;
        }
        gaps += usize::try_from(proof.regions.end - proof.regions.start)? - count;
        owned += count;
    }
    assert!(
        gaps > 0 && owned > 0,
        "the source must interleave distinct proof owners"
    );
    Ok(())
}

fn chain(depth: usize) -> Result<String, std::fmt::Error> {
    let mut source = "trait A{const C:usize;}struct X<const N:usize>;trait B{fn value()->usize;}impl B for X<{<S0 as A>::C}>{fn value()->usize{3}}".to_owned();
    for index in 0..depth {
        write!(
            source,
            "struct S{index};impl A for S{index}{{const C:usize="
        )?;
        if index + 1 == depth {
            source.push_str("3;");
        } else {
            write!(source, "<S{} as A>::C;", index + 1)?;
        }
        source.push('}');
    }
    source.push_str("fn main(){check(<X<3>as B>::value()==3);}");
    Ok(source)
}

#[test]
fn nested_header_dependencies_retain_bounded_body_work() -> Result<(), Box<dyn std::error::Error>> {
    for depth in [8, 32, 128] {
        let body = chain(depth)?;
        outcome(&format!("chain-{depth}"), &body, true)?;
        let source = format!("{}{body}", native::CHECK);
        let (_, facts) = semantic::analyze(&source)?;
        println!(
            "header-chain-{depth}: steps={} suspensions={}",
            facts.body_work.steps, facts.body_work.suspensions
        );
        assert!(facts.body_work.suspensions >= depth);
        assert!(facts.body_work.steps < 256 * (depth + 1));
    }
    Ok(())
}

#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn implementation_header_constants_preserve_native_values() -> Result<(), Box<dyn std::error::Error>>
{
    let chain = chain(32)?;
    let mut cases: Vec<_> = CASES
        .iter()
        .filter(|(_, _, accepted)| *accepted)
        .map(|&(name, source, _)| (name, source, false))
        .collect();
    cases.push(("interleaved-lifetimes", INTERLEAVED, false));
    cases.push(("chain-32", &chain, false));
    native::check_cases("impl-header-readiness", &cases)
}
