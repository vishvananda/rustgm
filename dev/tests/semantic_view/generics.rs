//! PA21 records binders and evidence while preserving the earlier inspection formats.
use super::{cli, rustc};

const SOURCE: &str = "trait A<'a>{}impl<'a>A<'a> for u8{}trait G{type V<'a>:Copy where Self:'a;}impl G for u8{type V<'a>=&'a u8;}fn take<T:for<'a>A<'a>>(_:T){}fn f<'b>(x:&'b u8)->impl Copy+use<'b>{x}fn main(){let a=7u8;take(a);let x=f(&a);let _=(x,x);}trait M{fn make()->impl Copy;}impl M for u16{fn make()->u8{7}}";

#[test]
fn advanced_records_have_stable_identities_and_exact_counts()
-> Result<(), Box<dyn std::error::Error>> {
    rustc::qualify("generic-view-records", SOURCE, true)?;
    let full = cli("--emit-generic-semantics", SOURCE)?;
    assert!(
        full.status.success(),
        "{}",
        String::from_utf8_lossy(&full.stderr)
    );
    assert!(full.stderr.is_empty());
    let text = std::str::from_utf8(&full.stdout)?;
    assert!(text.starts_with("generic-semantics 1\n"));
    let count = cli("--count-generic-semantics", SOURCE)?;
    assert!(count.status.success() && count.stderr.is_empty());
    assert!(
        full.stdout
            .ends_with(&[b"eof ".as_slice(), &count.stdout].concat())
    );
    let totals = std::str::from_utf8(&count.stdout)?;
    for (field, prefix) in [
        ("quantifiers", "quantifier "),
        ("families", "family "),
        ("family-bounds", "family-bound "),
        ("family-regions", "family-region "),
        ("opaques", "opaque "),
        ("opaque-bounds", "opaque-bound "),
        ("opaque-body-results", "opaque-body-result "),
        ("opaque-implementations", "opaque-implementation "),
        ("proofs", "proof "),
        ("proof-scopes", "proof-scope "),
        ("proof-blocks", "proof-block "),
        ("proof-sources", "proof-source "),
        ("proof-owned", "proof-owned "),
        ("proof-relations", "proof-relation "),
        ("proof-types", "proof-type "),
    ] {
        let value = totals
            .split_whitespace()
            .find_map(|entry| {
                let (key, value) = entry.split_once('=')?;
                (key == field).then_some(value)
            })
            .ok_or("missing count")?
            .parse::<usize>()?;
        assert_eq!(
            value,
            text.lines().filter(|line| line.starts_with(prefix)).count(),
            "{field}"
        );
    }
    assert!(text.lines().any(|line| line.starts_with("quantifier ")));
    assert!(
        text.lines()
            .any(|line| line.starts_with("opaque-implementation "))
    );
    assert!(text.lines().any(|line| line.starts_with("proof-scope ")));
    for _ in 0..3 {
        let again = cli("--emit-generic-semantics", SOURCE)?;
        assert!(again.status.success() && again.stderr.is_empty());
        assert_eq!(full.stdout, again.stdout);
    }
    Ok(())
}

#[test]
fn advanced_inspection_keeps_later_loan_failure_visible() -> Result<(), Box<dyn std::error::Error>>
{
    let source =
        "fn f<'a>(x:&'a u8)->impl Copy+use<'a>{x}fn main(){let x;{let a=7;x=f(&a);}let _=x;}";
    rustc::qualify("generic-view-pending-loan", source, false)?;
    for mode in ["--emit-generic-semantics", "--count-generic-semantics"] {
        let output = cli(mode, source)?;
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let output = cli("--emit-lowir", source)?;
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty() && !output.stderr.is_empty());
    Ok(())
}

const ORDERED_RESULTS: &[(&str, &str)] = &[
    (
        "shared-method-input",
        "trait A{fn f<'a,T:Copy>(x:&'a T)->(impl Copy+'a,impl Copy+'a);} impl A for u8{fn f<'a,T:Copy>(x:&'a T)->(impl Copy+'a,impl Copy+'a){(x,x)}} fn main(){let n=7u8;let _=<u8 as A>::f(&n);}",
    ),
    (
        "distinct-method-inputs",
        "trait A{fn f<'a,T:Copy,U:Copy>(x:&'a T,y:&'a U)->(impl Copy+'a,impl Copy+'a);} impl A for u8{fn f<'a,T:Copy,U:Copy>(x:&'a T,y:&'a U)->(impl Copy+'a,impl Copy+'a){(x,y)}} fn main(){let n=7u8;let m=9u16;let _=<u8 as A>::f(&n,&m);}",
    ),
    (
        "parent-and-method-lifetimes",
        "trait A<'p>{fn f<'a,T:Copy,U:Copy>(x:&'a T,y:&'p U)->(impl Copy+'a,impl Copy+'p);} impl<'p> A<'p> for u8{fn f<'a,T:Copy,U:Copy>(x:&'a T,y:&'p U)->(impl Copy+'a,impl Copy+'p){(x,y)}} fn main(){let n=7u8;let m=9u16;let _=<u8 as A>::f(&n,&m);}",
    ),
];

#[test]
fn multiple_trait_results_allocate_stable_types_across_processes()
-> Result<(), Box<dyn std::error::Error>> {
    for &(name, source) in ORDERED_RESULTS {
        rustc::qualify(name, source, true)?;
        for mode in ["--emit-generic-semantics", "--emit-lowir"] {
            let first = cli(mode, source)?;
            assert!(
                first.status.success() && first.stderr.is_empty(),
                "{name}: {first:?}"
            );
            for _ in 0..16 {
                let repeated = cli(mode, source)?;
                assert!(repeated.status.success() && repeated.stderr.is_empty());
                assert_eq!(first.stdout, repeated.stdout, "{name}: {mode}");
            }
        }
    }
    Ok(())
}

#[test]
#[ignore = "requires pinned external CPPGM backend"]
fn multiple_trait_results_execute_with_complete_captures() -> Result<(), Box<dyn std::error::Error>>
{
    let cases: Vec<_> = ORDERED_RESULTS
        .iter()
        .map(|&(name, source)| (name, source, false))
        .collect();
    super::native::check_cases("ordered-trait-results/native", &cases)
}
