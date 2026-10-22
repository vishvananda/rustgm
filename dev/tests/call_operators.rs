//! Callable invocation owns its receiver permission, input equations and result lifetime.
use crate::native_references as native;
use crate::qualify as rustc;
use crate::semantic;
const CASES: &[(&str, &str, bool)] = &[
    (
        "shared-item",
        "fn id(x:u8)->u8{x+1}fn apply<F:Fn(u8)->u8>(f:F)->u8{f(6)}fn main(){let _=apply(id);}",
        true,
    ),
    (
        "mutable-item",
        "fn id(x:u8)->u8{x+1}fn apply<F:FnMut(u8)->u8>(mut f:F)->u8{f(6)}fn main(){let _=apply(id);}",
        true,
    ),
    (
        "once-item",
        "fn id(x:u8)->u8{x+1}fn apply<F:FnOnce(u8)->u8>(f:F)->u8{f(6)}fn main(){let _=apply(id);}",
        true,
    ),
    (
        "pointer-reference",
        "fn id(x:u8)->u8{x}fn main(){let f:fn(u8)->u8=id;let _=(&f)(7);}",
        true,
    ),
    (
        "mutable-pointer-reference",
        "fn id(x:u8)->u8{x}fn main(){let mut f:fn(u8)->u8=id;let r=&mut f;let _=r(7);let _=r(8);}",
        true,
    ),
    (
        "shared-borrow",
        "fn apply<F:Fn(u8)->u8>(f:&F)->u8{f(6)+f(7)}fn main(){}",
        true,
    ),
    (
        "mutable-borrow",
        "fn apply<F:FnMut(u8)->u8>(f:&mut F)->u8{f(6)+f(7)}fn main(){}",
        true,
    ),
    (
        "mutable-binding-required",
        "fn apply<F:FnMut(u8)->u8>(f:F)->u8{f(6)}fn main(){}",
        false,
    ),
    (
        "mutable-through-shared",
        "fn apply<F:FnMut(u8)->u8>(f:&F)->u8{f(6)}fn main(){}",
        false,
    ),
    (
        "once-through-shared",
        "fn apply<F:FnOnce(u8)->u8>(f:&F)->u8{f(6)}fn main(){}",
        false,
    ),
    (
        "once-through-mutable",
        "fn apply<F:FnOnce(u8)->u8>(f:&mut F)->u8{f(6)}fn main(){}",
        false,
    ),
    (
        "once-consumed",
        "fn apply<F:FnOnce(u8)->u8>(f:F)->u8{f(6)+f(7)}fn main(){}",
        false,
    ),
    (
        "once-copy-repeated",
        "fn apply<F:FnOnce(u8)->u8+Copy>(f:F)->u8{f(6)+f(7)}fn main(){}",
        true,
    ),
    (
        "wrong-argument",
        "fn apply<F:Fn(u8)->u8>(f:F)->u8{f(6u16)}fn main(){}",
        false,
    ),
    (
        "wrong-arity",
        "fn apply<F:Fn(u8)->u8>(f:F)->u8{f(6,7)}fn main(){}",
        false,
    ),
    (
        "wrong-result",
        "fn apply<F:Fn(u8)->u16>(f:F)->u8{f(6)}fn main(){}",
        false,
    ),
    (
        "universal-input",
        "fn apply<F:for<'a>Fn(&'a u8)->&'a u8>(f:F,x:&u8)->&u8{f(x)}fn main(){}",
        true,
    ),
    (
        "static-input-reject",
        "fn apply<F:Fn(&'static u8)->&'static u8>(f:F,x:&u8)->&u8{f(x)}fn main(){}",
        false,
    ),
    (
        "universal-output-escape",
        "fn apply<F:for<'a>Fn(&'a u8)->&'a u8>(f:F)->&'static u8{let x=7;f(&x)}fn main(){}",
        false,
    ),
    (
        "associated-result",
        "trait A{type T;}fn apply<X:A,F:Fn()->X::T>(f:F)->X::T{f()}fn main(){}",
        true,
    ),
    (
        "tuple-single-input",
        "fn apply<F:Fn((u8,u16))->u16>(f:F)->u16{f((7,9))}fn main(){}",
        true,
    ),
    (
        "empty-input",
        "fn apply<F:Fn()->u8>(f:F)->u8{f()}fn main(){}",
        true,
    ),
    (
        "return-callable",
        "fn apply<F:Fn()->G,G:Fn()->u8>(f:F)->u8{f()()}fn main(){}",
        true,
    ),
    (
        "supertrait-call",
        "trait A:Fn(u8)->u8{}fn apply<F:A>(f:F)->u8{f(7)}fn main(){}",
        true,
    ),
    (
        "nested-reference",
        "fn apply<F:FnMut(u8)->u8>(f: &mut &mut F)->u8{f(7)}fn main(){}",
        true,
    ),
    (
        "shared-input-reference",
        "fn apply<F:Fn(&u8)->u8>(f:F)->u8{let x=7;f(&x)}fn main(){}",
        true,
    ),
    (
        "multiple-call-inputs",
        "fn apply<F:Fn(u8)->u8+Fn(u16)->u16>(f:F)->u8{f(7u8)}fn main(){}",
        true,
    ),
    (
        "different-call-kinds",
        "fn apply<F:Fn(u8)->u8+FnMut(u16)->u16>(mut f:F)->u16{f(7u16)}fn main(){}",
        false,
    ),
    (
        "shared-through-mutable",
        "fn apply<F:Fn(u8)->u8>(f:&mut F)->u8{f(7)}fn main(){}",
        true,
    ),
    (
        "explicit-reference-bound",
        "fn apply<F:?Sized>(f:&F)->u8 where for<'a>&'a F:Fn()->u8{f()}fn main(){}",
        false,
    ),
    (
        "explicit-mutable-reference-bound",
        "fn apply<F>(f:&mut F)->u8 where for<'a>&'a mut F:FnMut()->u8{f()}fn main(){}",
        false,
    ),
    (
        "explicit-mutable-reference-binding",
        "fn apply<F>(mut f:&mut F)->u8 where for<'a>&'a mut F:FnMut()->u8{f()}fn main(){}",
        false,
    ),
    (
        "once-reference-bound",
        "fn apply<'a,F>(f:&'a F)->u8 where &'a F:FnOnce()->u8{f()}fn main(){}",
        false,
    ),
    (
        "once-mutable-reference-twice",
        "fn apply<'a,F>(f:&'a mut F)->u8 where &'a mut F:FnOnce()->u8{f()+f()}fn main(){}",
        false,
    ),
    (
        "raw-function-pointer",
        "fn apply(f:*const fn()->u8)->u8{f()}fn main(){}",
        false,
    ),
    (
        "unsafe-reference",
        "fn apply(f:&unsafe fn()->u8)->u8{f()}fn main(){}",
        false,
    ),
    (
        "argument-context",
        "fn apply<F:Fn(u8)->u16>(f:F)->u16{let x=7;f(x)}fn main(){}",
        true,
    ),
    (
        "generic-output",
        "fn apply<F:Fn()->T,T>(f:F)->T{f()}fn main(){}",
        true,
    ),
    (
        "unsized-callable",
        "fn apply<F:Fn(u8)->u8+?Sized>(f:&F)->u8{f(7)}fn main(){}",
        true,
    ),
    (
        "opaque-function-item",
        "fn item(x:u8)->u8{x+1}fn make()->impl Fn(u8)->u8{item}fn main(){let f=make();let _=f(7);}",
        true,
    ),
    (
        "opaque-function-pointer",
        "fn item(x:u8)->u8{x+1}fn make()->impl FnMut(u8)->u8{item as fn(u8)->u8}fn main(){let mut f=make();let _=f(7);}",
        true,
    ),
    (
        "nonconst-operator",
        "const fn apply<F:Fn()->u8+Copy>(f:F)->u8{f()}fn main(){}",
        false,
    ),
    (
        "nested-function-argument",
        "fn item(x:&u8)->&u8{x}fn apply<F:Fn(for<'a>fn(&'a u8)->&'a u8)->u8>(f:F)->u8{f(item)}fn main(){}",
        true,
    ),
    (
        "opaque-once-item",
        "fn item(x:u8)->u8{x+1}fn make()->impl FnOnce(u8)->u8{item}fn main(){let f=make();let _=f(7);}",
        true,
    ),
    (
        "opaque-once-pointer",
        "fn item(x:u8)->u8{x+1}fn make()->impl FnOnce(u8)->u8{item as fn(u8)->u8}fn main(){let f=make();let _=f(7);}",
        true,
    ),
    (
        "projected-subject-bound",
        "trait A{type F;}fn apply<T:A>(f:T::F)->u8 where T::F:Fn(u8)->u8{f(7)}fn main(){}",
        true,
    ),
    (
        "projected-subject-contract",
        "trait A{type F:Fn(u8)->u8;}fn apply<T:A>(f:T::F)->u8{f(7)}fn main(){}",
        true,
    ),
    (
        "projected-mutable-contract",
        "trait A{type F:FnMut(u8)->u8;}fn apply<T:A>(mut f:T::F)->u8{f(7)}fn main(){}",
        true,
    ),
    (
        "projected-mutable-binding",
        "trait A{type F:FnMut(u8)->u8;}fn apply<T:A>(f:T::F)->u8{f(7)}fn main(){}",
        false,
    ),
    (
        "projected-once-contract",
        "trait A{type F:FnOnce(u8)->u8;}fn apply<T:A>(f:T::F)->u8{f(7)}fn main(){}",
        true,
    ),
    (
        "projected-once-consumed",
        "trait A{type F:FnOnce(u8)->u8;}fn apply<T:A>(f:T::F)->u8{f(7)+f(9)}fn main(){}",
        false,
    ),
    (
        "closed-projected-function",
        "trait A{type F;}struct S;impl A for S{type F=fn(u8)->u8;}fn apply(f:<S as A>::F)->u8{f(7)}fn main(){}",
        true,
    ),
];
#[test]
fn callable_receivers_preserve_permissions_and_result_lifetimes()
-> Result<(), Box<dyn std::error::Error>> {
    let mut failures = Vec::new();
    for &(name, source, accepted) in CASES {
        rustc::qualify(&format!("call-operator-{name}"), source, accepted)?;
        let result = checked(source);
        if result.is_ok() != accepted {
            failures.push(format!("{name}: {:?}", result.err()));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
fn checked(source: &str) -> Result<(), String> {
    let (mut report, mut facts) = semantic::analyze(source)?;
    let entry = rustgm::sema::entry::select(&mut report, &facts).map_err(|e| e.to_string())?;
    rustgm::lower::instances::build(&report, &mut facts, entry)
        .map(|_| ())
        .map_err(|e| e.to_string())
}
#[test]
#[ignore = "requires the qualified CPPGM backend"]
fn callable_operators_execute_through_the_existing_abi() -> Result<(), Box<dyn std::error::Error>> {
    native::check_cases(
        "pa24-invocations/native",
        &[
            (
                "shared-item",
                "fn id(x:u8)->u8{x+1}fn apply<F:Fn(u8)->u8>(f:F)->u8{f(6)+f(7)}fn main(){check(apply(id)==15);}",
                false,
            ),
            (
                "mutable-pointer",
                "fn id(x:u8)->u8{x+1}fn apply<F:FnMut(u8)->u8>(mut f:F)->u8{f(6)+f(7)}fn main(){let f:fn(u8)->u8=id;check(apply(f)==15);}",
                false,
            ),
            (
                "consumed-item",
                "fn id(x:u8)->u8{x+1}fn apply<F:FnOnce(u8)->u8>(f:F)->u8{f(6)}fn main(){check(apply(id)==7);}",
                false,
            ),
            (
                "borrowed-mutable",
                "fn id(x:u8)->u8{x+1}fn apply<F:FnMut(u8)->u8>(f:&mut F)->u8{f(6)+f(7)}fn main(){let mut f:fn(u8)->u8=id;check(apply(&mut f)==15);check(f(8)==9);}",
                false,
            ),
            (
                "reference-pointer",
                "fn id(x:u8)->u8{x+1}fn main(){let f:fn(u8)->u8=id;let r=&&f;check(r(6)==7);}",
                false,
            ),
            (
                "universal-result",
                "fn id(x:&u8)->&u8{x}fn apply<F:for<'a>Fn(&'a u8)->&'a u8>(f:F,x:&u8)->&u8{f(x)}fn main(){let n=7;let r=apply(id,&n);check(*r==7);}",
                false,
            ),
            (
                "aggregate-constructor",
                "struct P(u8,u16);fn apply<F:Fn(u8,u16)->P>(f:F)->P{f(7,9)}fn main(){let p=apply(P);check(p.0==7&&p.1==9);}",
                false,
            ),
            (
                "aggregate-return",
                "fn item(a:[u8;3])->[u8;3]{[a[2],a[1],a[0]]}fn apply<F:FnOnce([u8;3])->[u8;3]>(f:F)->[u8;3]{f([2,5,9])}fn main(){let p=apply(item);check(p[0]==9&&p[2]==2);}",
                false,
            ),
            (
                "pointer-argument",
                "fn id(x:u8)->u8{x+1}fn item(f:fn(u8)->u8)->u8{f(7)}fn apply<F:Fn(fn(u8)->u8)->u8>(f:F)->u8{f(id)}fn main(){check(apply(item)==8);}",
                false,
            ),
            (
                "opaque-item",
                "fn item(x:u8)->u8{x+1}fn make()->impl Fn(u8)->u8{item}fn main(){let f=make();check(f(7)==8);}",
                false,
            ),
            (
                "opaque-pointer",
                "fn item(x:u8)->u8{x+1}fn make()->impl FnMut(u8)->u8{item as fn(u8)->u8}fn main(){let mut f=make();check(f(7)==8);check(f(9)==10);}",
                false,
            ),
            (
                "opaque-once-item",
                "fn item(x:u8)->u8{x+1}fn make()->impl FnOnce(u8)->u8{item}fn main(){let f=make();check(f(7)==8);}",
                false,
            ),
            (
                "opaque-once-pointer",
                "fn item(x:u8)->u8{x+1}fn make()->impl FnOnce(u8)->u8{item as fn(u8)->u8}fn main(){let f=make();check(f(7)==8);}",
                false,
            ),
            (
                "projected-shared",
                "trait A{type F:Fn(u8)->u8;}struct S;impl A for S{type F=fn(u8)->u8;}fn item(x:u8)->u8{x+1}fn apply<T:A>(f:T::F)->u8{f(7)+f(9)}fn main(){check(apply::<S>(item)==18);}",
                false,
            ),
            (
                "projected-mutable",
                "trait A{type F:FnMut(u8)->u8;}struct S;impl A for S{type F=fn(u8)->u8;}fn item(x:u8)->u8{x+1}fn apply<T:A>(mut f:T::F)->u8{f(7)+f(9)}fn main(){check(apply::<S>(item)==18);}",
                false,
            ),
            (
                "projected-once",
                "trait A{type F:FnOnce(u8)->u8;}struct S;impl A for S{type F=fn(u8)->u8;}fn item(x:u8)->u8{x+1}fn apply<T:A>(f:T::F)->u8{f(7)}fn main(){check(apply::<S>(item)==8);}",
                false,
            ),
        ],
    )
}
