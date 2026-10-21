//! PA20 rows expose retained contracts, complete arguments and scoped normalization results.
use super::{Fails, cli, native, rustc, semantic};
use rustgm::sema::semantic_view::relationships;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const MEMBERS: &str = r"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{type Out:Copy;const C:u8=7;}struct S;struct U;impl A for S{type Out=u8;}impl A for U{type Out=u16;const C:u8=9;}const DEFAULT:u8=<S as A>::C;const OVERRIDE:u8=<U as A>::C;fn f<T:A>(x:T::Out)->T::Out{x}fn main(){check(f::<S>(DEFAULT)==7);check(f::<U>(OVERRIDE as u16)==9);}
";
const MIXED: &str = r"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A<'a,T:'a,const N:usize>{type Out;const C:usize=N;}struct S;impl<'a,T:'a,const N:usize>A<'a,T,N>for S{type Out=&'a T;}type X=<S as A<'static,u8,3>>::Out;const C:usize=<S as A<'static,u8,3>>::C;static V:u8=7;fn main(){let x:X=&V;check(*x==7&&C==3);}
";
const HEADER: &str = r"fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait B{fn value()->usize;}struct X<const N:usize>;trait A{type Out;const C:Self::Out;}impl B for X<{<u8 as A>::C}>{fn value()->usize{3}}impl A for u8{type Out=usize;const C:usize=3;}fn main(){check(<X<3>as B>::value()==3);}
";
const PENDING: &str = r"trait A<'a>{type Out;}impl<'a>A<'a>for u8{type Out=&'a u8;}fn bad<'a>(x:&'a u8)-><u8 as A<'static>>::Out{x}fn main(){}
";
const MISSING: &str = r"trait A{type Out;}struct S;impl A for S{}fn main(){}
";

fn inspect(source: &str) -> TestResult<String> {
    let report = semantic::resolve(source)?;
    // CLI inspection completes required initializer selection as well as body
    // inference; the lower-level helper intentionally stops before that gate.
    let facts = rustgm::pipeline::analyze_source(&report)?.facts;
    let mut bytes = Vec::new();
    relationships::write(&report, &facts, &mut bytes)?;
    let mut count = Vec::new();
    relationships::count(&report, &facts, &mut count)?;
    assert!(bytes.starts_with(b"trait-semantics 1\n"));
    assert!(bytes.ends_with(&[b"eof ".as_slice(), &count].concat()));
    let counted = cli("--count-trait-semantics", source)?;
    assert!(counted.status.success() && counted.stderr.is_empty());
    assert_eq!(counted.stdout, count);
    for _ in 0..3 {
        let output = cli("--emit-trait-semantics", source)?;
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stderr.is_empty());
        assert_eq!(output.stdout, bytes);
    }
    Ok(String::from_utf8(bytes)?)
}

#[test]
fn associated_contracts_and_default_overrides_have_hand_checked_rows() -> TestResult {
    rustc::qualify("relationship-members", MEMBERS, true)?;
    let text = inspect(MEMBERS)?;
    // Definition order follows the fixed source. The default selects the trait
    // constant itself, whereas U's override selects its distinct impl member.
    for row in [
        "associated-type def=17 type=22",
        "result-contract def=17 projection=22",
        "result-bound def=17 index=0 subject=22 bound=copy",
        "result-bound def=17 index=1 subject=22 bound=sized",
        "associated-type def=24 type=11",
        "associated-type def=26 type=12",
        "impl-constant owner=23 declaration=18 selected=18 default=true",
        "impl-constant owner=25 declaration=18 selected=27 default=false",
        "constant-dispatch declaration=18 arguments=[type:19] selected=18 arguments=[type:19]",
        "constant-dispatch declaration=18 arguments=[type:20] selected=27 arguments=[]",
    ] {
        assert!(text.lines().any(|line| line == row), "{row}: {text}");
    }
    assert!(text.ends_with("type-members=3 constant-members=2 result-contracts=1 result-bounds=2 impl-constants=2 constant-uses=2 constant-dispatches=2 normalized-nodes=4 normalized-definitions=0\n"));
    Ok(())
}

#[test]
fn mixed_arguments_and_early_dispatch_keep_complete_selected_identities() -> TestResult {
    for (name, source) in [
        ("relationship-mixed", MIXED),
        ("relationship-header", HEADER),
    ] {
        rustc::qualify(name, source, true)?;
        let text = inspect(source)?;
        if source == MIXED {
            assert!(text.lines().any(|line|line=="constant-use source=0:114 definition=18 arguments=[region:static,type:19,type:11,const:16:0x00000000000000000000000000000003]"));
            assert!(
                text.lines()
                    .any(|line| line == "normalized-definition def=23 original=29 result=31")
            );
        } else {
            assert!(text.lines().any(|line|line=="constant-dispatch declaration=23 arguments=[type:11] selected=28 arguments=[]"));
        }
    }
    Ok(())
}

#[test]
fn relationship_inspection_preserves_later_lifetime_errors_and_atomic_rejections() -> TestResult {
    rustc::qualify("relationship-pending-lifetime", PENDING, false)?;
    let text = inspect(PENDING)?;
    assert!(text.contains(" pending\n"));
    let output = cli("--emit-lowir", PENDING)?;
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    rustc::qualify("relationship-missing-definition", MISSING, false)?;
    for mode in ["--emit-trait-semantics", "--count-trait-semantics"] {
        let output = cli(mode, MISSING)?;
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        assert!(!output.stderr.is_empty());
    }
    Ok(())
}

#[test]
fn empty_relationship_counts_and_output_failures_are_explicit() -> TestResult {
    let text = inspect("fn main(){}\n")?;
    assert!(text.ends_with("eof owners=0 members=0 parameters=0 predicates=0 impls=0 calls=0 paths=0 impl-uses=0 type-members=0 constant-members=0 result-contracts=0 result-bounds=0 impl-constants=0 constant-uses=0 constant-dispatches=0 normalized-nodes=0 normalized-definitions=0\n"));
    let (report, facts) = semantic::analyze(MEMBERS)?;
    assert!(relationships::write(&report, &facts, &mut Fails).is_err());
    assert!(relationships::count(&report, &facts, &mut Fails).is_err());
    Ok(())
}

#[test]
#[ignore = "requires the qualified CPPGM LowIR-to-native backend"]
fn inspected_associated_values_execute_with_the_reported_member_choices() -> TestResult {
    let cases = [("members", MEMBERS), ("mixed", MIXED), ("header", HEADER)]
        .into_iter()
        .map(|(name, source)| {
            source
                .strip_prefix(native::CHECK)
                .map(|body| (name, body, false))
                .ok_or("fixture check prelude changed")
        })
        .collect::<Result<Vec<_>, _>>()?;
    native::check_cases("relationship-view/native", &cases)
}
