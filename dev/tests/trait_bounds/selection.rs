//! Validated impl evidence satisfies body goals under the actual caller environment.
use super::{cases, semantic};

#[test]
fn implementation_selection_1() -> Result<(), Box<dyn std::error::Error>> {
    cases(&[
        (
            "impl-selection-concrete",
            "trait A{}impl A for u8{}fn f<T:A>(x:T){}fn main(){f(7u8);}",
            true,
        ),
        (
            "impl-selection-unknown-self",
            "trait A{}impl A for u8{}fn f<T:A>(){}fn main(){f::<_>();}",
            false,
        ),
        (
            "impl-selection-infer-trait-argument",
            "trait A<U>{}impl A<u16> for u8{}fn f<T:A<U>,U>(x:T){}fn main(){f::<_,_>(7u8);}",
            true,
        ),
        (
            "impl-selection-ambiguous-trait-argument",
            "trait A<U>{}impl A<u16> for u8{}impl A<u32> for u8{}fn f<T:A<U>,U>(x:T){}fn main(){f::<_,_>(7u8);}",
            false,
        ),
        (
            "impl-selection-infer-nominal-argument",
            "trait A<U>{}struct S<T>(T);impl<T>A<T>for S<T>{}fn f<T:A<U>,U>(x:T){}fn main(){f::<_,_>(S(7u8));}",
            true,
        ),
    ])
}

#[test]
fn implementation_selection_2() -> Result<(), Box<dyn std::error::Error>> {
    cases(&[
        (
            "impl-selection-blanket-copy",
            "trait A{}impl<T:Copy>A for T{}fn f<T:A>(x:T){}fn main(){f(7u8);}",
            true,
        ),
        (
            "impl-selection-blanket-copy-missing",
            "trait A{}impl<T:Copy>A for T{}fn f<T:A>(x:T){}fn main(){let mut x=7u8;f(&mut x);}",
            false,
        ),
        (
            "impl-selection-nominal-conditional",
            "trait A{}struct S<T>(T);impl<T:Copy>A for S<T>{}fn f<T:A>(x:T){}fn main(){f(S(7u8));}",
            true,
        ),
        (
            "impl-selection-nominal-conditional-missing",
            "trait A{}struct S<T>(T);impl<T:Copy>A for S<T>{}fn f<T:A>(x:T){}fn main(){let mut x=7u8;f(S(&mut x));}",
            false,
        ),
        (
            "impl-selection-static-application",
            "trait A<'a>{}impl<'a>A<'a>for &'a u8{}fn f<T:A<'static>>(x:T){}static X:u8=7;fn main(){f(&X);}",
            true,
        ),
    ])
}

#[test]
fn implementation_selection_3() -> Result<(), Box<dyn std::error::Error>> {
    cases(&[
        (
            "impl-selection-static-application-missing",
            "trait A<'a>{}impl<'a>A<'a>for &'a u8{}fn f<T:A<'static>>(x:T){}fn main(){let x=7u8;f(&x);}",
            false,
        ),
        (
            "impl-selection-conditional-outlives",
            "trait A<'a>{}impl<'a,T:'a>A<'a>for T{}fn g<'b,T:A<'b>>(x:T,y:&'b()){}fn f<'a:'b,'b>(x:&'a u8,y:&'b()){g::<'b,&'a u8>(x,y)}fn main(){}",
            true,
        ),
        (
            "impl-selection-conditional-outlives-missing",
            "trait A<'a>{}impl<'a,T:'a>A<'a>for T{}fn g<'b,T:A<'b>>(x:T,y:&'b()){}fn f<'a,'b>(x:&'a u8,y:&'b()){g::<'b,&'a u8>(x,y)}fn main(){}",
            false,
        ),
        (
            "impl-selection-recursive-goal-base",
            "trait A{}struct S<T>(T);impl<T:A>A for S<T>{}impl A for u8{}fn f<T:A>(x:T){}fn main(){f(S(S(7u8)));}",
            true,
        ),
        (
            "impl-selection-recursive-goal-no-base",
            "trait A{}struct S<T>(T);impl<T:A>A for S<T>{}fn f<T:A>(x:T){}fn main(){f(S(S(7u8)));}",
            false,
        ),
    ])
}

