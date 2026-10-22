//! Shared closed selection evidence never supplies a caller lifetime assumption.
use super::{native, object_outcome, prepared};

const CASES: &[(&str, &str, bool)] = &[
    (
        "static-repeat",
        "trait A<'a,'b>{const C:usize;}struct S;impl<'a:'b,'b>A<'a,'b>for S{const C:usize=3;}fn main(){check(<S as A<'static,'static>>::C==3);check(<S as A<'static,'static>>::C==3);}",
        false,
    ),
    (
        "paid-caller",
        "trait A<'a,'b>{const C:usize;}struct S;impl<'a:'b,'b>A<'a,'b>for S{const C:usize=3;}fn value<'a:'b,'b>(_:&'a u8,_:&'b u8)->usize{<S as A<'a,'b>>::C}fn main(){let x=1;check(value(&x,&x)==3);}",
        false,
    ),
    (
        "unpaid-caller-after-static",
        "trait A<'a,'b>{const C:usize;}struct S;impl<'a:'b,'b>A<'a,'b>for S{const C:usize=3;}fn value<'a,'b>(_:&'a u8,_:&'b u8)->usize{<S as A<'a,'b>>::C}fn main(){check(<S as A<'static,'static>>::C==3);}",
        true,
    ),
    (
        "equal-caller",
        "trait A<'a,'b>{const C:usize;}struct S;impl<'a:'b,'b>A<'a,'b>for S{const C:usize=3;}fn value<'a>(_:&'a u8)->usize{<S as A<'a,'a>>::C}fn main(){let x=1;check(value(&x)==3);}",
        false,
    ),
    (
        // An earlier, wider body must not move the caller's own region numbering.
        "equal-caller-after-wide-body",
        concat!(
            "fn wide(){let _r0=&0u8;let _r1=&1u8;let _r2=&2u8;let _r3=&3u8;let _r4=&4u8;let _r5=&5u8;let _r6=&6u8;let _r7=&7u8;let _r8=&8u8;let _r9=&9u8;",
            "let _r10=&10u8;let _r11=&11u8;let _r12=&12u8;let _r13=&13u8;let _r14=&14u8;let _r15=&15u8;let _r16=&16u8;let _r17=&17u8;let _r18=&18u8;let _r19=&19u8;",
            "let _r20=&20u8;let _r21=&21u8;let _r22=&22u8;let _r23=&23u8;let _r24=&24u8;let _r25=&25u8;let _r26=&26u8;let _r27=&27u8;let _r28=&28u8;let _r29=&29u8;",
            "let _r30=&30u8;let _r31=&31u8;let _r32=&32u8;let _r33=&33u8;let _r34=&34u8;let _r35=&35u8;let _r36=&36u8;let _r37=&37u8;let _r38=&38u8;let _r39=&39u8;",
            "}trait A<'a,'b>{const C:usize;}struct S;impl<'a:'b,'b>A<'a,'b>for S{const C:usize=3;",
            "}fn value<'a>(_:&'a u8)->usize{<S as A<'a,'a>>::C}fn main(){wide();let x=1;check(value(&x)==3);}",
        ),
        false,
    ),
    (
        "unanchored-impl-region",
        "trait A{const C:usize;}struct S;impl<'a>A for S{const C:usize=3;}fn main(){check(<S as A>::C==3);check(<S as A>::C==3);}",
        false,
    ),
    (
        "default-constant",
        "trait A<'a>{const C:usize=2;}struct R<'a>(&'a u8);impl<'a>A<'a>for R<'a>{}fn main(){check(<R<'static>as A<'static>>::C==2);check(<R<'static>as A<'static>>::C==2);}",
        false,
    ),
    (
        "mixed-arguments",
        "trait A<'a,T,const N:usize>{const C:usize=2;}struct R<'a>(&'a u8);impl<'a,T:'a,const N:usize>A<'a,T,N>for R<'a>{const C:usize=N;}fn main(){check(<R<'static>as A<'static,u8,3>>::C==3);check(<R<'static>as A<'static,u16,7>>::C==7);}",
        false,
    ),
    (
        "local-callable-binder",
        "trait A<T>{const C:usize;}struct S;impl<T:'static>A<T>for S{const C:usize=3;}fn main(){check(<S as A<fn(&u8)->&u8>>::C==3);}",
        false,
    ),
    (
        "unpaid-type-bound",
        "trait A<T>{const C:usize;}struct S;impl<T:'static>A<T>for S{const C:usize=3;}fn value<'a>(_:&'a u8)->usize{<S as A<&'a u8>>::C}fn main(){check(<S as A<&'static u8>>::C==3);}",
        true,
    ),
    (
        "unused-invalid-result",
        "trait A<'a>{type Out:'a;const C:usize;}struct R<'a>(&'a u8);impl<'a,'b>A<'b>for R<'a>{type Out=&'a u8;const C:usize=3;}fn main(){check(<R<'static>as A<'static>>::C==3);}",
        true,
    ),
];
#[test]
fn closed_targets_preserve_complete_arguments_and_caller_debts()
-> Result<(), Box<dyn std::error::Error>> {
    for &(name, source, rejected) in CASES {
        let source = format!("{}{source}", native::CHECK);
        object_outcome(&format!("closed-dispatch-{name}"), &source, !rejected)?;
        let result = prepared::checked(&source).map(|_| ());
        assert_eq!(result.is_err(), rejected, "{name}: {result:?}");
    }
    Ok(())
}
#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn closed_dispatch_preserves_native_values() -> Result<(), Box<dyn std::error::Error>> {
    let cases = CASES
        .iter()
        .filter(|(_, _, rejected)| !rejected)
        .copied()
        .collect::<Vec<_>>();
    native::check_cases("closed-dispatch", &cases)
}
