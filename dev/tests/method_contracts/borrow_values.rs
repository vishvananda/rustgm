//! Sized expectations constrain value-producing borrow operands before formation.
use super::{native, rustc, semantic};
const CHECK: &str = "fn check(b:bool){if !b{let z=0;let _=1/z;}}";
const REQUIRED_GAP: &str = "borrow-readiness-v18-early-nominal-value";
const CASES: &[(&str, &str, bool)] = &[
    (
        "borrow-context-v15-known-block-reference",
        r"fn check(b:bool){if !b{let z=0;let _=1/z;}}fn take(r:&u8)->u8{*r}fn g(b:&u8)->u8{take(&{let _=1;b})}fn main(){let _=g(&2);}
",
        false,
    ),
    (
        "borrow-context-v15-known-group-place",
        r"fn check(b:bool){if !b{let z=0;let _=1/z;}}fn take(r:&u8)->u8{*r}fn g(b:&u8)->u8{take(&(b))}fn main(){check(g(&2)==2);}
",
        true,
    ),
    (
        "borrow-context-v15-known-group-block",
        r"fn check(b:bool){if !b{let z=0;let _=1/z;}}fn take(r:&u8)->u8{*r}fn g(b:&u8)->u8{take(&({let _=1;b}))}fn main(){let _=g(&2);}
",
        false,
    ),
    (
        "borrow-context-v15-known-block-value",
        r"fn check(b:bool){if !b{let z=0;let _=1/z;}}fn take(r:&u8)->u8{*r}fn g(b:&u8)->u8{take(&{let _=1;*b})}fn main(){check(g(&2)==2);}
",
        true,
    ),
    (
        "borrow-context-v15-known-if-reference",
        r"fn check(b:bool){if !b{let z=0;let _=1/z;}}fn take(r:&u8)->u8{*r}fn g(b:&u8)->u8{take(&if true{b}else{b})}fn main(){let _=g(&2);}
",
        false,
    ),
    (
        "borrow-context-v15-known-if-value",
        r"fn check(b:bool){if !b{let z=0;let _=1/z;}}fn take(r:&u8)->u8{*r}fn g(b:&u8)->u8{take(&if true{*b}else{*b})}fn main(){check(g(&2)==2);}
",
        true,
    ),
    (
        "borrow-context-v15-unknown-block-reference",
        r"fn check(b:bool){if !b{let z=0;let _=1/z;}}fn keep<T>(r:&T)->&T{r}fn g(b:&u8)->u8{**keep(&{let _=1;b})}fn main(){check(g(&2)==2);}
",
        true,
    ),
    (
        "borrow-context-v15-competing-block-reference",
        r"fn check(b:bool){if !b{let z=0;let _=1/z;}}trait P<R=Self>{fn f(&self,r:&R)->u8;}impl P for u8{fn f(&self,r:&u8)->u8{*self+*r}}impl P<&u8>for u8{fn f(&self,r:&&u8)->u8{*self+**r+1}}fn g(a:&u8,b:&u8)->u8{a.f(&{let _=1;b})}fn main(){check(g(&1,&2)==4);}
",
        true,
    ),
    (
        "borrow-context-v15-mutable-block-value",
        r"fn check(b:bool){if !b{let z=0;let _=1/z;}}fn take(r:&mut u8){*r+=1}fn g(b:&mut u8){take(&mut {let _=1;*b});*b+=1;}fn main(){let mut x=2;g(&mut x);check(x==3);}
",
        true,
    ),
    (
        "borrow-context-v15-mutable-block-reference",
        r"fn check(b:bool){if !b{let z=0;let _=1/z;}}fn take(r:&mut u8){*r+=1}fn g(b:&mut u8){take(&mut {let _=1;b});}fn main(){}
",
        false,
    ),
    (
        "borrow-context-v15-mutable-known-borrowed-tail",
        r"fn check(b:bool){if !b{let z=0;let _=1/z;}}fn take(r:&&mut u8)->u8{**r}fn g(b:&mut u8)->u8{let x=take(&{let _=1;b});*b+=1;x}fn main(){let mut x=2;check(g(&mut x)==2);check(x==3);}
",
        true,
    ),
    (
        "borrow-context-v15-mutable-unknown-moves-tail",
        r"fn check(b:bool){if !b{let z=0;let _=1/z;}}fn keep<T>(_:&T){}fn g(b:&mut u8){keep(&{let _=1;b});*b+=1;}fn main(){}
",
        false,
    ),
    (
        "borrow-context-v15-known-match-reference",
        r"fn check(b:bool){if !b{let z=0;let _=1/z;}}fn take(r:&u8)->u8{*r}fn g(b:&u8)->u8{take(&match true{true=>b,false=>b})}fn main(){}
",
        false,
    ),
    (
        "borrow-context-v15-known-match-value",
        r"fn check(b:bool){if !b{let z=0;let _=1/z;}}fn take(r:&u8)->u8{*r}fn g(b:&u8)->u8{take(&match true{true=>*b,false=>*b})}fn main(){check(g(&2)==2);}
",
        true,
    ),
    (
        "borrow-context-v15-known-loop-reference",
        r"fn check(b:bool){if !b{let z=0;let _=1/z;}}fn take(r:&u8)->u8{*r}fn g(b:&u8)->u8{take(&loop{break b;})}fn main(){}
",
        false,
    ),
    (
        "borrow-context-v15-known-loop-value",
        r"fn check(b:bool){if !b{let z=0;let _=1/z;}}fn take(r:&u8)->u8{*r}fn g(b:&u8)->u8{take(&loop{break *b;})}fn main(){check(g(&2)==2);}
",
        true,
    ),
    (
        "unsized-borrow-v16-array-literal-block",
        r"fn check(b:bool){if !b{let z=0;let _=1/z;}}fn take(r:&[u8])->u8{match r{[a,b]=>*a+*b,_=>0}}fn main(){check(take(&{[1,2]})==3);}
",
        true,
    ),
    (
        "unsized-borrow-v16-array-place-block",
        r"fn check(b:bool){if !b{let z=0;let _=1/z;}}fn take(r:&[u8])->u8{match r{[a,b]=>*a+*b,_=>0}}fn g(a:[u8;2])->u8{take(&{a})}fn main(){check(g([1,2])==3);}
",
        true,
    ),
    (
        "unsized-borrow-v16-slice-reference-block",
        r"fn check(b:bool){if !b{let z=0;let _=1/z;}}fn take(r:&[u8])->u8{match r{[a,b]=>*a+*b,_=>0}}fn g(a:&[u8])->u8{take(&{a})}fn main(){check(g(&[1,2])==3);}
",
        true,
    ),
    (
        "unsized-borrow-v16-str-reference-block",
        r#"fn check(b:bool){if !b{let z=0;let _=1/z;}}fn take(r:&str)->bool{match r{"yes"=>true,_=>false}}fn g(a:&str)->bool{take(&{a})}fn main(){check(g("yes"));}
"#,
        true,
    ),
    (
        "unsized-borrow-v16-dynamic-block",
        r"fn check(b:bool){if !b{let z=0;let _=1/z;}}trait A{fn f(&self)->u8;}struct S;impl A for S{fn f(&self)->u8{3}}fn take(r:&dyn A)->u8{r.f()}fn main(){check(take(&{S})==3);}
",
        true,
    ),
    (
        "unsized-borrow-v16-generic-unsized-target",
        r"fn check(b:bool){if !b{let z=0;let _=1/z;}}fn take<T:?Sized>(_:&T)->u8{3}fn g<T:?Sized>(r:&T)->u8{take::<T>(&{r})}fn main(){}
",
        false,
    ),
    (
        "unsized-borrow-v16-generic-sized-target",
        r"fn check(b:bool){if !b{let z=0;let _=1/z;}}fn take<T>(_:&T)->u8{3}fn g<T>(r:&T)->u8{take::<T>(&{r})}fn main(){}
",
        false,
    ),
    (
        "unsized-borrow-v16-array-element-mismatch",
        r"fn check(b:bool){if !b{let z=0;let _=1/z;}}fn take(r:&[u8])->u8{match r{[a,b]=>*a+*b,_=>0}}fn main(){let _=take(&{[1u16,2u16]});}
",
        false,
    ),
    (
        "borrow-readiness-v18-nominal-dst-block",
        r"fn check(b:bool){if !b{let z=0;let _=1/z;}}struct Tail<T:?Sized>{v:T}fn take(r:&Tail<[u8]>)->u8{match &r.v{[a,b]=>*a+*b,_=>0}}fn g(r:&Tail<[u8]>)->u8{take(&{r})}fn main(){let a=Tail{v:[1u8,2]};check(g(&a)==3);}
",
        true,
    ),
    (
        "borrow-readiness-v18-nominal-dst-value",
        r"fn check(b:bool){if !b{let z=0;let _=1/z;}}struct Tail<T:?Sized>{v:T}fn take(r:&Tail<[u8]>)->u8{match &r.v{[a,b]=>*a+*b,_=>0}}fn main(){check(take(&{Tail{v:[1u8,2]}})==3);}
",
        true,
    ),
    (
        "borrow-readiness-v18-fixed-dst-block",
        r"fn check(b:bool){if !b{let z=0;let _=1/z;}}struct Tail{v:[u8]}fn take(r:&Tail)->u8{match &r.v{[a,b]=>*a+*b,_=>0}}fn g(r:&Tail)->u8{take(&{r})}fn main(){}
",
        true,
    ),
    (
        "borrow-readiness-v18-generic-dst-hint",
        r"fn check(b:bool){if !b{let z=0;let _=1/z;}}struct Tail<T:?Sized>{v:T}fn take<T:?Sized>(_:&Tail<T>)->u8{3}fn g<T:?Sized>(r:&Tail<T>)->u8{take::<T>(&{r})}fn main(){}
",
        false,
    ),
    (
        "borrow-readiness-v18-nominal-sized-block",
        r"fn check(b:bool){if !b{let z=0;let _=1/z;}}struct S(u8);fn take(r:&S)->u8{r.0}fn g(r:&S)->u8{take(&{r})}fn main(){}
",
        false,
    ),
    (
        "borrow-readiness-v18-early-nominal-dst",
        r"fn check(b:bool){if !b{let z=0;let _=1/z;}}const A:Tail<[u8;2]>=Tail{v:[1,2]};const N:u8=take(&{&A});const fn take(r:&Tail<[u8]>)->u8{match &r.v{[a,b]=>*a+*b,_=>0}}struct Tail<T:?Sized>{v:T}fn main(){check(N==3);}
",
        false,
    ),
    (
        "borrow-readiness-v18-raw-pointer-block",
        r"fn check(b:bool){if !b{let z=0;let _=1/z;}}fn g(r:&u8)->*const u8{&{r}}fn main(){}
",
        false,
    ),
    (
        "borrow-readiness-v18-raw-pointer-group",
        r"fn check(b:bool){if !b{let z=0;let _=1/z;}}fn g(r:&u8)->*const u8{&(r)}fn main(){let x=2;check(unsafe{*g(&x)}==2);}
",
        false,
    ),
    (
        "borrow-readiness-v18-raw-pointer-value",
        r"fn check(b:bool){if !b{let z=0;let _=1/z;}}fn main(){let x=2;let p:*const u8=&{x};check(unsafe{*p}==2);}
",
        true,
    ),
    (
        "borrow-readiness-v18-reference-header-suspension",
        r"fn check(b:bool){if !b{let z=0;let _=1/z;}}trait A{const C:usize;}struct X<const N:usize>;trait K<R>{}impl K<u8> for X<{<S0 as A>::C}>{}const N:u8=accept(&&2u8);const fn accept<R>(_:&R)->u8 where X<3>:K<R>{3}struct S0;impl A for S0{const C:usize=3;}fn main(){check(N==3);}
",
        true,
    ),
    (
        "borrow-readiness-v18-early-nominal-value",
        r"fn check(b:bool){if !b{let z=0;let _=1/z;}}const N:u8=take(&{Tail{v:[1u8,2]}});const fn take(r:&Tail<[u8]>)->u8{match &r.v{[a,b]=>*a+*b,_=>0}}struct Tail<T:?Sized>{v:T}fn main(){check(N==3);}
",
        true,
    ),
    (
        "borrow-readiness-v18-raw-pointer-place",
        r"fn check(b:bool){if !b{let z=0;let _=1/z;}}fn g(r:&u8)->*const u8{&*(r)}fn main(){let x=2;check(unsafe{*g(&x)}==2);}
",
        true,
    ),
    (
        "early-nominal-dependency",
        r"fn check(b:bool){if !b{let z=0;let _=1/z;}}const N:u8=take(&{Tail{v:[1u8,2]}});const fn take(_:&Tail<[u8]>)->u8{3}struct Tail<T:?Sized>{v:T}fn main(){check(N==3);}
",
        true,
    ),
];
#[test]
fn borrowed_values_preserve_sized_expectations_and_unsized_hints()
-> Result<(), Box<dyn std::error::Error>> {
    let mut failures = Vec::new();
    for &(name, source, accepted) in CASES {
        rustc::qualify(&format!("borrow-value-{name}"), source, accepted)?;
        let result = (|| {
            let mut report = semantic::resolve(source)?;
            let rustgm::pipeline::Analysis {
                mut facts,
                prepared,
            } = rustgm::pipeline::analyze_source(&report).map_err(|e| e.to_string())?;
            if name.ends_with("reference-header-suspension") {
                eprintln!(
                    "reference input suspensions={}",
                    facts.body_work.suspensions
                );
                assert!(facts.body_work.suspensions > 0);
            }
            let entry =
                rustgm::sema::entry::select(&mut report, &facts).map_err(|e| e.to_string())?;
            rustgm::lower::instances::build_prepared(&report, &mut facts, entry, prepared)
                .map(|_| ())
                .map_err(|e| e.to_string())
        })();
        // The unchanged original is Rust-positive but still needs CTFE's
        // nested-DST unsizing metadata. Keep it required, not passing coverage.
        if name == REQUIRED_GAP {
            assert!(
                result.as_ref().is_err_and(
                    |error| error.contains("constant slice place lacks length metadata")
                ),
                "{result:?}"
            );
        } else if result.is_ok() != accepted {
            failures.push(format!("{name}: {result:?}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn borrowed_values_execute_without_losing_metadata_or_reborrows()
-> Result<(), Box<dyn std::error::Error>> {
    // Empty-main generic/fixed-DST controls are codegen-only; source checks own
    // those uninstantiated signatures, rather than claiming runtime invocation.
    let cases = CASES
        .iter()
        .filter(|case| case.2 && case.0 != REQUIRED_GAP)
        .map(|&(name, source, _)| {
            Ok((
                name,
                source.strip_prefix(CHECK).ok_or("check prefix")?,
                false,
            ))
        })
        .collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()?;
    native::check_cases("parallel/borrow-values", &cases)
}
