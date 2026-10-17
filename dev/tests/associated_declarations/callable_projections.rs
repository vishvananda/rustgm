//! Opened callable binders and projected arguments share caller universal identities.
use super::{native, object_outcome, semantic};

const CASES: &[(&str, &str, bool)] = &[
    (
        "plain-projection",
        "trait A{type Out;fn apply(f:fn(Self::Out)->Self::Out,x:Self::Out)->Self::Out{f(x)}}impl A for u8{type Out=u16;}fn keep(x:u16)->u16{x}fn main(){check(<u8 as A>::apply(keep,7)==7);}",
        true,
    ),
    (
        "free-function",
        "trait A<'a>{type Out;}impl<'a>A<'a>for u8{type Out=&'a u8;}fn apply<'a,T:A<'a>>(f:fn(T::Out)->T::Out,x:T::Out)->T::Out{f(x)}fn keep<'a>(x:&'a u8)->&'a u8{x}fn main(){let x=7u8;check(*apply::<u8>(keep,&x)==7);}",
        true,
    ),
    (
        "ordinary-capture",
        "fn apply<'a>(f:fn(&'a u8)->&'a u8,x:&'a u8)->&'a u8{f(x)}fn keep<'a>(x:&'a u8)->&'a u8{x}fn main(){let x=7u8;check(*apply(keep,&x)==7);}",
        true,
    ),
    (
        "self-capture",
        "trait A{type Out;fn apply(f:fn(Self::Out)->Self::Out,x:Self::Out)->Self::Out{f(x)}}impl<'a>A for &'a u8{type Out=&'a u8;}fn keep<'a>(x:&'a u8)->&'a u8{x}fn main(){let x=7u8;check(*<&u8 as A>::apply(keep,&x)==7);}",
        true,
    ),
    (
        "generic-call-trigger",
        "fn marker<T>(){}trait A<'a>{type Out;}impl<'a>A<'a>for u8{type Out=&'a u8;}fn apply<'a,T:A<'a>>(f:fn(T::Out)->T::Out,x:T::Out)->T::Out{marker::<u8>();f(x)}fn keep<'a>(x:&'a u8)->&'a u8{x}fn main(){let x=7u8;check(*apply::<u8>(keep,&x)==7);}",
        true,
    ),
    (
        "generic-call-after",
        "fn marker<T>(){}trait A<'a>{type Out;}impl<'a>A<'a>for u8{type Out=&'a u8;}fn apply<'a,T:A<'a>>(f:fn(T::Out)->T::Out,x:T::Out)->T::Out{let y=f(x);marker::<u8>();y}fn keep<'a>(x:&'a u8)->&'a u8{x}fn main(){let x=7u8;check(*apply::<u8>(keep,&x)==7);}",
        true,
    ),
    (
        "concrete-independent-lifetimes",
        "trait A<'a>{type Out;}impl<'a>A<'a>for u8{type Out=&'a u8;}fn apply<'a,'b>(f:fn(<u8 as A<'a>>::Out)-><u8 as A<'a>>::Out,x:<u8 as A<'b>>::Out)-><u8 as A<'a>>::Out{f(x)}fn main(){}",
        false,
    ),
    (
        "concrete-outlives",
        "trait A<'a>{type Out;}impl<'a>A<'a>for u8{type Out=&'a u8;}fn apply<'a,'b:'a>(f:fn(<u8 as A<'a>>::Out)-><u8 as A<'a>>::Out,x:<u8 as A<'b>>::Out)-><u8 as A<'a>>::Out{f(x)}fn keep<'a>(x:&'a u8)->&'a u8{x}fn main(){let x=7u8;check(*apply(keep,&x)==7);}",
        true,
    ),
    (
        "nested-callable",
        "trait A<'a>{type Out;fn apply(f:fn(Self::Out)->Self::Out,x:Self::Out)->Self::Out{f(x)}}impl<'a>A<'a> for u8{type Out=&'a u8;}fn keep<'a>(x:&'a u8)->&'a u8{x}fn main(){let x=7u8;check(*<u8 as A<'_>>::apply(keep,&x)==7);}",
        true,
    ),
    (
        "repeated-call",
        "trait A<'a>{type Out;}impl<'a>A<'a>for u8{type Out=&'a u8;}fn apply<'a,T:A<'a>>(f:fn(T::Out)->T::Out,x:T::Out)->T::Out{f(f(x))}fn keep<'a>(x:&'a u8)->&'a u8{x}fn main(){let x=7u8;check(*apply::<u8>(keep,&x)==7);}",
        true,
    ),
    (
        "borrowed-projection",
        "trait A<'a>{type Out;}impl<'a>A<'a>for u8{type Out=&'a u8;}fn apply<'a,T:A<'a>>(f:fn(&'a T::Out)->&'a T::Out,x:&'a T::Out)->&'a T::Out{f(x)}fn keep<'a>(x:&'a &'a u8)->&'a &'a u8{x}fn main(){let x=7u8;let r=&x;check(**apply::<u8>(keep,&r)==7);}",
        true,
    ),
    (
        "distinct-projected-subjects",
        "trait A<'a>{type Out;}fn apply<'a,T:A<'a>,U:A<'a>>(f:fn(T::Out)->T::Out,x:U::Out)->T::Out{f(x)}fn main(){}",
        false,
    ),
    (
        "equal-results-distinct-subjects",
        "trait A<'a>{type Out;}impl<'a>A<'a>for u8{type Out=&'a u8;}impl<'a>A<'a>for u16{type Out=&'a u8;}fn apply<'a,T:A<'a,Out=&'a u8>,U:A<'a,Out=&'a u8>>(f:fn(T::Out)->T::Out,x:U::Out)->T::Out{f(x)}fn keep<'a>(x:&'a u8)->&'a u8{x}fn main(){let x=7u8;check(*apply::<u8,u16>(keep,&x)==7);}",
        true,
    ),
];

#[test]
fn projected_callable_inputs_match_independent_outcomes() -> Result<(), Box<dyn std::error::Error>>
{
    let mut failures = Vec::new();
    for &(name, body, accepted) in CASES {
        let source = format!("{}{body}", native::CHECK);
        object_outcome(&format!("callable-projection-{name}"), &source, accepted)?;
        let result = semantic::analyze(&source).and_then(|(mut report, mut facts)| {
            let entry =
                rustgm::sema::entry::select(&mut report, &facts).map_err(|e| e.to_string())?;
            rustgm::lower::instances::build(&report, &mut facts, entry)
                .map(|_| ())
                .map_err(|e| e.to_string())
        });
        if result.is_ok() != accepted {
            failures.push(format!("{name}: {result:?}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn projected_callable_inputs_preserve_native_values() -> Result<(), Box<dyn std::error::Error>> {
    let cases = CASES
        .iter()
        .filter(|(_, _, accepted)| *accepted)
        .map(|&(name, source, _)| (name, source, false))
        .collect::<Vec<_>>();
    native::check_cases("callable-projections", &cases)
}