#[test]
fn implementation_selection_4() -> Result<(), Box<dyn std::error::Error>> {
    cases(&[
        (
            "impl-selection-vacuous-recursive-header",
            "trait A{}impl<T:A>A for T{}fn main(){}",
            true,
        ),
        (
            "impl-selection-cyclic-goal-is-not-proof",
            "trait A{}impl<T:A>A for T{}fn f<T:A>(x:T){}fn main(){f(7u8);}",
            false,
        ),
        (
            "impl-selection-unused-impl-lifetime",
            "trait A{}impl<'unused>A for u8{}fn f<T:A>(x:T){}fn main(){f(7u8);}",
            true,
        ),
        (
            "impl-selection-existential-static-lifetime",
            "trait A{}impl<'unused:'static>A for u8{}fn f<T:A>(x:T){}fn main(){f(7u8);}",
            true,
        ),
        (
            "impl-selection-infer-const-trait-argument",
            "trait A<const N:usize>{}impl<const N:usize>A<N>for [u8;N]{}fn f<T:A<N>,const N:usize>(x:T){}fn main(){f::<_,_>([0u8;3]);}",
            true,
        ),
    ])
}

#[test]
fn selected_impl_arguments_remain_typed_and_belong_to_validated_owners()
-> Result<(), Box<dyn std::error::Error>> {
    use rustgm::types::{Primitive, TypeId, arguments::Argument};
    let source = "trait A<U>{}struct S<T>(T);impl<T:Copy>A<T>for S<T>{}fn f<T:A<U>,U>(x:T){}fn main(){f::<_,_>(S(7u8));f::<_,_>(S(9u16));}";
    super::rustc::qualify("impl-selection-retained-arguments", source, true)?;
    let (_, facts) = semantic::analyze(source)?;
    let selections = facts
        .generics
        .trait_selections
        .values()
        .flatten()
        .collect::<Vec<_>>();
    assert_eq!(selections.len(), 2);
    assert_eq!(selections[0].owner, selections[1].owner);
    assert!(
        facts
            .generics
            .implementations()
            .any(|header| header.owner == selections[0].owner)
    );
    let mut types = Vec::new();
    for selection in selections {
        let arguments = facts
            .types
            .arguments(selection.arguments)
            .ok_or("missing selected impl arguments")?
            .collect::<Vec<_>>();
        assert_eq!(arguments.len(), 1);
        let Argument::Type(ty) = arguments[0] else {
            return Err("expected selected type parameter".into());
        };
        types.push(ty);
    }
    types.sort_unstable_by_key(|ty| ty.0);
    let mut expected = [
        TypeId::primitive(Primitive::U8),
        TypeId::primitive(Primitive::U16),
    ];
    expected.sort_unstable_by_key(|ty| ty.0);
    assert_eq!(types, expected);
    Ok(())
}

#[test]
fn implementation_selection_extended_1() -> Result<(), Box<dyn std::error::Error>> {
    cases(&[
        (
            "impl-selection-caller-copy",
            "trait A{}impl<T:Copy>A for T{}fn g<T:A>(x:T){}fn f<U:Copy>(x:U){g(x)}fn main(){}",
            true,
        ),
        (
            "impl-selection-caller-missing-copy",
            "trait A{}impl<T:Copy>A for T{}fn g<T:A>(x:T){}fn f<U>(x:U){g(x)}fn main(){}",
            false,
        ),
        (
            "impl-selection-nominal-contract",
            "trait A{}impl A for u8{}struct S<T:A>(T);fn main(){let _=S(7u8);}",
            true,
        ),
        (
            "impl-selection-conditions-disambiguate",
            "#[derive(Clone)]struct N;struct S<T>(T);trait A<U>{}impl<T:Copy>A<u8>for S<T>{}impl<T:Clone>A<u16>for S<T>{}fn g<T:A<U>,U>(x:T){}fn main(){g::<_,_>(S(N));}",
            true,
        ),
        (
            "impl-selection-conditions-still-ambiguous",
            "#[derive(Clone,Copy)]struct N;struct S<T>(T);trait A<U>{}impl<T:Copy>A<u8>for S<T>{}impl<T:Clone>A<u16>for S<T>{}fn g<T:A<U>,U>(x:T){}fn main(){g::<_,_>(S(N));}",
            false,
        ),
    ])
}

