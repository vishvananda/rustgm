//! Source equality contracts are qualified independently before full template proofs.
#[path = "harness/native_references.rs"]
mod native;
#[path = "helpers/qualify.rs"]
mod rustc;
#[path = "support/semantic.rs"]
mod semantic;
#[path = "helpers/template_check.rs"]
mod templates;

const CASES: &[(&str, &str, bool)] = &[
    (
        "unused-binding",
        "trait A{type Out;}fn f<T:A<Out=u8>>(){}fn main(){}",
        true,
    ),
    (
        "inherited-binding",
        "trait A{type Out;}trait B:A{}fn f<T:B<Out=u8>>(){}fn main(){}",
        true,
    ),
    (
        "forward-super-binding",
        "trait B:A<Out=u8>{}trait A{type Out;}fn main(){}",
        true,
    ),
    (
        "direct-and-inherited",
        "trait A{type Out;}trait B:A{type Out;}fn f<T:B<Out=u8>>(){}fn main(){}",
        true,
    ),
    (
        "repeated-unused-identical",
        "trait A{type Out;}fn f<T:A<Out=u8,Out=u8>>(){}fn main(){}",
        true,
    ),
    (
        "contradictory-separate",
        "trait A{type Out;}fn f<T:A<Out=u8>+A<Out=u16>>(){}fn main(){}",
        true,
    ),
    (
        "unused-copy-contradiction",
        "struct N;trait A{type Out:Copy;}fn f<T:A<Out=N>>(){}fn main(){}",
        true,
    ),
    (
        "unused-sized-contradiction",
        "trait A{type Out;}fn f<T:A<Out=str>>(){}fn main(){}",
        true,
    ),
    (
        "relaxed-result",
        "trait A{type Out:?Sized;}fn f<T:A<Out=str>>(){}fn main(){}",
        true,
    ),
    (
        "malformed-result",
        "trait A{type Out:?Sized;}fn f<T:A<Out=[str;1]>>(){}fn main(){}",
        false,
    ),
    (
        "closed-equality",
        "trait A{type Out;}impl A for u8{type Out=u16;}fn f()where u8:A<Out=u16>{}fn main(){f();}",
        true,
    ),
    (
        "closed-mismatch",
        "trait A{type Out;}impl A for u8{type Out=u16;}fn f()where u8:A<Out=u32>{}fn main(){}",
        false,
    ),
    (
        "rhs-only-parameter",
        "trait A{type Out;}impl A for u8{type Out=u16;}fn f<U>()where u8:A<Out=U>{}fn main(){f::<u16>();}",
        true,
    ),
    (
        "rhs-only-mismatch",
        "trait A{type Out;}impl A for u8{type Out=u16;}fn f<U>()where u8:A<Out=U>{}fn main(){f::<u32>();}",
        false,
    ),
    (
        "bound-before-argument",
        "trait A<U>{type Out;}fn f<T:A<Out=u8,u16>>(){}fn main(){}",
        false,
    ),
    (
        "raw-binding-name",
        "trait A{type r#type;}impl A for u8{type r#type=u16;}fn f<T:A<r#type=u16>>(x:u16)-><T as A>::r#type{x}fn main(){let _:u16=f::<u8>(7);}",
        true,
    ),
    (
        "binding-default",
        "trait A<U=u8>{type Out;}impl A for u16{type Out=u32;}fn f<T:A<Out=u32>>(x:u32)-><T as A>::Out{x}fn main(){let _:u32=f::<u16>(7);}",
        true,
    ),
    (
        "equality-parameter",
        "fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}trait A{type Out;}impl A for u8{type Out=u16;}fn id<T:A<Out=U>,U>(x:U)-><T as A>::Out{x}fn main(){check(id::<u8,_>(7u16)==7u16);}",
        true,
    ),
    (
        "equality-where",
        "fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}trait A{type Out;}impl A for u8{type Out=u16;}fn id<T>(x:u16)-><T as A>::Out where T:A<Out=u16>{x}fn main(){check(id::<u8>(7u16)==7u16);}",
        true,
    ),
    (
        "unrelated-result",
        "trait A{type Out;}impl A for u8{type Out=u16;}fn id<T:A,U>(x:U)-><T as A>::Out{x}fn main(){}",
        false,
    ),
    (
        "different-owner-results",
        "trait A{type Out;}impl A for u8{type Out=u16;}fn id<T:A,U:A>(x:<T as A>::Out)-><U as A>::Out{x}fn main(){}",
        false,
    ),
    (
        "equality-call-mismatch",
        "trait A{type Out;}impl A for u8{type Out=u16;}fn id<T:A<Out=u8>>(x:u8)-><T as A>::Out{x}fn main(){let _=id::<u8>(7u8);}",
        false,
    ),
    (
        "equality-trait-argument",
        "fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}trait A<U>{type Out;}impl A<u8>for u16{type Out=u32;}impl A<u16>for u16{type Out=u64;}fn id<T:A<u8,Out=u32>>(x:u32)-><T as A<u8>>::Out{x}fn main(){check(id::<u16>(7u32)==7u32);}",
        true,
    ),
    (
        "equality-supertrait",
        "fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}trait A{type Out;}impl A for u8{type Out=u16;}trait B:A<Out=u16>{}impl B for u8{}fn id<T:B>(x:u16)-><T as A>::Out{x}fn main(){check(id::<u8>(7u16)==7u16);}",
        true,
    ),
    (
        "equality-environment-isolation",
        "trait A{type Out;}impl A for u8{type Out=u16;}fn yes<T:A<Out=u16>>(x:u16)-><T as A>::Out{x}fn no<T:A>(x:u16)-><T as A>::Out{x}fn main(){}",
        false,
    ),
    (
        "equal-results-not-injective",
        "trait A{type Out;}impl A for u8{type Out=u16;}fn f<T:A<Out=u16>,U:A<Out=u16>>(x:T)->U{x}fn main(){}",
        false,
    ),
    (
        "unused-contradictory-equalities",
        "trait A{type Out;}impl A for u8{type Out=u16;}fn f<T:A<Out=u8,Out=u16>>(){}fn main(){}",
        true,
    ),
    (
        "contradictory-equalities-call",
        "trait A{type Out;}impl A for u8{type Out=u16;}fn f<T:A<Out=u8,Out=u16>>(){}fn main(){f::<u8>();}",
        false,
    ),
    (
        "bound-on-missing-type",
        "trait A{type Out;}impl A for u8{type Out=u16;}fn f<T:A<Missing=u8>>(){}fn main(){}",
        false,
    ),
    (
        "constraint-on-nominal",
        "struct S<T>(T);fn f(_:S<Out=u8>){}fn main(){}",
        false,
    ),
    (
        "shadow-direct",
        "trait A{type Out;}trait B:A{type Out;}fn f<T:B<Out=u8>>(x:u8)-><T as B>::Out{x}fn main(){}",
        true,
    ),
    (
        "shadow-super",
        "trait A{type Out;}trait B:A{type Out;}fn f<T:B<Out=u8>>(x:u8)-><T as A>::Out{x}fn main(){}",
        false,
    ),
    (
        "inherited-shadow",
        "trait A{type Out;}trait B:A{type Out;}trait C:B{}fn f<T:C<Out=u8>>(x:u8)-><T as B>::Out{x}fn main(){}",
        false,
    ),
    (
        "result-lifetime",
        "trait A{type Out;}fn f<'a,T:A<Out=&'a u8>>(x:&'a u8)-><T as A>::Out{x}fn main(){}",
        true,
    ),
    (
        "lifetime-argument",
        "trait A<'a>{type Out;}impl<'a>A<'a>for &'a u8{type Out=&'a u8;}fn f<'a,T:A<'a,Out=&'a u8>>(x:&'a u8)-><T as A<'a>>::Out{x}fn main(){let x=7u8;let y=f::<&u8>(&x);let _=*y;}",
        true,
    ),
    (
        "lifetime-escape",
        "trait A{type Out;}fn f<'a,T:A<Out=&'a u8>>(x:&'a u8)->&'static u8{let y:<T as A>::Out=x;y}fn main(){}",
        false,
    ),
    (
        "lifetime-missing-wf",
        "trait A{type Out;}fn f<'a,T:A<Out=&'a U>,U>(){}fn main(){}",
        false,
    ),
    (
        "lifetime-present-wf",
        "trait A{type Out;}fn f<'a,T:A<Out=&'a U>,U:'a>(){}fn main(){}",
        true,
    ),
    (
        "equal-result-lifetimes",
        "trait A{type Out;}fn f<'a:'b,'b:'a,T:A<Out=&'a u8>+A<Out=&'b u8>>(x:&'a u8)-><T as A>::Out{x}fn main(){}",
        false,
    ),
    (
        "unequal-result-lifetimes",
        "trait A{type Out;}fn f<'a,'b,T:A<Out=&'a u8>+A<Out=&'b u8>>(x:&'a u8)-><T as A>::Out{x}fn main(){}",
        false,
    ),
    (
        "callable-result",
        "trait A{type Out;}impl A for u8{type Out=for<'b>fn(&'b u8)->&'b u8;}fn f<T:A<Out=for<'a>fn(&'a u8)->&'a u8>>(x:for<'c>fn(&'c u8)->&'c u8)-><T as A>::Out{x}fn pass(x:&u8)->&u8{x}fn main(){let x=7u8;let y=f::<u8>(pass)(&x);let _=*y;}",
        true,
    ),
    (
        "same-lifetime-nonstatic",
        "trait A{type Out;}fn f<'a,T:A<Out=fn(&'a u8)>>(x:fn(&'a u8))-><T as A>::Out{x}fn main(){}",
        true,
    ),
];

