//! Trait overrides establish their existential result contract without identity equality.
#[path = "harness/native_references.rs"]
mod native;
#[path = "helpers/qualify.rs"]
mod rustc;
#[path = "support/semantic.rs"]
mod semantic;

const CONFORMANCE: &[(&str, &str, bool)] = &[
    (
        "opaque-copy",
        "trait A{fn f()->impl Copy;}impl A for u8{fn f()->impl Copy{7u8}}fn main(){}",
        true,
    ),
    (
        "refined-copy",
        "trait A{fn f()->impl Copy;}impl A for u8{fn f()->u8{7}}fn main(){}",
        true,
    ),
    (
        "opaque-user",
        "trait V{}impl V for u8{}trait A{fn f()->impl V;}impl A for u16{fn f()->impl V{7u8}}fn main(){}",
        true,
    ),
    (
        "refined-user",
        "trait V{}impl V for u8{}trait A{fn f()->impl V;}impl A for u16{fn f()->u8{7}}fn main(){}",
        true,
    ),
    (
        "quantified",
        "trait V<'a>{}impl<'a>V<'a> for u8{}trait A{fn f()->impl for<'a>V<'a>;}impl A for u16{fn f()->impl for<'b>V<'b>{7u8}}fn main(){}",
        true,
    ),
    (
        "pair",
        "trait A{fn f()->(impl Copy,impl Copy);}impl A for u8{fn f()->(u8,bool){(7,true)}}fn main(){}",
        true,
    ),
    (
        "parameter",
        "trait A{fn f<T:Copy>(x:T)->impl Copy;}impl A for u8{fn f<U:Copy>(x:U)->impl Copy{x}}fn main(){}",
        true,
    ),
    (
        "lifetime",
        "trait A{fn f<'a>(x:&'a u8)->impl Copy+'a;}impl A for u8{fn f<'b>(x:&'b u8)->impl Copy+'b{x}}fn main(){}",
        true,
    ),
    (
        "missing-output-wf",
        "trait A{fn f<'a,T>(x:&'a ())->impl Sized+'a;}impl A for u8{fn f<'a,T>(x:&'a ())->&'a T{loop{}}}fn main(){}",
        false,
    ),
    (
        "input-output-wf",
        "trait A{fn f<'a,T>(x:&'a T)->impl Sized+'a;}impl A for u8{fn f<'a,T>(x:&'a T)->&'a T{x}}fn main(){}",
        true,
    ),
    (
        "missing-parent-capture",
        "trait A<'a>{fn f()->impl Copy+use<Self>;}fn main(){}",
        false,
    ),
    (
        "parent-capture",
        "trait A<'a>{fn f()->impl Copy+use<'a,Self>;}impl<'a>A<'a> for u8{fn f()->impl Copy{7u8}}fn main(){}",
        true,
    ),
    (
        "stronger-output-hidden",
        "trait Value{fn value(&self)->u8;}impl Value for u8{fn value(&self)->u8{*self}}trait A{fn f()->impl Copy;}impl A for u8{fn f()->impl Copy+Value{7u8}}fn bad<T:A>(){let _=T::f().value();}fn main(){}",
        false,
    ),
    (
        "refinement-hidden",
        "trait A{fn f()->impl Copy;}impl A for u8{fn f()->u8{7}}fn bad<T:A>(){let _:u8=T::f();}fn main(){}",
        false,
    ),
    (
        "opaque-hidden",
        "trait A{fn f()->impl Copy;}impl A for u8{fn f()->impl Copy{7u8}}fn main(){let _:u8=<u8 as A>::f();}",
        false,
    ),
    (
        "excess-method-capture",
        "trait A{fn f<'a>(x:&'a u8)->impl Copy+use<Self>;}impl A for u8{fn f<'a>(x:&'a u8)->impl Copy{x}}fn main(){}",
        false,
    ),
    (
        "distinct-implementations",
        "trait A{fn f()->impl Copy;}impl A for u8{fn f()->impl Copy{7u8}}impl A for u16{fn f()->impl Copy{7u8}}fn same<T>(_:T,_:T){}fn main(){same(<u8 as A>::f(),<u16 as A>::f());}",
        false,
    ),
    (
        "missing-copy",
        "struct S;trait A{fn f()->impl Copy;}impl A for u8{fn f()->S{S}}fn main(){}",
        false,
    ),
    (
        "missing-user",
        "trait V{}trait A{fn f()->impl V;}impl A for u16{fn f()->u8{7}}fn main(){}",
        false,
    ),
    (
        "missing-quantifier",
        "trait V<'a>{}impl V<'static> for u8{}trait A{fn f()->impl for<'a>V<'a>;}impl A for u16{fn f()->u8{7}}fn main(){}",
        false,
    ),
    (
        "missing-static",
        "trait A{fn f<'a>(x:&'a u8)->impl Copy+'static;}impl A for u8{fn f<'a>(x:&'a u8)->impl Copy+'a{x}}fn main(){}",
        false,
    ),
    (
        "wrong-input",
        "trait A{fn f(x:u8)->impl Copy;}impl A for u8{fn f(x:u16)->impl Copy{x}}fn main(){}",
        false,
    ),
    (
        "stronger-input",
        "trait A{fn f<T>(x:T)->impl Copy;}impl A for u8{fn f<T:Copy>(x:T)->impl Copy{x}}fn main(){}",
        false,
    ),
];

