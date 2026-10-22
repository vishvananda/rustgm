//! Stored projected callbacks retain captured input identity in their actual scopes.
use crate::native_references as native;
use crate::qualify as rustc;
use crate::semantic;
const CASES: &[(&str, &str, bool, bool)] = &[
    (
        "stored-siblings",
        "trait A<'a>{type Out:Copy;}impl<'a>A<'a>for u8{type Out=u16;}fn f<'a,T:A<'a>>(p:for<'b>fn(&'b u8,&'a<T as A<'a>>::Out),q:for<'b>fn(&'b u8,&'a<T as A<'a>>::Out))->for<'b>fn(&'b u8,&'a<T as A<'a>>::Out){let _=p;let x=q;x}fn point<'a,'b>(_:&'b u8,x:&'a u16){check(*x==73);}fn other(_: &u8,x:&u16){check(*x==74);}fn main(){let p=f::<u8>(other,point);p(&1,&73);}\n",
        true,
        true,
    ),
    (
        "discard-and-return",
        "trait A<'a>{type Out:Copy;}impl<'a>A<'a>for u8{type Out=u16;}fn f<'a,T:A<'a>>(p:for<'b>fn(&'b u8,&'a<T as A<'a>>::Out))->for<'b>fn(&'b u8,&'a<T as A<'a>>::Out){let _=p;p}fn point<'a,'b>(_:&'b u8,x:&'a u16){check(*x==73);}fn main(){let p=f::<u8>(point);p(&1,&73);}\n",
        true,
        true,
    ),
    (
        "stored-empty-binder",
        "trait A<'a>{type Out:Copy;}impl<'a>A<'a>for u8{type Out=u16;}fn f<'a,T:A<'a>>(p:fn(&'a<T as A<'a>>::Out))->fn(&'a<T as A<'a>>::Out){let x=p;x}fn point(x:&u16){check(*x==73);}fn main(){let p=f::<u8>(point);p(&73);}\n",
        true,
        true,
    ),
    (
        "stored-double-empty-binder",
        "trait A<'a>{type Out:Copy;}fn f<'a,T:A<'a>>(p:fn(fn(&'a<T as A<'a>>::Out)))->fn(fn(&'a<T as A<'a>>::Out)){let x=p;x}fn main(){}\n",
        true,
        false,
    ),
    (
        "stored-projected-subject",
        "trait A{type Out:Copy;}impl<'a>A for &'a u8{type Out=u16;}fn f<'a,T>(p:fn(&'a<&'a T as A>::Out))->fn(&'a<&'a T as A>::Out)where &'a T:A{let x=p;x}fn point(x:&u16){check(*x==73);}fn main(){let p=f::<u8>(point);p(&73);}\n",
        true,
        true,
    ),
    (
        "stored-mixed-trait-argument",
        "trait A<X>{type Out:Copy;}impl<'a>A<&'a u8>for u8{type Out=u16;}fn f<'a,T:A<&'a u8>>(p:fn(&'a<T as A<&'a u8>>::Out))->fn(&'a<T as A<&'a u8>>::Out){let x=p;x}fn point(x:&u16){check(*x==73);}fn main(){let p=f::<u8>(point);p(&73);}\n",
        true,
        true,
    ),
    (
        "stored-nested-trait-callable",
        "trait A<F>{type Out:Copy;}type F<'a>=for<'b>fn(&'b &'a u8);fn f<'a,T:A<F<'a>>>(p:for<'c>fn(&'c<T as A<F<'a>>>::Out))->for<'d>fn(&'d<T as A<F<'a>>>::Out){let x=p;x}fn main(){}\n",
        true,
        false,
    ),
    (
        "stored-associated-return",
        "trait A<'a>{type Out:Copy;}impl<'a>A<'a>for u8{type Out=u16;}fn f<'a,T:A<'a>>(p:fn(&'a u8)-><T as A<'a>>::Out)->fn(&'a u8)-><T as A<'a>>::Out{let x=p;x}fn point(_: &u8)->u16{73}fn main(){let p=f::<u8>(point);check(p(&1)==73);}\n",
        true,
        true,
    ),
    (
        "wrong-subject",
        "trait A<'a>{type Out:Copy;}fn f<'a,T:A<'a>,U:A<'a>>(p:fn(&'a<T as A<'a>>::Out))->fn(&'a<U as A<'a>>::Out){let x=p;x}fn main(){}\n",
        false,
        false,
    ),
    (
        "wrong-trait-input",
        "trait A<X>{type Out:Copy;}fn f<'a,T:A<u8>+A<u16>>(p:fn(&'a<T as A<u8>>::Out))->fn(&'a<T as A<u16>>::Out){let x=p;x}fn main(){}\n",
        false,
        false,
    ),
    (
        "wrong-associated-item",
        "trait A<'a>{type Out:Copy;type Other:Copy;}fn f<'a,T:A<'a>>(p:fn(&'a<T as A<'a>>::Out))->fn(&'a<T as A<'a>>::Other){let x=p;x}fn main(){}\n",
        false,
        false,
    ),
    (
        "one-way-input-lifetime",
        "trait A<X>{type Out:Copy;}fn f<'a:'b,'b,T:A<&'a u8>+A<&'b u8>>(p:fn(&'a<T as A<&'a u8>>::Out))->fn(&'b<T as A<&'b u8>>::Out){let x=p;x}fn main(){}\n",
        false,
        false,
    ),
];
#[test]
fn stored_callbacks_preserve_associated_input_identity() -> Result<(), Box<dyn std::error::Error>> {
    for &(name, body, accepted, _) in CASES {
        let source = format!("{}{body}", native::CHECK);
        rustc::qualify(&format!("stored-projection-{name}"), &source, accepted)?;
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
fn stored_callbacks_execute_with_independent_results() -> Result<(), Box<dyn std::error::Error>> {
    let cases: Vec<_> = CASES
        .iter()
        .filter_map(|&(name, body, _, execute)| execute.then_some((name, body, false)))
        .collect();
    native::check_cases("stored-projections", &cases)
}
