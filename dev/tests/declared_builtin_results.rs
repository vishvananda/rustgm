//! Declared result promises reach body ownership without granting input abilities.
#[path = "harness/native_references.rs"]
mod native;
#[path = "helpers/qualify.rs"]
mod rustc;
#[path = "support/semantic.rs"]
mod semantic;

const CASES: &[(&str, &str, bool)] = &[
    (
        "declared-copy-value",
        "trait A{type Out:Copy;}impl A for u8{type Out=u16;}fn dup<T:A>(x:<T as A>::Out)->(<T as A>::Out,<T as A>::Out){(x,x)}fn main(){let x=dup::<u8>(73);check(x.0==73&&x.1==73);}\n",
        true,
    ),
    (
        "declared-copy-repeat",
        "trait A{type Out:Copy;}impl A for u8{type Out=u16;}fn dup<T:A>(x:<T as A>::Out)->[<T as A>::Out;2]{[x;2]}fn main(){let x=dup::<u8>(73);check(x[0]==73&&x[1]==73);}\n",
        true,
    ),
    (
        "declared-copy-predicate",
        "trait A{type Out:Copy;}impl A for u8{type Out=u16;}fn need<X:Copy>(x:X)->(X,X){(x,x)}fn dup<T:A>(x:<T as A>::Out)->(<T as A>::Out,<T as A>::Out){need(x)}fn main(){let x=dup::<u8>(73);check(x.0==73&&x.1==73);}\n",
        true,
    ),
    (
        "self-copy-is-not-result-copy",
        "trait A:Copy{type Out;}fn bad<T:A>(x:<T as A>::Out){let _=(x,x);}fn main(){}\n",
        false,
    ),
    (
        "other-member-is-not-copy",
        "trait A{type Out:Copy;type Other;}fn bad<T:A>(x:<T as A>::Other){let _=(x,x);}fn main(){}\n",
        false,
    ),
    (
        "other-input-is-not-copy",
        "trait A<U>{type Out;}fn bad<T:A<u8>+A<u16>>(x:<T as A<u16>>::Out)where<T as A<u8>>::Out:Copy{let _=(x,x);}fn main(){}\n",
        false,
    ),
    (
        "copy-is-not-parent-formation",
        "trait A{type Out;}fn bad<T>(x:<T as A>::Out)where<T as A>::Out:Copy{let _=(x,x);}fn main(){}\n",
        false,
    ),
    (
        "clone-is-not-copy",
        "trait A{type Out:Clone;}fn bad<T:A>(x:<T as A>::Out){let _=(x,x);}fn main(){}\n",
        false,
    ),
    (
        "result-copy-is-not-self-copy",
        "trait A{type Out:Copy;}fn bad<T:A>(x:T){let _=(x,x);}fn main(){}\n",
        false,
    ),
    (
        "copy-implies-clone",
        "trait A{type Out:Copy;}impl A for u8{type Out=u16;}fn need<X:Clone>(){}fn f<T:A>(_:T){need::<<T as A>::Out>();}fn main(){f(1u8);}\n",
        true,
    ),
    (
        "declared-clone-predicate",
        "trait A{type Out:Clone;}#[derive(Clone)]struct D(u8);impl A for u8{type Out=D;}fn need<X:Clone>(){}fn f<T:A>(_:T){need::<<T as A>::Out>();}fn main(){f(1u8);}\n",
        true,
    ),
    (
        "clone-cannot-prove-copy",
        "trait A{type Out:Clone;}fn need<X:Copy>(){}fn f<T:A>(){need::<<T as A>::Out>();}fn main(){}\n",
        false,
    ),
    (
        "noncopy-subject",
        "trait A{type Out:Copy;}struct D(u8);impl A for D{type Out=u16;}fn f<T:A>(x:<T as A>::Out)->(<T as A>::Out,<T as A>::Out){(x,x)}fn main(){let x=f::<D>(73);check(x.0==73&&x.1==73);}\n",
        true,
    ),
    (
        "unused-copy-body",
        "trait A{type Out:Copy;}fn f<T:A>(x:<T as A>::Out)->(<T as A>::Out,<T as A>::Out){(x,x)}fn main(){}\n",
        true,
    ),
    (
        "nested-repeat",
        "trait A{type Out:Copy;}impl A for u8{type Out=u16;}fn f<T:A>(x:<T as A>::Out)->[[<T as A>::Out;2];2]{[[x;2];2]}fn main(){let x=f::<u8>(73);check(x[0][0]==73&&x[1][1]==73);}\n",
        true,
    ),
    (
        "invalid-copy-implementation",
        "trait A{type Out:Copy;}struct D(u8);impl A for u8{type Out=D;}fn main(){}\n",
        false,
    ),
    (
        "invalid-clone-implementation",
        "trait A{type Out:Clone;}struct D(u8);impl A for u8{type Out=D;}fn main(){}\n",
        false,
    ),
    (
        "erased-static-rejected",
        "trait A{type Out:Copy;}impl<'a:'static>A for &'a u8{type Out=u16;}fn f<T:A>(_:T,x:<T as A>::Out)->(<T as A>::Out,<T as A>::Out){(x,x)}fn main(){let y=1;let x=f(&y,73);check(x.0==73&&x.1==73);}\n",
        false,
    ),
    (
        "relaxed-sized-copy",
        "trait A{type Out:?Sized+Copy;}impl A for u8{type Out=u16;}fn f<T:A>(x:<T as A>::Out)->[<T as A>::Out;2]{[x;2]}fn main(){let x=f::<u8>(73);check(x[0]==73&&x[1]==73);}\n",
        true,
    ),
];

#[test]
fn declared_result_abilities_preserve_input_and_formation_obligations()
-> Result<(), Box<dyn std::error::Error>> {
    for &(name, body, accepted) in CASES {
        let source = format!("{}{body}", native::CHECK);
        rustc::qualify(&format!("declared-ability-{name}"), &source, accepted)?;
        let result = semantic::analyze(&source).and_then(|(mut report, mut facts)| {
            let entry = rustgm::sema::entry::select(&mut report, &facts)
                .map_err(|error| error.to_string())?;
            rustgm::lower::instances::build(&report, &mut facts, entry)
                .map(|_| ())
                .map_err(|error| error.to_string())
        });
        assert_eq!(result.is_ok(), accepted, "{name}: {result:?}");
    }
    Ok(())
}

#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn declared_copy_values_and_clone_predicates_execute() -> Result<(), Box<dyn std::error::Error>> {
    let cases: Vec<_> = CASES
        .iter()
        .filter_map(|&(name, body, accepted)| {
            (accepted && name != "unused-copy-body").then_some((name, body, false))
        })
        .collect();
    native::check_cases("declared-result-abilities", &cases)
}
