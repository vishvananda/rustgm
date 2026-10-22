//! Projection inputs come from validated headers, never associated result matching.
use crate::native_references as native;
use crate::qualify as rustc;
use crate::semantic;
const CASES: &[(&str, &str, bool)] = &[
    (
        "captured-input-established-by-callable",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=T;}type P<T>=(fn(<T as A>::Out),T);fn take<T>(_:T){}fn f<'a>(x:&'a u8){let _:P<_>=(take::<&'a u8>,x);}fn main(){}",
        true,
    ),
    (
        "numeric-identity-result",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=T;}type P<T>=(T,<T as A>::Out);fn main(){let p:P<_>=(0,7u8);let _:u8=p.0;}",
        true,
    ),
    (
        "numeric-erased-result",
        "trait A{type Out;}impl<T:Copy>A for T{type Out=bool;}type P<T>=(T,<T as A>::Out);fn main(){let p:P<_>=(0,true);let _:i32=p.0;}",
        true,
    ),
    (
        "unknown-erased-input",
        "trait A{type Out;}impl<T>A for T{type Out=u8;}type Q<T>=<T as A>::Out;fn main(){let _:Q<_>=7u8;}",
        false,
    ),
    (
        "structured-identity-result",
        "struct W<T>(T);trait A{type Out;}impl<T:Copy>A for W<T>{type Out=T;}type P<T>=(W<T>,<W<T>as A>::Out);fn main(){let p:P<_>=(W(0),7u8);let _:W<u8>=p.0;}",
        true,
    ),
    (
        "unique-concrete-header",
        "struct W<T>(T);trait A{type Out;}type P<T>=(W<T>,<W<T>as A>::Out);fn g(_:u8){}impl A for W<u8>{type Out=fn(u8);}fn main(){let p:P<_>=(W(0),g);(p.1)(7);}",
        true,
    ),
    (
        "ambiguous-concrete-headers",
        "struct W<T>(T);trait A{type Out;}type P<T>=(W<T>,<W<T>as A>::Out);fn g(_:u8){}impl A for W<u8>{type Out=fn(u8);}impl A for W<u16>{type Out=fn(u8);}fn main(){let p:P<_>=(W(0),g);(p.1)(7);}",
        false,
    ),
    (
        "ambiguous-header-later-input",
        "struct W<T>(T);trait A{type Out;}type P<T>=(W<T>,<W<T>as A>::Out);fn g(_:u8){}impl A for W<u8>{type Out=fn(u8);}impl A for W<u16>{type Out=fn(u8);}fn main(){let p:P<_>=(W(0),g);let _:W<u16>=p.0;(p.1)(7);}",
        false,
    ),
    (
        "explicit-cast-later-input",
        "struct W<T>(T);trait A{type Out;}type P<T>=(W<T>,<W<T>as A>::Out);fn g(_:u8){}impl A for W<u8>{type Out=fn(u8);}impl A for W<u16>{type Out=fn(u8);}fn main(){let p:P<_>=(W(0),g as fn(u8));let _:W<u16>=p.0;(p.1)(7);}",
        true,
    ),
    (
        "output-cannot-select-header",
        "struct W<T>(T);trait A{type Out;}impl A for W<u8>{type Out=u16;}impl A for W<u16>{type Out=u8;}type P<T>=(W<T>,<W<T>as A>::Out);fn main(){let p:P<_>=(W(0),7u8);let _:u8=p.1;}",
        false,
    ),
    (
        "unique-reference-reborrow",
        "struct W<T>(T);trait A<'a>{type Out;}impl<'a>A<'a>for W<u8>{type Out=&'a mut u8;}type P<'a,T>=(W<T>,<W<T>as A<'a>>::Out);fn f(x:&mut u8){let p:P<'_,_>=(W(0),x);*p.1=19;*x=73;}fn main(){}",
        true,
    ),
    (
        "unique-borrow-cannot-escape",
        "struct W<T>(T);trait A<'a>{type Out;}impl<'a>A<'a>for W<u8>{type Out=&'a u8;}type P<'a,T>=(W<T>,<W<T>as A<'a>>::Out);fn f(x:&u8)->&'static u8{let p:P<'_,_>=(W(0),x);p.1}fn main(){}",
        false,
    ),
    (
        "unique-trait-argument",
        "struct W;trait A<T>{type Out;}impl A<u8>for W{type Out=fn(u8);}type P<T>=(T,<W as A<T>>::Out);fn g(_:u8){}fn main(){let p:P<_>=(0,g);(p.1)(7);}",
        true,
    ),
    (
        "unique-const-header",
        "struct W<const N:usize>;trait A{type Out;}impl A for W<3>{type Out=fn(u8);}type P<const N:usize>=(W<N>,<W<N>as A>::Out);fn g(_:u8){}fn main(){let p:P<_>=(W,g);(p.1)(7);}",
        true,
    ),
    (
        "ambient-reference-binder",
        "struct W<T>(T);trait A<'a>{type Out;}impl<'a>A<'a>for W<u8>{type Out=&'a u8;}type F<T>=for<'a>fn(<W<T>as A<'a>>::Out);fn g(_: &u8){}fn main(){let p:F<_>=g;p(&7);}",
        true,
    ),
    (
        "ambient-binder-cannot-be-static",
        "struct W<T>(T);trait A<'a>{type Out;}impl<'a>A<'a>for W<u8>{type Out=&'a u8;}type F<T>=for<'a>fn(<W<T>as A<'a>>::Out);fn g(_: &'static u8){}fn main(){let _:F<_>=g;}",
        false,
    ),
    (
        "shared-input-cannot-disagree",
        "struct W<T>(T);trait A{type Out;}impl A for W<u8>{type Out=fn(u8);}type P<T>=(W<T>,<W<T>as A>::Out);fn g(_:u8){}fn main(){let p:P<_>=(W(0u16),g);(p.1)(7);}",
        false,
    ),
    (
        "unique-conditional-header",
        "struct W<T>(T);struct C;trait B{}impl B for C{}trait A<T>{type Out;}impl<T:B>A<T>for W<u8>{type Out=fn(u8);}type P<T>=(W<T>,<W<T>as A<C>>::Out);fn g(_:u8){}fn main(){let p:P<_>=(W(0),g);(p.1)(7);}",
        true,
    ),
    (
        "unsatisfied-conditional-header",
        "struct W<T>(T);struct C;trait B{}trait A<T>{type Out;}impl<T:B>A<T>for W<u8>{type Out=fn(u8);}type P<T>=(W<T>,<W<T>as A<C>>::Out);fn g(_:u8){}fn main(){let p:P<_>=(W(0),g);(p.1)(7);}",
        false,
    ),
    (
        "cannot-capture-local-slot",
        "struct W<T>(T);trait A<'a>{type Out;}impl<'a>A<'a>for W<&'a u8>{type Out=u8;}type F<T>=for<'a>fn(<W<T>as A<'a>>::Out);fn g(_:u8){}fn main(){let _:F<_>=g;}",
        false,
    ),
    (
        "captured-outer-input",
        "struct W<T>(T);trait A<'a>{type Out;}impl<'a>A<'a>for W<&'a u8>{type Out=&'a u8;}type F<'b,T>=for<'a>fn(<W<T>as A<'b>>::Out,&'a u8);fn g(_: &u8,_: &u8){}fn f<'b>(x:&'b u8){let p:F<'b,_>=g;p(x,&7);}fn main(){}",
        true,
    ),
    (
        "captured-outer-cannot-be-static",
        "struct W<T>(T);trait A<'a>{type Out;}impl<'a>A<'a>for W<&'a u8>{type Out=&'a u8;}type F<'b,T>=for<'a>fn(<W<T>as A<'b>>::Out,&'a u8);fn g(_: &'static u8,_: &u8){}fn f<'b>(x:&'b u8){let p:F<'b,_>=g;p(x,&7);}fn main(){}",
        false,
    ),
    (
        "caller-bound-matches",
        "struct W<T>(T);trait B<'a>{}trait A<'a,U>{type Out;}impl<'a,U:B<'a>>A<'a,U>for W<u8>{type Out=&'a u8;}type P<'a,T,U>=(W<T>,<W<T>as A<'a,U>>::Out);fn f<'a,U:B<'a>>(x:&'a u8){let p:P<'a,_,U>=(W(0),x);let _: &'a u8=p.1;}fn main(){}",
        true,
    ),
    (
        "caller-bound-other-lifetime",
        "struct W<T>(T);trait B<'a>{}trait A<'a,U>{type Out;}impl<'a,U:B<'a>>A<'a,U>for W<u8>{type Out=&'a u8;}type P<'a,T,U>=(W<T>,<W<T>as A<'a,U>>::Out);fn f<'a,'b,U:B<'a>>(x:&'b u8){let p:P<'b,_,U>=(W(0),x);let _: &'b u8=p.1;}fn main(){}",
        false,
    ),
    (
        "caller-bound-equal-lifetimes",
        "struct W<T>(T);trait B<'a>{}trait A<'a,U>{type Out;}impl<'a,U:B<'a>>A<'a,U>for W<u8>{type Out=&'a u8;}type P<'a,T,U>=(W<T>,<W<T>as A<'a,U>>::Out);fn f<'a,'b,U:B<'a>>(x:&'b u8)where 'a:'b,'b:'a{let p:P<'b,_,U>=(W(0),x);let _: &'b u8=p.1;}fn main(){}",
        true,
    ),
];
#[test]
fn partial_projection_selection_matches_rustc() -> Result<(), Box<dyn std::error::Error>> {
    let mut failures = Vec::new();
    for &(name, source, accepted) in CASES {
        rustc::qualify(&format!("projection-selection-{name}"), source, accepted)?;
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
fn selected_projection_results_execute() -> Result<(), Box<dyn std::error::Error>> {
    native::check_cases(
        "partial-projection-selection",
        &[
            (
                "caller-bound",
                "struct W<T>(T);struct C;trait B<'a>{}impl<'a>B<'a>for C{}trait A<'a,U>{type Out;}impl<'a,U:B<'a>>A<'a,U>for W<u8>{type Out=&'a u8;}type P<'a,T,U>=(W<T>,<W<T>as A<'a,U>>::Out);fn f<'a,U:B<'a>>(x:&'a u8)->&'a u8{let p:P<'a,_,U>=(W(3),x);check(p.0.0==3);p.1}fn main(){let x=7;check(*f::<C>(&x)==7);}",
                false,
            ),
            (
                "captured",
                "struct W<T>(T);trait A<'a>{type Out;}impl<'a>A<'a>for W<&'a u8>{type Out=&'a u8;}type F<'b,T>=for<'a>fn(<W<T>as A<'b>>::Out,&'a u8);fn g(x:&u8,y:&u8){check(*x==7);check(*y==9);}fn f<'b>(x:&'b u8){let p:F<'b,_>=g;p(x,&9);}fn main(){f(&7);}",
                false,
            ),
            (
                "call",
                "struct W<T>(T);trait A{type Out;}impl A for W<u8>{type Out=fn(u8)->u8;}type P<T>=(W<T>,<W<T>as A>::Out);fn g(x:u8)->u8{x+17}fn main(){let p:P<_>=(W(3),g);check(p.0.0==3);check((p.1)(7)==24);}",
                false,
            ),
            (
                "borrow",
                "struct W<T>(T);trait A<'a>{type Out;}impl<'a>A<'a>for W<u8>{type Out=&'a mut u8;}type P<'a,T>=(W<T>,<W<T>as A<'a>>::Out);fn f(x:&mut u8){let p:P<'_,_>=(W(3),x);*p.1=19;check(p.0.0==3);*x=73;}fn main(){let mut x=0;f(&mut x);check(x==73);}",
                false,
            ),
            (
                "binder",
                "struct W<T>(T);trait A<'a>{type Out;}impl<'a>A<'a>for W<u8>{type Out=&'a u8;}type F<T>=for<'a>fn(<W<T>as A<'a>>::Out);fn g(x:&u8){check(*x==7);}fn main(){let f:F<_>=g;f(&7);}",
                false,
            ),
            (
                "constant",
                "struct W<const N:usize>;trait A{type Out;}impl A for W<3>{type Out=fn(u8)->u8;}type P<const N:usize>=(W<N>,<W<N>as A>::Out);fn g(x:u8)->u8{x+17}fn main(){let p:P<_>=(W,g);check((p.1)(7)==24);}",
                false,
            ),
        ],
    )
}

#[test]
fn wide_partial_inputs_wait_without_repeating_whole_graph_selection()
-> Result<(), Box<dyn std::error::Error>> {
    for width in [64, 256, 512, 1024] {
        let holes = vec!["_"; width].join(",");
        let values = vec!["0u8"; width].join(",");
        let source = format!(
            "#![recursion_limit=\"8192\"]\ntrait A{{type Out;}}impl<T:Copy>A for T{{type Out=u8;}}type P<T>=(T,<T as A>::Out);fn main(){{let _:P<({holes})>=(({values}),7u8);}}"
        );
        rustc::qualify(&format!("wide-partial-input-{width}"), &source, true)?;
        let (mut report, mut facts) = semantic::analyze(&source)?;
        let entry = rustgm::sema::entry::select(&mut report, &facts)?;
        rustgm::lower::instances::build(&report, &mut facts, entry)?;
    }
    Ok(())
}
