//! Opaque result promises are usable independently of their captured inputs.
#[path = "harness/native_references.rs"]
mod native;
#[path = "helpers/qualify.rs"]
mod rustc;
#[path = "support/semantic.rs"]
mod semantic;

const ACCEPTED: &[(&str, &str)] = &[
    (
        "custom-wrapper-drop",
        "struct W<T>(T);impl<T>Drop for W<T>{fn drop(&mut self){check(true);}}fn f<'a>(_:&'a u8)->impl Copy+'static{7u8}fn main(){let x;{let a=1;x=W(f(&a));}let _=x;}",
    ),
    (
        "static-among-bounds",
        "fn f<'a,'b>(_:&'a u8,_:&'b u8)->impl Copy+'a+'static+'b{7u8}fn take<T:'static>(_:T){}fn main(){let x;{let a=1;let b=2;x=f(&a,&b);}take(x);}",
    ),
    (
        "repeated-bound",
        "fn f<'a,'b>(x:&'a u8,_:&'b u8)->impl Copy+'a+'a{x}fn take<T>(_:T){}fn main(){let a=1;let x;{let b=2;x=f(&a,&b);}take(x);}",
    ),
    (
        "multiple-bounds-local",
        "fn f<'a,'b>(_:&'a u8,_:&'b u8)->impl Copy+'a+'b{7u8}fn take<T>(_:T){}fn main(){let a=1;let b=2;take(f(&a,&b));}",
    ),
    (
        "static-scalar",
        "fn f<'a>(_:&'a u8)->impl Copy+'static{7u8}fn take<T:'static>(_:T){}fn main(){let x;{let a=1;x=f(&a);}take(x);}",
    ),
    (
        "static-reference",
        "trait Value{fn value(&self)->u8;}impl<'a>Value for &'a u8{fn value(&self)->u8{**self}}fn f<'a>(_:&'a u8)->impl Value+'static{&7u8}fn main(){let x;{let a=1;x=f(&a);}check(x.value()==7);}",
    ),
    (
        "inherited-static",
        "trait Forever:'static{}impl Forever for u8{}fn f<'a>(_:&'a u8)->impl Forever{7u8}fn take<T:'static>(_:T){}fn main(){let x;{let a=1;x=f(&a);}take(x);}",
    ),
    (
        "inherited-trait",
        "trait Base{fn value(&self)->u8;}trait Derived:Base{}impl Base for u8{fn value(&self)->u8{*self}}impl Derived for u8{}fn f()->impl Derived{11u8}fn take<T:Base>(x:T)->u8{x.value()}fn main(){check(take(f())==11);}",
    ),
    (
        "type-capture",
        "fn f<T>(_:T)->impl Copy+'static{13u8}fn take<T:'static>(_:T){}fn main(){let x;{let a=1;x=f(&a);}take(x);}",
    ),
    (
        "named-lifetime",
        "fn f<'a>(x:&'a u8)->impl Copy+'a{x}fn take<'a,T:'a>(_:T,_:&'a u8){}fn main(){let a=17;take(f(&a),&a);}",
    ),
    (
        "excluded-live-input",
        "trait Value{fn value(&self)->u8;}impl<'a>Value for &'a u8{fn value(&self)->u8{**self}}fn f<'a,'b>(x:&'a u8,_:&'b u8)->impl Value+'a{x}fn main(){let a=19;let x;{let b=1;x=f(&a,&b);}check(x.value()==19);}",
    ),
];
const REJECTED: &[(&str, &str)] = &[
    (
        "no-reverse-capture-bound",
        "fn f<'a>(_:&'a u8)->impl Copy+'static{7u8}fn g<'a>(x:&'a u8)->&'static u8{let _=f(x);x}fn main(){}",
    ),
    (
        "custom-wrapper-keeps-input",
        "struct W<T>(T);impl<T>Drop for W<T>{fn drop(&mut self){}}fn f<'a>(x:&'a u8)->impl Copy{x}fn main(){let x;{let a=1;x=W(f(&a));}let _=x;}",
    ),
    (
        "multiple-bounds-escape",
        "fn f<'a,'b>(_:&'a u8,_:&'b u8)->impl Copy+'a+'b{7u8}fn take<T>(_:T){}fn main(){let a=1;let x;{let b=2;x=f(&a,&b);}take(x);}",
    ),
    (
        "named-keeps-loan",
        "fn f<'a>(x:&'a u8)->impl Copy+'a{x}fn take<T>(_:T){}fn main(){let mut a=1;let x=f(&a);a=2;take(x);}",
    ),
    (
        "false-static",
        "fn f<'a>(x:&'a u8)->impl Copy+'static{x}fn main(){}",
    ),
    (
        "false-inherited",
        "trait Forever:'static{}impl Forever for u8{}fn f<'a>(x:&'a u8)->impl Forever{x}fn main(){}",
    ),
    (
        "no-static-promise",
        "fn f<'a>(_:&'a u8)->impl Copy{7u8}fn take<T:'static>(_:T){}fn main(){let a=1;take(f(&a));}",
    ),
    (
        "named-escape",
        "fn f<'a>(x:&'a u8)->impl Copy+'a{x}fn take<T>(_:T){}fn main(){let x;{let a=1;x=f(&a);}take(x);}",
    ),
];

#[test]
fn public_promises_match_source_outcomes() -> Result<(), Box<dyn std::error::Error>> {
    let mut failures = Vec::new();
    for (cases, accepted) in [(ACCEPTED, true), (REJECTED, false)] {
        for &(name, body) in cases {
            let source = format!("{}{body}", native::CHECK);
            rustc::qualify(&format!("result-promises-{name}"), &source, accepted)?;
            let result = semantic::analyze(&source).and_then(|(mut report, mut facts)| {
                let entry =
                    rustgm::sema::entry::select(&mut report, &facts).map_err(|e| e.to_string())?;
                rustgm::lower::instances::build(&report, &mut facts, entry)
                    .map_err(|e| e.to_string())
            });
            if result.is_ok() != accepted {
                failures.push(format!("{name}: {:?}", result.err()));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn public_promises_execute_like_rustc() -> Result<(), Box<dyn std::error::Error>> {
    let cases: Vec<_> = ACCEPTED
        .iter()
        .map(|&(name, body)| (name, body, false))
        .collect();
    native::check_cases("opaque-result-promises/native", &cases)
}