#[test]
fn overrides_establish_the_declared_output_bounds() -> Result<(), Box<dyn std::error::Error>> {
    for &(name, source, accepted) in CONFORMANCE {
        rustc::qualify(&format!("method-output-{name}"), source, accepted)?;
        let result = semantic::analyze(source);
        assert_eq!(result.is_ok(), accepted, "{name}: {:?}", result.err());
    }
    Ok(())
}

const EXECUTION: &[(&str, &str, bool)] = &[
    (
        "quantified-result",
        "trait Read<'a>{fn read(&self,x:&'a u8)->u8;}impl<'a>Read<'a> for u8{fn read(&self,x:&'a u8)->u8{*self+*x}}trait Make{fn make()->impl for<'a>Read<'a>;}impl Make for u16{fn make()->impl for<'a>Read<'a>{3u8}}fn run<T:Make>()->u8{let x=7;T::make().read(&x)}fn main(){check(run::<u16>()==10);}",
        false,
    ),
    (
        "gat-result",
        "trait Get{type V<'a>where Self:'a;fn get<'a>(&'a self)->Self::V<'a>;}impl Get for u8{type V<'a>=&'a u8;fn get<'a>(&'a self)->&'a u8{self}}trait Make{fn make()->impl for<'a>Get<V<'a>=&'a u8>;}impl Make for u16{fn make()->impl for<'a>Get<V<'a>=&'a u8>{149u8}}fn main(){let x=<u16 as Make>::make();check(*x.get()==149);}",
        false,
    ),
    (
        "infer-const-result",
        "trait Make{fn make<const N:usize>()->impl Copy;}impl Make for u8{fn make<const N:usize>()->[u8;N]{[97;N]}}fn main(){let a:[u8;3]=<u8 as Make>::make();check(a[2]==97);}",
        false,
    ),
    (
        "infer-type-result",
        "trait Make{fn make<T:Copy>()->impl Copy;}impl Make for u8{fn make<T:Copy>()->[T;0]{[]}}fn main(){let a:[u16;0]=<u8 as Make>::make();let _=a;}",
        false,
    ),
    (
        "impl-parent-lifetime",
        "trait Make{fn make(self)->impl Copy;}struct S<'a>(&'a u8);impl<'a>Make for S<'a>{fn make(self)->&'a u8{self.0}}fn main(){let x=71;check(*S(&x).make()==71);}",
        false,
    ),
    (
        "trait-parent-lifetime",
        "trait Make<'a>{fn make(x:&'a u8)->impl Copy+'a;}impl<'a>Make<'a> for u16{fn make(x:&'a u8)->&'a u8{x}}fn main(){let x=73;check(*<u16 as Make>::make(&x)==73);}",
        false,
    ),
    (
        "two-implementations",
        "trait Make{fn make()->impl Copy;}impl Make for u8{fn make()->u16{79}}impl Make for u16{fn make()->bool{true}}fn main(){let x:u16=<u8 as Make>::make();let y:bool=<u16 as Make>::make();check(x==79);check(y);}",
        false,
    ),
    (
        "method-two-regions",
        "trait Make{fn make<'a,'b>(x:&'a u8,y:&'b u8)->impl Copy;}impl Make for u16{fn make<'x,'y>(x:&'x u8,y:&'y u8)->(&'y u8,&'x u8){(y,x)}}fn main(){let x=83;let y=89;let pair=<u16 as Make>::make(&x,&y);check(*pair.0==89);check(*pair.1==83);}",
        false,
    ),
    (
        "precise-method",
        "trait Make{fn make<'a>(x:&'a u8)->impl Copy+use<Self>;}impl Make for u8{fn make<'a>(x:&'a u8)->impl Copy+use<>{7u8}}fn main(){let value;{let x=1;value=<u8 as Make>::make(&x);}let _=(value,value);}",
        false,
    ),
    (
        "drop-result",
        "trait Value{}struct S;impl Value for S{}impl Drop for S{fn drop(&mut self){check(false);}}trait Make{fn make()->impl Value;}impl Make for u8{fn make()->impl Value{S}}fn main(){let _value=<u8 as Make>::make();}",
        true,
    ),
    (
        "parent-and-const",
        "trait Make<T:Copy>{fn make<const N:usize>(x:T)->impl Copy;}impl<T:Copy>Make<T> for u8{fn make<const N:usize>(x:T)->[T;N]{[x;N]}}fn main(){let a:[u16;3]=<u8 as Make<u16>>::make::<3>(61);check(a[2]==61);}",
        false,
    ),
    (
        "stronger-output",
        "trait Value{fn value(&self)->u8;}impl Value for u8{fn value(&self)->u8{*self}}trait Make{fn make()->impl Copy;}impl Make for u16{fn make()->impl Copy+Value{67u8}}fn main(){check(<u16 as Make>::make().value()==67);}",
        false,
    ),
    (
        "refined",
        "trait Make{fn make()->impl Copy;}impl Make for u8{fn make()->u16{37}}fn main(){let value:u16=<u8 as Make>::make();check(value==37);}",
        false,
    ),
    (
        "opaque",
        "trait Value{fn value(&self)->u8;}impl Value for u8{fn value(&self)->u8{*self}}trait Make{fn make()->impl Value;}impl Make for u16{fn make()->impl Value{41u8}}fn main(){check(<u16 as Make>::make().value()==41);}",
        false,
    ),
    (
        "generic-caller",
        "trait Value{fn value(&self)->u8;}impl Value for u8{fn value(&self)->u8{*self}}trait Make{fn make()->impl Value;}impl Make for u16{fn make()->impl Value{43u8}}fn run<T:Make>()->u8{T::make().value()}fn main(){check(run::<u16>()==43);}",
        false,
    ),
    (
        "method-parameter",
        "trait Make{fn make<T:Copy>(x:T)->impl Copy;}impl Make for u8{fn make<T:Copy>(x:T)->T{x}}fn main(){let x:u16=<u8 as Make>::make(47u16);check(x==47);}",
        false,
    ),
    (
        "reference",
        "trait Make{fn make<'a>(x:&'a u8)->impl Copy+'a;}impl Make for u8{fn make<'a>(x:&'a u8)->&'a u8{x}}fn main(){let x=53;check(*<u8 as Make>::make(&x)==53);}",
        false,
    ),
    (
        "pair",
        "trait Make{fn make()->(impl Copy,impl Copy);}impl Make for u8{fn make()->(u8,bool){(59,true)}}fn main(){let pair=<u8 as Make>::make();check(pair.0==59);check(pair.1);}",
        false,
    ),
];

