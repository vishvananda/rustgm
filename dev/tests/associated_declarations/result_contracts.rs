//! Finite demanded result promises retain parent proof and directed lifetime requirements.
use super::{native, object_outcome, semantic};

const CASES: &[(&str, &str, bool)] = &[
    (
        "member-recursive-move",
        "trait A{type Out:A;}impl A for u8{type Out=u8;}fn use_it<T:A>(x:<<T as A>::Out as A>::Out)-><<T as A>::Out as A>::Out{x}fn main(){check(use_it::<u8>(7)==7);}",
        true,
    ),
    (
        "member-recursive-copy",
        "trait A{type Out:A+Copy;}impl A for u8{type Out=u8;}fn use_it<T:A>(x:<<T as A>::Out as A>::Out)->(<<T as A>::Out as A>::Out,<<T as A>::Out as A>::Out){(x,x)}fn main(){let p=use_it::<u8>(7);check(p.0==7&&p.1==7);}",
        true,
    ),
    (
        "member-nested-copy",
        "trait A{type Out:B;}trait B{type Out:Copy;}impl A for u8{type Out=u16;}impl B for u16{type Out=u32;}fn use_it<T:A>(x:<<T as A>::Out as B>::Out)->(<<T as A>::Out as B>::Out,<<T as A>::Out as B>::Out){(x,x)}fn main(){let p=use_it::<u8>(7);check(p.0==7&&p.1==7);}",
        true,
    ),
    (
        "member-nested-optional-sized",
        "trait A{type Out:B;}trait B{type Out:?Sized+Copy;}impl A for u8{type Out=u16;}impl B for u16{type Out=u32;}fn use_it<T:A>(x:<<T as A>::Out as B>::Out)->(<<T as A>::Out as B>::Out,<<T as A>::Out as B>::Out){(x,x)}fn main(){let p=use_it::<u8>(7);check(p.0==7&&p.1==7);}",
        true,
    ),
    (
        "member-no-input-copy",
        "trait A{type Out:Copy;}fn wrong<T:A>(x:T)->(T,T){(x,x)}fn main(){}",
        false,
    ),
    (
        "header-recursive-move",
        "trait A where Self::Out:A{type Out;}impl A for u8{type Out=u8;}fn use_it<T:A>(x:<<T as A>::Out as A>::Out)-><<T as A>::Out as A>::Out{x}fn main(){check(use_it::<u8>(7)==7);}",
        true,
    ),
    (
        "header-recursive-copy",
        "trait A where Self::Out:A+Copy{type Out;}impl A for u8{type Out=u8;}fn use_it<T:A>(x:<<T as A>::Out as A>::Out)->(<<T as A>::Out as A>::Out,<<T as A>::Out as A>::Out){(x,x)}fn main(){let p=use_it::<u8>(7);check(p.0==7&&p.1==7);}",
        true,
    ),
    (
        "header-nested-copy",
        "trait A where Self::Out:B{type Out;}trait B where Self::Out:Copy{type Out:Sized;}impl A for u8{type Out=u16;}impl B for u16{type Out=u32;}fn use_it<T:A>(x:<<T as A>::Out as B>::Out)->(<<T as A>::Out as B>::Out,<<T as A>::Out as B>::Out){(x,x)}fn main(){let p=use_it::<u8>(7);check(p.0==7&&p.1==7);}",
        true,
    ),
    (
        "header-nested-optional-sized",
        "trait A where Self::Out:B{type Out;}trait B where Self::Out:Copy{type Out:?Sized;}impl A for u8{type Out=u16;}impl B for u16{type Out=u32;}fn use_it<T:A>(x:<<T as A>::Out as B>::Out)->(<<T as A>::Out as B>::Out,<<T as A>::Out as B>::Out){(x,x)}fn main(){let p=use_it::<u8>(7);check(p.0==7&&p.1==7);}",
        true,
    ),
    (
        "header-no-input-copy",
        "trait A where Self::Out:Copy{type Out;}fn wrong<T:A>(x:T)->(T,T){(x,x)}fn main(){}",
        false,
    ),
    (
        "member-recursive",
        "trait A{type Out:A;}impl A for u8{type Out=u8;}fn main(){}",
        true,
    ),
    (
        "header-recursive",
        "trait A where Self::Out:A{type Out;}impl A for u8{type Out=u8;}fn main(){}",
        true,
    ),
    (
        "header-recursive-demand",
        "trait A where Self::Out:A{type Out;}impl A for u8{type Out=u8;}fn use_it<T:A>(x:<T::Out as A>::Out)-><T::Out as A>::Out{x}fn main(){check(use_it::<u8>(7)==7);}",
        true,
    ),
    (
        "member-same-lifetime",
        "trait A<'a>{type Out:'a;}impl<'a>A<'a>for u8{type Out=&'a u8;}fn main(){let x=7u8;let y:<u8 as A<'_>>::Out=&x;check(*y==7);}",
        true,
    ),
    (
        "member-invalid-unused",
        "trait A<'a>{type Out:'a;}struct S<'b>(&'b u8);impl<'a,'b>A<'a>for S<'b>{type Out=&'b u8;}fn main(){}",
        false,
    ),
    (
        "member-explicit-outlives",
        "trait A<'a>{type Out:'a;}struct S<'b>(&'b u8);impl<'a,'b:'a>A<'a>for S<'b>{type Out=&'b u8;}fn main(){let x=7u8;let y:<S<'_> as A<'_>>::Out=&x;check(*y==7);}",
        true,
    ),
    (
        "member-generic-result",
        "trait A<'a>{type Out:'a;}struct S<T>(T);impl<'a,T:'a>A<'a>for S<T>{type Out=T;}fn main(){let x:<S<u16> as A<'_>>::Out=7;check(x==7);}",
        true,
    ),
    (
        "member-generic-result-invalid",
        "trait A<'a>{type Out:'a;}struct S<T>(T);impl<'a,T>A<'a>for S<T>{type Out=T;}fn main(){}",
        false,
    ),
    (
        "member-stronger-than-input",
        "trait A{type Out:'static;}impl<'a>A for &'a u8{type Out=u16;}fn require<T:'static>(x:T)->T{x}fn use_it<T:A>(x:T::Out)->T::Out{require(x)}fn main(){check(use_it::<&u8>(7)==7);}",
        true,
    ),
    (
        "member-no-reverse-input",
        "trait A{type Out:'static;}fn require<T:'static>(x:T)->T{x}fn use_it<T:A>(x:T)->T{require(x)}fn main(){}",
        false,
    ),
    (
        "header-same-lifetime",
        "trait A<'a>where Self::Out:'a{type Out;}impl<'a>A<'a>for u8{type Out=&'a u8;}fn main(){let x=7u8;let y:<u8 as A<'_>>::Out=&x;check(*y==7);}",
        true,
    ),
    (
        "header-invalid-unused",
        "trait A<'a>where Self::Out:'a{type Out;}struct S<'b>(&'b u8);impl<'a,'b>A<'a>for S<'b>{type Out=&'b u8;}fn main(){}",
        false,
    ),
    (
        "header-explicit-outlives",
        "trait A<'a>where Self::Out:'a{type Out;}struct S<'b>(&'b u8);impl<'a,'b:'a>A<'a>for S<'b>{type Out=&'b u8;}fn main(){let x=7u8;let y:<S<'_> as A<'_>>::Out=&x;check(*y==7);}",
        true,
    ),
    (
        "header-generic-result",
        "trait A<'a>where Self::Out:'a{type Out;}struct S<T>(T);impl<'a,T:'a>A<'a>for S<T>{type Out=T;}fn main(){let x:<S<u16> as A<'_>>::Out=7;check(x==7);}",
        true,
    ),
    (
        "header-generic-result-invalid",
        "trait A<'a>where Self::Out:'a{type Out;}struct S<T>(T);impl<'a,T>A<'a>for S<T>{type Out=T;}fn main(){}",
        false,
    ),
    (
        "header-stronger-than-input",
        "trait A where Self::Out:'static{type Out;}impl<'a>A for &'a u8{type Out=u16;}fn require<T:'static>(x:T)->T{x}fn use_it<T:A>(x:T::Out)->T::Out{require(x)}fn main(){check(use_it::<&u8>(7)==7);}",
        true,
    ),
    (
        "header-no-reverse-input",
        "trait A where Self::Out:'static{type Out;}fn require<T:'static>(x:T)->T{x}fn use_it<T:A>(x:T)->T{require(x)}fn main(){}",
        false,
    ),
    (
        "inherited-implied",
        "trait A:B where Self::Out:'static{}trait B{type Out;}impl B for u8{type Out=u16;}impl A for u8{}fn require<X:'static>(x:X)->X{x}fn use_it<T:A>(x:T::Out)->T::Out{require(x)}fn main(){check(use_it::<u8>(7)==7);}",
        false,
    ),
    (
        "inherited-explicit",
        "trait A:B where Self::Out:'static{}trait B{type Out;}impl B for u8{type Out=u16;}impl A for u8{}fn require<X:'static>(x:X)->X{x}fn use_it<T:A>(x:T::Out)->T::Out where T::Out:'static{require(x)}fn main(){check(use_it::<u8>(7)==7);}",
        true,
    ),
    (
        "member-indirect-lifetime",
        "trait Need<'a>:'a{}impl<'a>Need<'a>for &'a u8{}trait A<'a>{type Out:Need<'a>;}impl<'a>A<'a>for u8{type Out=&'a u8;}fn require<'a,X:'a>(x:X)->X{x}fn use_it<'a,T:A<'a>>(x:T::Out)->T::Out{require::<'a,T::Out>(x)}fn main(){let x=7u8;check(*use_it::<u8>(&x)==7);}",
        true,
    ),
    (
        "member-indirect-invalid",
        "trait Need<'a>:'a{}impl<'a,'b:'a>Need<'a>for &'b u8{}trait A<'a>{type Out:Need<'a>;}struct S<'b>(&'b u8);impl<'a,'b>A<'a>for S<'b>{type Out=&'b u8;}fn main(){}",
        false,
    ),
    (
        "member-indirect-static",
        "trait Need:'static{}impl Need for u16{}trait A{type Out:Need;}impl<'a>A for &'a u8{type Out=u16;}fn require<X:'static>(x:X)->X{x}fn use_it<T:A>(x:T::Out)->T::Out{require(x)}fn main(){check(use_it::<&u8>(7)==7);}",
        true,
    ),
    (
        "header-indirect-static",
        "trait Need:'static{}impl Need for u16{}trait A where Self::Out:Need{type Out;}impl<'a>A for &'a u8{type Out=u16;}fn require<X:'static>(x:X)->X{x}fn use_it<T:A>(x:T::Out)->T::Out{require(x)}fn main(){check(use_it::<&u8>(7)==7);}",
        true,
    ),
    (
        "member-recursive-static",
        "trait A{type Out:A+'static;}impl A for u8{type Out=u8;}impl<'a>A for &'a u8{type Out=u8;}fn keep<T:'static>(_:T){}fn use_it<T:A>(x:<<<T as A>::Out as A>::Out as A>::Out){keep(x)}fn scope<'a>(_:&'a u8){use_it::<&'a u8>(7)}fn main(){let x=7u8;scope(&x);}",
        true,
    ),
    (
        "member-nested-lifetime",
        "trait A<'a>{type Out:B<'a>;}trait B<'b>{type Out:'b;}impl<'a>A<'a>for u8{type Out=u16;}impl<'a>B<'a>for u16{type Out=&'a u8;}fn keep<'a,T:'a>(_:T){}fn use_it<'a,T:A<'a>>(x:<<T as A<'a>>::Out as B<'a>>::Out){keep::<'a,_>(x)}fn main(){let x=7u8;use_it::<u8>(&x);}",
        true,
    ),
    (
        "member-nested-static",
        "trait A{type Out:B;}trait B{type Out:'static;}impl A for u8{type Out=u16;}impl B for u16{type Out=u32;}fn keep<T:'static>(_:T){}fn use_it<T:A>(x:<<T as A>::Out as B>::Out){keep(x)}fn main(){use_it::<u8>(7);}",
        true,
    ),
    (
        "member-second-lifetime",
        "trait A<'a,'b>{type Out:'b;}impl<'a,'b>A<'a,'b>for u8{type Out=&'b u8;}fn keep<'b,T:'b>(_:T){}fn use_it<'a,'b,T:A<'a,'b>>(x:T::Out){keep::<'b,_>(x)}fn main(){let x=7u8;use_it::<u8>(&x);}",
        true,
    ),
    (
        "member-wrong-lifetime",
        "trait A<'a,'b>{type Out:'b;}fn keep<'a,T:'a>(_:T){}fn wrong<'a,'b,T:A<'a,'b>>(x:T::Out){keep::<'a,_>(x)}fn main(){}",
        false,
    ),
    (
        "member-conditional-parent",
        "trait A<'a>{type Out:'a;}impl<'a,T:'a>A<'a>for(T,){type Out=T;}fn wrong<'a,T>(x:T){let _y:<(T,)as A<'a>>::Out=x;}fn main(){}",
        false,
    ),
    (
        "member-missing-parent",
        "trait A<'a>{type Out:'a;}fn wrong<'a,T>(x:<T as A<'a>>::Out){}fn main(){}",
        false,
    ),
    (
        "member-invalid-unused-nested",
        "trait A{type Out:B;}trait B{type Out:'static;}impl A for u8{type Out=u16;}impl B for u16{type Out=&'static mut u8;}fn wrong<'a>(x:&'a mut u8)-><u16 as B>::Out{x}fn main(){}",
        false,
    ),
    (
        "member-concrete-conditional-valid",
        "trait A<'a>{type Out:'a;}impl<'a,T:'a>A<'a>for(T,){type Out=T;}fn use_it<'a,T:'a>(x:T)-><(T,)as A<'a>>::Out{x}fn main(){let x=7u8;check(*use_it(&x)==7);}",
        true,
    ),
    (
        "header-recursive-static",
        "trait A where Self::Out:A+'static{type Out;}impl A for u8{type Out=u8;}impl<'a>A for &'a u8{type Out=u8;}fn keep<T:'static>(_:T){}fn use_it<T:A>(x:<<<T as A>::Out as A>::Out as A>::Out){keep(x)}fn scope<'a>(_:&'a u8){use_it::<&'a u8>(7)}fn main(){let x=7u8;scope(&x);}",
        true,
    ),
    (
        "header-nested-lifetime",
        "trait A<'a> where Self::Out:B<'a>{type Out;}trait B<'b> where Self::Out:'b{type Out;}impl<'a>A<'a>for u8{type Out=u16;}impl<'a>B<'a>for u16{type Out=&'a u8;}fn keep<'a,T:'a>(_:T){}fn use_it<'a,T:A<'a>>(x:<<T as A<'a>>::Out as B<'a>>::Out){keep::<'a,_>(x)}fn main(){let x=7u8;use_it::<u8>(&x);}",
        true,
    ),
    (
        "header-nested-static",
        "trait A where Self::Out:B{type Out;}trait B where Self::Out:'static{type Out;}impl A for u8{type Out=u16;}impl B for u16{type Out=u32;}fn keep<T:'static>(_:T){}fn use_it<T:A>(x:<<T as A>::Out as B>::Out){keep(x)}fn main(){use_it::<u8>(7);}",
        true,
    ),
    (
        "header-second-lifetime",
        "trait A<'a,'b> where Self::Out:'b{type Out;}impl<'a,'b>A<'a,'b>for u8{type Out=&'b u8;}fn keep<'b,T:'b>(_:T){}fn use_it<'a,'b,T:A<'a,'b>>(x:T::Out){keep::<'b,_>(x)}fn main(){let x=7u8;use_it::<u8>(&x);}",
        true,
    ),
    (
        "header-wrong-lifetime",
        "trait A<'a,'b> where Self::Out:'b{type Out;}fn keep<'a,T:'a>(_:T){}fn wrong<'a,'b,T:A<'a,'b>>(x:T::Out){keep::<'a,_>(x)}fn main(){}",
        false,
    ),
    (
        "header-conditional-parent",
        "trait A<'a> where Self::Out:'a{type Out;}impl<'a,T:'a>A<'a>for(T,){type Out=T;}fn wrong<'a,T>(x:T){let _y:<(T,)as A<'a>>::Out=x;}fn main(){}",
        false,
    ),
    (
        "header-missing-parent",
        "trait A<'a> where Self::Out:'a{type Out;}fn wrong<'a,T>(x:<T as A<'a>>::Out){}fn main(){}",
        false,
    ),
    (
        "header-invalid-unused-nested",
        "trait A where Self::Out:B{type Out;}trait B where Self::Out:'static{type Out;}impl A for u8{type Out=u16;}impl B for u16{type Out=&'static mut u8;}fn wrong<'a>(x:&'a mut u8)-><u16 as B>::Out{x}fn main(){}",
        false,
    ),
    (
        "header-concrete-conditional-valid",
        "trait A<'a> where Self::Out:'a{type Out;}impl<'a,T:'a>A<'a>for(T,){type Out=T;}fn use_it<'a,T:'a>(x:T)-><(T,)as A<'a>>::Out{x}fn main(){let x=7u8;check(*use_it(&x)==7);}",
        true,
    ),
    (
        "header-nested-equality-1",
        "trait A where Self::Out:B{type Out;}trait B:C<Out=u8>{}trait C{type Out;}impl A for u8{type Out=u16;}impl B for u16{}impl C for u16{type Out=u8;}fn use_it<T:A>(x:<<T as A>::Out as C>::Out)->u8{x}fn main(){check(use_it::<u8>(7)==7);}",
        true,
    ),
    (
        "member-self-alias-copy",
        "type Id<T>=T;trait C where Id<Self>:Copy{}trait A{type Out:C;}impl C for u16{}impl A for u8{type Out=u16;}fn dup<T:A>(x:T::Out)->(T::Out,T::Out){(x,x)}fn main(){let p=dup::<u8>(7);check(p.0==7&&p.1==7);}",
        false,
    ),
    (
        "header-self-alias-copy",
        "type Id<T>=T;trait C where Id<Self>:Copy{}trait A where Self::Out:C{type Out;}impl C for u16{}impl A for u8{type Out=u16;}fn dup<T:A>(x:T::Out)->(T::Out,T::Out){(x,x)}fn main(){let p=dup::<u8>(7);check(p.0==7&&p.1==7);}",
        false,
    ),
    (
        "direct-self-alias-copy",
        "type Id<T>=T;trait C where Id<Self>:Copy{}impl C for u8{}fn dup<T:C>(x:T)->(T,T){(x,x)}fn main(){let p=dup(7u8);check(p.0==7&&p.1==7);}",
        false,
    ),
    (
        "member-self-alias-lifetime",
        "type Id<T>=T;trait C where Id<Self>:'static{}trait A{type Out:C;}impl C for u16{}impl A for u8{type Out=u16;}fn keep<T:'static>(x:T)->T{x}fn use_it<T:A>(x:T::Out)->T::Out{keep(x)}fn main(){check(use_it::<u8>(7)==7);}",
        false,
    ),
    (
        "header-self-alias-lifetime",
        "type Id<T>=T;trait C where Id<Self>:'static{}trait A where Self::Out:C{type Out;}impl C for u16{}impl A for u8{type Out=u16;}fn keep<T:'static>(x:T)->T{x}fn use_it<T:A>(x:T::Out)->T::Out{keep(x)}fn main(){check(use_it::<u8>(7)==7);}",
        false,
    ),
    (
        "alias-owned-result-copy",
        "type Out<T:?Sized+A>=<T as A>::Out;trait A where Out<Self>:Copy{type Out;}impl A for u8{type Out=u16;}fn dup<T:A>(x:T::Out)->(T::Out,T::Out){(x,x)}fn main(){let p=dup::<u8>(7);check(p.0==7&&p.1==7);}",
        true,
    ),
    (
        "member-nested-default",
        "trait A{type Out:A+Copy;}impl A for u8{type Out=u8;}trait Use<T:A>{fn take(x:<<T as A>::Out as A>::Out)->(<<T as A>::Out as A>::Out,<<T as A>::Out as A>::Out){(x,x)}}impl<T:A>Use<T>for u16{}fn main(){let p=<u16 as Use<u8>>::take(7);check(p.0==7&&p.1==7);}",
        true,
    ),
    (
        "member-nested-default-optional",
        "trait A{type Out:?Sized+A+Copy;}impl A for u8{type Out=u8;}trait Use<T:A>{fn take(x:<<T as A>::Out as A>::Out)->(<<T as A>::Out as A>::Out,<<T as A>::Out as A>::Out){(x,x)}}impl<T:A>Use<T>for u16{}fn main(){let p=<u16 as Use<u8>>::take(7);check(p.0==7&&p.1==7);}",
        true,
    ),
    (
        "header-nested-default",
        "trait A where Self::Out:A+Copy{type Out;}impl A for u8{type Out=u8;}trait Use<T:A>{fn take(x:<<T as A>::Out as A>::Out)->(<<T as A>::Out as A>::Out,<<T as A>::Out as A>::Out){(x,x)}}impl<T:A>Use<T>for u16{}fn main(){let p=<u16 as Use<u8>>::take(7);check(p.0==7&&p.1==7);}",
        true,
    ),
    (
        "header-nested-default-optional",
        "trait A where Self::Out:A+Copy{type Out:?Sized;}impl A for u8{type Out=u8;}trait Use<T:A>{fn take(x:<<T as A>::Out as A>::Out)->(<<T as A>::Out as A>::Out,<<T as A>::Out as A>::Out){(x,x)}}impl<T:A>Use<T>for u16{}fn main(){let p=<u16 as Use<u8>>::take(7);check(p.0==7&&p.1==7);}",
        true,
    ),
];

#[test]
fn associated_result_contracts_match_independent_outcomes() -> Result<(), Box<dyn std::error::Error>>
{
    let mut failures = Vec::new();
    for &(name, body, accepted) in CASES {
        let source = format!("{}{body}", native::CHECK);
        object_outcome(&format!("result-contract-{name}"), &source, accepted)?;
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
fn associated_result_contracts_preserve_native_values() -> Result<(), Box<dyn std::error::Error>> {
    let cases = CASES
        .iter()
        .filter(|(_, _, accepted)| *accepted)
        .map(|&(name, source, _)| (name, source, false))
        .collect::<Vec<_>>();
    native::check_cases("result-contract-projections", &cases)
}
