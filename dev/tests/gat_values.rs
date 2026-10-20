//! Public GAT declarations, bindings and calls retain complete input proofs.
#[path = "harness/native_references.rs"]
mod native;
#[path = "helpers/qualify.rs"]
mod rustc;
#[path = "support/semantic.rs"]
mod semantic;
const CASES: &[(&str, &str, bool)] = &[
    (
        "lending",
        "trait L{type Out<'a>where Self:'a;fn get<'a>(&'a self)->Self::Out<'a>;}struct S{v:u8}impl L for S{type Out<'a>=&'a u8;fn get<'a>(&'a self)->Self::Out<'a>{&self.v}}fn main(){let s=S{v:7};check(*s.get()==7);}",
        true,
    ),
    (
        "captured-lending",
        "trait L{type Out<'a>where Self:'a;fn get<'a>(&'a self)->Self::Out<'a>;}struct S<T>{v:T}impl<T>L for S<T>{type Out<'a>=&'a T where T:'a;fn get<'a>(&'a self)->Self::Out<'a>{&self.v}}fn main(){let s=S{v:7u8};check(*s.get()==7);}",
        true,
    ),
    (
        "type-family",
        "trait F{type Out<T>;fn make<T>(x:T)->Self::Out<T>;}struct S;impl F for S{type Out<T>=T;fn make<T>(x:T)->Self::Out<T>{x}}fn main(){check(S::make(7u8)==7);check(S::make(11u16)==11);}",
        true,
    ),
    (
        "const-family",
        "trait F{type Out<const N:usize>;fn make<const N:usize>(x:[u8;N])->Self::Out<N>;}struct S;impl F for S{type Out<const N:usize>=[u8;N];fn make<const N:usize>(x:[u8;N])->Self::Out<N>{x}}fn main(){let x=S::make([3u8,7,11]);check(x[1]==7);}",
        true,
    ),
    (
        "lifetime-equality",
        "trait F{type Out<'a>;}struct S;impl F for S{type Out<'a>=&'a u8;}fn get<'a,T:F<Out<'a>=&'a u8>>(x:T::Out<'a>)->u8{*x}fn main(){let v=7;check(get::<S>(&v)==7);}",
        true,
    ),
    (
        "quantified-equality",
        "trait F{type Out<'a>;}struct S;impl F for S{type Out<'a>=&'a u8;}fn get<'a,T:for<'b>F<Out<'b>=&'b u8>>(x:T::Out<'a>)->u8{*x}fn main(){let v=7;check(get::<S>(&v)==7);}",
        true,
    ),
    (
        "type-equality",
        "trait F{type Out<U>;}struct S;impl F for S{type Out<U>=U;}fn get<T:F<Out<u8>=u8>>(x:T::Out<u8>)->u8{x}fn main(){check(get::<S>(7)==7);}",
        true,
    ),
    (
        "const-equality",
        "trait F{type Out<const N:usize>;}struct S;impl F for S{type Out<const N:usize>=[u8;N];}fn get<T:F<Out<3>=[u8;3]>>(x:T::Out<3>)->u8{x[1]}fn main(){check(get::<S>([3,7,11])==7);}",
        true,
    ),
    (
        "inherited-equality",
        "trait F{type Out<U>;}trait A:F{}struct S;impl F for S{type Out<U>=U;}impl A for S{}fn get<T:A<Out<u8>=u8>>(x:T::Out<u8>)->u8{x}fn main(){check(get::<S>(7)==7);}",
        true,
    ),
    (
        "header-equality",
        "trait F{type Out<U>;}trait A:F<Out<u8>=u8>{}struct S;impl F for S{type Out<U>=U;}impl A for S{}fn get<T:A>(x:T::Out<u8>)->u8{x}fn main(){check(get::<S>(7)==7);}",
        true,
    ),
    (
        "wrong-equality",
        "trait F{type Out<U>;}struct S;impl F for S{type Out<U>=U;}fn get<T:F<Out<u8>=u16>>(x:T::Out<u8>)->u16{x}fn main(){let _=get::<S>(7);}",
        false,
    ),
    (
        "missing-family-argument",
        "trait F{type Out<U>;}fn get<T:F<Out=u8>>(){}fn main(){}",
        false,
    ),
    (
        "wrong-family-kind",
        "trait F{type Out<'a>;}fn get<T:F<Out<u8>=u8>>(){}fn main(){}",
        false,
    ),
    (
        "binding-input-debt",
        "trait F{type Out<U:'static>;}fn get<T:F<Out<U>=u8>,U>(){}fn main(){}",
        false,
    ),
    (
        "binding-independent-applications",
        "trait F{type Out<U>;}fn get<T:F<Out<u8>=u8,Out<u16>=u16>>(){}fn main(){}",
        true,
    ),
    (
        "shorten",
        "trait F{type Out<'a>;fn make<'a>(x:&'a u8)->Self::Out<'a>;}struct S;impl F for S{type Out<'a>=&'a u8;fn make<'a>(x:&'a u8)->Self::Out<'a>{x}}fn shorten<'a:'b,'b>(x:<S as F>::Out<'a>)-><S as F>::Out<'b>{x}fn main(){let v=7;check(*shorten(S::make(&v))==7);}",
        true,
    ),
    (
        "escape",
        "trait F{type Out<'a>;fn make<'a>(x:&'a u8)->Self::Out<'a>;}struct S;impl F for S{type Out<'a>=&'a u8;fn make<'a>(x:&'a u8)->Self::Out<'a>{x}}fn bad<'a>()-><S as F>::Out<'a>{let v=7;S::make(&v)}fn main(){}",
        false,
    ),
    (
        "invariant",
        "trait F{type Out<'a>;}struct S;impl F for S{type Out<'a>=&'a mut &'a u8;}fn shorten<'a:'b,'b>(x:<S as F>::Out<'a>)-><S as F>::Out<'b>{x}fn main(){}",
        false,
    ),
    (
        "mutation",
        "trait F{type Out<'a>;fn make<'a>(x:&'a mut u8)->Self::Out<'a>;}struct S;impl F for S{type Out<'a>=&'a mut u8;fn make<'a>(x:&'a mut u8)->Self::Out<'a>{x}}fn main(){let mut v=7;*S::make(&mut v)=11;check(v==11);}",
        true,
    ),
    (
        "aliasing",
        "trait F{type Out<'a>;fn make<'a>(x:&'a mut u8)->Self::Out<'a>;}struct S;impl F for S{type Out<'a>=&'a mut u8;fn make<'a>(x:&'a mut u8)->Self::Out<'a>{x}}fn main(){let mut v=7;let a=S::make(&mut v);let b=S::make(&mut v);*a=9;*b=11;}",
        false,
    ),
    (
        "owned-drop",
        "struct D<'a>(&'a mut u8);impl<'a>Drop for D<'a>{fn drop(&mut self){*self.0+=1;}}trait F{type Out<'a>;fn make<'a>(x:&'a mut u8)->Self::Out<'a>;}struct S;impl F for S{type Out<'a>=D<'a>;fn make<'a>(x:&'a mut u8)->Self::Out<'a>{D(x)}}fn consume<T:F>(){let mut v=0u8;{let _d=T::make(&mut v);}check(v==1);}fn main(){consume::<S>();}",
        true,
    ),
    (
        "borrowed-drop-escape",
        "struct D<'a>(&'a u8);impl<'a>Drop for D<'a>{fn drop(&mut self){let _=*self.0;}}trait F{type Out<'a>;}struct S;impl F for S{type Out<'a>=D<'a>;}fn bad<'a>()-><S as F>::Out<'a>{let v=7;D(&v)}fn main(){}",
        false,
    ),
    (
        "nominal-storage",
        "trait F{type Out<U>;}struct S;impl F for S{type Out<U>=U;}struct Wrap<T:F>{v:T::Out<u8>}fn main(){let x=Wrap::<S>{v:7};check(x.v==7);}",
        true,
    ),
    (
        "binding-static-explicit",
        "trait F{type Out<U:'static>;}fn get<T:F<Out<U>=u8>,U:'static>(){}fn main(){}",
        true,
    ),
    (
        "binding-lifetime-debt",
        "trait F{type Out<'a,U>where U:'a;}fn get<'a,T:F<Out<'a,U>=u8>,U>(){}fn main(){}",
        false,
    ),
    (
        "binding-lifetime-explicit",
        "trait F{type Out<'a,U>where U:'a;}fn get<'a,T:F<Out<'a,U>=u8>,U:'a>(){}fn main(){}",
        true,
    ),
    (
        "binding-copy-debt",
        "trait F{type Out<U:Copy>;}fn get<T:F<Out<U>=u8>,U>(){}fn main(){}",
        false,
    ),
    (
        "binding-copy-explicit",
        "trait F{type Out<U:Copy>;}fn get<T:F<Out<U>=u8>,U:Copy>(){}fn main(){}",
        true,
    ),
    (
        "binding-projection-debt",
        "trait B{type V;}trait F{type Out<U>;}fn get<T:F<Out<<U as B>::V>=u8>,U>(){}fn main(){}",
        false,
    ),
    (
        "binding-projection-explicit",
        "trait B{type V;}trait F{type Out<U>;}fn get<T:F<Out<<U as B>::V>=u8>,U:B>(){}fn main(){}",
        true,
    ),
    (
        "binding-independent-result",
        "trait F{type Out<U>;}fn get<T:F<Out<u8>=u8>>(x:T::Out<u16>)->u8{x}fn main(){}",
        false,
    ),
];

