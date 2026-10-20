//! Anonymous function inputs retain ordinary generic proofs and cannot be named in turbofish arguments.
#[path = "harness/native_references.rs"]
mod native;
#[path = "helpers/qualify.rs"]
mod rustc;
#[path = "support/semantic.rs"]
mod semantic;

const CASES: &[(&str, &str, bool)] = &[
    (
        "copy",
        "fn f(x:impl Copy){let _y=x;let _z=x;}fn main(){f(7u8);}",
        true,
    ),
    (
        "trait-call",
        "trait Value{fn value(&self)->u8;}impl Value for u8{fn value(&self)->u8{*self}}fn f(x:impl Value)->u8{x.value()}fn main(){check(f(7u8)==7);}",
        true,
    ),
    (
        "named-and-anonymous",
        "trait Value{fn value(&self)->u8;}impl Value for u8{fn value(&self)->u8{*self}}fn f<T:Copy>(t:T,x:impl Value)->u8{let _=t;x.value()}fn main(){check(f::<u16>(1u16,7u8)==7);}",
        true,
    ),
    (
        "explicit-anonymous",
        "fn f(x:impl Copy){}fn main(){f::<u8>(1);}",
        false,
    ),
    (
        "missing-named",
        "fn f<T:Copy,U:Copy>(t:T,u:U,x:impl Copy){}fn main(){f::<u8>(1,2,3);}",
        false,
    ),
    (
        "distinct-inputs",
        "fn f(x:impl Copy,y:impl Copy){let _=(x,y);}fn main(){f(7u8,true);}",
        true,
    ),
    (
        "reference",
        "trait Value{fn value(&self)->u8;}impl Value for u8{fn value(&self)->u8{*self}}fn f(x:&impl Value)->u8{x.value()}fn main(){check(f(&7u8)==7);}",
        true,
    ),
    (
        "nested-nominal",
        "trait Value{fn value(&self)->u8;}impl Value for u8{fn value(&self)->u8{*self}}struct W<T>(T);fn f(x:W<impl Value>)->u8{x.0.value()}fn main(){check(f(W(7u8))==7);}",
        true,
    ),
    (
        "trait-method",
        "trait Value{fn value(&self)->u8;}impl Value for u8{fn value(&self)->u8{*self}}trait A{fn f(x:impl Value)->u8;}struct S;impl A for S{fn f(x:impl Value)->u8{x.value()}}fn main(){check(S::f(7u8)==7);}",
        true,
    ),
    (
        "trait-method-mismatch",
        "trait A{fn f(x:impl Copy);}struct S;impl A for S{fn f<T:Copy>(x:T){}}fn main(){}",
        false,
    ),
    (
        "inherent-method",
        "trait Value{fn value(&self)->u8;}impl Value for u8{fn value(&self)->u8{*self}}struct S;impl S{fn f(&self,x:impl Value)->u8{x.value()}}fn main(){check(S.f(7u8)==7);}",
        true,
    ),
    (
        "method-turbofish",
        "trait Value{fn value(&self)->u8;}impl Value for u8{fn value(&self)->u8{*self}}struct S;impl S{fn f(&self,x:impl Value)->u8{x.value()}}fn main(){let _=S.f::<u8>(7u8);}",
        false,
    ),
    (
        "captured-lifetime",
        "trait Value{fn value(&self)->u8;}impl Value for u8{fn value(&self)->u8{*self}}fn f<'a>(x:impl Value+'a,y:&'a u8)->u8{x.value()+*y}fn main(){check(f(7u8,&1)==8);}",
        true,
    ),
    (
        "higher-ranked-input",
        "trait A<'a>{}impl<'a>A<'a>for u8{}fn f(x:impl for<'a>A<'a>){}fn main(){f(7u8);}",
        true,
    ),
    (
        "wrong-implementation",
        "trait Value{fn value(&self)->u8;}impl Value for u8{fn value(&self)->u8{*self}}fn f(x:impl Value)->u8{x.value()}fn main(){let _=f(true);}",
        false,
    ),
    ("empty-bound", "fn f(x:impl){}fn main(){}", false),
    ("lifetime-only", "fn f<'a>(x:impl 'a){}fn main(){}", false),
    (
        "precise-input",
        "fn f<T>(x:impl Copy+use<T>){}fn main(){}",
        false,
    ),
    (
        "relaxed-input",
        "fn f(x:&impl ?Sized){}fn main(){f(&7u8);}",
        true,
    ),
];
fn lower(source: &str) -> Result<(), String> {
    let (mut report, mut facts) = semantic::analyze(source)?;
    let entry = rustgm::sema::entry::select(&mut report, &facts).map_err(|e| e.to_string())?;
    rustgm::lower::instances::build(&report, &mut facts, entry)
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[test]
fn anonymous_inputs_match_independent_generic_and_call_rules()
-> Result<(), Box<dyn std::error::Error>> {
    let mut failures = Vec::new();
    for &(name, source, accepted) in CASES {
        let source = format!("{}{source}", native::CHECK);
        rustc::qualify(&format!("anonymous-inputs-{name}"), &source, accepted)?;
        let result = lower(&source);
        if result.is_ok() != accepted {
            failures.push(format!("{name}: expected {accepted}, got {result:?}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn anonymous_inputs_execute_through_generic_calls() -> Result<(), Box<dyn std::error::Error>> {
    let cases: Vec<_> = CASES
        .iter()
        .filter(|&&(_, source, accepted)| accepted && source.contains("check("))
        .map(|&(name, source, _)| (name, source, false))
        .collect();
    native::check_cases("anonymous-inputs/native", &cases)
}
