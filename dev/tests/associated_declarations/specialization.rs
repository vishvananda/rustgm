//! Specialized trait premises stay with proved callers and preserve independent lifetime scopes.
use super::{native, object_outcome, semantic};

const CASES: &[(&str, &str, bool)] = &[
    (
        "distinct",
        "trait A<'a>{type Out;}impl<'a,'b:'a>A<'a>for &'b u8{type Out=u16;}fn value<'a,T:A<'a>>(x:T::Out)->T::Out{x}fn call<'a,'b:'a>(_:&'a u8,_:&'b u8)->u16{value::<'a,&'b u8>(7)}fn main(){let x=7u8;check(call(&x,&x)==7);}",
        true,
    ),
    (
        "same",
        "trait A<'a>{type Out;}impl<'a,'b:'a>A<'a>for &'b u8{type Out=u16;}fn value<'a,T:A<'a>>(x:T::Out)->T::Out{x}fn call<'a>(_:&'a u8)->u16{value::<'a,&'a u8>(7)}fn main(){let x=7u8;check(call(&x)==7);}",
        true,
    ),
    (
        "explicit-type-bound",
        "trait A<'a>{type Out;}impl<'a,'b:'a>A<'a>for &'b u8{type Out=u16;}fn value<'a,T:A<'a>+'a>(x:T::Out)->T::Out{x}fn call<'a,'b:'a>(_:&'a u8,_:&'b u8)->u16{value::<'a,&'b u8>(7)}fn main(){let x=7u8;check(call(&x,&x)==7);}",
        true,
    ),
    (
        "result-outlives",
        "trait A<'a>{type Out:'a;}impl<'a,'b:'a>A<'a>for &'b u8{type Out=u16;}fn value<'a,T:A<'a>>(x:T::Out)->T::Out{x}fn call<'a,'b:'a>(_:&'a u8,_:&'b u8)->u16{value::<'a,&'b u8>(7)}fn main(){let x=7u8;check(call(&x,&x)==7);}",
        true,
    ),
    (
        "result-copy",
        "trait A<'a>{type Out:Copy;}impl<'a,'b:'a>A<'a>for &'b u8{type Out=u16;}fn value<'a,T:A<'a>>(x:T::Out)->T::Out{x}fn call<'a,'b:'a>(_:&'a u8,_:&'b u8)->u16{value::<'a,&'b u8>(7)}fn main(){let x=7u8;check(call(&x,&x)==7);}",
        true,
    ),
    (
        "static-type",
        "trait A<'a>{type Out;}impl<'a,'b:'a>A<'a>for &'b u8{type Out=u16;}fn value<'a,T:A<'a>>(x:T::Out)->T::Out{x}fn call<'a>(_:&'a u8)->u16{value::<'a,&'static u8>(7)}fn main(){let x=7u8;check(call(&x)==7);}",
        true,
    ),
    (
        "static-application",
        "trait A<'a>{type Out;}impl<'a,'b:'a>A<'a>for &'b u8{type Out=u16;}fn value<'a,T:A<'a>>(x:T::Out)->T::Out{x}fn main(){check(value::<'static,&'static u8>(7)==7);}",
        true,
    ),
    (
        "missing-outlives",
        "trait A<'a>{type Out;}impl<'a,'b:'a>A<'a>for &'b u8{type Out=u16;}fn value<'a,T:A<'a>>(x:T::Out)->T::Out{x}fn call<'a,'b>(_:&'a u8,_:&'b u8)->u16{value::<'a,&'b u8>(7)}fn main(){let x=7u8;check(call(&x,&x)==7);}",
        false,
    ),
    (
        "reversed-outlives",
        "trait A<'a>{type Out;}impl<'a,'b:'a>A<'a>for &'b u8{type Out=u16;}fn value<'a,T:A<'a>>(x:T::Out)->T::Out{x}fn call<'a:'b,'b>(_:&'a u8,_:&'b u8)->u16{value::<'a,&'b u8>(7)}fn main(){let x=7u8;check(call(&x,&x)==7);}",
        false,
    ),
    (
        "override",
        "trait A<'a>{type Out;}impl<'a,'b:'a>A<'a>for &'b u8{type Out=u16;}trait Use<'a,T:A<'a>>{fn take(x:T::Out)->T::Out;}impl<'a,T:A<'a>>Use<'a,T>for u32{fn take(x:T::Out)->T::Out{x}}fn call<'a,'b:'a>(_:&'a u8,_:&'b u8)->u16{<u32 as Use<'a,&'b u8>>::take(7)}fn main(){let x=7u8;check(call(&x,&x)==7);}",
        true,
    ),
    (
        "override-copy",
        "trait A<'a>{type Out:Copy;}impl<'a,'b:'a>A<'a>for &'b u8{type Out=u16;}trait Use<'a,T:A<'a>>{fn take(x:T::Out)->T::Out;}impl<'a,T:A<'a>>Use<'a,T>for u32{fn take(x:T::Out)->T::Out{x}}fn call<'a,'b:'a>(_:&'a u8,_:&'b u8)->u16{<u32 as Use<'a,&'b u8>>::take(7)}fn main(){let x=7u8;check(call(&x,&x)==7);}",
        true,
    ),
    (
        "default",
        "trait A<'a>{type Out;}impl<'a,'b:'a>A<'a>for &'b u8{type Out=u16;}trait Use<'a,T:A<'a>>{fn take(x:T::Out)->T::Out{x}}impl<'a,T:A<'a>>Use<'a,T>for u32{}fn call<'a,'b:'a>(_:&'a u8,_:&'b u8)->u16{<u32 as Use<'a,&'b u8>>::take(7)}fn main(){let x=7u8;check(call(&x,&x)==7);}",
        true,
    ),
    (
        "inherent",
        "trait A<'a>{type Out;}impl<'a,'b:'a>A<'a>for &'b u8{type Out=u16;}struct Use;impl Use{fn take<'a,T:A<'a>>(x:T::Out)->T::Out{x}}fn call<'a,'b:'a>(_:&'a u8,_:&'b u8)->u16{Use::take::<'a,&'b u8>(7)}fn main(){let x=7u8;check(call(&x,&x)==7);}",
        true,
    ),
    (
        "inherited",
        "trait A<'a>{type Out;}impl<'a,'b:'a>A<'a>for &'b u8{type Out=u16;}trait B<'a>:A<'a>{}impl<'a,'b:'a>B<'a>for &'b u8{}fn value<'a,T:B<'a>>(x:T::Out)->T::Out{x}fn call<'a,'b:'a>(_:&'a u8,_:&'b u8)->u16{value::<'a,&'b u8>(7)}fn main(){let x=7u8;check(call(&x,&x)==7);}",
        true,
    ),
    (
        "alias",
        "trait A<'a>{type Out;}impl<'a,'b:'a>A<'a>for &'b u8{type Out=u16;}type Out<'a,T>=<T as A<'a>>::Out;fn value<'a,T:A<'a>>(x:Out<'a,T>)->Out<'a,T>{x}fn call<'a,'b:'a>(_:&'a u8,_:&'b u8)->u16{value::<'a,&'b u8>(7)}fn main(){let x=7u8;check(call(&x,&x)==7);}",
        true,
    ),
    (
        "reified",
        "trait A<'a>{type Out;}impl<'a,'b:'a>A<'a>for &'b u8{type Out=u16;}fn value<'a,T:A<'a>>(x:T::Out)->T::Out{x}fn call<'a,'b:'a>(_:&'a u8,_:&'b u8)->u16{let f:fn(u16)->u16=value::<'a,&'b u8>;f(7)}fn main(){let x=7u8;check(call(&x,&x)==7);}",
        true,
    ),
    (
        "tuple-result",
        "trait A<'a>{type Out;}impl<'a,'b:'a>A<'a>for &'b u8{type Out=(u16,);}fn value<'a,T:A<'a>>(x:T::Out)->T::Out{x}fn call<'a,'b:'a>(_:&'a u8,_:&'b u8)->u16{value::<'a,&'b u8>((7,)).0}fn main(){let x=7u8;check(call(&x,&x)==7);}",
        true,
    ),
    (
        "array-result",
        "trait A<'a>{type Out;}impl<'a,'b:'a>A<'a>for &'b u8{type Out=[u16;2];}fn value<'a,T:A<'a>>(x:T::Out)->T::Out{x}fn call<'a,'b:'a>(_:&'a u8,_:&'b u8)->u16{value::<'a,&'b u8>([7,9])[0]}fn main(){let x=7u8;check(call(&x,&x)==7);}",
        true,
    ),
    (
        "late-before-early",
        "trait A<'a>{type Out;}impl<'a,'b:'a>A<'a>for &'b u8{type Out=u16;}fn value<'late,'early,T:A<'early>>(x:T::Out,_:&'late u8)->T::Out{x}fn call<'a,'b:'a>(_:&'a u8,_:&'b u8)->u16{value::<&'b u8>(7,&0)}fn main(){let x=7u8;check(call(&x,&x)==7);}",
        true,
    ),
    (
        "nested-late-binder",
        "trait A<'a>{type Out;}impl<'a,'b:'a>A<'a>for &'b u8{type Out=u16;}fn value<'a,T:A<'a>>(x:T::Out,f:fn(&u8)->&u8)->T::Out{let y=7u8;check(*f(&y)==7);x}fn id(x:&u8)->&u8{x}fn call<'a,'b:'a>(_:&'a u8,_:&'b u8)->u16{value::<'a,&'b u8>(7,id)}fn main(){let x=7u8;check(call(&x,&x)==7);}",
        true,
    ),
    (
        "two-callers",
        "trait A<'a>{type Out;}impl<'a,'b:'a>A<'a>for &'b u8{type Out=u16;}fn value<'a,T:A<'a>>(x:T::Out)->T::Out{x}fn call<'a,'b:'a>(_:&'a u8,_:&'b u8)->u16{value::<'a,&'b u8>(7)}fn other<'c,'d:'c>(_:&'c u8,_:&'d u8)->u16{value::<'c,&'d u8>(9)}fn main(){let x=7u8;check(call(&x,&x)==7);check(other(&x,&x)==9);}",
        true,
    ),
    (
        "wrong-caller-after-valid",
        "trait A<'a>{type Out;}impl<'a,'b:'a>A<'a>for &'b u8{type Out=u16;}fn value<'a,T:A<'a>>(x:T::Out)->T::Out{x}fn call<'a,'b:'a>(_:&'a u8,_:&'b u8)->u16{value::<'a,&'b u8>(7)}fn main(){let x=7u8;check(call(&x,&x)==7);}fn wrong<'c,'d>(_:&'c u8,_:&'d u8)->u16{value::<'c,&'d u8>(9)}",
        false,
    ),
    (
        "wrong-trait-argument",
        "trait A<'a,U>{type Out;}impl<'a,'b:'a>A<'a,u8>for &'b u8{type Out=u16;}fn value<'a,T:A<'a,u16>>(x:T::Out)->T::Out{x}fn call<'a,'b:'a>(_:&'a u8,_:&'b u8)->u16{value::<'a,&'b u8>(7)}fn main(){let x=7u8;check(call(&x,&x)==7);}",
        false,
    ),
    (
        "missing-parent",
        "trait A<'a>{type Out;}impl<'a,'b:'a>A<'a>for &'b u8{type Out=u16;}fn value<'a,T>(x:T::Out)->T::Out{x}fn call<'a,'b:'a>(_:&'a u8,_:&'b u8)->u16{value::<'a,&'b u8>(7)}fn main(){let x=7u8;check(call(&x,&x)==7);}",
        false,
    ),
    (
        "result-escape",
        "trait A<'a>{type Out;}impl<'a>A<'a>for u8{type Out=&'a u8;}fn value<'a,T:A<'a>>(x:T::Out)->T::Out{x}fn wrong<'a>(x:&'a u8)->&'static u8{value::<'a,u8>(x)}fn main(){}",
        false,
    ),
    (
        "input-not-result",
        "trait A<'a>{type Out:'static;}impl<'a,'b:'a>A<'a>for &'b u8{type Out=u16;}fn value<'a,T:A<'a>>(x:T::Out)->T::Out{x}fn wrong<'a,T:A<'a>>(x:&'a T)->&'static T{x}fn main(){}",
        false,
    ),
    (
        "nested-local-escape",
        "trait A<'a>{type Out;}impl<'a,'b:'a>A<'a>for &'b u8{type Out=u16;}fn value<'a,T:A<'a>>(x:T::Out,f:fn(&u8)->&u8)->T::Out{let y=7u8;let _: &'static u8=f(&y);x}fn call<'a,'b:'a>(_:&'a u8,_:&'b u8)->u16{value::<'a,&'b u8>(7,id)}fn main(){let x=7u8;check(call(&x,&x)==7);}fn id(x:&u8)->&u8{x}",
        false,
    ),
    (
        "equality-same",
        "trait A<'a>{type Out;}impl<'a>A<'a>for &'a u8{type Out=u16;}fn value<'a,T:A<'a>>(x:T::Out)->T::Out{x}fn call<'a>(_:&'a u8)->u16{value::<'a,&'a u8>(7)}fn main(){let x=7u8;check(call(&x)==7);}",
        true,
    ),
    (
        "equality-distinct",
        "trait A<'a>{type Out;}impl<'a>A<'a>for &'a u8{type Out=u16;}fn value<'a,T:A<'a>>(x:T::Out)->T::Out{x}fn call<'a,'b:'a>(_:&'a u8,_:&'b u8)->u16{value::<'a,&'b u8>(7)}fn main(){let x=7u8;check(call(&x,&x)==7);}",
        false,
    ),
    (
        "private-witness",
        "trait A<'a>{type Out;}impl<'a,'b:'a,'c:'a>A<'a>for &'b u8{type Out=u16;}fn value<'a,T:A<'a>>(x:T::Out)->T::Out{x}fn call<'a,'b:'a>(_:&'a u8,_:&'b u8)->u16{value::<'a,&'b u8>(7)}fn main(){let x=7u8;check(call(&x,&x)==7);}",
        true,
    ),
];

#[test]
fn specialized_trait_bounds_preserve_lifetime_conditions() -> Result<(), Box<dyn std::error::Error>>
{
    let mut failures = Vec::new();
    for &(name, source, accepted) in CASES {
        let source = format!("{}{source}", native::CHECK);
        object_outcome(&format!("specialization-{name}"), &source, accepted)?;
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
fn specialized_trait_bounds_preserve_native_values() -> Result<(), Box<dyn std::error::Error>> {
    let cases = CASES
        .iter()
        .filter(|(_, _, accepted)| *accepted)
        .map(|&(name, source, _)| (name, source, false))
        .collect::<Vec<_>>();
    native::check_cases("specialized-trait-lifetimes", &cases)
}
