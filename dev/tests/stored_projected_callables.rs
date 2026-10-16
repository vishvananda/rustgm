//! Stored callable signatures retain occurrence proofs and caller lifetime debts.
#[path = "harness/native_references.rs"]
mod native;
#[path = "helpers/qualify.rs"]
mod rustc;
#[path = "support/semantic.rs"]
mod semantic;

const CASES: &[(&str, &str, bool)] = &[
    (
        "direct-static",
        "trait A{type Out:Copy;}impl<'a:'static>A for &'a u8{type Out=u16;}fn f<T:A>(_:T,x:<T as A>::Out)->(<T as A>::Out,<T as A>::Out){(x,x)}static X:u8=1;fn main(){let x=f(&X,73);check(x.0==73&&x.1==73);}\n",
        true,
    ),
    (
        "stored-static",
        "trait A{type Out:Copy;}impl<'a:'static>A for &'a u8{type Out=u16;}fn f<T:A>(_:T,x:<T as A>::Out)->(<T as A>::Out,<T as A>::Out){(x,x)}static X:u8=1;fn main(){let p=f::<&u8>;let x=p(&X,73);check(x.0==73&&x.1==73);}\n",
        true,
    ),
    (
        "explicit-static",
        "trait A{type Out:Copy;}impl<'a:'static>A for &'a u8{type Out=u16;}fn f<T:A>(_:T,x:<T as A>::Out)->(<T as A>::Out,<T as A>::Out){(x,x)}static X:u8=1;fn main(){let p=f::<&'static u8>;let x=p(&X,73);check(x.0==73&&x.1==73);}\n",
        true,
    ),
    (
        "inferred-item",
        "trait A{type Out:Copy;}impl<'a:'static>A for &'a u8{type Out=u16;}fn f<T:A>(_:T,x:<T as A>::Out)->(<T as A>::Out,<T as A>::Out){(x,x)}static X:u8=1;fn main(){let p=f;let x=p(&X,73);check(x.0==73&&x.1==73);}\n",
        true,
    ),
    (
        "inferred-argument",
        "trait A{type Out:Copy;}impl<'a:'static>A for &'a u8{type Out=u16;}fn f<T:A>(_:T,x:<T as A>::Out)->(<T as A>::Out,<T as A>::Out){(x,x)}static X:u8=1;fn main(){let p=f::<_>;let x=p(&X,73);check(x.0==73&&x.1==73);}\n",
        true,
    ),
    (
        "copied-item",
        "trait A{type Out:Copy;}impl<'a:'static>A for &'a u8{type Out=u16;}fn f<T:A>(_:T,x:<T as A>::Out)->(<T as A>::Out,<T as A>::Out){(x,x)}static X:u8=1;fn main(){let p=f::<&u8>;let q=p;let x=q(&X,73);check(x.0==73&&x.1==73);}\n",
        true,
    ),
    (
        "tuple-item",
        "trait A{type Out:Copy;}impl<'a:'static>A for &'a u8{type Out=u16;}fn f<T:A>(_:T,x:<T as A>::Out)->(<T as A>::Out,<T as A>::Out){(x,x)}static X:u8=1;fn main(){let p=(f::<&u8>,73u16);let x=(p.0)(&X,p.1);check(x.0==73&&x.1==73);}\n",
        true,
    ),
    (
        "array-item",
        "trait A{type Out:Copy;}impl<'a:'static>A for &'a u8{type Out=u16;}fn f<T:A>(_:T,x:<T as A>::Out)->(<T as A>::Out,<T as A>::Out){(x,x)}static X:u8=1;fn main(){let p=[f::<&u8>;2];let x=p[1](&X,73);check(x.0==73&&x.1==73);}\n",
        true,
    ),
    (
        "joined-item",
        "trait A{type Out:Copy;}impl<'a:'static>A for &'a u8{type Out=u16;}fn f<T:A>(_:T,x:<T as A>::Out)->(<T as A>::Out,<T as A>::Out){(x,x)}static X:u8=1;fn main(){let p=if true{f::<&u8>}else{f::<&u8>};let x=p(&X,73);check(x.0==73&&x.1==73);}\n",
        true,
    ),
    (
        "reified-item",
        "trait A{type Out:Copy;}impl<'a:'static>A for &'a u8{type Out=u16;}fn f<T:A>(_:T,x:<T as A>::Out)->(<T as A>::Out,<T as A>::Out){(x,x)}static X:u8=1;fn main(){let p=f::<&u8>;let q:fn(&'static u8,u16)->(u16,u16)=p;let x=q(&X,73);check(x.0==73&&x.1==73);}\n",
        true,
    ),
    (
        "scalar-item",
        "trait A{type Out:Copy;}impl A for u8{type Out=u16;}fn f<T:A>(_:T,x:<T as A>::Out)->(<T as A>::Out,<T as A>::Out){(x,x)}fn main(){let p=f::<u8>;let x=p(1u8,73);check(x.0==73&&x.1==73);}\n",
        true,
    ),
    (
        "scalar-inferred-item",
        "trait A{type Out:Copy;}impl A for u8{type Out=u16;}fn f<T:A>(_:T,x:<T as A>::Out)->(<T as A>::Out,<T as A>::Out){(x,x)}fn main(){let p=f;let x=p(1u8,73);check(x.0==73&&x.1==73);}\n",
        true,
    ),
    (
        "scalar-inferred-argument",
        "trait A{type Out:Copy;}impl A for u8{type Out=u16;}fn f<T:A>(_:T,x:<T as A>::Out)->(<T as A>::Out,<T as A>::Out){(x,x)}fn main(){let p=f::<_>;let x=p(1u8,73);check(x.0==73&&x.1==73);}\n",
        true,
    ),
    (
        "nominal-item",
        "trait A{type Out:Copy;}impl<'a:'static>A for &'a u8{type Out=u16;}fn f<T:A>(_:T,x:<T as A>::Out)->(<T as A>::Out,<T as A>::Out){(x,x)}static X:u8=1;struct Holder<F>(F);fn main(){let p=Holder(f::<&u8>);let x=(p.0)(&X,73);check(x.0==73&&x.1==73);}\n",
        true,
    ),
    (
        "proved-universal",
        "trait A{type Out:Copy;}impl<'a:'static>A for &'a u8{type Out=u16;}fn f<T:A>(_:T,x:<T as A>::Out)->(<T as A>::Out,<T as A>::Out){(x,x)}static X:u8=1;fn g<'a:'static>(v:&'a u8)->u16{let p=f::<&'a u8>;p(v,73).0}fn main(){check(g(&X)==73);}\n",
        true,
    ),
    (
        "nested-binder",
        "trait A{type Out:Copy;}impl<'a:'static>A for &'a u8{type Out=u16;}fn f<T:A>(_:T,x:<T as A>::Out,p:for<'a>fn(&'a u16)->&'a u16)->(<T as A>::Out,<T as A>::Out){let y=73u16;check(*p(&y)==73);(x,x)}fn id(x:&u16)->&u16{x}static X:u8=1;fn main(){let p=f::<&u8>;let x=p(&X,73,id);check(x.0==73&&x.1==73);}\n",
        true,
    ),
    (
        "local-rejected",
        "trait A{type Out:Copy;}impl<'a:'static>A for &'a u8{type Out=u16;}fn f<T:A>(_:T,x:<T as A>::Out)->(<T as A>::Out,<T as A>::Out){(x,x)}static X:u8=1;fn main(){let p=f::<&u8>;let local=1;let x=p(&local,73);check(x.0==73&&x.1==73);}\n",
        false,
    ),
    (
        "inferred-local-rejected",
        "trait A{type Out:Copy;}impl<'a:'static>A for &'a u8{type Out=u16;}fn f<T:A>(_:T,x:<T as A>::Out)->(<T as A>::Out,<T as A>::Out){(x,x)}static X:u8=1;fn main(){let p=f;let local=1;let x=p(&local,73);check(x.0==73&&x.1==73);}\n",
        false,
    ),
    (
        "unused-local-rejected",
        "trait A{type Out:Copy;}impl<'a:'static>A for &'a u8{type Out=u16;}fn f<T:A>(_:T,x:<T as A>::Out)->(<T as A>::Out,<T as A>::Out){(x,x)}static X:u8=1;fn bad(){let p=f::<&u8>;let local=1;let _=p(&local,73);}fn main(){}\n",
        false,
    ),
    (
        "sibling-local-rejected",
        "trait A{type Out:Copy;}impl<'a:'static>A for &'a u8{type Out=u16;}fn f<T:A>(_:T,x:<T as A>::Out)->(<T as A>::Out,<T as A>::Out){(x,x)}static X:u8=1;fn main(){let p=f::<&u8>;let q=f::<&u8>;let _=p(&X,73);let local=1;let _=q(&local,73);}\n",
        false,
    ),
    (
        "unproved-universal",
        "trait A{type Out:Copy;}impl<'a:'static>A for &'a u8{type Out=u16;}fn f<T:A>(_:T,x:<T as A>::Out)->(<T as A>::Out,<T as A>::Out){(x,x)}static X:u8=1;fn g<'a>(v:&'a u8)->u16{let p=f::<&'a u8>;p(v,73).0}fn main(){check(g(&X)==73);}\n",
        false,
    ),
    (
        "missing-implementation",
        "trait A{type Out:Copy;}impl<'a:'static>A for &'a u8{type Out=u16;}fn f<T:A>(_:T,x:<T as A>::Out)->(<T as A>::Out,<T as A>::Out){(x,x)}static X:u8=1;fn main(){let _=f::<bool>;}\n",
        false,
    ),
];

