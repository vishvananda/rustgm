//! Outcomes independently checked by rustc 1.98.0; this tests the declaration rule itself.
use super::{Checker, TestResult, report};
use crate::resolve::model::{DefId, DefKind};

const CASES: &[(&str, &str, bool)] = &[
    (
        "result-supertrait-elaboration",
        "trait Has:'static{type Item;}trait A{type X<'a>;type Y<'a>:Has<Item=Self::X<'a>>;}",
        true,
    ),
    (
        "propagation-intersection",
        "trait Has{type Item;}trait A{type X<'a>;type Y<'a>:Has<Item=Self::X<'a>>where Self:'a;fn f<'a>(&'a self)->Self::Y<'a>;fn g<'a>(x:Self::X<'a>);}",
        true,
    ),
    (
        "other-trait-not-owner",
        "trait A{type Out<'a>;}trait B{fn f<'a,T:A>(x:&'a T)->T::Out<'a>;}",
        true,
    ),
    (
        "method-superbound-missing",
        "trait Long<'a>:'a{}trait A<T>{type Out<'x>;fn f<'a>()->Self::Out<'a>where T:Long<'a>;}",
        false,
    ),
    (
        "method-superbound-declared",
        "trait Long<'a>:'a{}trait A<T>{type Out<'x>where T:'x;fn f<'a>()->Self::Out<'a>where T:Long<'a>;}",
        true,
    ),
    (
        "parent-static-bound",
        "trait A<T:'static>{type Out<'a>;fn f<'a>()->Self::Out<'a>;}",
        true,
    ),
    (
        "constant-not-method",
        "trait A{type Out<'a>;const C:Self::Out<'static>;}",
        true,
    ),
    (
        "single-implied-missing",
        "trait A<T>{type Out<'x>;fn f<'a>(x:&'a T)->Self::Out<'a>;}",
        false,
    ),
    (
        "same-method-one-unconstrained",
        "trait A<T>{type Out<'x>;fn f<'a,'b>(x:&'a T,y:Self::Out<'b>)->Self::Out<'a>;}",
        false,
    ),
    (
        "same-method-both-implied",
        "trait A<T>{type Out<'x>;fn f<'a,'b>(x:&'a T,y:&'b T,z:Self::Out<'b>)->Self::Out<'a>;}",
        false,
    ),
    (
        "same-method-static-does-not-relax",
        "trait A<T>{type Out<'x>;fn f<'a>(x:&'a T,y:Self::Out<'static>)->Self::Out<'a>;}",
        false,
    ),
    (
        "different-family-types-mixed",
        "trait A{type Out<'x,T>;fn f<'a,'b,T,U>(x:&'a T,y:Self::Out<'b,U>)->Self::Out<'a,T>;}",
        false,
    ),
    (
        "different-family-types-both",
        "trait A{type Out<'x,T>;fn f<'a,'b,T,U>(x:&'a T,y:&'b U,z:Self::Out<'b,U>)->Self::Out<'a,T>;}",
        false,
    ),
    (
        "transitive-implied-missing",
        "trait A<T>{type Out<'x>;fn f<'a:'b,'b>(x:&'a T)->Self::Out<'b>;}",
        false,
    ),
    (
        "transitive-implied-declared",
        "trait A<T>{type Out<'x>where T:'x;fn f<'a:'b,'b>(x:&'a T)->Self::Out<'b>;}",
        true,
    ),
    (
        "two-family-slots-correct",
        "trait A<T>{type Out<'x,'y>where T:'x;fn f<'a,'b>(x:&'a T)->Self::Out<'a,'b>;}",
        true,
    ),
    (
        "two-family-slots-wrong",
        "trait A<T>{type Out<'x,'y>where T:'y;fn f<'a,'b>(x:&'a T)->Self::Out<'a,'b>;}",
        false,
    ),
    (
        "same-method-required-declared",
        "trait A<T>{type Out<'x>where T:'x;fn f<'a,'b>(x:&'a T,y:Self::Out<'b>)->Self::Out<'a>;}",
        true,
    ),
    (
        "different-family-types-required-declared",
        "trait A{type Out<'x,T>where T:'x;fn f<'a,'b,T,U>(x:&'a T,y:Self::Out<'b,U>)->Self::Out<'a,T>;}",
        true,
    ),
    (
        "lending-missing",
        "trait A{type Out<'a>;fn f<'a>(&'a self)->Self::Out<'a>;}",
        false,
    ),
    (
        "lending-declared",
        "trait A{type Out<'a>where Self:'a;fn f<'a>(&'a self)->Self::Out<'a>;}",
        true,
    ),
    (
        "different-methods-intersect",
        "trait A<T>{type Out<'a>;fn f<'a>(x:&'a T)->Self::Out<'a>;fn g<'b>(x:Self::Out<'b>);}",
        true,
    ),
    (
        "unused-method-no-intersection",
        "trait A<T>{type Out<'a>;fn f<'a>(x:&'a T)->Self::Out<'a>;fn g();}",
        false,
    ),
    (
        "static-only",
        "trait A{type Out<'a>;fn f(&self)->Self::Out<'static>;}",
        true,
    ),
    (
        "static-method-intersection",
        "trait A{type Out<'a>;fn f<'a>(&'a self)->Self::Out<'a>;fn g()->Self::Out<'static>;}",
        true,
    ),
    (
        "output-reference-not-assumed",
        "trait A{type Out<'a>;fn f<'a>()->(&'a Self,Self::Out<'a>);}",
        true,
    ),
    (
        "method-where-not-use",
        "trait A{type Out<'a>;fn f<'a>(&'a self)where Self::Out<'a>:Copy;}",
        true,
    ),
    (
        "forward-both-missing",
        "trait Has{type Item;}trait A{type X<'a>;type Y<'a>:Has<Item=Self::X<'a>>;fn f<'a>(&'a self)->Self::Y<'a>;}",
        false,
    ),
    (
        "forward-declared",
        "trait Has{type Item;}trait A{type X<'a>where Self:'a;type Y<'a>:Has<Item=Self::X<'a>>where Self:'a;fn f<'a>(&'a self)->Self::Y<'a>;}",
        true,
    ),
    (
        "forward-target-missing",
        "trait Has{type Item;}trait A{type X<'a>;type Y<'a>:Has<Item=Self::X<'a>>where Self:'a;fn f<'a>(&'a self)->Self::Y<'a>;}",
        false,
    ),
    (
        "unseeded-cycle",
        "trait Has{type Item;}trait A{type X<'a>:Has<Item=Self::Y<'a>>;type Y<'a>:Has<Item=Self::X<'a>>;}",
        true,
    ),
    (
        "region-missing",
        "trait A{type Out<'a,'b>;fn f<'a:'b,'b>(x:Self::Out<'a,'b>);}",
        false,
    ),
    (
        "region-declared",
        "trait A{type Out<'a,'b>where 'a:'b;fn f<'a:'b,'b>(x:Self::Out<'a,'b>);}",
        true,
    ),
    (
        "repeated-slot-not-new-equality",
        "trait A{type Out<'a,'b>;fn f<'a>(x:Self::Out<'a,'a>);}",
        true,
    ),
    (
        "concrete-type-input",
        "trait A{type Out<'a,T>;fn f<'a>()->Self::Out<'a,u8>;}",
        false,
    ),
    (
        "nested-local",
        "trait A{type Out<'a>;fn f(x:for<'a>fn(&'a Self)->Self::Out<'a>);}",
        true,
    ),
    (
        "nested-capture",
        "trait A{type Out<'a>;fn f<'a>(x:&'a Self,y:fn()->Self::Out<'a>);}",
        false,
    ),
];

