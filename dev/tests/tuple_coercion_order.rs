//! Tuple fields retain the context established by earlier coercions.
#[path = "harness/native_references.rs"]
mod native;
#[path = "helpers/qualify.rs"]
mod rustc;
#[path = "support/semantic.rs"]
mod semantic;
const CASES: &[(&str, &str, bool)] = &[
    (
        "ordinary-shared-tuple-second-reborrow",
        "type P<T>=(T,T);fn f(x:&mut u8,y:&mut u8){let p:P<_>=(x,y);*p.1=19;*y=73;}fn main(){}",
        true,
    ),
    (
        "ordinary-shared-tuple-first-moves",
        "type P<T>=(T,T);fn f(x:&mut u8,y:&mut u8){let p:P<_>=(x,y);*p.1=19;*x=73;}fn main(){}",
        false,
    ),
    (
        "ordinary-independent-tuple-moves",
        "fn f(x:&mut u8,y:&mut u8){let p:(_,_)=(x,y);*p.1=19;*y=73;}fn main(){}",
        false,
    ),
    (
        "known-element-identity-reborrow",
        "trait A<'a>{type Out;}impl<'a,T:Copy>A<'a>for T{type Out=&'a mut u8;}type P<'a,T>=(T,<T as A<'a>>::Out);fn f<'a>(x:&'a mut u8){let p:P<'_,_>=(0u16,x);let _:u16=p.0;*p.1=19;*x=73;}fn main(){}",
        true,
    ),
    (
        "fully-known-identity-reborrow",
        "trait A<'a>{type Out;}impl<'a,T:Copy>A<'a>for T{type Out=&'a mut u8;}type P<'a,T>=(T,<T as A<'a>>::Out);fn f<'a>(x:&'a mut u8){let p:P<'_,u16>=(0,x);let _:u16=p.0;*p.1=19;*x=73;}fn main(){}",
        true,
    ),
    (
        "nested-shared-tuple",
        "type P<T>=(T,(T,T));fn f(x:&mut u8,y:&mut u8,z:&mut u8){let p:P<_>=(x,(y,z));*p.1.0=19;*p.1.1=23;*y=73;*z=79;}fn main(){}",
        true,
    ),
    (
        "block-shared-tuple",
        "type P<T>=(T,T);fn f(x:&mut u8,y:&mut u8){let p:P<_>=({x},{y});*p.1=19;*y=73;}fn main(){}",
        true,
    ),
];
#[test]
fn tuple_coercion_order_matches_rustc() -> Result<(), Box<dyn std::error::Error>> {
    let mut failures = Vec::new();
    for &(name, source, accepted) in CASES {
        rustc::qualify(&format!("tuple-order-{name}"), source, accepted)?;
        let result = semantic::analyze(source).and_then(|(mut report, mut facts)| {
            let entry =
                rustgm::sema::entry::select(&mut report, &facts).map_err(|e| e.to_string())?;
            rustgm::lower::instances::build(&report, &mut facts, entry)
                .map(|_| ())
                .map_err(|e| e.to_string())
        });
        if result.is_ok() != accepted {
            failures.push(format!("{name}: {:?}", result.err()));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn tuple_coercions_preserve_reborrows_at_runtime() -> Result<(), Box<dyn std::error::Error>> {
    native::check_cases(
        "tuple-coercion-order",
        &[
            (
                "pair",
                "type P<T>=(T,T);fn f(x:&mut u8,y:&mut u8){let p:P<_>=(x,y);*p.0=17;*p.1=19;*y=73;}fn main(){let mut x=0;let mut y=0;f(&mut x,&mut y);check(x==17);check(y==73);}",
                false,
            ),
            (
                "block",
                "type P<T>=(T,T);fn f(x:&mut u8,y:&mut u8){let p:P<_>=({x},{y});*p.0=17;*p.1=19;*y=73;}fn main(){let mut x=0;let mut y=0;f(&mut x,&mut y);check(x==17);check(y==73);}",
                false,
            ),
            (
                "nested",
                "type P<T>=(T,(T,T));fn f(x:&mut u8,y:&mut u8,z:&mut u8){let p:P<_>=(x,(y,z));*p.0=17;*p.1.0=19;*p.1.1=23;*y=73;*z=79;}fn main(){let mut x=0;let mut y=0;let mut z=0;f(&mut x,&mut y,&mut z);check(x==17);check(y==73);check(z==79);}",
                false,
            ),
            (
                "projected",
                "trait A<'a>{type Out;}impl<'a,T:Copy>A<'a>for T{type Out=&'a mut u8;}type P<'a,T>=(T,<T as A<'a>>::Out);fn f<'a>(x:&'a mut u8){let p:P<'_,_>=(0u16,x);let _:u16=p.0;*p.1=19;*x=73;}fn main(){let mut x=0;f(&mut x);check(x==73);}",
                false,
            ),
        ],
    )
}
