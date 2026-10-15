//! Quantified alias uses retain each placeholder and conditional impl requirement.
#[path = "harness/native_references.rs"]
mod native;
#[path = "helpers/qualify.rs"]
mod rustc;
#[path = "support/semantic.rs"]
mod semantic;
const CASES: &[(&str, &str, bool)] = &[
    (
        "unused-outer-short",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}type P<'a,'b>=<u8 as A<'a,'b>>::Out;type F<'a>=for<'b>fn(P<'a,'b>);fn main(){}",
        true,
    ),
    (
        "outer-short-used",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}type P<'a,'b>=<u8 as A<'a,'b>>::Out;type F<'a>=for<'b>fn(P<'a,'b>);fn f<'a>()->F<'a>{loop{}}fn main(){}",
        false,
    ),
    (
        "unused-outer-long",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}type P<'a,'b>=<u8 as A<'a,'b>>::Out;type F<'a>=for<'b>fn(P<'b,'a>);fn main(){}",
        true,
    ),
    (
        "outer-long-used",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}type P<'a,'b>=<u8 as A<'a,'b>>::Out;type F<'a>=for<'b>fn(P<'b,'a>);fn f<'a>()->F<'a>{loop{}}fn main(){}",
        false,
    ),
    (
        "outer-long-static",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}type P<'a,'b>=<u8 as A<'a,'b>>::Out;type F<'a>=for<'b>fn(P<'b,'a>);fn f()->F<'static>{loop{}}fn main(){}",
        true,
    ),
    (
        "outer-long-bound",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}type P<'a,'b>=<u8 as A<'a,'b>>::Out;type F<'a>=for<'b>fn(P<'b,'a>);fn f<'a:'static>()->F<'a>{loop{}}fn main(){}",
        true,
    ),
    (
        "closed-self",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}type P<'a,'b>=<u8 as A<'a,'b>>::Out;type F=for<'a>fn(P<'a,'a>);fn f()->F{loop{}}fn main(){}",
        true,
    ),
    (
        "closed-long-static",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}type P<'a,'b>=<u8 as A<'a,'b>>::Out;type F=for<'a>fn(P<'a,'static>);fn f()->F{loop{}}fn main(){}",
        true,
    ),
    (
        "closed-short-static-unused",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}type P<'a,'b>=<u8 as A<'a,'b>>::Out;type F=for<'a>fn(P<'static,'a>);fn main(){}",
        true,
    ),
    (
        "conditional-input-unused",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}type P<'a,'b>=<u8 as A<'a,'b>>::Out;type F<'a>=for<'b>fn(&'b &'a (),P<'b,'a>);fn main(){}",
        true,
    ),
    (
        "conditional-input-used",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}type P<'a,'b>=<u8 as A<'a,'b>>::Out;type F<'a>=for<'b>fn(&'b &'a (),P<'b,'a>);fn f<'a>()->F<'a>{loop{}}fn main(){}",
        false,
    ),
    (
        "conditional-unrelated-missing",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}type P<'a,'b>=<u8 as A<'a,'b>>::Out;type F<'a,'c>=for<'b>fn(&'b &'a (),P<'b,'c>);fn f<'a,'c>()->F<'a,'c>{loop{}}fn main(){}",
        false,
    ),
    (
        "nested-quantified-unused",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}type P<'a,'b>=<u8 as A<'a,'b>>::Out;type F<'a>=for<'b>fn(for<'c>fn(P<'b,'c>),&'a u8);fn main(){}",
        true,
    ),
    (
        "nested-quantified-used",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}type P<'a,'b>=<u8 as A<'a,'b>>::Out;type F<'a>=for<'b>fn(for<'c>fn(P<'b,'c>),&'a u8);fn f<'a>()->F<'a>{loop{}}fn main(){}",
        false,
    ),
    (
        "dependent-direct-unused",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}type P<'a,'b>=<u8 as A<'a,'b>>::Out;type F<'a>=for<'b>fn(<u8 as A<'b,'a>>::Out);fn main(){}",
        true,
    ),
    (
        "local-static",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}type P<'a,'b>=<u8 as A<'a,'b>>::Out;type F<'a>=for<'b>fn(P<'b,'a>);fn take(x:&u8){}fn main(){let _:F<'static>=take;}",
        true,
    ),
    (
        "local-missing",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}type P<'a,'b>=<u8 as A<'a,'b>>::Out;type F<'a>=for<'b>fn(P<'b,'a>);fn take(x:&u8){}fn f<'a>(){let _:F<'a>=take;}fn main(){}",
        false,
    ),
    (
        "local-bound",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}type P<'a,'b>=<u8 as A<'a,'b>>::Out;type F<'a>=for<'b>fn(P<'b,'a>);fn take(x:&u8){}fn f<'a:'static>(){let _:F<'a>=take;}fn main(){}",
        true,
    ),
    (
        "local-inferred",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}type P<'a,'b>=<u8 as A<'a,'b>>::Out;type F<'a>=for<'b>fn(P<'b,'a>);fn take(x:&u8){}fn main(){let _:F<'_>=take;}",
        true,
    ),
    (
        "local-diverging-inferred",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}type P<'a,'b>=<u8 as A<'a,'b>>::Out;type F<'a>=for<'b>fn(P<'b,'a>);fn main(){let _:F<'_>=loop{};}",
        true,
    ),
    (
        "scope-sharing-missing",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}type P<'a,'b>=<u8 as A<'a,'b>>::Out;type F<'a>=for<'b>fn(P<'b,'a>);fn f<'a:'static,'b>(x:F<'a>,y:F<'b>){}fn main(){}",
        false,
    ),
    (
        "scope-sharing-valid",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}type P<'a,'b>=<u8 as A<'a,'b>>::Out;type F<'a>=for<'b>fn(P<'b,'a>);fn f<'a:'static,'b:'static>(x:F<'a>,y:F<'b>){}fn main(){}",
        true,
    ),
    (
        "default-unused",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}type P<'a,'b>=<u8 as A<'a,'b>>::Out;type F<'a>=for<'b>fn(P<'b,'a>);type D<'a,T=F<'a>>=T;fn main(){}",
        true,
    ),
    (
        "default-overridden",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}type P<'a,'b>=<u8 as A<'a,'b>>::Out;type F<'a>=for<'b>fn(P<'b,'a>);type D<'a,T=F<'a>>=T;fn f<'a>()->D<'a,u8>{7}fn main(){}",
        true,
    ),
    (
        "default-static",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}type P<'a,'b>=<u8 as A<'a,'b>>::Out;type F<'a>=for<'b>fn(P<'b,'a>);type D<'a,T=F<'a>>=T;fn f()->D<'static>{loop{}}fn main(){}",
        true,
    ),
    (
        "default-missing",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}type P<'a,'b>=<u8 as A<'a,'b>>::Out;type F<'a>=for<'b>fn(P<'b,'a>);type D<'a,T=F<'a>>=T;fn f<'a>()->D<'a>{loop{}}fn main(){}",
        false,
    ),
    (
        "nested-capture-static",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}type P<'a,'b>=<u8 as A<'a,'b>>::Out;type F<'a>=for<'b>fn(P<'b,'a>);type G<'a>=for<'b>fn(F<'a>,&'b u8);fn f()->G<'static>{loop{}}fn main(){}",
        true,
    ),
    (
        "nested-capture-missing",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}type P<'a,'b>=<u8 as A<'a,'b>>::Out;type F<'a>=for<'b>fn(P<'b,'a>);type G<'a>=for<'b>fn(F<'a>,&'b u8);fn f<'a>()->G<'a>{loop{}}fn main(){}",
        false,
    ),
    (
        "nested-local-unused",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}type P<'a,'b>=<u8 as A<'a,'b>>::Out;type F<'a>=for<'b>fn(P<'b,'a>);type G<'a>=for<'b>fn(F<'b>,&'a u8);fn main(){}",
        true,
    ),
    (
        "nested-local-missing",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}type P<'a,'b>=<u8 as A<'a,'b>>::Out;type F<'a>=for<'b>fn(P<'b,'a>);type G<'a>=for<'b>fn(F<'b>,&'a u8);fn f<'a>()->G<'a>{loop{}}fn main(){}",
        false,
    ),
    (
        "erased-static",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}type P<'a,'b>=<u8 as A<'a,'b>>::Out;type F<'a>=for<'b>fn(P<'b,'a>);trait E{type Out;}impl<T>E for T{type Out=u8;}type G<'a>=<F<'a>as E>::Out;fn f()->G<'static>{7}fn main(){}",
        true,
    ),
    (
        "erased-missing",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}type P<'a,'b>=<u8 as A<'a,'b>>::Out;type F<'a>=for<'b>fn(P<'b,'a>);trait E{type Out;}impl<T>E for T{type Out=u8;}type G<'a>=<F<'a>as E>::Out;fn f<'a>()->G<'a>{7}fn main(){}",
        false,
    ),
    (
        "zero-array-missing",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}type P<'a,'b>=<u8 as A<'a,'b>>::Out;type F<'a>=for<'b>fn(P<'b,'a>);type G<'a>=[F<'a>;0];fn f<'a>()->G<'a>{[]}fn main(){}",
        false,
    ),
    (
        "type-bound-unused",
        "trait A<'a>{type Out;}impl<'a,T:'a>A<'a>for T{type Out=&'a T;}type F<T>=for<'a>fn(<T as A<'a>>::Out);fn main(){}",
        true,
    ),
    (
        "type-bound",
        "trait A<'a>{type Out;}impl<'a,T:'a>A<'a>for T{type Out=&'a T;}type F<T>=for<'a>fn(<T as A<'a>>::Out);fn f<T:'static>()->F<T>{loop{}}fn main(){}",
        true,
    ),
    (
        "type-bound-missing",
        "trait A<'a>{type Out;}impl<'a,T:'a>A<'a>for T{type Out=&'a T;}type F<T>=for<'a>fn(<T as A<'a>>::Out);fn f<T>()->F<T>{loop{}}fn main(){}",
        false,
    ),
    (
        "type-bound-local",
        "trait A<'a>{type Out;}impl<'a,T:'a>A<'a>for T{type Out=&'a T;}type F<T>=for<'a>fn(<T as A<'a>>::Out);fn f<T:'static>(){let _:F<T>=loop{};}fn main(){}",
        true,
    ),
    (
        "type-bound-local-missing",
        "trait A<'a>{type Out;}impl<'a,T:'a>A<'a>for T{type Out=&'a T;}type F<T>=for<'a>fn(<T as A<'a>>::Out);fn f<T>(){let _:F<T>=loop{};}fn main(){}",
        false,
    ),
    (
        "direct-default-unused",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}type P<'a,'b>=<u8 as A<'a,'b>>::Out;type D<'a,T=for<'b>fn(P<'b,'a>)>=T;fn main(){}",
        true,
    ),
    (
        "direct-default-overridden",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}type P<'a,'b>=<u8 as A<'a,'b>>::Out;type D<'a,T=for<'b>fn(P<'b,'a>)>=T;fn f<'a>()->D<'a,u8>{7}fn main(){}",
        true,
    ),
    (
        "direct-default-static",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}type P<'a,'b>=<u8 as A<'a,'b>>::Out;type D<'a,T=for<'b>fn(P<'b,'a>)>=T;fn f()->D<'static>{loop{}}fn main(){}",
        true,
    ),
    (
        "direct-default-missing",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}type P<'a,'b>=<u8 as A<'a,'b>>::Out;type D<'a,T=for<'b>fn(P<'b,'a>)>=T;fn f<'a>()->D<'a>{loop{}}fn main(){}",
        false,
    ),
    (
        "associated-definition-valid",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}type P<'a,'b>=<u8 as A<'a,'b>>::Out;type F<'a>=for<'b>fn(P<'b,'a>);trait B{type Out;}impl<'a:'static>B for &'a (){type Out=F<'a>;}fn main(){}",
        true,
    ),
    (
        "associated-definition-missing",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}type P<'a,'b>=<u8 as A<'a,'b>>::Out;type F<'a>=for<'b>fn(P<'b,'a>);trait B{type Out;}impl<'a>B for &'a (){type Out=F<'a>;}fn main(){}",
        false,
    ),
    (
        "late-slot-static",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}type P<'a,'b>=<u8 as A<'a,'b>>::Out;type F<'a>=for<'b,'c>fn(P<'b,'a>,P<'c,'c>);fn f()->F<'static>{loop{}}fn main(){}",
        true,
    ),
    (
        "late-slot-missing",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}type P<'a,'b>=<u8 as A<'a,'b>>::Out;type F<'a>=for<'b,'c>fn(P<'b,'a>,P<'c,'c>);fn f<'a>()->F<'a>{loop{}}fn main(){}",
        false,
    ),
    (
        "empty-parent-static",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}type P<'a,'b>=<u8 as A<'a,'b>>::Out;type F<'a>=fn(for<'b>fn(P<'b,'a>));fn f()->F<'static>{loop{}}fn main(){}",
        true,
    ),
    (
        "empty-parent-missing",
        "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}type P<'a,'b>=<u8 as A<'a,'b>>::Out;type F<'a>=fn(for<'b>fn(P<'b,'a>));fn f<'a>()->F<'a>{loop{}}fn main(){}",
        false,
    ),
];
#[test]
fn independently_qualified_quantified_aliases() -> Result<(), Box<dyn std::error::Error>> {
    let mut failures = Vec::new();
    for &(name, source, accepted) in CASES {
        rustc::qualify(&format!("quantified-alias-{name}"), source, accepted)?;
        let result = semantic::analyze(source).and_then(|(mut report, mut facts)| {
            let entry =
                rustgm::sema::entry::select(&mut report, &facts).map_err(|e| e.to_string())?;
            rustgm::lower::instances::build(&report, &mut facts, entry)
                .map(|_| ())
                .map_err(|e| e.to_string())
        });
        if result.is_ok() != accepted {
            failures.push(format!("{name}: {:?}", result.err()));
        } else if let Err(error) = result
            && !error.contains("required lifetime does not outlive its target")
        {
            failures.push(format!(
                "{name}: rejected before its lifetime proof: {error}"
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn quantified_aliases_execute_after_template_and_instance_proofs()
-> Result<(), Box<dyn std::error::Error>> {
    native::check_cases(
        "quantified-aliases",
        &[
            (
                "private-witness",
                "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}type P<'a,'b>=<u8 as A<'a,'b>>::Out;type F=for<'a>fn(P<'a,'a>)->u8;fn read(x:&u8)->u8{*x}fn main(){let f:F=read;check(f(&74)==74)}",
                false,
            ),
            (
                "generic-copy",
                "trait A<'a>{type Out;}impl<'a,T:'a>A<'a>for T{type Out=&'a T;}type F<T>=for<'a>fn(<T as A<'a>>::Out)->T;fn copy<T:Copy>(x:&T)->T{*x}fn run<T:Copy+'static>(x:T)->T{let f:F<T>=copy::<T>;f(&x)}fn main(){check(run(73u8)==73);let x:&'static u8=&74;check(*run(x)==74)}",
                false,
            ),
            (
                "erased-result",
                "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}type P<'a,'b>=<u8 as A<'a,'b>>::Out;type F<'a>=for<'b>fn(P<'b,'a>);trait E{type Out;}impl<T>E for T{type Out=u8;}type G<'a>=<F<'a>as E>::Out;fn deliver<'a:'static>()->G<'a>{73}fn main(){check(deliver::<'static>()==73)}",
                false,
            ),
            (
                "trait-dispatch",
                "trait A<'a,'b>{type Out;}impl<'a,'b:'a>A<'a,'b>for u8{type Out=&'b u8;}type P<'a,'b>=<u8 as A<'a,'b>>::Out;type F<'a>=for<'b>fn(P<'b,'a>)->u8;trait V{fn get(&self)->u8;}impl V for u8{fn get(&self)->u8{*self}}fn read(x:&u8)->u8{*x}fn run<T:V>(x:T)->u8{let f:F<'static>=read;f(&73)+x.get()}fn main(){check(run(7u8)==80)}",
                false,
            ),
        ],
    )
}
