//! Inferred projection parents retain their premises until later output equations settle.
#[path = "harness/native_references.rs"]
mod native;
#[path = "helpers/qualify.rs"]
mod rustc;
#[path = "support/semantic.rs"]
mod semantic;
const CASES: &[(&str, &str, bool)] = &[
    (
        "known-subject",
        "struct W<T>(T);trait Q<P>{type Out;}impl<T:Copy>Q<u16>for W<T>{type Out=T;}fn main(){let _: <W<u8>as Q<_>>::Out=7u8;}",
        true,
    ),
    (
        "parent-default",
        "struct W<T>(T);trait Q<P=u16>{type Out;}impl<T:Copy>Q for W<T>{type Out=T;}fn main(){let _: <W<_>as Q>::Out=7u8;}",
        true,
    ),
    (
        "dependent-default",
        "struct W<T>(T);trait Q<P,R=P>{type Out;}impl<T:Copy>Q<u16>for W<T>{type Out=T;}fn main(){let _: <W<_>as Q<_>>::Out=7u8;}",
        true,
    ),
    (
        "self-default",
        "struct W<T>(T);trait Q<P=Self>:Sized{type Out;}impl<T:Copy>Q for W<T>{type Out=T;}fn main(){let _: <W<_>as Q>::Out=7u8;}",
        true,
    ),
    (
        "const-parent",
        "struct W<T>(T);trait Q<const N:usize>{type Out;}impl<T:Copy>Q<3>for W<T>{type Out=T;}fn main(){let _: <W<_>as Q<_>>::Out=7u8;}",
        true,
    ),
    (
        "direct-universal",
        "struct W<T>(T);trait Q<'a,P>{type Out;}impl<'a,T:Copy>Q<'a,u16>for W<T>{type Out=T;}fn f(_: &u8)->u8{7}fn main(){let _:for<'a>fn(&'a u8)-><W<_>as Q<'a,_>>::Out=f;}",
        true,
    ),
    (
        "direct-static",
        "struct W<T>(T);trait Q<'a,P>{type Out;}impl<T:Copy>Q<'static,u16>for W<T>{type Out=T;}fn f(_: &u8)->u8{7}fn main(){let _:for<'a>fn(&'a u8)-><W<_>as Q<'a,_>>::Out=f;}",
        false,
    ),
    (
        "direct-nested-binders",
        "struct W<T>(T);trait Q<'a,'b,P>{type Out;}impl<'a,'b,T:Copy>Q<'a,'b,u16>for W<T>{type Out=T;}fn f<'a>(_:for<'b>fn(&'a u8,&'b u8)->u8){}fn main(){let _:for<'a>fn(for<'b>fn(&'a u8,&'b u8)-><W<_>as Q<'a,'b,_>>::Out)=f;}",
        true,
    ),
    (
        "copy-parent",
        "struct W<T>(T);trait Q<P>{type Out;}impl<T:Copy>Q<u16>for W<T>{type Out=T;}fn main(){let _: <W<_>as Q<_>>::Out=7u8;}",
        true,
    ),
    (
        "noncopy-parent",
        "struct W<T>(T);trait Q<P>{type Out;}impl<T:Copy>Q<u16>for W<T>{type Out=T;}fn main(){let mut x=0u8;let _: <W<_>as Q<_>>::Out=&mut x;}",
        false,
    ),
    (
        "trait-parent",
        "struct W<T>(T);trait Mark{}impl Mark for u8{}trait Q<P>{type Out;}impl<T:Mark>Q<u16>for W<T>{type Out=T;}fn main(){let _: <W<_>as Q<_>>::Out=7u8;}",
        true,
    ),
    (
        "missing-trait-parent",
        "struct W<T>(T);trait Mark{}impl Mark for u8{}trait Q<P>{type Out;}impl<T:Mark>Q<u16>for W<T>{type Out=T;}fn main(){let _: <W<_>as Q<_>>::Out=7u32;}",
        false,
    ),
    (
        "ambiguous-parent",
        "struct W<T>(T);trait Q<P>{type Out;}impl<T:Copy>Q<u8>for W<T>{type Out=T;}impl<T:Copy>Q<u16>for W<T>{type Out=T;}fn main(){let _: <W<_>as Q<_>>::Out=7u8;}",
        false,
    ),
    (
        "erased-unresolved",
        "struct W<T>(T);trait Q<P>{type Out;}impl<T:Copy>Q<u16>for W<T>{type Out=u8;}fn main(){let _: <W<_>as Q<_>>::Out=7u8;}",
        false,
    ),
    (
        "callable-parent",
        "struct W<T>(T);trait Q<P>{type Out;}impl<T:Copy>Q<u16>for W<T>{type Out=T;}type F<T,P>=for<'a>fn(&'a u8)-><W<T>as Q<P>>::Out;fn f(_: &u8)->u8{7}fn main(){let _:F<_,_>=f;}",
        true,
    ),
    (
        "universal-parent",
        "struct W<T>(T);trait Q<'a,P>{type Out;}impl<'a,T:Copy>Q<'a,u16>for W<T>{type Out=T;}type F<T,P>=for<'a>fn(&'a u8)-><W<T>as Q<'a,P>>::Out;fn f(_: &u8)->u8{7}fn main(){let _:F<_,_>=f;}",
        true,
    ),
    (
        "static-parent",
        "struct W<T>(T);trait Q<'a,P>{type Out;}impl<T:Copy>Q<'static,u16>for W<T>{type Out=T;}type F<T,P>=for<'a>fn(&'a u8)-><W<T>as Q<'a,P>>::Out;fn f(_: &u8)->u8{7}fn main(){let _:F<_,_>=f;}",
        false,
    ),
    (
        "nested-binders",
        "struct W<T>(T);trait Q<'a,'b,P>{type Out;}impl<'a,'b,T:Copy>Q<'a,'b,u16>for W<T>{type Out=T;}type F<T,P>=for<'a>fn(for<'b>fn(&'a u8,&'b u8)-><W<T>as Q<'a,'b,P>>::Out);fn f<'a>(_:for<'b>fn(&'a u8,&'b u8)->u8){}fn main(){let _:F<_,_>=f;}",
        true,
    ),
    (
        "nested-correlations",
        "struct W<T>(T);trait Q<'a,'b,P>{type Out;}impl<'a,T:Copy>Q<'a,'a,u16>for W<T>{type Out=T;}type F<T,P>=for<'a>fn(for<'b>fn(&'a u8,&'b u8)-><W<T>as Q<'a,'b,P>>::Out);fn f<'a>(_:for<'b>fn(&'a u8,&'b u8)->u8){}fn main(){let _:F<_,_>=f;}",
        false,
    ),
    (
        "two-refinements",
        "struct W<T>(T);trait Mark<P>{}impl Mark<u32>for u8{}trait Q<P,R>{type Out;}impl<T:Mark<R>,R>Q<u16,R>for W<T>{type Out=T;}fn main(){let _: <W<_>as Q<_,_>>::Out=7u8;}",
        true,
    ),
];
#[test]
fn source_expectations_match_pinned_rustc() -> Result<(), Box<dyn std::error::Error>> {
    for &(name, source, accepted) in CASES {
        rustc::qualify(&format!("parent-proof-{name}"), source, accepted)?;
    }
    Ok(())
}
#[test]
fn partial_parents_complete_before_projection_results_are_accepted() {
    let mut failures = Vec::new();
    for &(name, source, accepted) in CASES {
        let result = semantic::analyze(source).and_then(|(mut report, mut facts)| {
            let entry =
                rustgm::sema::entry::select(&mut report, &facts).map_err(|e| e.to_string())?;
            rustgm::lower::instances::build(&report, &mut facts, entry)
                .map(|_| ())
                .map_err(|e| e.to_string())
        });
        eprintln!("parent-proof {name} expected={accepted} actual={result:?}");
        if result.is_ok() != accepted {
            failures.push(format!("{name}: {:?}", result.err()));
        } else if let Err(message) = result {
            let required = match name {
                "noncopy-parent" | "missing-trait-parent" => {
                    "parent requirements are not satisfied"
                }
                "ambiguous-parent" | "erased-unresolved" => "projection inputs remain unresolved",
                "direct-static" | "static-parent" | "nested-correlations" => {
                    "required lifetime does not outlive"
                }
                _ => "unclassified negative control",
            };
            if !message.contains(required) {
                failures.push(format!("{name}: expected {required:?}, got {message:?}"));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn inferred_parents_execute_after_all_conditions_complete() -> Result<(), Box<dyn std::error::Error>>
{
    native::check_cases(
        "projection-parents/native",
        &[
            (
                "copy-sum",
                "struct W<T>(T);trait Q<P>{type Out;}impl<T:Copy>Q<u16>for W<T>{type Out=(T,T);}fn main(){let p:<W<_>as Q<_>>::Out=(7u8,9u8);check(p.0+p.1==16);}",
                false,
            ),
            (
                "two-refinements",
                "struct W<T>(T);trait Mark<P>{}impl Mark<u32>for u8{}trait Q<P,R>{type Out;}impl<T:Mark<R>,R>Q<u16,R>for W<T>{type Out=T;}fn main(){let p:<W<_>as Q<_,_>>::Out=7u8;check(p*6==42);}",
                false,
            ),
            (
                "self-default",
                "struct W<T>(T);trait Q<P=Self>:Sized{type Out;}impl<T:Copy>Q for W<T>{type Out=T;}fn main(){let p:<W<_>as Q>::Out=7u8;check(p+5==12);}",
                false,
            ),
            (
                "const-array",
                "struct W<T,const N:usize>(T);trait Q<P>{type Out;}impl<T:Copy,const N:usize>Q<u16>for W<T,N>{type Out=[T;N];}fn main(){let p:<W<_,_>as Q<_>>::Out=[7u8,9u8,11u8];check(p[0]+p[2]==18);}",
                false,
            ),
            (
                "universal-call",
                "struct W<T>(T);trait Q<'a,P>{type Out;}impl<'a,T:Copy>Q<'a,u16>for W<T>{type Out=T;}fn f(x:&u8)->u8{*x+5}fn main(){let g:for<'a>fn(&'a u8)-><W<_>as Q<'a,_>>::Out=f;check(g(&7)==12);}",
                false,
            ),
            (
                "nested-call",
                "struct W<T>(T);trait Q<'a,'b,P>{type Out;}impl<'a,'b,T:Copy>Q<'a,'b,u16>for W<T>{type Out=T;}static X:u8=7;fn add(x:&u8,y:&u8)->u8{*x+*y}fn f<'a>(g:for<'b>fn(&'a u8,&'b u8)->u8){check(g(&X,&9)==16);}fn main(){let g:for<'a>fn(for<'b>fn(&'a u8,&'b u8)-><W<_>as Q<'a,'b,_>>::Out)=f;g(add);}",
                false,
            ),
        ],
    )
}
