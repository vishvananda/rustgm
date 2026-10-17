//! Independently qualified projected fields retain their nominal lifetime contracts.
#[path = "harness/native_references.rs"]
mod native;
#[path = "helpers/qualify.rs"]
mod rustc;
#[path = "support/semantic.rs"]
mod semantic;
const CASES: &[(&str, &str, bool, bool)] = &[
    (
        "reference-field",
        "trait A{type Out:Copy;}struct S<'a,T:A>{value:&'a<T as A>::Out}fn main(){}\n",
        true,
        false,
    ),
    (
        "reference-field-executed",
        "trait A{type Out:Copy;}struct S<'a,T:A>{value:&'a<T as A>::Out}impl A for u8{type Out=u16;}fn main(){let x=73u16;let s=S::<u8>{value:&x};check(*s.value==73);}\n",
        true,
        true,
    ),
    (
        "ordinary-reference-field",
        "struct S<'a,T>{value:&'a T}fn main(){let x=73u16;let s=S{value:&x};check(*s.value==73);}\n",
        true,
        false,
    ),
    (
        "aliased-reference-field",
        "trait A{type Out:Copy;}type R<'a,T>=&'a<T as A>::Out;struct S<'a,T:A>{value:R<'a,T>}fn main(){}\n",
        true,
        false,
    ),
    (
        "callback-field",
        "trait A{type Out:Copy;}struct S<'a,T:A>{value:fn(&'a<T as A>::Out)}fn main(){}\n",
        true,
        false,
    ),
    (
        "nested-nominal",
        "trait A{type Out:Copy;}struct S<'a,T:A>{value:&'a<T as A>::Out}struct W<'a,T:A>(S<'a,T>);impl A for u8{type Out=u16;}fn main(){let x=73u16;let w=W::<u8>(S{value:&x});check(*w.0.value==73);}\n",
        true,
        true,
    ),
    (
        "borrowed-result-field",
        "trait A{type Out:Copy;}struct S<'a,T:A>{value:&'a<T as A>::Out}impl<'a>A for &'a u8{type Out=&'a u8;}fn main(){let x=73u8;let p=&x;let s=S::<&u8>{value:&p};check(**s.value==73);}\n",
        true,
        true,
    ),
    (
        "escaping-field",
        "trait A{type Out:Copy;}struct S<'a,T:A>{value:&'a<T as A>::Out}impl A for u8{type Out=u16;}fn bad()->S<'static,u8>{let x=73u16;S{value:&x}}fn main(){}\n",
        false,
        false,
    ),
    (
        "output-does-not-bound-input",
        "trait A{type Out:Copy;}struct S<'a,T:A>{value:&'a<T as A>::Out}fn need<T:'static>(){}fn bad<T:A>(_:S<'static,T>){need::<T>();}fn main(){}\n",
        false,
        false,
    ),
    (
        "generic-field-forward",
        "trait A{type Out:Copy;}struct S<'a,T:A>{value:&'a<T as A>::Out}fn get<'a,T:A>(x:S<'a,T>)->&'a<T as A>::Out{x.value}fn main(){}\n",
        true,
        false,
    ),
    (
        "unbounded-static-field",
        "trait A{type Out:Copy;}struct S<T:A>{value:&'static<T as A>::Out}fn main(){}\n",
        false,
        false,
    ),
    (
        "input-static-field",
        "trait A{type Out:Copy;}struct S<T:A+'static>{value:&'static<T as A>::Out}fn main(){}\n",
        true,
        false,
    ),
];
#[test]
fn projected_nominal_fields_preserve_well_formedness() -> Result<(), Box<dyn std::error::Error>> {
    for &(name, body, accepted, _) in CASES {
        let source = format!("{}{body}", native::CHECK);
        rustc::qualify(&format!("nominal-projection-{name}"), &source, accepted)?;
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
fn projected_nominal_fields_execute() -> Result<(), Box<dyn std::error::Error>> {
    let cases: Vec<_> = CASES
        .iter()
        .filter_map(|&(name, body, _, execute)| execute.then_some((name, body, false)))
        .collect();
    native::check_cases("nominal-results", &cases)
}
