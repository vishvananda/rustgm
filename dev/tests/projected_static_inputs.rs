//! Completed caller proofs preserve static implementation inputs across instances.
use crate::native_references as native;
use crate::qualify as rustc;
use crate::semantic;

const CASES: &[(&str, &str, bool)] = &[
    (
        "declared-copy-value",
        "trait A{type Out:Copy;}impl A for u8{type Out=u16;}fn dup<T:A>(x:<T as A>::Out)->(<T as A>::Out,<T as A>::Out){(x,x)}fn main(){let x=dup::<u8>(73);check(x.0==73&&x.1==73);}\n",
        true,
    ),
    (
        "erased-static-accepted",
        "trait A{type Out:Copy;}impl<'a:'static>A for &'a u8{type Out=u16;}fn f<T:A>(_:T,x:<T as A>::Out)->(<T as A>::Out,<T as A>::Out){(x,x)}static X:u8=1;fn main(){let x=f(&X,73);check(x.0==73&&x.1==73);}\n",
        true,
    ),
    (
        "erased-static-rejected",
        "trait A{type Out:Copy;}impl<'a:'static>A for &'a u8{type Out=u16;}fn f<T:A>(_:T,x:<T as A>::Out)->(<T as A>::Out,<T as A>::Out){(x,x)}fn main(){let y=1;let x=f(&y,73);check(x.0==73&&x.1==73);}\n",
        false,
    ),
    (
        "erased-formation-accepted",
        "trait A{type Out:Copy;}impl<'a:'static>A for &'a u8{type Out=u16;}fn need<X:Copy>(){}fn f<T:A>(_:T){need::<<T as A>::Out>();}static X:u8=1;fn main(){f(&X);}\n",
        true,
    ),
    (
        "erased-formation-rejected",
        "trait A{type Out:Copy;}impl<'a:'static>A for &'a u8{type Out=u16;}fn need<X:Copy>(){}fn f<T:A>(_:T){need::<<T as A>::Out>();}fn main(){let x=1;f(&x);}\n",
        false,
    ),
    (
        "explicit-static",
        "trait A{type Out:Copy;}impl<'a:'static>A for &'a u8{type Out=u16;}fn f<T:A>(_:T,x:<T as A>::Out)->(<T as A>::Out,<T as A>::Out){(x,x)}static X:u8=1;fn main(){let x=f::<&'static u8>(&X,73);check(x.0==73&&x.1==73);}\n",
        true,
    ),
    (
        "reified-function",
        "trait A{type Out:Copy;}impl<'a:'static>A for &'a u8{type Out=u16;}fn f<T:A>(_:T,x:<T as A>::Out)->(<T as A>::Out,<T as A>::Out){(x,x)}static X:u8=1;fn main(){let p:fn(&'static u8,u16)->(u16,u16)=f::<&u8>;let x=p(&X,73);check(x.0==73&&x.1==73);}\n",
        true,
    ),
    (
        "stored-item-local-rejected",
        "trait A{type Out:Copy;}impl<'a:'static>A for &'a u8{type Out=u16;}fn f<T:A>(_:T,x:<T as A>::Out)->(<T as A>::Out,<T as A>::Out){(x,x)}static X:u8=1;fn main(){let p=f::<&u8>;let local=1;let x=p(&local,73);check(x.0==73);}\n",
        false,
    ),
    (
        "nested-generic-call",
        "trait A{type Out:Copy;}impl<'a:'static>A for &'a u8{type Out=u16;}fn f<T:A>(_:T,x:<T as A>::Out)->(<T as A>::Out,<T as A>::Out){(x,x)}static X:u8=1;fn wrap<T:A>(x:T,y:<T as A>::Out)->(<T as A>::Out,<T as A>::Out){f(x,y)}fn main(){let x=wrap(&X,73);check(x.0==73&&x.1==73);}\n",
        true,
    ),
    (
        "two-static-inputs",
        "trait A{type Out:Copy;}impl<'a:'static>A for &'a u8{type Out=u16;}fn f<T:A>(_:T,x:<T as A>::Out)->(<T as A>::Out,<T as A>::Out){(x,x)}static X:u8=1;static Y:u8=2;fn main(){let x=f(&X,73);let y=f(&Y,9);check(x.0==73&&y.1==9);}\n",
        true,
    ),
    (
        "separate-caller-bodies",
        "trait A{type Out:Copy;}impl<'a:'static>A for &'a u8{type Out=u16;}fn f<T:A>(_:T,x:<T as A>::Out)->(<T as A>::Out,<T as A>::Out){(x,x)}static X:u8=1;static Y:u8=2;fn left()->u16{f(&X,73).0}fn right()->u16{f(&Y,9).1}fn main(){check(left()==73&&right()==9);}\n",
        true,
    ),
    (
        "proved-universal",
        "trait A{type Out:Copy;}impl<'a:'static>A for &'a u8{type Out=u16;}fn f<T:A>(_:T,x:<T as A>::Out)->(<T as A>::Out,<T as A>::Out){(x,x)}static X:u8=1;fn g<'a:'static>(x:&'a u8)->u16{f(x,73).0}fn main(){check(g(&X)==73);}\n",
        true,
    ),
    (
        "unproved-universal",
        "trait A{type Out:Copy;}impl<'a:'static>A for &'a u8{type Out=u16;}fn f<T:A>(_:T,x:<T as A>::Out)->(<T as A>::Out,<T as A>::Out){(x,x)}static X:u8=1;fn g<'a>(x:&'a u8)->u16{f(x,73).0}fn main(){check(g(&X)==73);}\n",
        false,
    ),
    (
        "unrelated-local-retained",
        "trait A{type Out:Copy;}impl<'a:'static>A for &'a u8{type Out=u16;}fn f<T:A>(_:T,x:<T as A>::Out)->(<T as A>::Out,<T as A>::Out){(x,x)}static X:u8=1;fn id<T>(x:T)->T{x}fn main(){let local=9;let y=id(&local);let x=f(&X,73);check(*y==9&&x.0==73);}\n",
        true,
    ),
    (
        "explicit-static-constant",
        "trait A{type Out:Copy;}impl<'a:'static>A for &'a u8{type Out=u16;}fn f<T:A>(_:T,x:<T as A>::Out)->(<T as A>::Out,<T as A>::Out){(x,x)}static X:u8=1;const P:fn(&'static u8,u16)->(u16,u16)=f::<&'static u8>;fn main(){let x=P(&X,73);check(x.0==73&&x.1==73);}\n",
        true,
    ),
    (
        "nested-binder-preserved",
        "trait A{type Out:Copy;}impl<'a:'static>A for &'a u8{type Out=u16;}fn f<T:A>(_:T,x:<T as A>::Out,p:for<'a>fn(&'a u16)->&'a u16)->(<T as A>::Out,<T as A>::Out){let y=73u16;check(*p(&y)==73);(x,x)}fn id(x:&u16)->&u16{x}fn g<'a:'static>(x:&'a u8)->u16{f(x,73,id).0}static X:u8=1;fn main(){check(g(&X)==73);}\n",
        true,
    ),
];

