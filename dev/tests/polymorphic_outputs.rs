//! Return contracts preserve quantified subjects, inherited bounds and equalities.
use crate::native_references as native;
use crate::qualify as rustc;
use crate::semantic;

const ACCEPTED: &[(&str, &str)] = &[
    (
        "quantified-static-superbound",
        "trait All<'a>:'static{}impl<'a>All<'a> for u8{}fn f<'b>(_:&'b u8)->impl for<'a>All<'a>{7u8}fn take<T:'static>(_:T){}fn main(){let x;{let a=1;x=f(&a);}take(x);}",
    ),
    (
        "quantified-captured-bound",
        "trait All<'a,'b>:'b{}impl<'a,'b>All<'a,'b> for &'b u8{}fn f<'b,'c>(x:&'b u8,_:&'c u8)->impl for<'a>All<'a,'b>{x}fn take<T>(_:T){}fn main(){let a=1;let x;{let b=2;x=f(&a,&b);}take(x);}",
    ),
    (
        "inherited-equality",
        "trait Get{type Item;fn get(&self)->Self::Item;}trait Derived:Get<Item=u8>{}impl Get for u8{type Item=u8;fn get(&self)->u8{*self}}impl Derived for u8{}fn f()->impl Derived{29u8}fn main(){check(f().get()==29);}",
    ),
    (
        "nested-callable",
        "trait Read<T>{fn value(&self)->u8;}impl<'a>Read<fn(&'a u8)> for u8{fn value(&self)->u8{*self}}fn f()->impl for<'a>Read<fn(&'a u8)>{31u8}fn take<T:for<'a>Read<fn(&'a u8)>>(x:T)->u8{x.value()}fn main(){check(take(f())==31);}",
    ),
    (
        "universal-call",
        "trait Read<'a>{fn read(&self,x:&'a u8)->u8;}impl<'a>Read<'a> for u8{fn read(&self,x:&'a u8)->u8{*self+*x}}fn f()->impl for<'a>Read<'a>{7u8}fn main(){let a=3;check(f().read(&a)==10);}",
    ),
    (
        "universal-forward",
        "trait Read<'a>{fn read(&self,x:&'a u8)->u8;}impl<'a>Read<'a> for u8{fn read(&self,x:&'a u8)->u8{*self+*x}}fn f()->impl for<'a>Read<'a>{7u8}fn g()->impl for<'b>Read<'b>{f()}fn take<T:for<'c>Read<'c>>(x:T)->u8{let a=5;x.read(&a)}fn main(){check(take(g())==12);}",
    ),
    (
        "captured-hidden",
        "trait Read<'a>{fn read(&self,x:&'a u8)->u8;}struct S<'b>(&'b u8);impl<'a,'b>Read<'a> for S<'b>{fn read(&self,x:&'a u8)->u8{*self.0+*x}}fn f<'b>(x:&'b u8)->impl for<'a>Read<'a>+use<'b>{S(x)}fn main(){let a=7;let b=11;check(f(&a).read(&b)==18);}",
    ),
    (
        "independent-slots",
        "trait Pair<'a,'b>{fn read(&self,x:&'a u8,y:&'b u8)->u8;}impl<'a,'b>Pair<'a,'b> for u8{fn read(&self,x:&'a u8,y:&'b u8)->u8{*self+*x+*y}}fn f()->impl for<'a,'b>Pair<'a,'b>{7u8}fn main(){let a=2;let b=3;check(f().read(&a,&b)==12);}",
    ),
    (
        "inherited-universal",
        "trait Read<'a>{fn read(&self,x:&'a u8)->u8;}trait All:for<'a>Read<'a>{}impl<'a>Read<'a> for u8{fn read(&self,x:&'a u8)->u8{*self+*x}}impl All for u8{}fn f()->impl All{7u8}fn take<T:for<'b>Read<'b>>(x:T)->u8{let a=5;x.read(&a)}fn main(){check(take(f())==12);}",
    ),
    (
        "associated-value",
        "trait Get{type Item;fn get(&self)->Self::Item;}impl Get for u8{type Item=u8;fn get(&self)->u8{*self}}fn f()->impl Get<Item=u8>{17u8}fn main(){check(f().get()==17);}",
    ),
    (
        "associated-binder",
        "trait Get<'a>{type Item;fn get(&self,x:&'a u8)->Self::Item;}impl<'a>Get<'a> for u8{type Item=&'a u8;fn get(&self,x:&'a u8)->&'a u8{x}}fn f()->impl for<'a>Get<'a,Item=&'a u8>{7u8}fn main(){let a=23;check(*f().get(&a)==23);}",
    ),
    (
        "empty-binder",
        "fn f()->impl for<>Copy{7u8}fn main(){let x=f();let _=(x,x);}",
    ),
    (
        "unused-binder",
        "fn f()->impl for<'a>Copy{7u8}fn main(){let x=f();let _=(x,x);}",
    ),
];
const REJECTED: &[(&str, &str)] = &[
    // Rust 1.98 retains the captured input here despite the quantified superbound.
    (
        "quantified-every-lifetime",
        "trait All<'a>:'a{}impl<'a>All<'a> for u8{}fn f<'b>(_:&'b u8)->impl for<'a>All<'a>{7u8}fn take<T:'static>(_:T){}fn main(){let x;{let a=1;x=f(&a);}take(x);}",
    ),
    (
        "false-every-lifetime",
        "trait All<'a>:'a{}impl<'a,'b:'a>All<'a> for &'b u8{}fn f<'b>(x:&'b u8)->impl for<'a>All<'a>{x}fn main(){}",
    ),
    (
        "false-quantified-equality",
        "trait Get<'a>{type Item;}impl<'a>Get<'a> for u8{type Item=&'a u8;}fn f()->impl for<'a>Get<'a,Item=&'static u8>{7u8}fn main(){}",
    ),
    (
        "static-is-not-universal",
        "trait Read<'a>{}impl Read<'static> for u8{}fn f()->impl for<'a>Read<'a>{7u8}fn main(){}",
    ),
    (
        "correlated-is-not-independent",
        "trait Pair<'a,'b>{}impl<'a>Pair<'a,'a> for u8{}fn f()->impl for<'a,'b>Pair<'a,'b>{7u8}fn main(){}",
    ),
    (
        "wrong-equality",
        "trait Get{type Item;}impl Get for u8{type Item=u16;}fn f()->impl Get<Item=u8>{7u8}fn main(){}",
    ),
    (
        "captured-static-only",
        "trait Read<'a>{}struct S<'a>(&'a u8);impl Read<'static> for S<'static>{}fn f<'b>(x:&'b u8)->impl for<'a>Read<'a>{S(x)}fn main(){}",
    ),
    (
        "excluded-capture",
        "trait Read<'a>{}struct S<'b>(&'b u8);impl<'a,'b>Read<'a> for S<'b>{}fn f<'b>(x:&'b u8)->impl for<'a>Read<'a>+use<>{S(x)}fn main(){}",
    ),
];

#[test]
fn published_quantifiers_match_source_outcomes() -> Result<(), Box<dyn std::error::Error>> {
    let mut failures = Vec::new();
    for (cases, accepted) in [(ACCEPTED, true), (REJECTED, false)] {
        for &(name, body) in cases {
            let source = format!("{}{body}", native::CHECK);
            rustc::qualify(&format!("polymorphic-outputs-{name}"), &source, accepted)?;
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
fn published_quantifiers_execute_like_rustc() -> Result<(), Box<dyn std::error::Error>> {
    let cases: Vec<_> = ACCEPTED
        .iter()
        .map(|&(name, body)| (name, body, false))
        .collect();
    native::check_cases("opaque-polymorphic-outputs/native", &cases)
}
