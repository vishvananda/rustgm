//! Selected override signatures normalize before concrete body rewrite and layout.
use super::{native, object_outcome, semantic};

const CASES: &[(&str, &str, bool)] = &[
    (
        "member-nested-method",
        "trait A{type Out:A+Copy;}impl A for u8{type Out=u8;}trait Use<T:A>{fn take(x:<<T as A>::Out as A>::Out)->(<<T as A>::Out as A>::Out,<<T as A>::Out as A>::Out);}impl<T:A>Use<T>for u16{fn take(x:<<T as A>::Out as A>::Out)->(<<T as A>::Out as A>::Out,<<T as A>::Out as A>::Out){(x,x)}}fn main(){let p=<u16 as Use<u8>>::take(7);check(p.0==7&&p.1==7);}",
        true,
    ),
    (
        "member-nested-method-optional",
        "trait A{type Out:?Sized+A+Copy;}impl A for u8{type Out=u8;}trait Use<T:A>{fn take(x:<<T as A>::Out as A>::Out)->(<<T as A>::Out as A>::Out,<<T as A>::Out as A>::Out);}impl<T:A>Use<T>for u16{fn take(x:<<T as A>::Out as A>::Out)->(<<T as A>::Out as A>::Out,<<T as A>::Out as A>::Out){(x,x)}}fn main(){let p=<u16 as Use<u8>>::take(7);check(p.0==7&&p.1==7);}",
        true,
    ),
    (
        "header-nested-method",
        "trait A where Self::Out:A+Copy{type Out;}impl A for u8{type Out=u8;}trait Use<T:A>{fn take(x:<<T as A>::Out as A>::Out)->(<<T as A>::Out as A>::Out,<<T as A>::Out as A>::Out);}impl<T:A>Use<T>for u16{fn take(x:<<T as A>::Out as A>::Out)->(<<T as A>::Out as A>::Out,<<T as A>::Out as A>::Out){(x,x)}}fn main(){let p=<u16 as Use<u8>>::take(7);check(p.0==7&&p.1==7);}",
        true,
    ),
    (
        "header-nested-method-optional",
        "trait A where Self::Out:A+Copy{type Out:?Sized;}impl A for u8{type Out=u8;}trait Use<T:A>{fn take(x:<<T as A>::Out as A>::Out)->(<<T as A>::Out as A>::Out,<<T as A>::Out as A>::Out);}impl<T:A>Use<T>for u16{fn take(x:<<T as A>::Out as A>::Out)->(<<T as A>::Out as A>::Out,<<T as A>::Out as A>::Out){(x,x)}}fn main(){let p=<u16 as Use<u8>>::take(7);check(p.0==7&&p.1==7);}",
        true,
    ),
    (
        "retained-projected-lifetime",
        "trait Pick{type Out;fn pick(&self)->Self::Out;}struct Wrap<'a>{value:&'a u8}impl<'a>Pick for Wrap<'a>{type Out=&'a u8;fn pick(&self)->Self::Out{self.value}}fn projected<T:Pick>(x:&T)->T::Out{x.pick()}fn main(){let n=7;let s=Wrap{value:&n};check(*projected(&s)==7);}",
        true,
    ),
    (
        "direct-result",
        "trait A{type Out;}impl A for u8{type Out=u16;}trait Use<T:A>{fn take(x:T::Out)->T::Out;}impl<T:A>Use<T>for u32{fn take(x:T::Out)->T::Out{x}}fn main(){check(<u32 as Use<u8>>::take(7)==7);}",
        true,
    ),
    (
        "function-pointer",
        "trait A{type Out;}impl A for u8{type Out=u16;}trait Use<T:A>{fn take(x:T::Out)->T::Out;}impl<T:A>Use<T>for u32{fn take(x:T::Out)->T::Out{x}}fn main(){let f:fn(u16)->u16=<u32 as Use<u8>>::take;check(f(7)==7);}",
        true,
    ),
    (
        "tuple",
        "trait A{type Out;}impl A for u8{type Out=u16;}trait Use<T:A>{fn take(x:(T::Out,))->(T::Out,);}impl<T:A>Use<T>for u32{fn take(x:(T::Out,))->(T::Out,){x}}fn main(){let p=<u32 as Use<u8>>::take((7,));check(p.0==7);}",
        true,
    ),
    (
        "array",
        "trait A{type Out;}impl A for u8{type Out=u16;}trait Use<T:A>{fn take(x:[T::Out;2])->[T::Out;2];}impl<T:A>Use<T>for u32{fn take(x:[T::Out;2])->[T::Out;2]{x}}fn main(){let p=<u32 as Use<u8>>::take([7,9]);check(p[0]==7&&p[1]==9);}",
        true,
    ),
    (
        "trait-lifetime",
        "trait A<'a>{type Out:Copy+'a;}impl<'a>A<'a>for u8{type Out=&'a u8;}trait Use<'a,T:A<'a>>{fn take(x:T::Out)->T::Out;}impl<'a,T:A<'a>>Use<'a,T>for u32{fn take(x:T::Out)->T::Out{x}}fn scope<'a>(x:&'a u8)->&'a u8{<u32 as Use<'a,u8>>::take(x)}fn main(){let x=7u8;check(*scope(&x)==7);}",
        true,
    ),
    (
        "method-parameters",
        "trait A{type Out;}impl A for u8{type Out=u16;}trait Use<T:A>{fn take<U:Copy>(x:T::Out,y:U)->(T::Out,U);}impl<T:A>Use<T>for u32{fn take<U:Copy>(x:T::Out,y:U)->(T::Out,U){(x,y)}}fn main(){let p=<u32 as Use<u8>>::take::<u64>(7,9);check(p.0==7&&p.1==9);}",
        true,
    ),
    (
        "nested-callable",
        "trait A{type Out;}impl A for u8{type Out=u16;}trait Use<T:A>{fn take(f:fn(T::Out)->T::Out,x:T::Out)->T::Out;}impl<T:A>Use<T>for u32{fn take(f:fn(T::Out)->T::Out,x:T::Out)->T::Out{f(x)}}fn id(x:u16)->u16{x}fn main(){check(<u32 as Use<u8>>::take(id,7)==7);}",
        true,
    ),
    (
        "alias-result",
        "trait A{type Out;}impl A for u8{type Out=u16;}type Output<T>=<T as A>::Out;trait Use<T:A>{fn take(x:Output<T>)->Output<T>;}impl<T:A>Use<T>for u32{fn take(x:Output<T>)->Output<T>{x}}fn main(){check(<u32 as Use<u8>>::take(7)==7);}",
        true,
    ),
    (
        "conditional-valid",
        "trait A{type Out;}impl A for u8{type Out=u16;}trait Use<T:A>{fn take(x:T::Out)->T::Out;}impl<T:A+Copy>Use<T>for u32{fn take(x:T::Out)->T::Out{x}}fn use_it<X:A+Copy>(x:X::Out)->X::Out{<u32 as Use<X>>::take(x)}fn main(){check(use_it::<u8>(7)==7);}",
        true,
    ),
    (
        "conditional-missing",
        "trait A{type Out;}impl A for u8{type Out=u16;}trait Use<T:A>{fn take(x:T::Out)->T::Out;}impl<T:A+Copy>Use<T>for u32{fn take(x:T::Out)->T::Out{x}}fn use_it<X:A>(x:X::Out)->X::Out{<u32 as Use<X>>::take(x)}fn main(){}",
        false,
    ),
    (
        "invalid-unused-signature",
        "trait A{type Out;}trait Use<T:A>{fn take(x:T::Out)->T::Out;}impl<T:A>Use<T>for u32{fn take(x:T::Out)->u8{7}}fn main(){}",
        false,
    ),
    (
        "result-no-copy",
        "trait A{type Out;}trait Use<T:A>{fn take(x:T::Out)->(T::Out,T::Out);}impl<T:A>Use<T>for u32{fn take(x:T::Out)->(T::Out,T::Out){(x,x)}}fn main(){}",
        false,
    ),
    (
        "result-not-input-copy",
        "trait A{type Out:Copy;}trait Use<T:A>{fn take(x:T)->(T,T);}impl<T:A>Use<T>for u32{fn take(x:T)->(T,T){(x,x)}}fn main(){}",
        false,
    ),
    (
        "missing-parent",
        "trait A{type Out;}impl A for u8{type Out=u16;}trait Use<T:A>{fn take(x:T::Out)->T::Out;}impl<T:A>Use<T>for u32{fn take(x:T::Out)->T::Out{x}}fn wrong<X>(x:<X as A>::Out)-><X as A>::Out{<u32 as Use<X>>::take(x)}fn main(){}",
        false,
    ),
    (
        "lifetime-escape",
        "trait A<'a>{type Out:'a;}impl<'a>A<'a>for u8{type Out=&'a u8;}trait Use<'a,T:A<'a>>{fn take(x:T::Out)->T::Out;}impl<'a,T:A<'a>>Use<'a,T>for u32{fn take(x:T::Out)->T::Out{x}}fn wrong<'a>(x:&'a u8)->&'static u8{<u32 as Use<'a,u8>>::take(x)}fn main(){}",
        false,
    ),
    (
        "late-reference",
        "trait A{type Out;}impl A for u8{type Out=u16;}trait Use<T:A>{fn take<'a>(x:&'a T::Out)->&'a T::Out;}impl<T:A>Use<T>for u32{fn take<'a>(x:&'a T::Out)->&'a T::Out{x}}fn main(){let x=7u16;check(*<u32 as Use<u8>>::take(&x)==7);}",
        true,
    ),
    (
        "late-function-pointer",
        "trait A{type Out;}impl A for u8{type Out=u16;}trait Use<T:A>{fn take<'a>(x:&'a T::Out)->&'a T::Out;}impl<T:A>Use<T>for u32{fn take<'a>(x:&'a T::Out)->&'a T::Out{x}}fn main(){let f:fn(&u16)->&u16=<u32 as Use<u8>>::take;let x=7u16;check(*f(&x)==7);}",
        true,
    ),
    (
        "late-lifetime-escape",
        "trait A{type Out;}impl A for u8{type Out=u16;}trait Use<T:A>{fn take<'a>(x:&'a T::Out)->&'a T::Out;}impl<T:A>Use<T>for u32{fn take<'a>(x:&'a T::Out)->&'a T::Out{x}}fn wrong<'a>(x:&'a u16)->&'static u16{<u32 as Use<u8>>::take(x)}fn main(){}",
        false,
    ),
    (
        "erased-parent-copy-missing",
        "trait A<'a>{type Out:Copy;}impl<'a,'b:'a>A<'a>for &'b u8{type Out=u16;}trait Use<'a,T:A<'a>>{fn take(x:T::Out)->T::Out;}impl<'a,T:A<'a>>Use<'a,T>for u32{fn take(x:T::Out)->T::Out{x}}fn use_it<'a,'b>(_:&'a u8,_:&'b u8)->u16{<u32 as Use<'a,&'b u8>>::take(7)}fn main(){}",
        false,
    ),
    (
        "erased-parent-missing",
        "trait A<'a>{type Out;}impl<'a,'b:'a>A<'a>for &'b u8{type Out=u16;}trait Use<'a,T:A<'a>>{fn take(x:T::Out)->T::Out;}impl<'a,T:A<'a>>Use<'a,T>for u32{fn take(x:T::Out)->T::Out{x}}fn use_it<'a,'b>(_:&'a u8,_:&'b u8)->u16{<u32 as Use<'a,&'b u8>>::take(7)}fn main(){}",
        false,
    ),
    (
        "nested-late-callable",
        "trait A{type Out;}impl A for u8{type Out=u16;}trait Use<T:A>{fn take(f:fn(&T::Out)->&T::Out,x:&T::Out)->T::Out where T::Out:Copy;}impl<T:A>Use<T>for u32{fn take(f:fn(&T::Out)->&T::Out,x:&T::Out)->T::Out where T::Out:Copy{*f(x)}}fn id(x:&u16)->&u16{x}fn main(){let x=7u16;check(<u32 as Use<u8>>::take(id,&x)==7);}",
        true,
    ),
];

#[test]
fn selected_override_signatures_match_independent_outcomes()
-> Result<(), Box<dyn std::error::Error>> {
    let mut failures = Vec::new();
    for &(name, body, accepted) in CASES {
        let source = format!("{}{body}", native::CHECK);
        object_outcome(&format!("projected-dispatch-{name}"), &source, accepted)?;
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
fn selected_override_signatures_preserve_native_values() -> Result<(), Box<dyn std::error::Error>> {
    let cases = CASES
        .iter()
        .filter(|(_, _, accepted)| *accepted)
        .map(|&(name, source, _)| (name, source, false))
        .collect::<Vec<_>>();
    native::check_cases("projected-dispatch-projections", &cases)
}
