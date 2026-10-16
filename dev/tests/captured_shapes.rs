//! Scoped shape premises preserve local binders and complete captured inputs.
#[path = "harness/native_references.rs"]
mod native;
#[path = "helpers/qualify.rs"]
mod rustc;
#[path = "support/semantic.rs"]
mod semantic;
const CASES: &[(&str, &str, bool, bool)] = &[
    (
        "captured-function",
        "trait A<'a>{type Out:Copy;}fn get<'a,T:A<'a>>(p:for<'b>fn(&'b u8,&'a<T as A<'a>>::Out))->for<'c>fn(&'c u8,&'a<T as A<'a>>::Out){p}fn main(){}\n",
        true,
        false,
    ),
    (
        "captured-function-called",
        "trait A<'a>{type Out:Copy;}impl<'a>A<'a>for u8{type Out=u16;}fn get<'a,T:A<'a>>(p:for<'b>fn(&'b u8,&'a<T as A<'a>>::Out))->for<'c>fn(&'c u8,&'a<T as A<'a>>::Out){p}fn point<'a,'b>(_:&'b u8,x:&'a u16){check(*x==73);}fn main(){let p=get::<u8>(point);p(&1,&73);}\n",
        true,
        true,
    ),
    (
        "empty-binder-capture",
        "trait A<'a>{type Out:Copy;}impl<'a>A<'a>for u8{type Out=u16;}fn get<'a,T:A<'a>>(p:fn(&'a<T as A<'a>>::Out))->fn(&'a<T as A<'a>>::Out){p}fn point(x:&u16){check(*x==73);}fn main(){let p=get::<u8>(point);p(&73);}\n",
        true,
        true,
    ),
    (
        "double-empty-binder-capture",
        "trait A<'a>{type Out:Copy;}fn get<'a,T:A<'a>>(p:fn(fn(&'a<T as A<'a>>::Out)))->fn(fn(&'a<T as A<'a>>::Out)){p}fn main(){}\n",
        true,
        false,
    ),
    (
        "mixed-trait-argument-capture",
        "trait A<X>{type Out:Copy;}impl<'a>A<&'a u8>for u8{type Out=u16;}fn get<'a,T:A<&'a u8>>(p:fn(&'a<T as A<&'a u8>>::Out))->fn(&'a<T as A<&'a u8>>::Out){p}fn point(x:&u16){check(*x==73);}fn main(){let p=get::<u8>(point);p(&73);}\n",
        true,
        true,
    ),
    (
        "projected-subject-capture",
        "trait A{type Out:Copy;}impl<'a>A for &'a u8{type Out=u16;}fn get<'a,T>(p:fn(&'a<&'a T as A>::Out))->fn(&'a<&'a T as A>::Out)where &'a T:A{p}fn point(x:&u16){check(*x==73);}fn main(){let p=get::<u8>(point);p(&73);}\n",
        true,
        true,
    ),
    (
        "shifted-alias-capture",
        "trait A<'a>{type Out:Copy;}type Out<'a,T>=<T as A<'a>>::Out;fn get<'a,T:A<'a>>(p:fn(&'a Out<'a,T>))->fn(&'a Out<'a,T>){p}fn main(){}\n",
        true,
        false,
    ),
    (
        "wrong-outer-lifetime",
        "trait A<'a>{type Out:Copy;}fn bad<'a,'b,T:A<'a>>(_:fn(&'b<T as A<'b>>::Out)){}fn main(){}\n",
        false,
        false,
    ),
    (
        "wrong-subject-lifetime",
        "trait A{type Out:Copy;}fn bad<'a,'b,T>(_:fn(&'b<&'b T as A>::Out))where &'a T:A{}fn main(){}\n",
        false,
        false,
    ),
    (
        "wrong-trait-type-argument",
        "trait A<X>{type Out:Copy;}fn bad<'a,T:A<&'a u8>>(_:fn(&'a<T as A<&'a u16>>::Out)){}fn main(){}\n",
        false,
        false,
    ),
    (
        "sized-result-capture",
        "trait A<'a>{type Out:?Sized;}impl<'a>A<'a>for u8{type Out=u16;}fn get<'a,T:A<'a>>(p:fn(&'a[<T as A<'a>>::Out;1]))->fn(&'a[<T as A<'a>>::Out;1])where<T as A<'a>>::Out:Sized{p}fn point(x:&[u16;1]){check(x[0]==73);}fn main(){let p=get::<u8>(point);p(&[73]);}\n",
        true,
        true,
    ),
    (
        "missing-sized-capture",
        "trait A<'a>{type Out:?Sized;}fn get<'a,T:A<'a>>(p:fn(&'a[<T as A<'a>>::Out;1]))->fn(&'a[<T as A<'a>>::Out;1]){p}fn main(){}\n",
        false,
        false,
    ),
    (
        "nested-trait-callable-capture",
        "trait A<F>{type Out:Copy;}type F<'a>=for<'b>fn(&'b &'a u8);fn get<'a,T:A<F<'a>>>(p:for<'c>fn(&'c<T as A<F<'a>>>::Out))->for<'d>fn(&'d<T as A<F<'a>>>::Out){p}fn main(){}\n",
        true,
        false,
    ),
    (
        "wrong-nested-trait-callable-capture",
        "trait A<F>{type Out:Copy;}type F<'a>=for<'b>fn(&'b &'a u8);fn get<'a,'b,T:A<F<'a>>>(p:for<'c>fn(&'c<T as A<F<'b>>>::Out))->for<'d>fn(&'d<T as A<F<'b>>>::Out){p}fn main(){}\n",
        false,
        false,
    ),
    (
        "normalized-sized-call",
        "trait A{type Out:?Sized;}impl A for u8{type Out=u16;}fn need<T:A>()where<T as A>::Out:Sized{}fn main(){need::<u8>();}\n",
        true,
        false,
    ),
    (
        "normalized-sized-forward",
        "trait A{type Out:?Sized;}impl A for u8{type Out=u16;}fn need<T:A>()where<T as A>::Out:Sized{}fn forward<T:A>()where<T as A>::Out:Sized{need::<T>();}fn main(){forward::<u8>();}\n",
        true,
        false,
    ),
    (
        "normalized-unsized-call",
        "trait A{type Out:?Sized;}impl A for u8{type Out=str;}fn need<T:A>()where<T as A>::Out:Sized{}fn main(){need::<u8>();}\n",
        false,
        false,
    ),
];
#[test]
fn captured_shapes_and_sized_call_results_retain_their_permissions()
-> Result<(), Box<dyn std::error::Error>> {
    for &(name, body, accepted, _) in CASES {
        let source = format!("{}{body}", native::CHECK);
        rustc::qualify(&format!("captured-shape-{name}"), &source, accepted)?;
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
fn captured_shapes_execute_with_independent_results() -> Result<(), Box<dyn std::error::Error>> {
    let cases: Vec<_> = CASES
        .iter()
        .filter_map(|&(name, body, _, execute)| execute.then_some((name, body, false)))
        .collect();
    native::check_cases("captured-shapes", &cases)
}
