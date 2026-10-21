//! Constants and promotions preserve closure code identity without permitting const calls.
use super::{native, rustc, semantic};
type TestResult = Result<(), Box<dyn std::error::Error>>;
const CASES: &[(&str, &str, bool)] = &[
    (
        "named",
        "const F:fn(u8)->u8=|x|x+1;fn main(){check(F(7)==8);}",
        true,
    ),
    (
        "static",
        "static F:fn(u8)->u8=|x|x+1;fn main(){check(F(7)==8);}",
        true,
    ),
    (
        "array",
        "const F:[fn(u8)->u8;2]=[|x|x+1,|x|x+2];fn main(){check(F[0](7)==8);check(F[1](7)==9);}",
        true,
    ),
    (
        "static-array",
        "static F:[fn(u8)->u8;2]=[|x|x+1,|x|x+2];fn main(){check(F[0](7)==8);check(F[1](7)==9);}",
        true,
    ),
    (
        "repeat",
        "const F:[fn(u8)->u8;3]=[|x|x+2;3];fn main(){check(F[2](7)==9);}",
        true,
    ),
    (
        "factory",
        "const fn make()->fn(u8)->u8{|x|x+1}const F:fn(u8)->u8=make();fn main(){check(F(7)==8);}",
        true,
    ),
    (
        "generic-factory",
        "const fn make<const N:u8>()->fn(u8)->u8{|x|x+N}const F:fn(u8)->u8=make::<2>();const G:fn(u8)->u8=make::<3>();fn main(){check(F(7)==9);check(G(7)==10);}",
        true,
    ),
    (
        "local",
        "const F:fn(u8)->u8={let f=|x|x+1;f};fn main(){check(F(7)==8);}",
        true,
    ),
    (
        "cast",
        "const F:fn(u8)->u8=(|x:u8|x+1) as fn(u8)->u8;fn main(){check(F(7)==8);}",
        true,
    ),
    (
        "borrowed-result",
        "const F:fn(&u8)->&u8=|x|x;fn main(){let x=9;check(*F(&x)==9);}",
        true,
    ),
    (
        "promotion",
        "fn main(){let fs:&[fn(u8)->u8;2]=&[|x|x+1,|x|x+2];check(fs[0](7)==8);check(fs[1](7)==9);}",
        true,
    ),
    (
        "cast-promotion",
        "fn main(){let f=&((|x:u8|x+1) as fn(u8)->u8);check(f(7)==8);}",
        true,
    ),
    (
        "function-control",
        "fn add(x:u8)->u8{x+1}const F:fn(u8)->u8=add;fn main(){check(F(7)==8);}",
        true,
    ),
    (
        "generic-promotion",
        "fn make<const N:u8>()->&'static fn(u8)->u8{&((|x:u8|x+N) as fn(u8)->u8)}fn main(){check(make::<2>()(7)==9);check(make::<3>()(7)==10);}",
        true,
    ),
    ("runtime-only", "const X:u8=(||1)();fn main(){}", false),
    (
        "pointer-runtime-only",
        "const F:fn(u8)->u8=|x|x+1;const X:u8=F(7);fn main(){}",
        false,
    ),
    (
        "capture",
        "const F:fn()->u8={let x=7;move||x};fn main(){}",
        false,
    ),
    (
        "unit-capture",
        "const F:fn()={let x=();move||x};fn main(){}",
        false,
    ),
];
#[test]
fn source_outcomes_match_rustc() -> TestResult {
    let mut failures = Vec::new();
    for &(name, source, expected) in CASES {
        let source = format!("{}{source}", native::CHECK);
        rustc::qualify(&format!("closure-constant-{name}"), &source, expected)?;
        let mut report = semantic::resolve(&source)?;
        let actual = rustgm::pipeline::analyze_source(&report).and_then(|analysis| {
            let mut facts = analysis.facts;
            let entry = rustgm::sema::entry::select(&mut report, &facts)?;
            rustgm::lower::instances::build_prepared(&report, &mut facts, entry, analysis.prepared)
        });
        if matches!(name, "runtime-only" | "pointer-runtime-only") {
            assert!(
                actual
                    .as_ref()
                    .err()
                    .is_some_and(|error| error.to_string().contains("non-const calls")),
                "{name}: {:?}",
                actual.as_ref().err()
            );
        }
        if actual.is_ok() != expected {
            failures.push(format!("{name}: {:?}", actual.err()));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
#[test]
#[ignore = "requires the pinned CPPGM native backend"]
fn constants_and_promotions_execute_selected_closures() -> TestResult {
    let cases: Vec<_> = CASES
        .iter()
        .filter(|case| case.2)
        .map(|&(name, source, _)| (name, source, false))
        .collect();
    native::check_cases("parallel/closure-constants/native", &cases)
}
