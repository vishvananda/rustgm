//! Local checked definitions do not erase family-application requirements during conformance.
use super::{Checker, DefKind, TestResult, prepare, report};
use crate::resolve::model::DefId;

const CASES: &[(&str, &str, bool)] = &[
    (
        "erased-late-actual",
        "trait Q{type Out<'a> where 'a:'static;fn f<'a>(x:u8);}impl Q for u8{type Out<'a>=u8 where 'a:'static;fn f<'a>(x:Self::Out<'a>) {}}",
        false,
    ),
    (
        "erased-late-declared",
        "trait Q{type Out<'a> where 'a:'static;fn f<'a>(x:Self::Out<'a>);}impl Q for u8{type Out<'a>=u8 where 'a:'static;fn f<'a>(x:u8) {}}",
        true,
    ),
    (
        "erased-late-proven",
        "trait Q{type Out<'a> where 'a:'static;fn f<'a:'static>(x:u8);}impl Q for u8{type Out<'a>=u8 where 'a:'static;fn f<'a:'static>(x:Self::Out<'a>) {}}",
        true,
    ),
    (
        "erased-late-pair",
        "trait Q{type Out<'a,'b> where 'a:'b;fn f<'a,'b>(x:u8);}impl Q for u8{type Out<'a,'b>=u8 where 'a:'b;fn f<'a,'b>(x:Self::Out<'a,'b>) {}}",
        false,
    ),
    (
        "erased-late-type",
        "trait Q<T>{type Out<'a> where T:'a;fn f<'a>(x:u8);}impl<T>Q<T>for u8{type Out<'a>=u8 where T:'a;fn f<'a>(x:Self::Out<'a>) {}}",
        false,
    ),
    (
        "constant-actual-only",
        "trait Q<'a,'b>{type Out where 'a:'b;const C:u8;}impl<'a,'b>Q<'a,'b>for u8{type Out=u8 where 'a:'b;const C:Self::Out=0;}",
        false,
    ),
    (
        "constant-actual-proven",
        "trait Q<'a,'b>{type Out where 'a:'b;const C:u8;}impl<'a:'b,'b>Q<'a,'b>for u8{type Out=u8 where 'a:'b;const C:Self::Out=0;}",
        true,
    ),
    (
        "constant-declared-life",
        "trait Q<'a:'b,'b>{type Out where 'a:'b;const C:Self::Out;}impl<'a:'b,'b>Q<'a,'b>for u8{type Out=u8 where 'a:'b;const C:u8=0;}",
        true,
    ),
    (
        "constant-unproven-life",
        "trait Q<'a,'b>{type Out where 'a:'b;const C:Self::Out;}impl<'a,'b>Q<'a,'b>for u8{type Out=u8 where 'a:'b;const C:u8=0;}",
        false,
    ),
    (
        "actual-only-condition",
        "trait Q<'a,'b>{type Out where 'a:'b;fn f()->u8;}impl<'a,'b>Q<'a,'b> for u8{type Out=u8 where 'a:'b;fn f()->Self::Out{0}}",
        false,
    ),
    (
        "expected-implies-actual-where",
        "trait Q<'a,'b>{type Out where 'a:'b;fn f()->Self::Out;}impl<'a,'b>Q<'a,'b> for u8{type Out=u8 where 'a:'b;fn f()->u8 where 'a:'b{0}}",
        true,
    ),
    (
        "outer-capture-nested",
        "trait Q<'a,'b>{type Out where 'a:'b;fn f()->fn()->Self::Out;}impl<'a,'b>Q<'a,'b> for u8{type Out=u8 where 'a:'b;fn f()->fn()->u8{loop{}}}",
        true,
    ),
    (
        "nested-local-no-permission",
        "trait Q{type Out<'a,'b> where 'a:'b;fn f()->for<'a,'b>fn(&'a u8,&'b u8)->Self::Out<'a,'b>;}impl Q for u8{type Out<'a,'b>=u8 where 'a:'b;fn f()->for<'a,'b>fn(&'a u8,&'b u8)->u8{loop{}}}",
        false,
    ),
    (
        "type-outlives-method",
        "trait Q<'a,T>{type Out where T:'a;fn f()->Self::Out;}impl<'a,T>Q<'a,T> for u8{type Out=u8 where T:'a;fn f()->u8{0}}",
        true,
    ),
    (
        "method-lifetime-debt",
        "trait Q{type Out<'a,'b> where 'a:'b;fn f<'a:'b,'b>()->Self::Out<'a,'b>;}impl Q for u8{type Out<'a,'b>=u8 where 'a:'b;fn f<'x:'y,'y>()->u8{0}}",
        true,
    ),
    (
        "method-lifetime-omitted",
        "trait Q{type Out<'a,'b> where 'a:'b;fn f<'a,'b>()->Self::Out<'a,'b>;}impl Q for u8{type Out<'a,'b>=u8 where 'a:'b;fn f<'x,'y>()->u8{0}}",
        false,
    ),
    (
        "method-zero-lifetimes",
        "trait Q<'a,'b>{type Out where 'a:'b;fn f()->Self::Out where 'a:'b;}impl<'a,'b>Q<'a,'b> for u8{type Out=u8 where 'a:'b;fn f()->u8 where 'a:'b{0}}",
        true,
    ),
    (
        "method-zero-lifetimes-omitted",
        "trait Q<'a,'b>{type Out where 'a:'b;fn f()->Self::Out;}impl<'a,'b>Q<'a,'b> for u8{type Out=u8 where 'a:'b;fn f()->u8{0}}",
        true,
    ),
    (
        "method-identity",
        "trait Q{type Out<T>;fn f<T>(x:T)->Self::Out<T>;}impl Q for u8{type Out<T>=T;fn f<T>(x:T)->T{x}}",
        true,
    ),
    (
        "method-distinct",
        "trait Q{type Out<T>;fn f()->(Self::Out<u8>,Self::Out<u16>);}impl Q for u8{type Out<T>=T;fn f()->(u8,u16){(1,2)}}",
        true,
    ),
    (
        "method-swapped",
        "trait Q{type Out<T>;fn f()->(Self::Out<u8>,Self::Out<u16>);}impl Q for u8{type Out<T>=T;fn f()->(u16,u8){(1,2)}}",
        false,
    ),
    (
        "method-copy",
        "trait Q{type Out<T:Copy>;fn f<T:Copy>(x:T)->Self::Out<T>;}impl Q for u8{type Out<T:Copy>=u8;fn f<T:Copy>(x:T)->u8{0}}",
        true,
    ),
    (
        "method-omitted-copy",
        "trait Q{type Out<T:Copy>;fn f<T>(x:T)->Self::Out<T>;}impl Q for u8{type Out<T:Copy>=u8;fn f<T>(x:T)->u8{0}}",
        false,
    ),
    (
        "method-parent",
        "trait Q<P>{type Out<T>;fn f<T>(p:P,t:T)->Self::Out<T>;}impl<P>Q<P> for u8{type Out<T>=(P,T);fn f<U>(p:P,t:U)->(P,U){(p,t)}}",
        true,
    ),
    (
        "method-lifetime",
        "trait Q{type Out<'a> where Self:'a;fn f<'a>(x:&'a Self)->Self::Out<'a>;}impl Q for u8{type Out<'a>=&'a u8;fn f<'b>(x:&'b u8)->&'b u8{x}}",
        true,
    ),
    (
        "method-capture",
        "struct W<'p>(&'p u8);trait Q{type Out<'a> where Self:'a;fn f<'a>(x:&'a Self)->Self::Out<'a>;}impl<'p>Q for W<'p>{type Out<'a>=&'a &'p u8 where Self:'a;fn f<'b>(x:&'b Self)->&'b &'p u8{&x.0}}",
        true,
    ),
    (
        "method-nested",
        "trait Q{type Out<'a>;fn f()->for<'a>fn(&'a u8)->Self::Out<'a>;}impl Q for u8{type Out<'a>=&'a u8;fn f()->for<'a>fn(&'a u8)->&'a u8{loop{}}}",
        true,
    ),
    (
        "method-correlated",
        "trait Q{type Out<'a,'b>;fn f()->for<'a,'b>fn(&'a u8,&'b u8)->Self::Out<'a,'b>;}impl Q for u8{type Out<'a,'b>=(&'a u8,&'b u8);fn f()->for<'a,'b>fn(&'a u8,&'b u8)->(&'a u8,&'b u8){loop{}}}",
        true,
    ),
    (
        "method-swapped-lifetimes",
        "trait Q{type Out<'a,'b>;fn f()->for<'a,'b>fn(&'a u8,&'b u8)->Self::Out<'a,'b>;}impl Q for u8{type Out<'a,'b>=(&'a u8,&'b u8);fn f()->for<'a,'b>fn(&'a u8,&'b u8)->(&'b u8,&'a u8){loop{}}}",
        false,
    ),
    (
        "method-zero-parameters",
        "trait Q<T>{type Out where T:Copy;fn f()->Self::Out where T:Copy;}impl<T>Q<T> for u8{type Out=u8 where T:Copy;fn f()->u8 where T:Copy{0}}",
        true,
    ),
    (
        "method-zero-omitted",
        "trait Q<T>{type Out where T:Copy;fn f()->Self::Out;}impl<T>Q<T> for u8{type Out=u8 where T:Copy;fn f()->u8{0}}",
        false,
    ),
    (
        "constant-array",
        "trait Q{type Out<const N:usize>;const C:Self::Out<3>;}impl Q for u8{type Out<const N:usize>=[u8;N];const C:[u8;3]=[0;3];}",
        true,
    ),
    (
        "constant-wrong-array",
        "trait Q{type Out<const N:usize>;const C:Self::Out<3>;}impl Q for u8{type Out<const N:usize>=[u8;N];const C:[u8;4]=[0;4];}",
        false,
    ),
    (
        "constant-copy",
        "trait Q{type Out<T:Copy>;const C:Self::Out<u8>;}impl Q for u8{type Out<T:Copy>=u8;const C:u8=0;}",
        true,
    ),
    (
        "constant-unproved-copy",
        "trait Q{type Out<T:Copy>;const C:Self::Out<&'static mut u8>;}impl Q for u8{type Out<T:Copy>=u8;const C:u8=0;}",
        false,
    ),
    (
        "constant-parent",
        "trait Q<P:Copy>{type Out<T>;const C:Self::Out<P>;}impl<P:Copy>Q<P> for u8{type Out<T>=u8;const C:u8=0;}",
        true,
    ),
    (
        "dependent-quantifier",
        "trait Q{type Out<T> where for<'a>T:'a;fn f<T>(x:Self::Out<T>);}impl Q for u8{type Out<T>=u8 where for<'a>T:'a;fn f<T>(x:u8){}}",
        false,
    ),
    (
        "explicit-static",
        "trait Q{type Out<T:'static>;fn f<T>(x:Self::Out<T>);}impl Q for u8{type Out<T>=u8 where T:'static;fn f<T>(x:u8){}}",
        true,
    ),
    (
        "dependent-proven",
        "trait Q{type Out<T> where for<'a>T:'a;fn f<T:'static>(x:Self::Out<T>);}impl Q for u8{type Out<T>=u8 where for<'a>T:'a;fn f<T:'static>(x:u8){}}",
        true,
    ),
    (
        "unused-quantifier",
        "trait Q{type Out<T> where for<'a>T:'static;fn f<T>(x:Self::Out<T>);}impl Q for u8{type Out<T>=u8 where for<'a>T:'static;fn f<T>(x:u8){}}",
        true,
    ),
    (
        "dependent-composite",
        "trait Q{type Out<T> where for<'a>(&'a u8,T):'a;fn f<T>(x:Self::Out<T>);}impl Q for u8{type Out<T>=u8 where for<'a>(&'a u8,T):'a;fn f<T>(x:u8){}}",
        false,
    ),
    (
        "independent-composite",
        "trait Q{type Out<T> where for<'a>(u8,T):'static;fn f<T>(x:Self::Out<T>);}impl Q for u8{type Out<T>=u8 where for<'a>(u8,T):'static;fn f<T>(x:u8){}}",
        true,
    ),
];

