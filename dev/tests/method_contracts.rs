//! Associated method signatures keep impl authority, callable binders and call normalization.
use crate::native_references as native;
use crate::qualify as rustc;
use crate::semantic;

const CASES: &[(&str, &str, bool)] = &[
    (
        "qualified-result",
        "trait C{type Item;fn get()-><Self as C>::Item;}impl C for u8{type Item=u16;fn get()->u16{7}}fn main(){let _=(<u8 as C>::get()==7);}\n",
        true,
    ),
    (
        "qualified-input",
        "trait C{type Item;fn get(x:<Self as C>::Item)->u8;}impl C for u8{type Item=u16;fn get(x:u16)->u8{x as u8}}fn main(){let _=(<u8 as C>::get(7)==7);}\n",
        true,
    ),
    (
        "wrong-qualified-result",
        "trait C{type Item;fn get()-><Self as C>::Item;}impl C for u8{type Item=u16;fn get()->u32{7}}fn main(){}\n",
        false,
    ),
    (
        "wrong-qualified-input",
        "trait C{type Item;fn get(x:<Self as C>::Item)->u8;}impl C for u8{type Item=u16;fn get(x:u32)->u8{x as u8}}fn main(){}\n",
        false,
    ),
    (
        "qualified-generic",
        "trait C{type Item;fn id(x:<Self as C>::Item)-><Self as C>::Item;}impl<T>C for [T;1]{type Item=T;fn id(x:T)->T{x}}fn main(){let _=(<[u16;1]as C>::id(7)==7);}\n",
        true,
    ),
    (
        "qualified-reference",
        "trait C{type Item;fn id<'a>(x:&'a <Self as C>::Item)->&'a <Self as C>::Item;}impl C for u8{type Item=u16;fn id<'a>(x:&'a u16)->&'a u16{x}}fn main(){let x=7;let _=(*<u8 as C>::id(&x)==7);}\n",
        true,
    ),
    (
        "stronger-qualified-reference",
        "trait C{type Item;fn id<'a>(x:&'a <Self as C>::Item)->&'a <Self as C>::Item;}impl C for u8{type Item=u16;fn id(x:&'static u16)->&'static u16{x}}fn main(){}\n",
        false,
    ),
    (
        "qualified-distinct-supertraits",
        "trait A{type Item;}trait B{type Item;}trait C:A+B{fn id(x:<Self as A>::Item)-><Self as B>::Item;}fn main(){}\n",
        true,
    ),
    (
        "unused-qualified-result",
        "trait C{type Item;fn get()-><Self as C>::Item;}impl C for u8{type Item=u16;fn get()->u16{7}}fn main(){}\n",
        true,
    ),
    (
        "unused-qualified-input",
        "trait C{type Item;fn get(x:<Self as C>::Item)->u8;}impl C for u8{type Item=u16;fn get(x:u16)->u8{x as u8}}fn main(){}\n",
        true,
    ),
    (
        "unused-wrong-qualified-result",
        "trait C{type Item;fn get()-><Self as C>::Item;}impl C for u8{type Item=u16;fn get()->u32{7}}fn main(){}\n",
        false,
    ),
    (
        "unused-wrong-qualified-input",
        "trait C{type Item;fn get(x:<Self as C>::Item)->u8;}impl C for u8{type Item=u16;fn get(x:u32)->u8{x as u8}}fn main(){}\n",
        false,
    ),
    (
        "unused-qualified-generic",
        "trait C{type Item;fn id(x:<Self as C>::Item)-><Self as C>::Item;}impl<T>C for [T;1]{type Item=T;fn id(x:T)->T{x}}fn main(){}\n",
        true,
    ),
    (
        "unused-qualified-reference",
        "trait C{type Item;fn id<'a>(x:&'a <Self as C>::Item)->&'a <Self as C>::Item;}impl C for u8{type Item=u16;fn id<'a>(x:&'a u16)->&'a u16{x}}fn main(){}\n",
        true,
    ),
    (
        "unused-stronger-qualified-reference",
        "trait C{type Item;fn id<'a>(x:&'a <Self as C>::Item)->&'a <Self as C>::Item;}impl C for u8{type Item=u16;fn id(x:&'static u16)->&'static u16{x}}fn main(){}\n",
        false,
    ),
    (
        "nested-function",
        "trait C{type Out;fn wrap(x:fn()-><Self as C>::Out)->fn()-><Self as C>::Out;}impl C for u8{type Out=u16;fn wrap(x:fn()->u16)->fn()->u16{x}}fn main(){}\n",
        true,
    ),
    (
        "nested-captured-function",
        "trait C{type Out;fn wrap(x:fn()-><Self as C>::Out)->fn()-><Self as C>::Out;}impl<'a>C for &'a u16{type Out=&'a u16;fn wrap(x:fn()->&'a u16)->fn()->&'a u16{x}}fn main(){}\n",
        true,
    ),
    (
        "mixed-trait-inputs",
        "trait C<'a,T,const N:usize>{type Out;fn id(x:<Self as C<'a,T,N>>::Out)-><Self as C<'a,T,N>>::Out;}impl<'a,T:'a>C<'a,T,3>for u8{type Out=&'a T;fn id(x:&'a T)->&'a T{x}}fn main(){}\n",
        true,
    ),
    (
        "stronger-generic-requirement",
        "trait C{type Out;fn make<T>(x:T)-><Self as C>::Out;}impl C for u8{type Out=u16;fn make<T:Copy>(x:T)->u16{7}}fn main(){}\n",
        false,
    ),
    (
        "unvalidated-result",
        "trait Need{}trait C{type Out:Need;fn get()-><Self as C>::Out;}impl C for u8{type Out=u16;fn get()->u16{7}}fn main(){}\n",
        false,
    ),
    (
        "original-formation-escape",
        "trait A{type Out;}impl<'a,T>A for &'a T{type Out=u8;}trait C<'a>{type Out;fn promote<'b>(_:<&'a &'b u8 as A>::Out,x:&'b u8)-><Self as C<'a>>::Out;}impl<'a>C<'a>for u8{type Out=&'a u8;fn promote<'b>(_:<&'a &'b u8 as A>::Out,x:&'b u8)->&'a u8{x}}fn escape()->&'static u8{let x=7;<u8 as C<'static>>::promote(0,&x)}fn main(){let _=escape();}\n",
        false,
    ),
    (
        "different-trait-application",
        "trait C<T>{type Out;fn id(x:<Self as C<T>>::Out)->u8;}impl C<u8>for u16{type Out=u32;fn id(x:<Self as C<u16>>::Out)->u8{x as u8}}impl C<u16>for u16{type Out=u64;fn id(x:u64)->u8{x as u8}}fn main(){}\n",
        false,
    ),
    (
        "function-pointer",
        "trait C{type Out;fn get()-><Self as C>::Out;}impl C for u8{type Out=u16;fn get()->u16{73}}fn main(){let f:fn()->u16=<u8 as C>::get;let _=f();}\n",
        true,
    ),
    (
        "wrong-function-pointer",
        "trait C{type Out;fn get()-><Self as C>::Out;}impl C for u8{type Out=u16;fn get()->u16{73}}fn main(){let f:fn()->u32=<u8 as C>::get;let _=f();}\n",
        false,
    ),
    (
        "constrained-generic-call",
        "trait C{type Out;fn id(x:<Self as C>::Out)->u16;}impl C for u8{type Out=u16;fn id(x:u16)->u16{x}}fn f<T:C<Out=u16>>()->u16{<T as C>::id(73)}fn main(){let _=f::<u8>();}\n",
        true,
    ),
    (
        "unconstrained-generic-call",
        "trait C{type Out;fn id(x:<Self as C>::Out)->u16;}impl C for u8{type Out=u16;fn id(x:u16)->u16{x}}fn f<T:C>()->u16{<T as C>::id(73u16)}fn main(){}\n",
        false,
    ),
    (
        "late-reference-function-pointer",
        "trait C{type Out;fn id<'a>(x:&'a <Self as C>::Out)->&'a <Self as C>::Out;}impl C for u8{type Out=u16;fn id<'a>(x:&'a u16)->&'a u16{x}}fn main(){let f:for<'a>fn(&'a u16)->&'a u16=<u8 as C>::id;let x=73;let _=f(&x);}\n",
        true,
    ),
    (
        "dot-call",
        "trait C{type Out;fn id(&self,x:<Self as C>::Out)-><Self as C>::Out;}impl C for u8{type Out=u16;fn id(&self,x:u16)->u16{x}}fn main(){let _=1u8.id(73u16);}\n",
        true,
    ),
    (
        "independent-method-lifetimes",
        "trait C{type Out;fn pair<'a,'b>(p:fn(&'a <Self as C>::Out)->u16,q:fn(&'b <Self as C>::Out)->u16)->(fn(&'a <Self as C>::Out)->u16,fn(&'b <Self as C>::Out)->u16);}impl C for u8{type Out=u16;fn pair<'a,'b>(p:fn(&'a u16)->u16,q:fn(&'b u16)->u16)->(fn(&'a u16)->u16,fn(&'b u16)->u16){(p,q)}}fn main(){}\n",
        true,
    ),
    (
        "swapped-method-lifetimes",
        "trait C{type Out;fn pair<'a,'b>(p:fn(&'a <Self as C>::Out)->u16,q:fn(&'b <Self as C>::Out)->u16)->(fn(&'a <Self as C>::Out)->u16,fn(&'b <Self as C>::Out)->u16);}impl C for u8{type Out=u16;fn pair<'a,'b>(p:fn(&'a u16)->u16,q:fn(&'b u16)->u16)->(fn(&'b u16)->u16,fn(&'a u16)->u16){loop{}}}fn main(){}\n",
        false,
    ),
    (
        "repeated-universal-components",
        "trait C{type Out;fn first(p:for<'a>fn(&'a <Self as C>::Out)->u16,q:for<'a>fn(&'a <Self as C>::Out)->u16)->for<'a>fn(&'a <Self as C>::Out)->u16;}impl C for u8{type Out=u16;fn first(p:for<'a>fn(&'a u16)->u16,q:for<'a>fn(&'a u16)->u16)->for<'a>fn(&'a u16)->u16{p}}fn main(){}\n",
        true,
    ),
    (
        "narrowed-universal-result",
        "trait C{type Out;fn first(p:for<'a>fn(&'a <Self as C>::Out)->u16,q:for<'a>fn(&'a <Self as C>::Out)->u16)->for<'a>fn(&'a <Self as C>::Out)->u16;}impl C for u8{type Out=u16;fn first(p:for<'a>fn(&'a u16)->u16,q:for<'a>fn(&'a u16)->u16)->fn(&'static u16)->u16{loop{}}}fn main(){}\n",
        false,
    ),
    (
        "result-does-not-infer-self",
        "trait C{type Out;fn get()-><Self as C>::Out;}impl C for u8{type Out=u16;fn get()->u16{73}}fn main(){let _:u16=<_ as C>::get();}\n",
        false,
    ),
    (
        "ordinary-input-infers-self",
        "trait C{type Out;fn get(x:Self)-><Self as C>::Out;}impl C for u8{type Out=u16;fn get(x:u8)->u16{x as u16}}fn main(){let _:u16=<_ as C>::get(73u8);}\n",
        true,
    ),
];
#[test]
fn overrides_and_calls_retain_complete_associated_signatures()
-> Result<(), Box<dyn std::error::Error>> {
    for &(name, source, accepted) in CASES {
        rustc::qualify(&format!("associated-method-{name}"), source, accepted)?;
        let result = analyze(source);
        assert_eq!(result.is_ok(), accepted, "{name}: {result:?}");
    }
    Ok(())
}
fn analyze(source: &str) -> Result<(), String> {
    let (mut report, mut facts) = semantic::analyze(source)?;
    let entry = rustgm::sema::entry::select(&mut report, &facts).map_err(|e| e.to_string())?;
    rustgm::lower::instances::build(&report, &mut facts, entry)
        .map(|_| ())
        .map_err(|e| e.to_string())
}
#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn associated_method_callables_execute_against_rustc() -> Result<(), Box<dyn std::error::Error>> {
    native::check_cases(
        "associated-method-signatures",
        &[
            (
                "qualified-result",
                "trait C{type Item;fn get()-><Self as C>::Item;}impl C for u8{type Item=u16;fn get()->u16{7}}fn main(){check(<u8 as C>::get()==7);}",
                false,
            ),
            (
                "qualified-input",
                "trait C{type Item;fn get(x:<Self as C>::Item)->u8;}impl C for u8{type Item=u16;fn get(x:u16)->u8{x as u8}}fn main(){check(<u8 as C>::get(7)==7);}",
                false,
            ),
            (
                "qualified-generic",
                "trait C{type Item;fn id(x:<Self as C>::Item)-><Self as C>::Item;}impl<T>C for [T;1]{type Item=T;fn id(x:T)->T{x}}fn main(){check(<[u16;1]as C>::id(7)==7);}",
                false,
            ),
            (
                "qualified-reference",
                "trait C{type Item;fn id<'a>(x:&'a <Self as C>::Item)->&'a <Self as C>::Item;}impl C for u8{type Item=u16;fn id<'a>(x:&'a u16)->&'a u16{x}}fn main(){let x=7;check(*<u8 as C>::id(&x)==7);}",
                false,
            ),
            (
                "function-pointer",
                "trait C{type Out;fn get()-><Self as C>::Out;}impl C for u8{type Out=u16;fn get()->u16{73}}fn main(){let f:fn()->u16=<u8 as C>::get;check(f()==73);}",
                false,
            ),
            (
                "late-reference-function-pointer",
                "trait C{type Out;fn id<'a>(x:&'a <Self as C>::Out)->&'a <Self as C>::Out;}impl C for u8{type Out=u16;fn id<'a>(x:&'a u16)->&'a u16{x}}fn main(){let f:for<'a>fn(&'a u16)->&'a u16=<u8 as C>::id;let x=73;check(*f(&x)==73);}",
                false,
            ),
            (
                "constrained-generic-call",
                "trait C{type Out;fn id(x:<Self as C>::Out)->u16;}impl C for u8{type Out=u16;fn id(x:u16)->u16{x}}fn f<T:C<Out=u16>>()->u16{<T as C>::id(73)}fn main(){check(f::<u8>()==73);}",
                false,
            ),
            (
                "dot-call",
                "trait C{type Out;fn id(&self,x:<Self as C>::Out)-><Self as C>::Out;}impl C for u8{type Out=u16;fn id(&self,x:u16)->u16{x}}fn main(){check(1u8.id(73u16)==73);}",
                false,
            ),
            (
                "nested-captured-function",
                "static V:u16=73;fn v()->&'static u16{&V}trait C{type Out;fn wrap(x:fn()-><Self as C>::Out)->fn()-><Self as C>::Out;}impl<'a>C for &'a u16{type Out=&'a u16;fn wrap(x:fn()->&'a u16)->fn()->&'a u16{x}}fn main(){let f=<&'static u16 as C>::wrap(v);check(*f()==73);}",
                false,
            ),
            (
                "independent-method-lifetimes",
                "trait C{type Out;fn pair<'a,'b>(p:fn(&'a <Self as C>::Out)->u16,q:fn(&'b <Self as C>::Out)->u16)->(fn(&'a <Self as C>::Out)->u16,fn(&'b <Self as C>::Out)->u16);}impl C for u8{type Out=u16;fn pair<'a,'b>(p:fn(&'a u16)->u16,q:fn(&'b u16)->u16)->(fn(&'a u16)->u16,fn(&'b u16)->u16){(p,q)}}fn left(x:&u16)->u16{*x}fn right(x:&u16)->u16{*x+1}fn main(){let (f,g)=<u8 as C>::pair(left,right);let x=73;check(f(&x)==73);check(g(&x)==74);}",
                false,
            ),
            (
                "repeated-universal-components",
                "trait C{type Out;fn first(p:for<'a>fn(&'a <Self as C>::Out)->u16,q:for<'a>fn(&'a <Self as C>::Out)->u16)->for<'a>fn(&'a <Self as C>::Out)->u16;}impl C for u8{type Out=u16;fn first(p:for<'a>fn(&'a u16)->u16,q:for<'a>fn(&'a u16)->u16)->for<'a>fn(&'a u16)->u16{p}}fn left(x:&u16)->u16{*x}fn right(x:&u16)->u16{*x+1}fn main(){let f=<u8 as C>::first(left,right);let x=73;check(f(&x)==73);}",
                false,
            ),
            (
                "ordinary-input-infers-self",
                "trait C{type Out;fn get(x:Self)-><Self as C>::Out;}impl C for u8{type Out=u16;fn get(x:u8)->u16{x as u16}}fn main(){let x:u16=<_ as C>::get(73u8);check(x==73);}",
                false,
            ),
        ],
    )
}
