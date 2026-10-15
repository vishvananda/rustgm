//! Alias obligations defer at definitions and remain mandatory at every concrete use.
#[path = "harness/native_references.rs"]
mod native;
#[path = "helpers/qualify.rs"]
mod rustc;
#[path = "support/semantic.rs"]
mod semantic;
const CASES: &[(&str, &str, bool)] = &[
    (
        "unused",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}type P<'a,'b>=<u8 as A<'a,'b>>::Out;fn main(){}",
        true,
    ),
    (
        "signature",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}type P<'a,'b>=<u8 as A<'a,'b>>::Out;fn f<'a,'b:'a>(x:&'b u8)->P<'a,'b>{x}fn main(){}",
        true,
    ),
    (
        "signature-missing",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}type P<'a,'b>=<u8 as A<'a,'b>>::Out;fn f<'a,'b>(x:&'b u8)->P<'a,'b>{x}fn main(){}",
        false,
    ),
    (
        "local",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}type P<'a,'b>=<u8 as A<'a,'b>>::Out;fn f<'a,'b:'a>(x:&'b u8){let _:P<'a,'b>=x;}fn main(){}",
        true,
    ),
    (
        "local-missing",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}type P<'a,'b>=<u8 as A<'a,'b>>::Out;fn f<'a,'b>(x:&'b u8){let _:P<'a,'b>=x;}fn main(){}",
        false,
    ),
    (
        "static",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}type P<'a,'b>=<u8 as A<'a,'b>>::Out;fn f<'a>()->P<'a,'static>{&7}fn main(){}",
        true,
    ),
    (
        "static-missing",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}type P<'a,'b>=<u8 as A<'a,'b>>::Out;fn f<'a>(x:&'a u8)->P<'static,'a>{x}fn main(){}",
        false,
    ),
    (
        "permuted",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}type P<'a,'b>=<u8 as A<'a,'b>>::Out;type Q<'x,'y>=P<'y,'x>;fn f<'x:'y,'y>(x:&'x u8)->Q<'x,'y>{x}fn main(){}",
        true,
    ),
    (
        "permuted-missing",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}type P<'a,'b>=<u8 as A<'a,'b>>::Out;type Q<'x,'y>=P<'y,'x>;fn f<'x,'y>(x:&'x u8)->Q<'x,'y>{x}fn main(){}",
        false,
    ),
    (
        "wrapper",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}type P<'a,'b>=<u8 as A<'a,'b>>::Out;type Q<'a,'b>=(P<'a,'b>,u8);fn f<'a,'b:'a>(x:&'b u8)->Q<'a,'b>{(x,7)}fn main(){}",
        true,
    ),
    (
        "wrapper-missing",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}type P<'a,'b>=<u8 as A<'a,'b>>::Out;type Q<'a,'b>=(P<'a,'b>,u8);fn f<'a,'b>(x:&'b u8)->Q<'a,'b>{(x,7)}fn main(){}",
        false,
    ),
    (
        "erased-unused",
        "trait A<'a>{type Out;}impl<'a,T:'a>A<'a>for T{type Out=u8;}type P<'a,T>=<T as A<'a>>::Out;fn main(){}",
        true,
    ),
    (
        "erased",
        "trait A<'a>{type Out;}impl<'a,T:'a>A<'a>for T{type Out=u8;}type P<'a,T>=<T as A<'a>>::Out;fn f<'a,T:'a>()->P<'a,T>{7}fn main(){}",
        true,
    ),
    (
        "erased-missing",
        "trait A<'a>{type Out;}impl<'a,T:'a>A<'a>for T{type Out=u8;}type P<'a,T>=<T as A<'a>>::Out;fn f<'a,T>()->P<'a,T>{7}fn main(){}",
        false,
    ),
    (
        "erased-dependent-unused",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}type P<'a,'b>=<u8 as A<'a,'b>>::Out;trait E{type Out;}impl<T>E for T{type Out=u8;}type Q<'a,'b>=<P<'a,'b>as E>::Out;fn main(){}",
        true,
    ),
    (
        "erased-dependent-use",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}type P<'a,'b>=<u8 as A<'a,'b>>::Out;trait E{type Out;}impl<T>E for T{type Out=u8;}type Q<'a,'b>=<P<'a,'b>as E>::Out;fn f<'a,'b>()->Q<'a,'b>{7}fn main(){}",
        false,
    ),
    (
        "alias-default-unused",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}type P<'a,'b>=<u8 as A<'a,'b>>::Out;type Q<'a,'b,T=P<'a,'b>>=T;fn main(){}",
        true,
    ),
    (
        "alias-default-overridden",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}type P<'a,'b>=<u8 as A<'a,'b>>::Out;type Q<'a,'b,T=P<'a,'b>>=T;fn f<'a,'b>()->Q<'a,'b,u8>{7}fn main(){}",
        true,
    ),
    (
        "alias-default-selected",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}type P<'a,'b>=<u8 as A<'a,'b>>::Out;type Q<'a,'b,T=P<'a,'b>>=T;fn f<'a,'b>(x:&'b u8)->Q<'a,'b>{x}fn main(){}",
        false,
    ),
    (
        "alias-nested-unused",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}type P<'a,'b>=<u8 as A<'a,'b>>::Out;type F<'a,'b>=fn(P<'a,'b>);fn main(){}",
        true,
    ),
    (
        "alias-nested-use",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}type P<'a,'b>=<u8 as A<'a,'b>>::Out;type F<'a,'b>=fn(P<'a,'b>);fn f<'a,'b>()->F<'a,'b>{loop{}}fn main(){}",
        false,
    ),
    (
        "nested-valid",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}type P<'a,'b>=<u8 as A<'a,'b>>::Out;type F<'a,'b>=fn(P<'a,'b>);fn f<'a,'b:'a>()->F<'a,'b>{loop{}}fn main(){}",
        true,
    ),
    (
        "direct-nested-unused",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}type P<'a,'b>=<u8 as A<'a,'b>>::Out;type F<'a,'b>=fn(<u8 as A<'a,'b>>::Out);fn main(){}",
        true,
    ),
    (
        "direct-nested-valid",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}type P<'a,'b>=<u8 as A<'a,'b>>::Out;type F<'a,'b>=fn(<u8 as A<'a,'b>>::Out);fn f<'a,'b:'a>()->F<'a,'b>{loop{}}fn main(){}",
        true,
    ),
    (
        "direct-nested-missing",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}type P<'a,'b>=<u8 as A<'a,'b>>::Out;type F<'a,'b>=fn(<u8 as A<'a,'b>>::Out);fn f<'a,'b>()->F<'a,'b>{loop{}}fn main(){}",
        false,
    ),
    (
        "nested-default-unused",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}type P<'a,'b>=<u8 as A<'a,'b>>::Out;type Q<'a,'b,T=fn(P<'a,'b>)>=T;fn main(){}",
        true,
    ),
    (
        "nested-default-overridden",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}type P<'a,'b>=<u8 as A<'a,'b>>::Out;type Q<'a,'b,T=fn(P<'a,'b>)>=T;fn f<'a,'b>()->Q<'a,'b,u8>{7}fn main(){}",
        true,
    ),
    (
        "nested-default-valid",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}type P<'a,'b>=<u8 as A<'a,'b>>::Out;type Q<'a,'b,T=fn(P<'a,'b>)>=T;fn f<'a,'b:'a>()->Q<'a,'b>{loop{}}fn main(){}",
        true,
    ),
    (
        "nested-default-missing",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}type P<'a,'b>=<u8 as A<'a,'b>>::Out;type Q<'a,'b,T=fn(P<'a,'b>)>=T;fn f<'a,'b>()->Q<'a,'b>{loop{}}fn main(){}",
        false,
    ),
    (
        "nominal-default-unused",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}type P<'a,'b>=<u8 as A<'a,'b>>::Out;struct S<'a,'b,T=P<'a,'b>>(&'a (),&'b (),T);fn main(){}",
        true,
    ),
    (
        "alias-bound-not-granted",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}type P<'a,'b>=<u8 as A<'a,'b>>::Out;type Q<'a,'b:'a>=P<'a,'b>;fn f<'a,'b>(x:&'b u8)->Q<'a,'b>{x}fn main(){}",
        false,
    ),
    (
        "associated-use-valid",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}type P<'a,'b>=<u8 as A<'a,'b>>::Out;trait B{type Out;}impl<'a,'b:'a>B for (&'a (),&'b ()){type Out=P<'a,'b>;}fn main(){}",
        true,
    ),
    (
        "associated-use-missing",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}type P<'a,'b>=<u8 as A<'a,'b>>::Out;trait B{type Out;}impl<'a,'b>B for (&'a (),&'b ()){type Out=P<'a,'b>;}fn main(){}",
        false,
    ),
    (
        "same-scope-swapped-missing",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}type P<'a,'b>=<u8 as A<'a,'b>>::Out;fn f<'a,'b:'a>(x:&'b u8,y:&'a u8)->(P<'a,'b>,P<'b,'a>){(x,y)}fn main(){}",
        false,
    ),
    (
        "independent-scope-missing",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}type P<'a,'b>=<u8 as A<'a,'b>>::Out;fn f<'a,'b:'a>(x:&'b u8)->P<'a,'b>{x}fn g<'a,'b>(x:&'b u8)->P<'a,'b>{x}fn main(){}",
        false,
    ),
    (
        "default-valid",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}type P<'a,'b>=<u8 as A<'a,'b>>::Out;type Q<'a,'b,T=P<'a,'b>>=T;fn f<'a,'b:'a>(x:&'b u8)->Q<'a,'b>{x}fn main(){}",
        true,
    ),
    (
        "nominal-default-valid",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}type P<'a,'b>=<u8 as A<'a,'b>>::Out;struct S<'a,'b,T=P<'a,'b>>(&'a (),&'b (),T);fn f<'a,'b:'a>(a:&'a (),b:&'b (),x:&'b u8)->S<'a,'b>{S(a,b,x)}fn main(){}",
        true,
    ),
    (
        "nominal-default-missing",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}type P<'a,'b>=<u8 as A<'a,'b>>::Out;struct S<'a,'b,T=P<'a,'b>>(&'a (),&'b (),T);fn f<'a,'b>(a:&'a (),b:&'b (),x:&'b u8)->S<'a,'b>{S(a,b,x)}fn main(){}",
        false,
    ),
    (
        "nominal-default-overridden",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}type P<'a,'b>=<u8 as A<'a,'b>>::Out;struct S<'a,'b,T=P<'a,'b>>(&'a (),&'b (),T);fn f<'a,'b>(a:&'a (),b:&'b ())->S<'a,'b,u8>{S(a,b,7)}fn main(){}",
        true,
    ),
];
#[test]
fn independently_qualified_alias_uses() -> Result<(), Box<dyn std::error::Error>> {
    let mut failures = Vec::new();
    for &(name, source, accepted) in CASES {
        rustc::qualify(&format!("alias-obligation-{name}"), source, accepted)?;
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

#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn aliases_execute_with_instantiated_lifetime_contracts() -> Result<(), Box<dyn std::error::Error>>
{
    native::check_cases(
        "conditional-aliases",
        &[
            (
                "reference-chain",
                "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}type P<'a,'b>=<u8 as A<'a,'b>>::Out;type Q<'a,'b>=P<'b,'a>;fn f<'a:'b,'b>(x:&'a u8)->Q<'a,'b>{x}fn main(){let x=73u8;check(*f(&x)==73)}",
                false,
            ),
            (
                "default",
                "trait A<'a>{type Out;}impl<'a,T:'a>A<'a>for T{type Out=T;}type P<'a,T>=<T as A<'a>>::Out;type D<'a,T,U=P<'a,T>>=(T,U);fn f<'a>(x:u8)->D<'a,u8>{(x,x+1)}fn main(){let x=f(72);check(x.0+x.1==145)}",
                false,
            ),
            (
                "captured-pointer",
                "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}type P<'a,'b>=<u8 as A<'a,'b>>::Out;type F<'a,'b>=fn(P<'a,'b>)->u8;fn read(x:&u8)->u8{*x}fn main(){let f:F<'static,'static>=read;check(f(&73)==73)}",
                false,
            ),
        ],
    )
}
