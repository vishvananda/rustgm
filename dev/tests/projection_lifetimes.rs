//! Scoped projection lifetimes retain independently qualified constraints through normalization.
#[path = "harness/native_references.rs"]
mod native;
#[path = "helpers/qualify.rs"]
mod rustc;
#[path = "support/semantic.rs"]
mod semantic;
const ENVIRONMENT_ORDER: &str = "trait A<'a>{type Out;}impl<'a>A<'a>for u8{type Out=u8;}fn f<'a,'b,'c,T>(x:&'a T,y:&'b [T])where &'a T:Copy,&'b [T]:Copy{let _: <u8 as A<'c>>::Out=7u8;}fn main(){}";
const CASES: &[(&str, &str, bool)] = &[
    ("environment-order", ENVIRONMENT_ORDER, true),
    (
        "nested-caller-outlives",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}fn f<'a,'b:'a>()->for<'c>fn(&'c u8,<u8 as A<'a,'b>>::Out){loop{}}fn main(){}",
        true,
    ),
    (
        "nested-caller-outlives-missing",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}fn f<'a,'b>()->for<'c>fn(&'c u8,<u8 as A<'a,'b>>::Out){loop{}}fn main(){}",
        false,
    ),
    (
        "static-input",
        "trait A{type Out;}impl<'a>A for &'a u8{type Out=&'a u8;}fn main(){let _: <&'static u8 as A>::Out=&7u8;}\n",
        true,
    ),
    (
        "signature",
        "trait A{type Out;}impl<'a>A for &'a u8{type Out=&'a u8;}fn f<'a>(x:&'a u8)-><&'a u8 as A>::Out{x}fn main(){}\n",
        true,
    ),
    (
        "cannot-strengthen",
        "trait A{type Out;}impl<'a>A for &'a u8{type Out=&'a u8;}fn f<'a>(x:&'a u8)-><&'static u8 as A>::Out{x}fn main(){}\n",
        false,
    ),
    (
        "trait-argument",
        "trait A<'a>{type Out;}impl<'a>A<'a>for u8{type Out=&'a u8;}fn f<'a>(x:&'a u8)-><u8 as A<'a>>::Out{x}fn main(){}\n",
        true,
    ),
    (
        "permuted",
        "trait A<'a,'b>{type Out;}impl<'a,'b>A<'b,'a>for u8{type Out=(&'a u8,&'b u8);}fn f<'a,'b>(x:&'a u8,y:&'b u8)-><u8 as A<'b,'a>>::Out{(x,y)}fn main(){}\n",
        true,
    ),
    (
        "permuted-wrong",
        "trait A<'a,'b>{type Out;}impl<'a,'b>A<'b,'a>for u8{type Out=(&'a u8,&'b u8);}fn f<'a,'b>(x:&'a u8,y:&'b u8)-><u8 as A<'b,'a>>::Out{(y,x)}fn main(){}\n",
        false,
    ),
    (
        "outlives",
        "trait A<'a>{type Out;}impl<'a,'b:'a>A<'a>for &'b u8{type Out=&'a u8;}fn f<'a,'b:'a>(x:&'b u8)-><&'b u8 as A<'a>>::Out{x}fn main(){}\n",
        true,
    ),
    (
        "outlives-missing",
        "trait A<'a>{type Out;}impl<'a,'b:'a>A<'a>for &'b u8{type Out=&'a u8;}fn f<'a,'b>(x:&'b u8)-><&'b u8 as A<'a>>::Out{x}fn main(){}\n",
        false,
    ),
    (
        "unused-impl-lifetime",
        "trait A{type Out;}impl<'a>A for u8{type Out=u16;}fn main(){let _: <u8 as A>::Out=7u16;}\n",
        true,
    ),
    (
        "higher-ranked-result",
        "trait A{type Out;}impl<'a>A for &'a u8{type Out=for<'b>fn(&'b u8,&'a u8)->&'b u8;}fn pair<'a,'b>(x:&'b u8,_:&'a u8)->&'b u8{x}fn f<'a>()-><&'a u8 as A>::Out{pair}fn main(){}\n",
        true,
    ),
    (
        "within-callable-binder",
        "trait A{type Out;}impl<'a>A for &'a u8{type Out=&'a u8;}type F=for<'a>fn(<&'a u8 as A>::Out)->&'a u8;fn id(x:&u8)->&u8{x}fn main(){let _:F=id;}\n",
        false,
    ),
    (
        "static-impl-not-universal",
        "trait A{type Out;}impl A for &'static u8{type Out=&'static u8;}fn f<'a>(x:&'a u8)-><&'a u8 as A>::Out{x}fn main(){}\n",
        false,
    ),
    (
        "local-annotation",
        "trait A{type Out;}impl<'a>A for &'a u8{type Out=&'a u8;}fn f<'a>(x:&'a u8)->&'a u8{let y:<&'a u8 as A>::Out=x;y}fn main(){}\n",
        true,
    ),
    (
        "local-cannot-strengthen",
        "trait A{type Out;}impl<'a>A for &'a u8{type Out=&'a u8;}fn f<'a>(x:&'a u8)->&'static u8{let y:<&'a u8 as A>::Out=x;y}fn main(){}\n",
        false,
    ),
    (
        "callable-direct-input",
        "trait A{type Out;}impl<'a>A for &'a u8{type Out=&'a u8;}type F=for<'a>fn(&'a u8,<&'a u8 as A>::Out)->&'a u8;fn id<'a>(x:&'a u8,_:&'a u8)->&'a u8{x}fn main(){let _:F=id;}\n",
        true,
    ),
    (
        "callable-alias-input",
        "trait A{type Out;}impl<'a>A for &'a u8{type Out=&'a u8;}type P<'a>=<&'a u8 as A>::Out;type F=for<'a>fn(P<'a>)->&'a u8;fn main(){}\n",
        false,
    ),
    (
        "callable-projected-output",
        "trait A{type Out;}impl<'a>A for &'a u8{type Out=&'a u8;}type F=for<'a>fn()-><&'a u8 as A>::Out;fn main(){}\n",
        false,
    ),
    (
        "callable-both-projected",
        "trait A{type Out;}impl<'a>A for &'a u8{type Out=&'a u8;}type F=for<'a>fn(<&'a u8 as A>::Out)-><&'a u8 as A>::Out;fn main(){}\n",
        false,
    ),
    (
        "local-elided",
        "trait A{type Out;}impl<'a>A for &'a u8{type Out=&'a u8;}fn f(x:&u8)->&u8{let y:<&u8 as A>::Out=x;y}fn main(){}",
        true,
    ),
    (
        "local-elided-escape",
        "trait A{type Out;}impl<'a>A for &'a u8{type Out=&'a u8;}fn f(x:&u8)->&'static u8{let y:<&u8 as A>::Out=x;y}fn main(){}",
        false,
    ),
    (
        "nested-capture",
        "trait A{type Out;}impl<'a>A for &'a u8{type Out=&'a u8;}type F<'a>=for<'b>fn(&'b u8,<&'a u8 as A>::Out)->&'b u8;fn main(){}",
        true,
    ),
    (
        "deep-capture",
        "trait A{type Out;}impl<'a>A for &'a u8{type Out=&'a u8;}type F=for<'a>fn(&'a u8,for<'b>fn(&'b u8,<&'a u8 as A>::Out)->&'b u8);fn main(){}",
        true,
    ),
    (
        "erased-impl-lifetime",
        "trait A{type Out;}trait E{type Out;}impl<'a>E for &'a u8{type Out=u8;}impl<'a>A for u8{type Out=<&'a u8 as E>::Out;}fn main(){}",
        false,
    ),
    (
        "local-outlives",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}fn f<'a,'b:'a>(x:&'b u8){let _: <u8 as A<'a,'b>>::Out=x;}fn main(){}",
        true,
    ),
    (
        "local-outlives-missing",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}fn f<'a,'b>(x:&'b u8){let _: <u8 as A<'a,'b>>::Out=x;}fn main(){}",
        false,
    ),
    (
        "early-projected-input",
        "trait A{type Out;}impl<'a>A for &'a u8{type Out=&'a u8;}fn f<'a>(x:<&'a u8 as A>::Out)->&'a u8{x}fn main(){let _=f::<'static>;}",
        true,
    ),
    (
        "early-erased-output",
        "trait A{type Out;}impl<'a>A for &'a u8{type Out=u8;}fn f<'a>(x:u8)-><&'a u8 as A>::Out{x}fn main(){let _=f::<'static>;}",
        true,
    ),
    (
        "late-erased-input",
        "trait A{type Out;}impl<'a>A for &'a u8{type Out=u8;}fn f<'a>(x:<&'a u8 as A>::Out)->u8{x}fn main(){let _=f::<'static>;}",
        false,
    ),
    (
        "output-type-outlives",
        "trait A<'a>{type Out;}impl<'a,T:'a>A<'a>for T{type Out=&'a T;}fn f<'a,T>()-><T as A<'a>>::Out{loop{}}fn main(){}",
        true,
    ),
    (
        "output-type-outlives-explicit",
        "trait A<'a>{type Out;}impl<'a,T:'a>A<'a>for T{type Out=&'a T;}fn f<'a,T:'a>()-><T as A<'a>>::Out{loop{}}fn main(){}",
        true,
    ),
    (
        "input-type-outlives",
        "trait A<'a>{type Out;}impl<'a,T:'a>A<'a>for T{type Out=&'a T;}fn f<'a,T>(x:<T as A<'a>>::Out){}fn main(){}",
        true,
    ),
    (
        "input-type-outlives-direct",
        "trait A<'a>{type Out;}impl<'a,T:'a>A<'a>for T{type Out=&'a T;}fn f<'a,T>(x:&'a T)-><T as A<'a>>::Out{x}fn main(){}",
        true,
    ),
    (
        "erased-type-outlives",
        "trait A<'a>{type Out;}impl<'a,T:'a>A<'a>for T{type Out=u8;}fn f<'a,T:'a>()-><T as A<'a>>::Out{7}fn main(){}",
        true,
    ),
    (
        "erased-type-outlives-missing",
        "trait A<'a>{type Out;}impl<'a,T:'a>A<'a>for T{type Out=u8;}fn f<'a,T>()-><T as A<'a>>::Out{7}fn main(){}",
        false,
    ),
    (
        "local-erased-type-outlives",
        "trait A<'a>{type Out;}impl<'a,T:'a>A<'a>for T{type Out=u8;}fn f<'a,T:'a>(){let _: <T as A<'a>>::Out=7u8;}fn main(){}",
        true,
    ),
    (
        "local-erased-type-outlives-missing",
        "trait A<'a>{type Out;}impl<'a,T:'a>A<'a>for T{type Out=u8;}fn f<'a,T>(){let _: <T as A<'a>>::Out=7u8;}fn main(){}",
        false,
    ),
];