#[test]
fn caller_static_equalities_preserve_implementation_debts() -> Result<(), Box<dyn std::error::Error>>
{
    for &(name, body, accepted) in CASES {
        let source = format!("{}{body}", native::CHECK);
        rustc::qualify(&format!("static-target-{name}"), &source, accepted)?;
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
fn proved_static_targets_execute_across_bodies_and_binders()
-> Result<(), Box<dyn std::error::Error>> {
    let cases: Vec<_> = CASES
        .iter()
        .filter_map(|&(name, body, accepted)| accepted.then_some((name, body, false)))
        .collect();
    native::check_cases("projected-static-inputs", &cases)
}

#[test]
fn repeated_static_inputs_fit_the_runtime_region_budget() -> Result<(), Box<dyn std::error::Error>>
{
    let mut source = String::from(
        "trait A{type Out:Copy;}impl<'a:'static>A for &'a u8{type Out=u16;}static X:u8=1;fn f<T:A>(_:T,x:<T as A>::Out)->(<T as A>::Out,<T as A>::Out){(x,x)}fn main(){",
    );
    for _ in 0..1024 {
        source.push_str("let _=f(&X,73);");
    }
    source.push_str("}\n");
    rustc::qualify("static-region-scaling", &source, true)?;
    let (mut report, mut facts) = semantic::analyze(&source)?;
    let entry = rustgm::sema::entry::select(&mut report, &facts)?;
    rustgm::lower::instances::build(&report, &mut facts, entry)?;
    Ok(())
}