fn lower(source: &str) -> Result<(), String> {
    let (mut report, mut facts) = semantic::analyze(source)?;
    let entry = rustgm::sema::entry::select(&mut report, &facts).map_err(|e| e.to_string())?;
    rustgm::lower::instances::build(&report, &mut facts, entry)
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[test]
fn public_family_applications_preserve_argument_kinds_and_input_debts()
-> Result<(), Box<dyn std::error::Error>> {
    let mut failures = Vec::new();
    for &(name, source, accepted) in CASES {
        let source = format!("{}{source}", native::CHECK);
        rustc::qualify(&format!("gat-values-{name}"), &source, accepted)?;
        let result = lower(&source);
        if result.is_ok() != accepted {
            failures.push(format!("{name}: expected {accepted}, got {result:?}"));
        } else if name == "binding-input-debt"
            && result.as_ref().err().is_none_or(|e| {
                !e.contains("associated type lifetime requirement is not satisfied")
            })
        {
            failures.push(format!("{name}: wrong rejection: {result:?}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn generic_family_values_and_equalities_execute() -> Result<(), Box<dyn std::error::Error>> {
    let cases: Vec<_> = CASES
        .iter()
        .filter(|&&(_, source, accepted)| accepted && source.contains("check("))
        .map(|&(name, source, _)| (name, source, false))
        .collect();
    native::check_cases("family-values/native", &cases)
}
