//! Argument contexts use only the current call's proved input frontier.
use super::{analyze, native, rustc};
const CHECK: &str = "fn check(b:bool){if !b{let z=0;let _=1/z;}}";
const CASES: &[(&str, &str, bool)] = &[
    (
        "sole-dot",
        r"fn check(b:bool){if !b{let z=0;let _=1/z;}}trait P<R=Self>{fn f(&self,r:&R)->u8;}impl P for u8{fn f(&self,r:&u8)->u8{*self+*r}}fn g(a:&u8,b:&u8)->u8{a.f(&b)}fn main(){check(g(&1,&2)==3);}
",
        true,
    ),
    (
        "sole-qualified",
        r"fn check(b:bool){if !b{let z=0;let _=1/z;}}trait P<R=Self>{fn f(&self,r:&R)->u8;}impl P for u8{fn f(&self,r:&u8)->u8{*self+*r}}fn g(a:&u8,b:&u8)->u8{<u8 as P>::f(a,&b)}fn main(){check(g(&1,&2)==3);}
",
        true,
    ),
    (
        "sole-nondefault-dot",
        r"fn check(b:bool){if !b{let z=0;let _=1/z;}}trait P<R=Self>{fn f(&self,r:&R)->u8;}impl P<u16> for u8{fn f(&self,r:&u16)->u8{*self+*r as u8}}fn g(a:&u8,b:&u16)->u8{a.f(&b)}fn main(){check(g(&1,&2)==3);}
",
        true,
    ),
    (
        "competing-reference-dot",
        r"fn check(b:bool){if !b{let z=0;let _=1/z;}}trait P<R=Self>{fn f(&self,r:&R)->u8;}impl P for u8{fn f(&self,r:&u8)->u8{*self+*r}}impl P<&u8> for u8{fn f(&self,r:&&u8)->u8{*self+**r+1}}fn g(a:&u8,b:&u8)->u8{a.f(&b)}fn main(){check(g(&1,&2)==4);}
",
        true,
    ),
    (
        "competing-reference-reversed",
        r"fn check(b:bool){if !b{let z=0;let _=1/z;}}trait P<R=Self>{fn f(&self,r:&R)->u8;}impl P<&u8> for u8{fn f(&self,r:&&u8)->u8{*self+**r+1}}impl P for u8{fn f(&self,r:&u8)->u8{*self+*r}}fn g(a:&u8,b:&u8)->u8{a.f(&b)}fn main(){check(g(&1,&2)==4);}
",
        true,
    ),
    (
        "generic-impl-exact-reference",
        r"fn check(b:bool){if !b{let z=0;let _=1/z;}}trait P<R=Self>{fn f(&self,r:&R)->u8;}trait Marker{}impl Marker for &u8{}impl<T:Marker>P<T> for u8{fn f(&self,_:&T)->u8{9}}fn g(a:&u8,b:&u8)->u8{a.f(&b)}fn main(){check(g(&1,&2)==9);}
",
        true,
    ),
    (
        "free-bound-does-not-preselect",
        r"fn check(b:bool){if !b{let z=0;let _=1/z;}}trait P<R=Self>{fn f(&self,r:&R)->u8;}impl P for u8{fn f(&self,r:&u8)->u8{*self+*r}}fn take<R>(a:u8,r:&R)->u8 where u8:P<R>{a.f(r)}fn g(b:&u8)->u8{take(1,&b)}fn main(){let _=g(&2);}
",
        true,
    ),
    (
        "later-source-context",
        r"fn check(b:bool){if !b{let z=0;let _=1/z;}}trait P<R=Self>{fn f(&self,r:&R)->u8;}impl P for u8{fn f(&self,r:&u8)->u8{*self+*r}}fn take<T>(_:T){}fn g(a:&u8,b:&u8)->u8{a.f(&{take(1u8);b})}fn main(){check(g(&1,&2)==3);}
",
        false,
    ),
    (
        "shared-reference-deref",
        r"fn check(b:bool){if !b{let z=0;let _=1/z;}}trait P<R=Self>{fn f(&self,r:&R)->u8;}impl P for u8{fn f(&self,r:&u8)->u8{*self+*r}}fn g(a:&u8,b:&mut u8)->u8{a.f(&b)}fn main(){let mut b=2;check(g(&1,&mut b)==3);}
",
        true,
    ),
    (
        "suffixed-mismatch",
        r"fn check(b:bool){if !b{let z=0;let _=1/z;}}trait P<R=Self>{fn f(&self,r:&R)->u8;}impl P for u8{fn f(&self,r:&u8)->u8{*self+*r}}fn g(a:&u8,b:&u16)->u8{a.f(b)}fn main(){let _=g(&1,&2);}
",
        false,
    ),
    (
        "mutable-reborrow-depth",
        r"fn check(b:bool){if !b{let z=0;let _=1/z;}}trait Q<R=Self>{fn f(&self,r:&mut R)->u8;}impl Q for u8{fn f(&self,r:&mut u8)->u8{*r+=*self;*r}}fn g(a:&u8,r:&mut &mut u8)->u8{let x=a.f(r);**r+=1;x}fn main(){let mut b=2;let mut r=&mut b;check(g(&1,&mut r)==3);check(b==4);}
",
        true,
    ),
    (
        "shared-to-mutable-denied",
        r"fn check(b:bool){if !b{let z=0;let _=1/z;}}trait Q<R=Self>{fn f(&self,r:&mut R)->u8;}impl Q for u8{fn f(&self,r:&mut u8)->u8{*r+=*self;*r}}fn g(a:&u8,r:&u8)->u8{a.f(r)}fn main(){let _=g(&1,&2);}
",
        false,
    ),
    (
        "unpaid-lifetime",
        r"fn check(b:bool){if !b{let z=0;let _=1/z;}}trait P<R=Self>{fn f(&self,r:&R)->u8;}struct W<'a>(&'a u8);impl<'a>P<W<'a>> for u8 where 'a:'static{fn f(&self,r:&W<'a>)->u8{*self+*r.0}}fn g<'a>(a:&u8,b:&W<'a>)->u8{a.f(&b)}fn main(){let b=2;let _=g(&1,&W(&b));}
",
        false,
    ),
    (
        "invariant-unpaid",
        r"fn check(b:bool){if !b{let z=0;let _=1/z;}}trait P<R=Self>{fn f(&self,r:&R)->u8;}impl<'a>P<&'a mut &'a u8> for u8{fn f(&self,r:&&'a mut &'a u8)->u8{*self+***r}}fn g<'a,'b>(a:&u8,b:&'b mut &'a u8)->u8{a.f(&b)}fn main(){}
",
        false,
    ),
    (
        "invariant-paid",
        r"fn check(b:bool){if !b{let z=0;let _=1/z;}}trait P<R=Self>{fn f(&self,r:&R)->u8;}impl<'a>P<&'a mut &'a u8> for u8{fn f(&self,r:&&'a mut &'a u8)->u8{*self+***r}}fn g<'a,'b>(a:&u8,b:&'b mut &'a u8)->u8 where 'b:'a{a.f(&b)}fn main(){let x=2;let mut y=&x;check(g(&1,&mut y)==3);}
",
        true,
    ),
    (
        "ambiguous-rhs",
        r"fn check(b:bool){if !b{let z=0;let _=1/z;}}trait P<R=Self>{fn f(&self,r:&R)->u8;}impl P for u8{fn f(&self,r:&u8)->u8{*r}}impl P<u16> for u8{fn f(&self,r:&u16)->u8{*r as u8}}fn make<T>()->T{loop{}}fn g(a:&u8)->u8{a.f(&make())}fn main(){}
",
        false,
    ),
    (
        "independent-applications",
        r"fn check(b:bool){if !b{let z=0;let _=1/z;}}trait P<R=Self>{fn f(&self,r:&R)->u8;}impl P<u16> for u8{fn f(&self,r:&u16)->u8{*self+*r as u8}}impl P<u8> for u16{fn f(&self,r:&u8)->u8{*self as u8+*r}}fn g(a:&u8,b:&u16)->u8{let x=a.f(&b);let y=b.f(&a);x+y}fn main(){check(g(&1,&2)==6);}
",
        true,
    ),
    (
        "known-reference-without-bound",
        r"fn check(b:bool){if !b{let z=0;let _=1/z;}}fn keep<T>(a:&T)->&T{a}fn main(){let x=2u8;let r=&x;let rr=keep(&r);check(**rr==2);}
",
        true,
    ),
    (
        "nested-binder-positive",
        r"fn check(b:bool){if !b{let z=0;let _=1/z;}}trait P<R=Self>{fn f(&self,r:&R)->u8;}type F=for<'a>fn(&'a u8)->&'a u8;impl P<F> for u8{fn f(&self,r:&F)->u8{let x=*self;*r(&x)}}fn g(a:&u8,r:&F)->u8{a.f(&r)}fn id(x:&u8)->&u8{x}fn main(){let f:F=id;check(g(&7,&f)==7);}
",
        true,
    ),
    (
        "nested-binder-negative",
        r"fn check(b:bool){if !b{let z=0;let _=1/z;}}trait P<R=Self>{fn f(&self,r:&R)->u8;}type F=for<'a>fn(&'a u8)->&'a u8;impl P<F> for u8{fn f(&self,_:&F)->u8{*self}}fn g(a:&u8,r:&fn(&'static u8)->&'static u8)->u8{a.f(&r)}fn main(){}
",
        false,
    ),
    (
        "canonical-nominal-eq",
        r"fn check(b:bool){if !b{let z=0;let _=1/z;}}struct W(u8);impl core::cmp::PartialEq<W>for u8{fn eq(&self,r:&W)->bool{*self==r.0}}fn main(){check(1u8.eq(&W(1)));}
",
        true,
    ),
    (
        "canonical-nominal-order",
        r"fn check(b:bool){if !b{let z=0;let _=1/z;}}struct W(u8);impl core::cmp::PartialEq<W>for u8{fn eq(&self,r:&W)->bool{*self==r.0}}impl core::cmp::PartialOrd<W>for u8{fn partial_cmp(&self,r:&W)->Option<core::cmp::Ordering>{if *self<r.0{Some(core::cmp::Ordering::Less)}else{Some(core::cmp::Ordering::Equal)}}}fn main(){check(match 1u8.partial_cmp(&W(2)){Some(core::cmp::Ordering::Less)=>true,_=>false});}
",
        true,
    ),
    (
        "canonical-nominal-order-reversed",
        r"fn check(b:bool){if !b{let z=0;let _=1/z;}}struct W(u8);impl core::cmp::PartialOrd<W>for u8{fn partial_cmp(&self,r:&W)->Option<core::cmp::Ordering>{if *self<r.0{Some(core::cmp::Ordering::Less)}else{Some(core::cmp::Ordering::Equal)}}}impl core::cmp::PartialEq<W>for u8{fn eq(&self,r:&W)->bool{*self==r.0}}fn main(){check(match 1u8.partial_cmp(&W(2)){Some(core::cmp::Ordering::Less)=>true,_=>false});}
",
        true,
    ),
    (
        "canonical-nominal-wrong-rhs",
        r"fn check(b:bool){if !b{let z=0;let _=1/z;}}struct W(u8);impl core::cmp::PartialEq<W>for u8{fn eq(&self,r:&W)->bool{*self==r.0}}impl core::cmp::PartialOrd<W>for u8{fn partial_cmp(&self,r:&W)->Option<core::cmp::Ordering>{if *self<r.0{Some(core::cmp::Ordering::Less)}else{Some(core::cmp::Ordering::Equal)}}}fn main(){let _=1u8.partial_cmp(&2u16);}
",
        false,
    ),
];
#[test]
fn reference_arguments_retain_prefix_inference_and_coercion_boundaries()
-> Result<(), Box<dyn std::error::Error>> {
    let mut failures = Vec::new();
    for &(name, source, accepted) in CASES {
        rustc::qualify(&format!("reference-input-{name}"), source, accepted)?;
        let result = analyze(source);
        if result.is_ok() != accepted {
            failures.push(format!("{name}: {result:?}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn reference_inputs_execute_selected_rhs_and_preserve_reborrows()
-> Result<(), Box<dyn std::error::Error>> {
    let cases = CASES
        .iter()
        .filter(|case| case.2)
        .map(|&(name, source, _)| {
            Ok((
                name,
                source.strip_prefix(CHECK).ok_or("check prefix")?,
                false,
            ))
        })
        .collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()?;
    native::check_cases("parallel/method-reference-inputs", &cases)
}
