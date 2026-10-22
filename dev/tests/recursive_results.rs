//! Defining recursion preserves capture identity and independently checked hidden types.
use crate::native_references as native;
use crate::qualify as rustc;
use crate::semantic;

const CASES: &[(&str, &str, bool)] = &[
    ("direct", "fn f()->impl Copy{f()}fn main(){}", true),
    (
        "never-bound",
        "trait A{}fn f()->impl A{f()}fn main(){}",
        false,
    ),
    (
        "unit-bound",
        "trait A{}impl A for (){}fn f()->impl A{f()}fn main(){}",
        true,
    ),
    (
        "anchored",
        "fn f(n:u8)->impl Copy{if n==0{1u8}else{f(n-1)}}fn main(){}",
        true,
    ),
    (
        "reverse",
        "fn f(n:u8)->impl Copy{if n!=0{f(n-1)}else{1u8}}fn main(){}",
        true,
    ),
    (
        "generic",
        "fn f<T:Copy>(n:u8,x:T)->impl Copy{if n==0{x}else{f(n-1,x)}}fn main(){}",
        true,
    ),
    (
        "changed",
        "fn f<T:Copy>(n:u8,x:T)->impl Copy{if n==0{1u8}else{f(n-1,true)}}fn main(){}",
        false,
    ),
    (
        "changed-hidden",
        "fn f<T:Copy>(n:u8,x:T)->impl Copy{if n==0{x}else{f(n-1,true)}}fn main(){}",
        false,
    ),
    (
        "ref",
        "fn f<'a>(n:u8,x:&'a u8)->impl Copy+'a{if n==0{x}else{f(n-1,x)}}fn main(){}",
        true,
    ),
    (
        "pair",
        "fn f(n:u8)->(impl Copy,impl Copy){if n==0{(1u8,false)}else{f(n-1)}}fn main(){}",
        true,
    ),
    (
        "nested",
        "fn f(n:u8)->impl Copy{if n==0{(1u8,)}else{(f(n-1),)}}fn main(){}",
        false,
    ),
    (
        "mutual",
        "fn f(n:u8)->impl Copy{if n==0{1u8}else{g(n-1)}}fn g(n:u8)->impl Copy{f(n)}fn main(){}",
        false,
    ),
    (
        "nested-ref",
        "fn f<'a>(n:u8,x:&'a u8)->impl Copy+'a{if n==0{x}else{let y=2;f(n-1,&y)}}fn main(){}",
        false,
    ),
    (
        "permuted",
        "fn f<T:Copy,U:Copy>(n:u8,x:T,y:U)->impl Copy{if n==0{1u8}else{f(n-1,y,x)}}fn main(){}",
        false,
    ),
    (
        "permuted-hidden",
        "fn f<T:Copy,U:Copy>(n:u8,x:T,y:U)->impl Copy{if n==0{x}else{f(n-1,y,x)}}fn main(){}",
        false,
    ),
    (
        "ignored-change",
        "fn f<T:Copy>(n:u8,x:T)->impl Copy{if n!=0{let _=f(n-1,true);}x}fn main(){}",
        true,
    ),
    (
        "unused-lifetime",
        "fn f<'a>(n:u8,x:&'a u8)->impl Copy{if n==0{1u8}else{let y=2;f(n-1,&y)}}fn main(){}",
        false,
    ),
    (
        "unused-explicit",
        "fn f<'a>(n:u8,x:&'a u8)->impl Copy+use<>{if n==0{1u8}else{let y=2;f(n-1,&y)}}fn main(){}",
        true,
    ),
    (
        "const-changed",
        "fn f<const N:usize>(n:u8)->impl Copy{if n==0{1u8}else{f::<3>(n-1)}}fn main(){}",
        false,
    ),
    (
        "const-permuted",
        "fn f<const N:usize,const M:usize>(n:u8)->impl Copy{if n==0{1u8}else{f::<M,N>(n-1)}}fn main(){}",
        false,
    ),
    (
        "early-return",
        "fn f(n:u8)->impl Copy{if n!=0{return f(n-1);}1u8}fn main(){}",
        true,
    ),
    (
        "repeated-param",
        "fn f<T:Copy,U:Copy>(n:u8,x:T,y:U)->impl Copy{if n==0{1u8}else{f(n-1,x,x)}}fn main(){}",
        false,
    ),
    (
        "identity-only",
        "fn f<T:Copy>(x:T)->impl Copy{f(x)}fn main(){}",
        true,
    ),
];
#[test]
fn defining_recursion_matches_rustc() -> Result<(), Box<dyn std::error::Error>> {
    let mut failures = Vec::new();
    for &(name, source, accepted) in CASES {
        rustc::qualify(&format!("recursive-result-{name}"), source, accepted)?;
        let result = semantic::analyze(source).and_then(|(mut report, mut facts)| {
            let entry =
                rustgm::sema::entry::select(&mut report, &facts).map_err(|e| e.to_string())?;
            rustgm::lower::instances::build(&report, &mut facts, entry).map_err(|e| e.to_string())
        });
        if result.is_ok() != accepted {
            failures.push(format!("{name}: expected {accepted}; {:?}", result.err()));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

const VALUE: &str = "trait Value{fn value(&self)->u8;}impl Value for u8{fn value(&self)->u8{*self}}impl<'a>Value for &'a u8{fn value(&self)->u8{**self}}";
const EXECUTION: &[(&str, &str)] = &[
    (
        "anchored",
        "fn f(n:u8)->impl Value{if n==0{103u8}else{f(n-1)}}fn main(){check(f(4).value()==103);}",
    ),
    (
        "reverse",
        "fn f(n:u8)->impl Value{if n!=0{f(n-1)}else{107u8}}fn main(){check(f(4).value()==107);}",
    ),
    (
        "generic",
        "fn f<T:Copy+Value>(n:u8,x:T)->impl Value{if n==0{x}else{f(n-1,x)}}fn main(){check(f(4,109u8).value()==109);}",
    ),
    (
        "reference",
        "fn f<'a>(n:u8,x:&'a u8)->impl Value+'a{if n==0{x}else{f(n-1,x)}}fn main(){let x=113;check(f(4,&x).value()==113);}",
    ),
    (
        "pair",
        "fn f(n:u8)->(impl Value,impl Value){if n==0{(127u8,131u8)}else{f(n-1)}}fn main(){let p=f(4);check(p.0.value()==127);check(p.1.value()==131);}",
    ),
    (
        "precise",
        "fn f<'a>(n:u8,x:&'a u8)->impl Value+use<>{if n==0{137u8}else{let y=2;f(n-1,&y)}}fn main(){check(f(4,&1).value()==137);}",
    ),
    (
        "inherent",
        "struct S;impl S{fn f(n:u8)->impl Value{if n==0{139u8}else{Self::f(n-1)}}}fn main(){check(S::f(4).value()==139);}",
    ),
];
#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn recursive_results_execute_like_rustc() -> Result<(), Box<dyn std::error::Error>> {
    let owned: Vec<_> = EXECUTION
        .iter()
        .map(|(name, body)| (*name, format!("{VALUE}{body}")))
        .collect();
    let cases: Vec<_> = owned
        .iter()
        .map(|(name, source)| (*name, source.as_str(), false))
        .collect();
    native::check_cases("recursive-results/native", &cases)
}
