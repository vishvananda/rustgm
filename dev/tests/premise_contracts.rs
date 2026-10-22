//! Equality premises retain trait proofs, result equations and caller lifetime checks.
use crate::native_references as native;
use crate::qualify as rustc;
use crate::semantic;

const CASES: &[(&str, &str, bool)] = &[
    (
        "impl-equality",
        "trait A{type Out;}impl A for u8{type Out=u16;}impl A for u16{type Out=u32;}trait B{fn get()->u16;}impl<T:A<Out=u16>>B for T{fn get()->u16{7}}fn main(){check(<u8 as B>::get()==7);}",
        true,
    ),
    (
        "impl-equality-mismatch",
        "trait A{type Out;}impl A for u8{type Out=u16;}impl A for u16{type Out=u32;}trait B{}impl<T:A<Out=u16>>B for T{}fn require<T:B>(){}fn main(){require::<u16>();}",
        false,
    ),
    (
        "impl-equality-wrapper",
        "trait A{type Out;}impl A for u8{type Out=u16;}impl A for u16{type Out=u32;}struct W<T>(T);trait B{fn get()->u16;}impl<T:A<Out=u16>>B for W<T>{fn get()->u16{7}}fn main(){check(<W<u8>as B>::get()==7);}",
        true,
    ),
    (
        "impl-equality-caller",
        "trait A{type Out;}impl A for u8{type Out=u16;}impl A for u16{type Out=u32;}struct W<T>(T);trait B{}impl<T:A<Out=u16>>B for W<T>{}fn require<T:B>(){}fn f<T:A<Out=u16>>(){require::<W<T>>();}fn main(){f::<u8>();}",
        true,
    ),
    (
        "impl-equality-unproved",
        "trait A{type Out;}impl A for u8{type Out=u16;}impl A for u16{type Out=u32;}struct W<T>(T);trait B{}impl<T:A<Out=u16>>B for W<T>{}fn require<T:B>(){}fn f<T:A>(){require::<W<T>>();}fn main(){}",
        false,
    ),
    (
        "inherent-equality",
        "trait A{type Out;}impl A for u8{type Out=u16;}impl A for u16{type Out=u32;}struct W<T>(T);impl<T:A<Out=u16>>W<T>{fn get()->u16{7}}fn main(){check(W::<u8>::get()==7);}",
        true,
    ),
    (
        "inherent-equality-mismatch",
        "trait A{type Out;}impl A for u8{type Out=u16;}impl A for u16{type Out=u32;}struct W<T>(T);impl<T:A<Out=u16>>W<T>{fn get()->u16{7}}fn main(){let _=W::<u16>::get();}",
        false,
    ),
    (
        "receiver-equality",
        "trait A{type Out;}impl A for u8{type Out=u16;}impl A for u16{type Out=u32;}struct W<T>(T);impl<T:A<Out=u16>>W<T>{fn get(&self)->u16{7}}fn main(){check(W(1u8).get()==7);}",
        true,
    ),
    (
        "receiver-equality-mismatch",
        "trait A{type Out;}impl A for u8{type Out=u16;}impl A for u16{type Out=u32;}struct W<T>(T);impl<T:A<Out=u16>>W<T>{fn get(&self)->u16{7}}fn main(){let _=W(1u16).get();}",
        false,
    ),
    (
        "impl-result-inference",
        "trait A{type Out;}impl A for u8{type Out=u16;}impl A for u16{type Out=u32;}struct W<T>(T);trait B<U>{fn get(x:U)->U;}impl<T:A<Out=U>,U>B<U>for W<T>{fn get(x:U)->U{x}}fn main(){check(<W<u8>as B<_>>::get(7u16)==7);}",
        true,
    ),
    (
        "impl-borrowed-result",
        "trait A{type Out;}impl<'a>A for &'a u8{type Out=&'a u8;}trait B<'a>{}impl<'a,T:A<Out=&'a u8>>B<'a>for T{}fn require<'a,T:B<'a>>(_:T,x:&'a u8)->&'a u8{x}fn main(){let x=7u8;check(*require(&x,&x)==7);}",
        true,
    ),
    (
        "impl-borrowed-escape",
        "trait A{type Out;}impl<'a>A for &'a u8{type Out=&'a u8;}trait B{}impl<T:A<Out=&'static u8>>B for T{}fn require<T:B>(_:T){}fn f<'a>(x:&'a u8){require(x);}fn main(){}",
        false,
    ),
    (
        "impl-callable-result",
        "trait A{type Out;}impl A for u8{type Out=for<'a>fn(&'a u8)->&'a u8;}trait B{}impl<T:A<Out=for<'b>fn(&'b u8)->&'b u8>>B for T{}fn require<T:B>(){}fn main(){require::<u8>();}",
        true,
    ),
    (
        "impl-equality-cycle",
        "trait A{type Out;}trait B{}impl<T:B>A for T{type Out=u16;}impl<T:A<Out=u16>>B for T{}fn require<T:B>(){}fn main(){require::<u8>();}",
        false,
    ),
    (
        "two-independent-equalities",
        "trait A{type Out;}impl A for u8{type Out=u16;}impl A for u16{type Out=u32;}trait C{type Out;}impl C for u8{type Out=u32;}trait B{}impl<T:A<Out=u16>+C<Out=u32>>B for T{}fn require<T:B>(){}fn main(){require::<u8>();}",
        true,
    ),
];

#[test]
fn implementation_and_receiver_premises_require_complete_proofs()
-> Result<(), Box<dyn std::error::Error>> {
    for &(name, source, accepted) in CASES {
        let source = format!("{}{source}", native::CHECK);
        rustc::qualify(&format!("premise-{name}"), &source, accepted)?;
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
fn equality_premises_select_executable_implementations() -> Result<(), Box<dyn std::error::Error>> {
    let cases: Vec<_> = CASES
        .iter()
        .filter(|(_, _, accepted)| *accepted)
        .map(|&(name, source, _)| (name, source, false))
        .collect();
    native::check_cases("equality-premises", &cases)
}