#[test]
fn opened_environments_have_deterministic_semantic_identities()
-> Result<(), Box<dyn std::error::Error>> {
    let mut expected = None;
    for _ in 0..24 {
        let (report, facts) = semantic::analyze(ENVIRONMENT_ORDER)?;
        let mut bytes = Vec::new();
        rustgm::sema::semantic_view::write(&report, &facts, &mut bytes)?;
        if let Some(expected) = &expected {
            assert_eq!(&bytes, expected);
        } else {
            expected = Some(bytes);
        }
    }
    Ok(())
}

#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn projected_lifetimes_execute_with_preserved_arguments() -> Result<(), Box<dyn std::error::Error>>
{
    native::check_cases(
        "projection-lifetimes",
        &[
            (
                "reference",
                "trait A{type Out;}impl<'a>A for &'a u16{type Out=&'a u16;}fn f<'a>(x:&'a u16)-><&'a u16 as A>::Out{x}fn main(){let x=399u16;check(*f(&x)==399)}",
                false,
            ),
            (
                "permutation",
                "trait A<'a,'b>{type Out;}impl<'a,'b>A<'b,'a>for u8{type Out=(&'a u16,&'b u16);}fn f<'a,'b>(x:&'a u16,y:&'b u16)-><u8 as A<'b,'a>>::Out{(x,y)}fn main(){let x=399u16;let y=401u16;let p=f(&x,&y);check(*p.0==399);check(*p.1==401)}",
                false,
            ),
            (
                "captured-callable",
                "trait A{type Out;}impl<'a>A for &'a u8{type Out=for<'b>fn(&'b u8,&'a u8)->&'b u8;}fn pair<'a,'b>(x:&'b u8,_:&'a u8)->&'b u8{x}fn f<'a>(_:&'a u8)-><&'a u8 as A>::Out{pair}fn main(){let x=17u8;let y=23u8;let p=f(&x);check(*p(&y,&x)==23)}",
                false,
            ),
        ],
    )
}
#[test]
fn qualified_lifetime_projections() -> Result<(), Box<dyn std::error::Error>> {
    let mut failures = Vec::new();
    for &(name, source, accepted) in CASES {
        rustc::qualify(&format!("scoped-projection-{name}"), source, accepted)?;
        let result = semantic::analyze(source).and_then(|(mut report, mut facts)| {
            let entry =
                rustgm::sema::entry::select(&mut report, &facts).map_err(|e| e.to_string())?;
            rustgm::lower::instances::build(&report, &mut facts, entry)
                .map(|_| ())
                .map_err(|e| e.to_string())
        });
        if result.is_ok() != accepted {
            failures.push(format!("{name}: {:?}", result.err()));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
