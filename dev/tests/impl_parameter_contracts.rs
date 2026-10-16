//! Impl outputs are determined by directed, seeded associated equalities.
#[path = "harness/native_references.rs"]
mod native;
#[path = "helpers/qualify.rs"]
mod rustc;
#[path = "support/semantic.rs"]
mod semantic;

const CASES: &[(&str, &str, bool)] = &[
    (
        "type-result",
        "trait A{type Out;}impl A for u8{type Out=u16;}impl A for u16{type Out=u32;}trait B{fn get()->usize;}impl<T:A<Out=U>,U>B for T{fn get()->usize{7}}fn main(){check(<u8 as B>::get()==7);}",
        true,
    ),
    (
        "array-length",
        "trait A{type Out;}impl A for u8{type Out=[u16;3];}trait B{fn get()->usize;}impl<T:A<Out=[u16;N]>,const N:usize>B for T{fn get()->usize{N}}fn main(){check(<u8 as B>::get()==3);}",
        true,
    ),
    (
        "inherent-result",
        "trait A{type Out;}impl A for u8{type Out=u16;}impl A for u16{type Out=u32;}trait B{fn get()->usize;}struct W<T>(T);impl<T:A<Out=U>,U>W<T>{fn get(x:U)->U{x}}fn main(){check(W::<u8>::get(7u16)==7);}",
        true,
    ),
    (
        "receiver-result",
        "trait A{type Out;}impl A for u8{type Out=u16;}impl A for u16{type Out=u32;}trait B{fn get()->usize;}struct W<T>(T);impl<T:A<Out=U>,U>W<T>{fn get(&self,x:U)->U{x}}fn main(){check(W(1u8).get(7u16)==7);}",
        true,
    ),
    (
        "dependent-chain",
        "trait A{type Out;}impl A for u8{type Out=u16;}impl A for u16{type Out=u32;}trait B{fn get()->usize;}impl<T:A<Out=U>,U:A<Out=V>,V>B for T{fn get()->usize{7}}fn main(){check(<u8 as B>::get()==7);}",
        true,
    ),
    (
        "reversed-chain",
        "trait A{type Out;}impl A for u8{type Out=u16;}impl A for u16{type Out=u32;}trait B{fn get()->usize;}impl<V,U:A<Out=V>,T:A<Out=U>>B for T{fn get()->usize{7}}fn main(){check(<u8 as B>::get()==7);}",
        true,
    ),
    (
        "trait-argument-seed",
        "trait A{type Out;}impl A for u8{type Out=u16;}trait B<T>{fn get()->usize;}impl<T:A<Out=U>,U>B<T>for u32{fn get()->usize{7}}fn main(){check(<u32 as B<u8>>::get()==7);}",
        true,
    ),
    (
        "tuple-output",
        "trait A{type Out;}impl A for u8{type Out=(u16,u32);}trait B{fn get()->usize;}impl<T:A<Out=(U,V)>,U,V>B for T{fn get()->usize{7}}fn main(){check(<u8 as B>::get()==7);}",
        true,
    ),
    (
        "callable-output",
        "trait A{type Out;}impl A for u8{type Out=fn(u16)->u32;}trait B{fn get()->usize;}impl<T:A<Out=fn(U)->V>,U,V>B for T{fn get()->usize{7}}fn main(){check(<u8 as B>::get()==7);}",
        true,
    ),
    (
        "argument-dependency",
        "trait A<P>{type Out;}impl A<u32>for u8{type Out=u16;}struct W<T,P>(T,P);trait B{fn get()->usize;}impl<T:A<P,Out=U>,P,U>B for W<T,P>{fn get()->usize{7}}fn main(){check(<W<u8,u32>as B>::get()==7);}",
        true,
    ),
    (
        "own-trait-other-subject",
        "trait A{type Out;fn get()->u16;}impl A for u8{type Out=u16;fn get()->u16{7}}struct W<T>(T);impl<T:A<Out=U>,U>A for W<T>{type Out=U;fn get()->u16{7}}fn main(){check(<W<u8>as A>::get()==7);}",
        true,
    ),
    (
        "own-trait-other-argument",
        "trait A<P>{type Out;fn get()->u16;}impl A<u8>for u8{type Out=u16;fn get()->u16{7}}impl<T:A<u8,Out=U>,U>A<u16>for T{type Out=U;fn get()->u16{7}}fn main(){check(<u8 as A<u16>>::get()==7);}",
        true,
    ),
    (
        "unseeded-input",
        "trait A{type Out;}impl A for u8{type Out=u16;}impl A for u16{type Out=u32;}trait B{fn get()->usize;}struct W;impl<T:A<Out=U>,U>W{fn get(x:U)->U{x}}fn main(){}",
        false,
    ),
    (
        "unseeded-cycle",
        "trait A{type Out;}impl A for u8{type Out=u16;}impl A for u16{type Out=u32;}trait B{fn get()->usize;}struct W;impl<T:A<Out=U>,U:A<Out=T>>W{}fn main(){}",
        false,
    ),
    (
        "reverse-dependency",
        "trait A{type Out;}impl A for u8{type Out=u16;}impl A for u16{type Out=u32;}trait B{fn get()->usize;}struct W<T>(T);impl<T,U:A<Out=T>>W<T>{}fn main(){}",
        false,
    ),
    (
        "unseeded-trait-argument",
        "trait A<P>{type Out;}trait B{}impl<T:A<P,Out=U>,P,U>B for T{}fn main(){}",
        false,
    ),
    (
        "noninjective-output",
        "trait A{type Out;}impl A for u8{type Out=u16;}impl A for u16{type Out=u32;}trait B{fn get()->usize;}impl<T:A,U:A<Out=<T as A>::Out>>B for U{fn get()->usize{7}}fn main(){}",
        false,
    ),
    (
        "own-trait-cycle",
        "trait A{type Out;}impl<U>A for u8 where u8:A<Out=U>{type Out=U;}fn main(){}",
        false,
    ),
    (
        "unseeded-lifetime-input",
        "trait A<'a>{type Out;}trait B{}impl<'a,T:A<'a,Out=U>,U>B for T{}fn main(){}",
        false,
    ),
    (
        "seeded-lifetime-input",
        "trait A<'a>{type Out;}impl<'a>A<'a>for u8{type Out=u16;}trait B<'a>{fn get()->usize;}impl<'a,T:A<'a,Out=U>,U>B<'a>for T{fn get()->usize{7}}fn main(){check(<u8 as B<'static>>::get()==7);}",
        true,
    ),
    (
        "lifetime-result-chain",
        "trait A{type Out;}impl<'a>A for &'a u8{type Out=&'a u8;}trait C<'a>{type Out;}impl<'a>C<'a>for u16{type Out=u32;}trait B{fn get()->usize;}impl<'a,T:A<Out=&'a u8>,U:C<'a,Out=V>,V>B for (T,U){fn get()->usize{7}}fn main(){check(<(&'static u8,u16)as B>::get()==7);}",
        true,
    ),
    (
        "wrong-result-call",
        "trait A{type Out;}impl A for u8{type Out=u16;}impl A for u16{type Out=u32;}trait B{fn get()->usize;}impl<T:A<Out=U>,U>B for T{fn get()->usize{7}}fn f<T:A<Out=u32>>(_:T){}fn main(){f(1u8);}",
        false,
    ),
];

#[test]
fn equality_dependencies_determine_only_constrained_impl_parameters()
-> Result<(), Box<dyn std::error::Error>> {
    for &(name, source, accepted) in CASES {
        let source = format!("{}{source}", native::CHECK);
        rustc::qualify(&format!("impl-parameter-{name}"), &source, accepted)?;
        let result = semantic::analyze(&source).and_then(|(mut report, mut facts)| {
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
fn determined_impl_arguments_reach_executable_methods() -> Result<(), Box<dyn std::error::Error>> {
    let cases: Vec<_> = CASES
        .iter()
        .filter(|(_, _, accepted)| *accepted)
        .map(|&(name, source, _)| (name, source, false))
        .collect();
    native::check_cases("impl-parameter-constraints", &cases)
}
