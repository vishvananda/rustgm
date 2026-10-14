//! Constant eligibility distinguishes transferred results from temporary cleanup.
#[path = "helpers/qualify.rs"]
mod rustc;
#[path = "support/semantic.rs"]
mod semantic;
use rustgm::{resolve::model::DefId, sema::values::Evaluator};

const HEADER: &str = "struct S{n:usize}impl Drop for S{fn drop(&mut self){}}";
const ACCEPTED: &[(&str, &str)] = &[
    ("returned-struct", "const C:S=S{n:3};"),
    ("returned-tuple", "const C:(S,usize)=(S{n:3},4);"),
    (
        "returned-conditional",
        "const C:S=if true{S{n:3}}else{S{n:4}};",
    ),
    ("one-repeat", "const C:[S;1]=[S{n:3};1];"),
    (
        "zero-repeat-constant",
        "const V:S=S{n:3};const C:[S;0]=[V;0];",
    ),
    (
        "zero-repeat-projection",
        "const V:S=S{n:3};const C:usize=([V;0],3).1;",
    ),
    ("direct-reference", "const C:&S=&S{n:3};"),
    ("field-reference", "const C:&usize=&S{n:3}.n;"),
    ("tuple-reference", "const C:(&S,)=(&S{n:3},);"),
    ("array-reference", "const C:[&S;1]=[&S{n:3}];"),
    ("if-reference", "const C:&S=if true{&S{n:3}}else{&S{n:4}};"),
    (
        "constructor-reference",
        "struct W(&'static S);const C:W=W(&S{n:3});",
    ),
    ("static-field-read", "static V:S=S{n:3};const C:usize=V.n;"),
    (
        "static-borrow-read",
        "static V:S=S{n:3};const C:usize=(&V).n;",
    ),
    (
        "const-reference-field-read",
        "const V:&S=&S{n:3};const C:usize=V.n;",
    ),
];
const REJECTED: &[(&str, &str)] = &[
    (
        "constant-drop-enum-cast",
        "enum E{A=3}impl Drop for E{fn drop(&mut self){}}const C:usize=E::A as usize;",
    ),
    (
        "runtime-drop-enum-cast",
        "enum E{A=3}impl Drop for E{fn drop(&mut self){}}const C:usize=3;fn f(e:E)->usize{e as usize}",
    ),
    ("owned-projection", "const C:usize=S{n:3}.n;"),
    ("dead-branch", "const C:usize=if true{3}else{S{n:3}.n};"),
    ("short-circuit", "const C:bool=true || (S{n:3}.n==3);"),
    ("temporary-borrow-read", "const C:usize=(&S{n:3}).n;"),
    ("zero-repeat-expression", "const C:[S;0]=[S{n:3};0];"),
    ("tuple-transfer", "const C:S=(S{n:3},0).0;"),
    ("tuple-drop-sibling", "const C:S=(S{n:3},S{n:4}).0;"),
    ("tuple-scalar-projection", "const C:usize=(S{n:3},4).1;"),
    (
        "nested-wrapper-projection",
        "struct W{a:S,n:usize}const C:usize=W{a:S{n:3},n:4}.n;",
    ),
    (
        "wrapper-transfer",
        "struct W{a:S,n:usize}const C:S=W{a:S{n:3},n:4}.a;",
    ),
    (
        "update-retains-drop",
        "struct W{a:S,n:usize}const C:W=W{n:4,..W{a:S{n:3},n:2}};",
    ),
    (
        "update-replaces-drop",
        "struct W{a:S,n:usize}const C:W=W{a:S{n:4},..W{a:S{n:3},n:2}};",
    ),
    ("const-field-read", "const V:S=S{n:3};const C:usize=V.n;"),
    ("repeat-borrow", "const C:[&S;1]=[&S{n:3};1];"),
];

fn evaluate(source: &str) -> Result<(), String> {
    let (report, mut facts) = semantic::analyze(source)?;
    let index = report
        .resolution
        .definitions
        .iter()
        .position(|definition| report.package.names.get(definition.name) == "C")
        .ok_or("missing declared constant C")?;
    Evaluator::new(&report, &mut facts)
        .definition(DefId(u32::try_from(index).map_err(|e| e.to_string())?))
        .map_err(|e| e.to_string())?;
    Ok(())
}
#[test]
fn constant_results_and_extended_storage_do_not_run_destructors() -> Result<(), String> {
    for &(name, body) in ACCEPTED {
        evaluate(&format!("{HEADER}{body}")).map_err(|e| format!("{name}: {e}"))?;
    }
    Ok(())
}
#[test]
fn owned_temporaries_remain_ineligible_in_dead_branches_and_partial_moves() {
    for &(name, body) in REJECTED {
        assert!(evaluate(&format!("{HEADER}{body}")).is_err(), "{name}");
    }
}
#[test]
fn destruction_eligibility_matches_independent_rustc() -> Result<(), Box<dyn std::error::Error>> {
    for (cases, accepted) in [(ACCEPTED, true), (REJECTED, false)] {
        for &(name, body) in cases {
            rustc::qualify(
                &format!("const-destruction-{name}"),
                &format!("{HEADER}{body}"),
                accepted,
            )?;
        }
    }
    Ok(())
}
#[test]
fn scalar_shape_arguments_do_not_discard_owned_destruction() -> Result<(), String> {
    let alias = "type A<const N:usize>=[u8;N];";
    for body in [
        "fn f(){let _:A<{S{n:3}.n}>=[0;3];}",
        "const V:S=S{n:3};fn f(){let _:A<{V.n}>=[0;3];}",
        "fn f(){let _:A<{(&S{n:3}).n}>=[0;3];}",
    ] {
        let error = semantic::analyze(&format!("{HEADER}{alias}{body}"))
            .err()
            .ok_or("shape constant accepted invalid destruction")?;
        assert!(
            error.contains("constant expression cannot destroy"),
            "{error}"
        );
    }
    semantic::analyze(&format!(
        "{HEADER}{alias}static V:S=S{{n:3}};fn f(){{let _:A<{{V.n}}>=[0;3];}}"
    ))?;
    Ok(())
}

#[test]
fn reference_shape_constants_keep_a_typed_lifetime_proof_body() -> Result<(), String> {
    let (report, facts) = semantic::analyze(
        "const R:&u8=&3;type A<const N:usize>=[u8;N];fn f(){let _:A<{*R as usize}>=[0;3];}",
    )?;
    assert_eq!(facts.constant_bodies().len(), 1);
    let program = rustgm::body::build(&report, &facts).map_err(|e| e.to_string())?;
    let contracts = rustgm::body::regions::collect(&facts).map_err(str::to_owned)?;
    let source = facts.constant_bodies()[0].0;
    assert!(program.bodies.iter().any(
        |b| b.source == source && matches!(b.owner, rustgm::body::ir::Owner::AnonymousConstant)
    ));
    assert!(
        contracts
            .bodies
            .iter()
            .any(|b| b.source == source && !b.obligations.is_empty())
    );
    let (_, scalar) =
        semantic::analyze("type A<const N:usize>=[u8;N];fn f(){let _:A<{1+2}>=[0;3];}")?;
    assert!(scalar.constant_bodies().is_empty());
    Ok(())
}