#[test]
fn stored_items_normalize_before_proving_their_lifetime_debts()
-> Result<(), Box<dyn std::error::Error>> {
    for &(name, body, accepted) in CASES {
        let source = format!("{}{body}", native::CHECK);
        rustc::qualify(&format!("stored-projection-{name}"), &source, accepted)?;
        let typed = semantic::analyze(&source);
        assert_eq!(typed.is_ok(), name != "missing-implementation", "{name}");
        let result = typed.and_then(|(mut report, mut facts)| {
            let entry = rustgm::sema::entry::select(&mut report, &facts)
                .map_err(|error| error.to_string())?;
            rustgm::lower::instances::build(&report, &mut facts, entry)
                .map(|_| ())
                .map_err(|error| error.to_string())
        });
        assert_eq!(result.is_ok(), accepted, "{name}: {result:?}");
        if !accepted && name != "missing-implementation" {
            let error = result.err().ok_or("missing lifetime rejection")?;
            assert!(
                error.contains("required lifetime does not outlive its target"),
                "{name}: {error}"
            );
        }
    }
    Ok(())
}

#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn stored_projected_items_execute_with_independent_rustc_results()
-> Result<(), Box<dyn std::error::Error>> {
    let cases: Vec<_> = CASES
        .iter()
        .filter_map(|&(name, body, accepted)| accepted.then_some((name, body, false)))
        .collect();
    native::check_cases("stored-projected-callables", &cases)
}

#[test]
fn stored_static_projections_fit_the_runtime_storage_budget()
-> Result<(), Box<dyn std::error::Error>> {
    let mut source = String::from(
        "static X:u8=1;trait A{type Out:Copy;}impl<'a:'static>A for &'a u8{type Out=u16;}fn f<T:A>(_:T,x:<T as A>::Out)->(<T as A>::Out,<T as A>::Out){(x,x)}fn main(){",
    );
    for _ in 0..1024 {
        source.push_str("let p=f::<&u8>;let _=p(&X,73);");
    }
    source.push_str("}\n");
    rustc::qualify("stored-static-projection-scaling", &source, true)?;
    let (mut report, mut facts) = semantic::analyze(&source)?;
    let entry = rustgm::sema::entry::select(&mut report, &facts)?;
    rustgm::lower::instances::build(&report, &mut facts, entry)?;
    Ok(())
}