#[test]
fn selected_results_match_independent_source_outcomes() -> Result<(), Box<dyn std::error::Error>> {
    let mut failures = Vec::new();
    for &(name, body, _) in EXECUTION {
        let source = format!("{}{body}", native::CHECK);
        rustc::qualify(&format!("selected-result-{name}"), &source, true)?;
        let result = semantic::analyze(&source).and_then(|(mut report, mut facts)| {
            let entry =
                rustgm::sema::entry::select(&mut report, &facts).map_err(|e| e.to_string())?;
            rustgm::lower::instances::build(&report, &mut facts, entry).map_err(|e| e.to_string())
        });
        if let Err(error) = result {
            failures.push(format!("{name}: {error}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn selected_results_execute_like_rustc() -> Result<(), Box<dyn std::error::Error>> {
    native::check_cases("trait-return-results/native", EXECUTION)
}

#[test]
fn retained_result_inputs_still_owe_their_contract() -> Result<(), Box<dyn std::error::Error>> {
    for (name, source) in [
        (
            "inferred-copy-debt",
            "struct S;trait Make{fn make<T:Copy>()->impl Copy;}impl Make for u8{fn make<T:Copy>()->[T;0]{[]}}fn main(){let a:[S;0]=<u8 as Make>::make();let _=a;}",
        ),
        (
            "refined-reference-escape",
            "trait Make{fn make<'a>(x:&'a u8)->impl Copy+'a;}impl Make for u8{fn make<'a>(x:&'a u8)->&'a u8{x}}fn main(){let x;{let y=1;x=<u8 as Make>::make(&y);}let _=x;}",
        ),
        (
            "opaque-reference-escape",
            "trait Make{fn make<'a>(x:&'a u8)->impl Copy+'a;}impl Make for u8{fn make<'a>(x:&'a u8)->impl Copy+'a{x}}fn main(){let x;{let y=1;x=<u8 as Make>::make(&y);}let _=x;}",
        ),
    ] {
        rustc::qualify(name, source, false)?;
        let result = semantic::analyze(source).and_then(|(mut report, mut facts)| {
            let entry =
                rustgm::sema::entry::select(&mut report, &facts).map_err(|e| e.to_string())?;
            rustgm::lower::instances::build(&report, &mut facts, entry).map_err(|e| e.to_string())
        });
        assert!(result.is_err(), "{name}: unproved caller contract accepted");
    }
    Ok(())
}