fn check(source: &str) -> TestResult<usize> {
    let report = report(source)?;
    let mut checker = Checker::new(&report)?;
    super::conformance::prepare(&mut checker)?;
    for (index, definition) in report.resolution.definitions.iter().enumerate() {
        if definition.kind == DefKind::Function {
            checker.definition(DefId(u32::try_from(index)?), false)?;
            checker.drain(&mut 0, 100_000)?;
        }
    }
    checker.complete_deferred_declarations(&mut 0, 100_000)?;
    checker.validate_associated_type_bounds()?;
    checker
        .validate_required_family_bounds()
        .map_err(Into::into)
}

#[test]
fn required_family_where_clauses_match_independent_source_outcomes() {
    let mut failures = Vec::new();
    for &(name, source, accepted) in CASES {
        let result = check(source);
        println!("{name}: {result:?}");
        if result.is_ok() != accepted {
            failures.push(format!(
                "{name}: expected accepted={accepted}, result={result:?}"
            ));
        } else if let Err(error) = result
            && !error
                .to_string()
                .contains("associated type is missing required where clauses")
        {
            failures.push(format!("{name}: unexpected rejection: {error}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn independent_required_bound_sites_visit_only_their_own_applications() -> TestResult {
    use std::fmt::Write;
    for width in [16, 64, 256] {
        let mut source = String::from("trait A{type Out<'a>;");
        for index in 0..width {
            write!(source, "fn f{index}<'a>(x:Self::Out<'a>);")?;
        }
        source.push('}');
        let steps = check(&source)?;
        println!("required-bound sites={width} steps={steps}");
        assert!(steps <= 80 * width + 128, "{width}: {steps}");
    }
    Ok(())
}
