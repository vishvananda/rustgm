//! Projected implementation conditions retain proofs, inference and builtin requirements.
#[path = "harness/native_references.rs"]
mod native;
#[path = "helpers/qualify.rs"]
mod rustc;
#[path = "support/semantic.rs"]
mod semantic;

const CASES: &[(&str, &str, bool)] = &[
    (
        "nested-equality",
        "trait A{type Out;}impl A for u8{type Out=u16;}impl A for u16{type Out=u32;}trait C{type Out;}impl C for u16{type Out=u32;}struct W<T>(T);trait B{}fn require<T:B>(){}impl<T:A>B for W<T>where <T as A>::Out:C<Out=u32>{}fn main(){require::<W<u8>>();}",
        true,
    ),
    (
        "nested-trait",
        "trait A{type Out;}impl A for u8{type Out=u16;}impl A for u16{type Out=u32;}trait C{type Out;}impl C for u16{type Out=u32;}struct W<T>(T);trait B{}fn require<T:B>(){}impl<T:A>B for W<T>where <T as A>::Out:C{}fn main(){require::<W<u8>>();}",
        true,
    ),
    (
        "nested-wrong-result",
        "trait A{type Out;}impl A for u8{type Out=u16;}impl A for u16{type Out=u32;}trait C{type Out;}impl C for u16{type Out=u32;}struct W<T>(T);trait B{}fn require<T:B>(){}impl<T:A>B for W<T>where <T as A>::Out:C<Out=u16>{}fn main(){require::<W<u8>>();}",
        false,
    ),
    (
        "nested-missing-impl",
        "trait A{type Out;}impl A for u8{type Out=u16;}impl A for u16{type Out=u32;}trait C{type Out;}impl C for u16{type Out=u32;}struct W<T>(T);trait B{}fn require<T:B>(){}impl<T:A>B for W<T>where <T as A>::Out:C{}fn main(){require::<W<u16>>();}",
        false,
    ),
    (
        "nested-caller-equality",
        "trait A{type Out;}impl A for u8{type Out=u16;}impl A for u16{type Out=u32;}trait C{type Out;}impl C for u16{type Out=u32;}struct W<T>(T);trait B{}fn require<T:B>(){}impl<T:A>B for W<T>where <T as A>::Out:C<Out=u32>{}fn f<T:A>()where <T as A>::Out:C<Out=u32>{require::<W<T>>();}fn main(){f::<u8>();}",
        true,
    ),
    (
        "nested-caller-missing-equality",
        "trait A{type Out;}impl A for u8{type Out=u16;}impl A for u16{type Out=u32;}trait C{type Out;}impl C for u16{type Out=u32;}struct W<T>(T);trait B{}fn require<T:B>(){}impl<T:A>B for W<T>where <T as A>::Out:C<Out=u32>{}fn f<T:A>()where <T as A>::Out:C{require::<W<T>>();}fn main(){}",
        false,
    ),
    (
        "inherent-projected-bound",
        "trait A{type Out;}impl A for u8{type Out=u16;}impl A for u16{type Out=u32;}trait C{type Out;}impl C for u16{type Out=u32;}struct W<T>(T);trait B{}fn require<T:B>(){}impl<T:A>W<T>where <T as A>::Out:C<Out=u32>{fn get(&self)->u8{7}}fn main(){let _=W(1u8).get();}",
        true,
    ),
    (
        "projected-trait-argument",
        "trait A{type Out;}impl A for u8{type Out=u16;}trait C<P>{}impl C<u16>for u8{}trait B{}impl<T:A+C<<T as A>::Out>>B for T{}fn require<T:B>(){}fn main(){require::<u8>();}",
        true,
    ),
    (
        "projected-copy-bound",
        "trait A{type Out;}impl A for u8{type Out=u16;}impl A for u16{type Out=u32;}trait C{type Out;}impl C for u16{type Out=u32;}struct W<T>(T);trait B{}fn require<T:B>(){}impl<T:A>B for W<T>where <T as A>::Out:Copy{}fn main(){require::<W<u8>>();}",
        true,
    ),
    (
        "projected-output-parameter",
        "trait A{type Out;}impl A for u8{type Out=u16;}impl A for u16{type Out=u32;}trait C{type Out;}impl C for u16{type Out=u32;}struct W<T>(T);trait B{}fn require<T:B>(){}impl<T:A,U>B for W<T>where <T as A>::Out:C<Out=U>{}fn main(){require::<W<u8>>();}",
        true,
    ),
    (
        "missing-projection-premise",
        "trait A{type Out;}trait C{}struct W<T>(T);impl<T>W<T>where <T as A>::Out:C{}fn main(){}",
        false,
    ),
    (
        "projected-inductive-cycle",
        "trait A{type Out;}trait B{}struct W<T>(T);impl<T:B>A for T{type Out=u8;}impl<T:A>B for T where <T as A>::Out:B{}fn require<T:B>(){}fn main(){require::<u8>();}",
        false,
    ),
    (
        "projected-copy-negative",
        "struct N;trait A{type Out;}impl A for u8{type Out=N;}trait B{}fn require<T:B>(){}impl<T:A>B for T where <T as A>::Out:Copy{}fn main(){require::<u8>();}",
        false,
    ),
    (
        "projected-clone",
        "trait A{type Out;}impl A for u8{type Out=u16;}trait B{}fn require<T:B>(){}impl<T:A>B for T where <T as A>::Out:Clone{}fn main(){require::<u8>();}",
        true,
    ),
    (
        "projected-clone-negative",
        "struct N;trait A{type Out;}impl A for u8{type Out=N;}trait B{}fn require<T:B>(){}impl<T:A>B for T where <T as A>::Out:Clone{}fn main(){require::<u8>();}",
        false,
    ),
    (
        "projected-sized",
        "trait A{type Out:?Sized;}impl A for u8{type Out=u16;}trait B{}fn require<T:B>(){}impl<T:A>B for T where <T as A>::Out:Sized{}fn main(){require::<u8>();}",
        true,
    ),
    (
        "projected-sized-negative",
        "trait A{type Out:?Sized;}impl A for u8{type Out=str;}trait B{}fn require<T:B>(){}impl<T:A>B for T where <T as A>::Out:Sized{}fn main(){require::<u8>();}",
        false,
    ),
    (
        "projected-mixed-arguments",
        "trait A{type Out;}impl A for u8{type Out=u16;}trait C<'a,P,const N:usize>{}impl<'a>C<'a,u16,3>for u8{}trait B<'a>{}impl<'a,T:A+C<'a,<T as A>::Out,3>>B<'a>for T{}fn require<T:B<'static>>(){}fn main(){require::<u8>();}",
        true,
    ),
    (
        "projected-nested-argument",
        "trait A{type Out;}impl A for u8{type Out=u16;}trait C<P>{}impl C<(u16,[u16;2])>for u8{}trait B{}fn require<T:B>(){}impl<T:A+C<(<T as A>::Out,[<T as A>::Out;2])>>B for T{}fn main(){require::<u8>();}",
        true,
    ),
    (
        "projected-inferred-trait-argument",
        "trait A{type Out;}impl A for u8{type Out=u16;}trait C<U>{}impl C<u32>for u16{}trait B<U>{}struct W<T>(T);impl<T:A,U>B<U>for W<T>where <T as A>::Out:C<U>{}fn choose<T:B<U>,U>(_:T)->U{loop{}}fn main(){let _:u32=choose(W(1u8));}",
        true,
    ),
    (
        "projected-inferred-output-hole",
        "trait A{type Out;}impl A for u8{type Out=u16;}trait C<U>{}impl C<u32>for u16{}trait B<U>{}struct W<T>(T);impl<T:A,U>B<U>for W<T>where <T as A>::Out:C<U>{}fn choose<T:B<U>,U>(_:T){}fn main(){choose(W(1u8));}",
        true,
    ),
    (
        "projected-result-not-injective",
        "trait A{type Out;}impl A for u8{type Out=u32;}impl A for u16{type Out=u32;}trait C<P>{}impl C<u8>for u32{}trait B{}impl<T:A>B for T where <T as A>::Out:C<u16>{}fn require<T:B>(){}fn main(){require::<u8>();}",
        false,
    ),
    (
        "projected-caller-copy",
        "trait A{type Out;}trait B{}impl<T:A>B for T where <T as A>::Out:Copy{}fn require<T:B>(){}fn f<T:A>()where <T as A>::Out:Copy{require::<T>();}fn main(){}",
        true,
    ),
    (
        "projected-inferred-nested-hole",
        "trait A{type Out;}impl A for u8{type Out=u16;}trait C<P>{}impl C<(u16,u32)>for u8{}trait B<U>{}impl<T:A,U>B<U>for T where T:C<(<T as A>::Out,U)>{}fn choose<T:B<U>,U>(_:T){}fn main(){choose(1u8);}",
        true,
    ),
    (
        "projected-region-capture",
        "trait A{type Out;}impl<'a>A for &'a u8{type Out=&'a u16;}trait C<'a>{}impl<'a>C<'a>for &'a u16{}trait B<'a>{}impl<'a,T:A>B<'a>for T where <T as A>::Out:C<'a>{}fn require<T:B<'static>>(_:T){}static X:u8=7;fn main(){require(&X);}",
        true,
    ),
    (
        "projected-region-escape",
        "trait A{type Out;}impl<'a>A for &'a u8{type Out=&'a u16;}trait C<'a>{}impl<'a>C<'a>for &'a u16{}trait B<'a>{}impl<'a,T:A>B<'a>for T where <T as A>::Out:C<'a>{}fn require<T:B<'static>>(_:T){}fn f<'a>(x:&'a u8){require(x);}fn main(){}",
        false,
    ),
    (
        "normalized-caller-trait-argument",
        "trait A{type Out;}trait C<P>{}trait B{}impl<T:A+C<<T as A>::Out>>B for T{}fn require<T:B>(){}fn f<T:A<Out=u16>+C<u16>>(){require::<T>();}fn main(){}",
        true,
    ),
    (
        "wrong-normalized-caller-trait-argument",
        "trait A{type Out;}trait C<P>{}trait B{}impl<T:A+C<<T as A>::Out>>B for T{}fn require<T:B>(){}fn f<T:A<Out=u16>+C<u32>>(){require::<T>();}fn main(){}",
        false,
    ),
];
#[test]
fn projected_conditions_are_proved_at_use_sites() -> Result<(), Box<dyn std::error::Error>> {
    for &(name, source, accepted) in CASES {
        rustc::qualify(&format!("projected-condition-{name}"), source, accepted)?;
        let result = semantic::analyze(source).and_then(|(mut report, mut facts)| {
            let entry =
                rustgm::sema::entry::select(&mut report, &facts).map_err(|e| e.to_string())?;
            rustgm::lower::instances::build(&report, &mut facts, entry)
                .map(|_| ())
                .map_err(|e| e.to_string())
        });
        assert_eq!(result.is_ok(), accepted, "{name}: {result:?}");
    }
    Ok(())
}