#[test]
fn implementation_selection_extended_2() -> Result<(), Box<dyn std::error::Error>> {
    cases(&[
        (
            "impl-selection-callable-inference",
            "trait A{}impl<T>A for fn(T)->T{}fn g<T:A>(x:T){}fn f(x:fn(u8)->u8){g(x)}fn main(){}",
            true,
        ),
        (
            "impl-selection-callable-no-capture",
            "trait A{}impl<T>A for fn(T)->T{}fn g<T:A>(x:T){}fn f(x:for<'a>fn(&'a u8)->&'a u8){g(x)}fn main(){}",
            false,
        ),
        (
            "impl-selection-callable-outer",
            "trait A{}impl<T>A for fn(T){}fn g<T:A>(x:T){}fn f<'a>(x:fn(&'a u8)){g(x)}fn main(){}",
            true,
        ),
        (
            "impl-selection-callable-alpha",
            "trait A{}impl A for for<'a>fn(&'a u8)->&'a u8{}fn g<T:A>(x:T){}fn f(x:for<'b>fn(&'b u8)->&'b u8){g(x)}fn main(){}",
            true,
        ),
        (
            "impl-selection-callable-correlation",
            "trait A{}impl A for for<'a>fn(&'a u8)->&'a u8{}fn g<T:A>(x:T){}fn f(x:for<'a,'b>fn(&'a u8)->&'b u8){g(x)}fn main(){}",
            false,
        ),
    ])
}

#[test]
fn implementation_selection_extended_3() -> Result<(), Box<dyn std::error::Error>> {
    cases(&[
        (
            "impl-selection-recursive-lifetimes",
            "trait A<'a>{}struct S<T>(T);impl<'a,T:A<'a>>A<'a>for S<T>{}impl<'a>A<'a>for &'a u8{}fn g<T:A<'static>>(x:T){}static X:u8=7;fn main(){g(S(S(&X)));}",
            true,
        ),
        (
            "impl-selection-recursive-lifetimes-missing",
            "trait A<'a>{}struct S<T>(T);impl<'a,T:A<'a>>A<'a>for S<T>{}impl<'a>A<'a>for &'a u8{}fn g<T:A<'static>>(x:T){}fn main(){let x=7u8;g(S(S(&x)));}",
            false,
        ),
        (
            "impl-selection-recursive-siblings",
            "trait A<'a>{}struct S<T,U>(T,U);impl<'a,T:A<'a>,U:A<'a>>A<'a>for S<T,U>{}impl<'a>A<'a>for &'a u8{}fn g<'a,T:A<'a>>(x:T,y:&'a()){}fn f<'a>(x:&'a u8,y:&'a u8,z:&'a()){g::<'a,_>(S(x,y),z)}fn main(){}",
            true,
        ),
        (
            "impl-selection-numeric-fallback",
            "trait A{}impl A for i32{}fn g<T:A>(x:T){}fn main(){g(7);}",
            true,
        ),
        (
            "impl-selection-inherent-condition",
            "trait A{}impl A for u8{}struct S<T>(T);impl<T:A>S<T>{fn get(self)->T{self.0}}fn main(){let _=S(7u8).get();}",
            true,
        ),
    ])
}

#[test]
fn implementation_selection_extended_4() -> Result<(), Box<dyn std::error::Error>> {
    cases(&[
        (
            "impl-selection-default-proof-no-permission",
            "trait A{}impl<T:A>A for T{}fn g<T:A>(x:T){}fn f<U>(x:U){g(x)}fn main(){}",
            false,
        ),
        (
            "impl-selection-unique-u8",
            "trait A{}impl A for u8{}fn g<T:A>(x:T){}fn main(){g(7);}",
            true,
        ),
        (
            "impl-selection-unique-f32",
            "trait A{}impl A for f32{}fn g<T:A>(x:T){}fn main(){g(7.0);}",
            true,
        ),
        (
            "impl-selection-integer-fallback",
            "trait A{}impl A for u8{}impl A for i32{}fn g<T:A>(x:T){}fn main(){g(7);}",
            true,
        ),
        (
            "impl-selection-integer-missing-default",
            "trait A{}impl A for u8{}impl A for u16{}fn g<T:A>(x:T){}fn main(){g(7);}",
            false,
        ),
    ])
}

#[test]
fn implementation_selection_extended_5() -> Result<(), Box<dyn std::error::Error>> {
    cases(&[
        (
            "impl-selection-float-fallback",
            "trait A{}impl A for f32{}impl A for f64{}fn g<T:A>(x:T){}fn main(){g(7.0);}",
            true,
        ),
        (
            "impl-selection-numeric-kind",
            "trait A{}impl A for u8{}impl A for f32{}fn g<T:A>(x:T){}fn main(){g(7);g(7.0);}",
            true,
        ),
        (
            "impl-selection-numeric-arguments",
            "trait A<U>{}impl A<u8>for u16{}fn g<T:A<U>,U>(x:T,y:U){}fn main(){g(7,9);}",
            true,
        ),
        (
            "impl-selection-blanket-fallback",
            "trait A{}impl<T:Copy>A for T{}fn g<T:A>(x:T){}fn main(){g(7);g(7.0);}",
            true,
        ),
    ])
}

