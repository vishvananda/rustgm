//! Body projection queries retain exact selections and caller evidence across definition work.
use super::{native, object_outcome, semantic};
#[path = "definition_readiness/growth.rs"]
mod growth;

const CASES: &[(&str, &str, bool)] = &[
    (
        "v1-projected-value",
        "struct X<const N:usize>([u8;N]);struct V(u8);fn f()->X<{<u8 as A>::C.0 as usize}>{X([7;3])}trait A{type Out;const C:Self::Out;}impl A for u8{type Out=V;const C:V=V(3);}fn main(){check(f().0[2]==7);}",
        true,
    ),
    (
        "v1-projected-reference",
        "struct X<const N:usize>([u8;N]);struct V(u8);fn f()->X<{<u8 as A>::C.0 as usize}>{X([7;3])}trait A{type Out;const C:Self::Out;}impl A for u8{type Out=&'static V;const C:&'static V=&V(3);}fn main(){check(f().0[2]==7);}",
        true,
    ),
    (
        "v2-nested-projected-field",
        "struct X<const N:usize>([u8;N]);struct V(u8);fn f()->X<{<u8 as A>::C.v.0 as usize}>{X([7;3])}trait P{type Out;}impl P for u8{type Out=V;}struct W<T:P>{v:T::Out}trait A{const C:W<u8>;}impl A for u8{const C:W<u8>=W{v:V(3)};}fn main(){check(f().0[2]==7);}",
        true,
    ),
    (
        "v2-projected-subject",
        "struct X<const N:usize>([u8;N]);struct V(u8);fn f()->X<{<<u16 as P>::Out as A>::C.0 as usize}>{X([7;3])}trait P{type Out;}impl P for u16{type Out=u8;}trait A{type Out;const C:Self::Out;}impl A for u8{type Out=V;const C:V=V(3);}fn main(){check(f().0[2]==7);}",
        true,
    ),
    (
        "v2-different-selected-results",
        "struct X<const N:usize>([u8;N]);struct V(u8);fn f()->X<{<u8 as A>::C.0 as usize+<u16 as A>::C.1 as usize}>{X([7;3])}trait A{type Out;const C:Self::Out;}impl A for u8{type Out=V;const C:V=V(1);}impl A for u16{type Out=(u8,u8);const C:(u8,u8)=(0,2);}fn main(){check(f().0[2]==7);}",
        true,
    ),
    (
        "v2-tuple-of-reference",
        "struct X<const N:usize>([u8;N]);struct V(u8);fn f()->X<{<u8 as A>::C.1.0 as usize}>{X([7;3])}trait A{type Out;const C:Self::Out;}impl A for u8{type Out=(u8,&'static V);const C:(u8,&'static V)=(0,&V(3));}fn main(){check(f().0[2]==7);}",
        true,
    ),
    (
        "v2-projected-unknown-field",
        "struct X<const N:usize>([u8;N]);struct V(u8);fn f()->X<{<u8 as A>::C.1 as usize}>{X([7;3])}trait A{type Out;const C:Self::Out;}impl A for u8{type Out=V;const C:V=V(3);}fn main(){check(f().0[2]==7);}",
        false,
    ),
    (
        "v2-projected-invalid-unused-member",
        "struct X<const N:usize>([u8;N]);struct V(u8);fn f()->X<{<u8 as A>::C.0 as usize}>{X([7;3])}trait A{type Out;const C:Self::Out;}impl A for u8{type Out=V;const C:V=V(3);}impl A for u16{type Out=V;const C:u8=3;}fn main(){check(f().0[2]==7);}",
        false,
    ),
    (
        "v3-type-dependency-early-forward",
        "struct X<const N:usize>([u8;N]);fn f()->X<{<u8 as A>::C}>{X([7;3])}trait A{type Out;type Other;const C:Self::Out;}impl A for u8{type Out=usize;type Other=X<{<u8 as A>::C}>;const C:usize=3;}fn main(){check(f().0[2]==7);}",
        true,
    ),
    (
        "v3-type-dependency-early-reverse",
        "struct X<const N:usize>([u8;N]);fn f()->X<{<u8 as A>::C}>{X([7;3])}trait A{type Out;type Other;const C:Self::Out;}impl A for u8{type Other=X<{<u8 as A>::C}>;type Out=usize;const C:usize=3;}fn main(){check(f().0[2]==7);}",
        true,
    ),
    (
        "v3-type-dependency-late-forward",
        "struct X<const N:usize>([u8;N]);trait A{type Out;type Other;const C:Self::Out;}impl A for u8{type Out=usize;type Other=X<{<u8 as A>::C}>;const C:usize=3;}fn f()->X<{<u8 as A>::C}>{X([7;3])}fn main(){check(f().0[2]==7);}",
        true,
    ),
    (
        "v3-type-dependency-late-reverse",
        "struct X<const N:usize>([u8;N]);trait A{type Out;type Other;const C:Self::Out;}impl A for u8{type Other=X<{<u8 as A>::C}>;type Out=usize;const C:usize=3;}fn f()->X<{<u8 as A>::C}>{X([7;3])}fn main(){check(f().0[2]==7);}",
        true,
    ),
    (
        "paid-early-out-first",
        "struct X<const N:usize>([u8;N]);fn f()->X<{*<S<'static>as A<'static>>::C.0 as usize}>{X([7;3])}struct S<'a>(&'a u8);struct R<'b>(&'b u8);trait B<'b>{const N:usize;}impl<'b>B<'b>for R<'b>{const N:usize=1;}struct V<'a,const N:usize>(&'a u8,[u8;N]);trait A<'a>{type Out:'a;type Other;const C:Self::Out;}impl<'a>A<'a>for S<'a>{type Out=V<'a,{<R<'static>as B<'static>>::N}>;type Other=X<{*<S<'static>as A<'static>>::C.0 as usize}>;const C:V<'a,1>=V(&3,[0;1]);}fn main(){check(f().0[2]==7);}",
        true,
    ),
    (
        "paid-early-other-first",
        "struct X<const N:usize>([u8;N]);fn f()->X<{*<S<'static>as A<'static>>::C.0 as usize}>{X([7;3])}struct S<'a>(&'a u8);struct R<'b>(&'b u8);trait B<'b>{const N:usize;}impl<'b>B<'b>for R<'b>{const N:usize=1;}struct V<'a,const N:usize>(&'a u8,[u8;N]);trait A<'a>{type Out:'a;type Other;const C:Self::Out;}impl<'a>A<'a>for S<'a>{type Other=X<{*<S<'static>as A<'static>>::C.0 as usize}>;type Out=V<'a,{<R<'static>as B<'static>>::N}>;const C:V<'a,1>=V(&3,[0;1]);}fn main(){check(f().0[2]==7);}",
        true,
    ),
    (
        "paid-late-out-first",
        "struct X<const N:usize>([u8;N]);struct S<'a>(&'a u8);struct R<'b>(&'b u8);trait B<'b>{const N:usize;}impl<'b>B<'b>for R<'b>{const N:usize=1;}struct V<'a,const N:usize>(&'a u8,[u8;N]);trait A<'a>{type Out:'a;type Other;const C:Self::Out;}impl<'a>A<'a>for S<'a>{type Out=V<'a,{<R<'static>as B<'static>>::N}>;type Other=X<{*<S<'static>as A<'static>>::C.0 as usize}>;const C:V<'a,1>=V(&3,[0;1]);}fn f()->X<{*<S<'static>as A<'static>>::C.0 as usize}>{X([7;3])}fn main(){check(f().0[2]==7);}",
        true,
    ),
    (
        "paid-late-other-first",
        "struct X<const N:usize>([u8;N]);struct S<'a>(&'a u8);struct R<'b>(&'b u8);trait B<'b>{const N:usize;}impl<'b>B<'b>for R<'b>{const N:usize=1;}struct V<'a,const N:usize>(&'a u8,[u8;N]);trait A<'a>{type Out:'a;type Other;const C:Self::Out;}impl<'a>A<'a>for S<'a>{type Other=X<{*<S<'static>as A<'static>>::C.0 as usize}>;type Out=V<'a,{<R<'static>as B<'static>>::N}>;const C:V<'a,1>=V(&3,[0;1]);}fn f()->X<{*<S<'static>as A<'static>>::C.0 as usize}>{X([7;3])}fn main(){check(f().0[2]==7);}",
        true,
    ),
    (
        "unpaid-early-out-first",
        "struct X<const N:usize>([u8;N]);fn f()->X<{*<S<'static>as A<'static>>::C.0 as usize}>{X([7;3])}struct S<'a>(&'a u8);struct R<'b>(&'b u8);trait B<'b>{const N:usize;}impl<'b>B<'b>for R<'b>{const N:usize=1;}struct V<'a,const N:usize>(&'a u8,[u8;N]);trait A<'a>{type Out:'static;type Other;const C:Self::Out;}impl<'a>A<'a>for S<'a>{type Out=V<'a,{<R<'static>as B<'static>>::N}>;type Other=X<{*<S<'static>as A<'static>>::C.0 as usize}>;const C:V<'a,1>=V(&3,[0;1]);}fn main(){check(f().0[2]==7);}",
        false,
    ),
    (
        "unpaid-early-other-first",
        "struct X<const N:usize>([u8;N]);fn f()->X<{*<S<'static>as A<'static>>::C.0 as usize}>{X([7;3])}struct S<'a>(&'a u8);struct R<'b>(&'b u8);trait B<'b>{const N:usize;}impl<'b>B<'b>for R<'b>{const N:usize=1;}struct V<'a,const N:usize>(&'a u8,[u8;N]);trait A<'a>{type Out:'static;type Other;const C:Self::Out;}impl<'a>A<'a>for S<'a>{type Other=X<{*<S<'static>as A<'static>>::C.0 as usize}>;type Out=V<'a,{<R<'static>as B<'static>>::N}>;const C:V<'a,1>=V(&3,[0;1]);}fn main(){check(f().0[2]==7);}",
        false,
    ),
    (
        "unpaid-late-out-first",
        "struct X<const N:usize>([u8;N]);struct S<'a>(&'a u8);struct R<'b>(&'b u8);trait B<'b>{const N:usize;}impl<'b>B<'b>for R<'b>{const N:usize=1;}struct V<'a,const N:usize>(&'a u8,[u8;N]);trait A<'a>{type Out:'static;type Other;const C:Self::Out;}impl<'a>A<'a>for S<'a>{type Out=V<'a,{<R<'static>as B<'static>>::N}>;type Other=X<{*<S<'static>as A<'static>>::C.0 as usize}>;const C:V<'a,1>=V(&3,[0;1]);}fn f()->X<{*<S<'static>as A<'static>>::C.0 as usize}>{X([7;3])}fn main(){check(f().0[2]==7);}",
        false,
    ),
    (
        "unpaid-late-other-first",
        "struct X<const N:usize>([u8;N]);struct S<'a>(&'a u8);struct R<'b>(&'b u8);trait B<'b>{const N:usize;}impl<'b>B<'b>for R<'b>{const N:usize=1;}struct V<'a,const N:usize>(&'a u8,[u8;N]);trait A<'a>{type Out:'static;type Other;const C:Self::Out;}impl<'a>A<'a>for S<'a>{type Other=X<{*<S<'static>as A<'static>>::C.0 as usize}>;type Out=V<'a,{<R<'static>as B<'static>>::N}>;const C:V<'a,1>=V(&3,[0;1]);}fn f()->X<{*<S<'static>as A<'static>>::C.0 as usize}>{X([7;3])}fn main(){check(f().0[2]==7);}",
        false,
    ),
    (
        "reference-only",
        "struct X<const N:usize>([u8;N]);struct V(u8);fn f()->X<{(<u8 as A>::C,3usize).1}>{X([7;3])}trait P{type Out;}impl P for u8{type Out=V;}struct W<T:P>{v:T::Out}enum O<T>{None,Some(T)}trait A{const C:O<&'static W<u8>>;}impl A for u8{const C:O<&'static W<u8>>=O::None;}fn main(){check(f().0[2]==7);}",
        true,
    ),
    (
        "initializer-result-projection",
        "struct X<const N:usize>([u8;N]);struct V(u8);fn f()->X<{<u8 as A>::C.0 as usize}>{X([7;3])}trait A{type Out;const C:Self::Out;}impl A for u8{type Out=V;const C:Self::Out=V(3);}fn main(){check(f().0[2]==7);}",
        true,
    ),
    (
        "callable-initializer",
        "struct X<const N:usize>([u8;N]);struct V(u8);fn f()->X<{(C,3usize).1}>{X([7;3])}trait P{type Out;}impl P for u8{type Out=V;}fn value()->V{V(3)}const C:fn()-><u8 as P>::Out=value;fn main(){check(f().0[2]==7);}",
        true,
    ),
    (
        "generic-callable-early",
        "struct X<const N:usize>([u8;N]);struct V(u8);fn f()->X<{(<S as A<u8>>::C,3usize).1}>{X([7;3])}struct S;trait P{type Out;const F:fn()->Self::Out;}impl P for u8{type Out=V;const F:fn()->V=value;}fn value()->V{V(3)}trait A<T:P>{type Out;const C:Self::Out;}impl<T:P>A<T>for S{type Out=fn()->T::Out;const C:Self::Out=<T as P>::F;}fn main(){check(f().0[2]==7);}",
        true,
    ),
    (
        "generic-callable-late",
        "struct X<const N:usize>([u8;N]);struct V(u8);struct S;trait P{type Out;const F:fn()->Self::Out;}impl P for u8{type Out=V;const F:fn()->V=value;}fn value()->V{V(3)}trait A<T:P>{type Out;const C:Self::Out;}impl<T:P>A<T>for S{type Out=fn()->T::Out;const C:Self::Out=<T as P>::F;}fn f()->X<{(<S as A<u8>>::C,3usize).1}>{X([7;3])}fn main(){check(f().0[2]==7);}",
        true,
    ),
    (
        "partial-unique-header",
        "struct X<const N:usize>([u8;N]);struct V(u8);struct G<T>(T);fn f()->X<{<G<_>as A>::C.0 as usize}>{X([7;3])}trait A{type Out;const C:Self::Out;}impl A for G<u8>{type Out=V;const C:V=V(3);}fn main(){check(f().0[2]==7);}",
        true,
    ),
    (
        "partial-ambiguous-headers",
        "struct X<const N:usize>([u8;N]);struct V(u8);struct G<T>(T);fn f()->X<{<G<_>as A>::C.0 as usize}>{X([7;3])}trait A{type Out;const C:Self::Out;}impl A for G<u8>{type Out=V;const C:V=V(3);}impl A for G<u16>{type Out=V;const C:V=V(3);}fn main(){check(f().0[2]==7);}",
        false,
    ),
    (
        "partial-unpaid-result",
        "struct X<const N:usize>([u8;N]);struct V(u8);struct G<T>(T);fn f()->X<{<G<_>as A>::C.0 as usize}>{X([7;3])}trait A{type Out;const C:Self::Out;}impl A for G<u8>{type Out=V;const C:V=V(3);}trait Bad{type Out:Copy;}impl Bad for bool{type Out=V;}fn main(){check(f().0[2]==7);}",
        false,
    ),
    (
        "partial-unique-reference-header",
        "struct X<const N:usize>([u8;N]);struct V(u8);struct G<T>(T);fn f()->X<{*<G<_>as A>::C as usize}>{X([7;3])}trait A{type Out;const C:Self::Out;}impl<'a>A for G<&'a u8>{type Out=&'a u8;const C:&'a u8=&3;}fn main(){check(f().0[2]==7);}",
        true,
    ),
    (
        "partial-definition-cycle",
        "struct X<const N:usize>([u8;N]);struct V(u8);struct G<T>(T);fn f()->X<{<G<_>as A>::C.0 as usize}>{X([7;3])}struct W<const N:usize>(u8,[u8;N]);trait A{type Out;const C:Self::Out;}impl A for G<u8>{type Out=W<{<G<u8>as A>::C.0 as usize}>;const C:Self::Out=W(3,[0;3]);}fn main(){check(f().0[2]==7);}",
        false,
    ),
];

#[test]
fn selected_definitions_preserve_source_order_and_lifetime_outcomes()
-> Result<(), Box<dyn std::error::Error>> {
    for &(name, body, accepted) in CASES {
        let source = format!("{}{body}", native::CHECK);
        object_outcome(&format!("projection-readiness-{name}"), &source, accepted)?;
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
fn selected_definition_waits_preserve_native_values() -> Result<(), Box<dyn std::error::Error>> {
    let cases = CASES
        .iter()
        .filter(|(_, _, accepted)| *accepted)
        .map(|&(name, body, _)| (name, body, false))
        .collect::<Vec<_>>();
    native::check_cases("projection-readiness", &cases)
}