fn check(source: &str) -> TestResult {
    let report = report(source)?;
    let mut checker = Checker::new(&report)?;
    prepare(&mut checker)?;
    for (index, definition) in report.resolution.definitions.iter().enumerate() {
        if matches!(definition.kind, DefKind::Function | DefKind::Const) {
            checker.definition(DefId(u32::try_from(index)?), false)?;
            checker.drain(&mut 0, 100_000)?;
        }
    }
    checker.complete_deferred_declarations(&mut 0, 100_000)?;
    // The production entry point checks types, then local method/constant
    // signatures, and only then marks the implementation complete.
    checker.validate_trait_implementations()?;
    Ok(())
}

#[test]
fn local_family_methods_and_constants_preserve_complete_applications() {
    let mut failures = Vec::new();
    for &(name, source, accepted) in CASES {
        let result = check(source);
        if result.is_ok() != accepted {
            failures.push(format!(
                "{name}: expected accepted={accepted}, result={result:?}"
            ));
        } else if let Err(error) = result {
            let message = error.to_string();
            if ![
                "associated type requires an unambiguous implementation",
                "associated type requires an unambiguous validated implementation",
                "implementation method early lifetime parameters disagree with trait",
                "implementation method lifetime contract does not satisfy trait",
                "implementation constant type does not satisfy trait",
                "associated family lifetime contract is not satisfied",
                "incompatible types",
            ]
            .iter()
            .any(|class| message.contains(class))
            {
                failures.push(format!("{name}: unexpected diagnostic: {message}"));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