const EXECUTIONS: &[(&str, &str, bool)] = &[
    (
        "conditional",
        "trait A{}impl<T:Copy>A for T{}fn id<T:A>(x:T)->T{x}fn main(){check(id(7u8)==7);check(id(9.0f32)==9.0);}",
        false,
    ),
    (
        "recursive",
        "trait A{}struct S<T>(T);impl A for u8{}impl<T:A>A for S<T>{}fn id<T:A>(x:T)->T{x}fn main(){let S(S(x))=id(S(S(9u8)));check(x==9);}",
        false,
    ),
    (
        "lifetimes",
        "trait A<'a>{}impl<'a>A<'a>for &'a u8{}fn id<'a,T:A<'a>>(x:T,y:&'a())->T{x}static X:u8=11;fn main(){let value=id(&X,&());check(*value==11);}",
        false,
    ),
    (
        "const-argument",
        "trait A<const N:usize>{}impl<const N:usize>A<N>for[u8;N]{}fn size<T:A<N>,const N:usize>(x:T)->usize{N}fn main(){check(size::<_,_>([7u8;3])==3);}",
        false,
    ),
];

#[test]
fn selected_impl_programs_lower_with_their_declared_arguments()
-> Result<(), Box<dyn std::error::Error>> {
    for &(name, source, _) in EXECUTIONS {
        cases(&[(name, &format!("{}{source}", super::native::CHECK), true)])?;
    }
    Ok(())
}

#[test]
#[ignore = "requires the qualified CPPGM LowIR-to-native backend"]
fn selected_impl_programs_execute_against_rustc() -> Result<(), Box<dyn std::error::Error>> {
    super::native::check_cases("trait-implementation-selection", EXECUTIONS)
}

#[test]
fn recursive_predicate_diamonds_share_completed_proofs() -> Result<(), Box<dyn std::error::Error>> {
    for depth in [4, 16, 32] {
        let value = format!("{}7u8{}", "S(".repeat(depth), ")".repeat(depth));
        let source = format!(
            "trait A{{}}trait B{{}}struct S<T>(T);impl A for u8{{}}impl B for u8{{}}impl<T:A+B>A for S<T>{{}}impl<T:A+B>B for S<T>{{}}fn g<T:A>(x:T){{}}fn main(){{g({value});}}"
        );
        cases(&[(&format!("impl-selection-diamond-{depth}"), &source, true)])?;
    }
    Ok(())
}

#[test]
fn failed_candidates_cannot_publish_lifetime_variables() -> Result<(), Box<dyn std::error::Error>> {
    use rustgm::types::arguments::Argument;
    for (copy_parameters, clone_parameters, expected_regions) in [
        ("'unused:'static,T:Copy", "T:Clone", 0),
        ("T:Copy", "'unused:'static,T:Clone", 2),
    ] {
        let source = format!(
            "#[derive(Clone)]struct N;struct S<T>(T);trait A<U>{{}}impl<{copy_parameters}>A<u8>for S<T>{{}}impl<{clone_parameters}>A<u16>for S<T>{{}}fn g<T:A<U>,U>(x:T){{}}fn main(){{g::<_,_>(S(N));g::<_,_>(S(N));}}"
        );
        super::rustc::qualify(
            &format!("impl-selection-lifetime-publication-{expected_regions}"),
            &source,
            true,
        )?;
        super::prove(&source)?;
        let (_, facts) = semantic::analyze(&source)?;
        assert_eq!(facts.body_regions.log().len(), expected_regions);
        let mut regions = Vec::new();
        for selection in facts.generics.trait_selections.values().flatten() {
            for argument in facts
                .types
                .arguments(selection.arguments)
                .ok_or("missing selected arguments")?
            {
                if let Argument::Lifetime(region) = argument {
                    regions.push(region.0);
                }
            }
        }
        regions.sort_unstable();
        assert_eq!(
            regions,
            (1..=u32::try_from(expected_regions)?).collect::<Vec<_>>()
        );
    }
    Ok(())
}