#[test]
fn source_bindings_preserve_full_template_requirements() -> Result<(), Box<dyn std::error::Error>> {
    for &(name, source, accepted) in CASES {
        rustc::qualify(&format!("equality-{name}"), source, accepted)?;
        let result = templates::check(source).map(|_| ());
        assert_eq!(result.is_ok(), accepted, "{name}: {result:?}");
    }
    Ok(())
}

#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn equality_constraints_execute_with_their_selected_results()
-> Result<(), Box<dyn std::error::Error>> {
    native::check_cases(
        "bound-equalities",
        &[
            (
                "closed-equality",
                "trait A{type Out;}impl A for u8{type Out=u16;}fn f()->u16 where u8:A<Out=u16>{7}fn main(){check(f()==7);}",
                false,
            ),
            (
                "raw-binding-name",
                "trait A{type r#type;}impl A for u8{type r#type=u16;}fn f<T:A<r#type=u16>>(x:u16)-><T as A>::r#type{x}fn main(){check(f::<u8>(7)==7);}",
                false,
            ),
            (
                "binding-default",
                "trait A<U=u8>{type Out;}impl A for u16{type Out=u32;}fn f<T:A<Out=u32>>(x:u32)-><T as A>::Out{x}fn main(){check(f::<u16>(7)==7);}",
                false,
            ),
            (
                "equality-parameter",
                "trait A{type Out;}impl A for u8{type Out=u16;}fn id<T:A<Out=U>,U>(x:U)-><T as A>::Out{x}fn main(){check(id::<u8,_>(7u16)==7u16);}",
                false,
            ),
            (
                "equality-where",
                "trait A{type Out;}impl A for u8{type Out=u16;}fn id<T>(x:u16)-><T as A>::Out where T:A<Out=u16>{x}fn main(){check(id::<u8>(7u16)==7u16);}",
                false,
            ),
            (
                "equality-trait-argument",
                "trait A<U>{type Out;}impl A<u8>for u16{type Out=u32;}impl A<u16>for u16{type Out=u64;}fn id<T:A<u8,Out=u32>>(x:u32)-><T as A<u8>>::Out{x}fn main(){check(id::<u16>(7u32)==7u32);}",
                false,
            ),
            (
                "equality-supertrait",
                "trait A{type Out;}impl A for u8{type Out=u16;}trait B:A<Out=u16>{}impl B for u8{}fn id<T:B>(x:u16)-><T as A>::Out{x}fn main(){check(id::<u8>(7u16)==7u16);}",
                false,
            ),
            (
                "lifetime-argument",
                "trait A<'a>{type Out;}impl<'a>A<'a>for &'a u8{type Out=&'a u8;}fn f<'a,T:A<'a,Out=&'a u8>>(x:&'a u8)-><T as A<'a>>::Out{x}fn main(){let x=7u8;let y=f::<&u8>(&x);check(*y==7);}",
                false,
            ),
            (
                "callable-result",
                "trait A{type Out;}impl A for u8{type Out=for<'b>fn(&'b u8)->&'b u8;}fn f<T:A<Out=for<'a>fn(&'a u8)->&'a u8>>(x:for<'c>fn(&'c u8)->&'c u8)-><T as A>::Out{x}fn pass(x:&u8)->&u8{x}fn main(){let x=7u8;let y=f::<u8>(pass)(&x);check(*y==7);}",
                false,
            ),
        ],
    )
}