#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn selected_projected_conditions_execute() -> Result<(), Box<dyn std::error::Error>> {
    native::check_cases(
        "projected-impl-conditions",
        &[
            (
                "normalized-caller-trait-argument",
                "trait A{type Out;}impl A for u8{type Out=u16;}trait C<P>{}impl C<u16>for u8{}trait B{fn value()->u8{73}}impl<T:A+C<<T as A>::Out>>B for T{}fn require<T:B>()->u8{<T as B>::value()}fn f<T:A<Out=u16>+C<u16>>()->u8{require::<T>()}fn main(){check(f::<u8>()==73);}",
                false,
            ),
            (
                "nested-equality",
                "trait A{type Out;}impl A for u8{type Out=u16;}impl A for u16{type Out=u32;}trait C{type Out;}impl C for u16{type Out=u32;}struct W<T>(T);trait B{fn value()->u8{73}}fn require<T:B>()->u8{<T as B>::value()}impl<T:A>B for W<T>where <T as A>::Out:C<Out=u32>{}fn main(){check(require::<W<u8>>()==73);}",
                false,
            ),
            (
                "inherent-projected-bound",
                "trait A{type Out;}impl A for u8{type Out=u16;}impl A for u16{type Out=u32;}trait C{type Out;}impl C for u16{type Out=u32;}struct W<T>(T);trait B{}fn require<T:B>(){}impl<T:A>W<T>where <T as A>::Out:C<Out=u32>{fn get(&self)->u8{7}}fn main(){let _=check(W(1u8).get()==7);}",
                false,
            ),
            (
                "projected-trait-argument",
                "trait A{type Out;}impl A for u8{type Out=u16;}trait C<P>{}impl C<u16>for u8{}trait B{fn value()->u8{73}}impl<T:A+C<<T as A>::Out>>B for T{}fn require<T:B>()->u8{<T as B>::value()}fn main(){check(require::<u8>()==73);}",
                false,
            ),
            (
                "projected-copy-bound",
                "trait A{type Out;}impl A for u8{type Out=u16;}impl A for u16{type Out=u32;}trait C{type Out;}impl C for u16{type Out=u32;}struct W<T>(T);trait B{fn value()->u8{73}}fn require<T:B>()->u8{<T as B>::value()}impl<T:A>B for W<T>where <T as A>::Out:Copy{}fn main(){check(require::<W<u8>>()==73);}",
                false,
            ),
            (
                "projected-output-parameter",
                "trait A{type Out;}impl A for u8{type Out=u16;}impl A for u16{type Out=u32;}trait C{type Out;}impl C for u16{type Out=u32;}struct W<T>(T);trait B{fn value()->u8{73}}fn require<T:B>()->u8{<T as B>::value()}impl<T:A,U>B for W<T>where <T as A>::Out:C<Out=U>{}fn main(){check(require::<W<u8>>()==73);}",
                false,
            ),
        ],
    )
}
