//! Symbolic promotion eligibility is independent of concrete constant materialization.
use crate::qualify as rustc;
use crate::semantic;
use crate::template_check as template;
use rustgm::{body, borrow, types::copy::Copies};

const CASES: &[(&str, bool, &str)] = &[
    ("const", true, "fn f<const N:usize>()->&'static usize{&N}"),
    ("sum", true, "fn f<const N:usize>()->&'static usize{&(N+1)}"),
    (
        "repeat",
        true,
        "fn f<const N:usize>()->&'static[u8;N]{&[3;N]}",
    ),
    (
        "repeat-value",
        true,
        "fn f<const N:usize>()->&'static[usize;N]{&[N;N]}",
    ),
    (
        "element",
        true,
        "fn f<const N:usize>()->&'static u8{&[3;N][0]}",
    ),
    ("zero", true, "fn f<T>()->&'static[T;0]{&[]}"),
    (
        "zero-divisor",
        false,
        "fn f<const N:usize>()->&'static usize{&(N/0)}",
    ),
    (
        "known-divisor",
        true,
        "fn f<const N:usize>()->&'static usize{&(N/2)}",
    ),
    (
        "symbolic-divisor",
        false,
        "fn f<const N:usize>()->&'static usize{&(1/N)}",
    ),
    (
        "negative-one",
        false,
        "fn f<const N:i8>()->&'static i8{&(N/-1)}",
    ),
    (
        "indexed-value",
        false,
        "fn f<const N:usize>()->&'static u8{&([3;N][0]+1)}",
    ),
    (
        "indexed-borrow",
        true,
        "fn f<const N:usize,const I:usize>()->&'static u8{&[3;N][I]}",
    ),
    (
        "runtime-element",
        false,
        "fn f<const N:usize>(x:u8)->&'static[u8;N]{&[x;N]}",
    ),
    (
        "nested",
        true,
        "fn f<const N:usize>()->&'static(usize,[usize;N]){&(N,[N;N])}",
    ),
    (
        "cast",
        true,
        "fn f<const N:u8>()->&'static u16{&(N as u16)}",
    ),
    ("bool", true, "fn f<const B:bool>()->&'static bool{&!B}"),
    ("char", true, "fn f<const C:char>()->&'static char{&C}"),
    (
        "known-remainder",
        true,
        "fn f<const N:i8>()->&'static i8{&(N%2)}",
    ),
    (
        "symbolic-remainder",
        false,
        "fn f<const N:i8>()->&'static i8{&(2%N)}",
    ),
    (
        "constructor",
        true,
        "struct S<const N:usize>([u8;N]);fn f<const N:usize>()->&'static S<N>{&S([3;N])}",
    ),
    (
        "nested-borrow",
        true,
        "fn f<const N:usize>()->&'static &'static usize{&&N}",
    ),
    (
        "field",
        true,
        "fn f<const N:usize>()->&'static usize{&((N,2).0+1)}",
    ),
    (
        "fixed-index",
        true,
        "fn f<const N:usize>()->&'static usize{&([N;2][0]+1)}",
    ),
    (
        "unknown-index",
        false,
        "fn f<const N:usize>()->&'static usize{&([N;2][N]+1)}",
    ),
    (
        "unknown-mutable-length",
        false,
        "fn f<const N:usize>()->&'static mut[u8;N]{&mut[3;N]}",
    ),
    (
        "effects",
        true,
        "fn effect(){}fn f<const N:usize>()->&'static usize{&{effect();N}}",
    ),
    (
        "runtime-call",
        false,
        "fn effect()->u8{3}fn f<const N:usize>()->&'static[u8;N]{&[effect();N]}",
    ),
    (
        "drop",
        false,
        "struct D<const N:usize>;impl<const N:usize>Drop for D<N>{fn drop(&mut self){}}fn f<const N:usize>()->&'static D<N>{&D}",
    ),
];

#[test]
fn declared_outcomes_match_rustc_and_full_template_proofs() -> Result<(), Box<dyn std::error::Error>>
{
    for &(name, accepted, source) in CASES {
        rustc::qualify(&format!("deferred-promotion-{name}"), source, accepted)?;
    }
    let mut failures = Vec::new();
    for &(name, accepted, source) in CASES {
        let result = template::check(source);
        if result.is_ok() != accepted {
            failures.push(format!("{name}: wanted {accepted}, {:?}", result.err()));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

#[test]
fn deferred_roots_are_cached_without_inventing_constant_values()
-> Result<(), Box<dyn std::error::Error>> {
    let source =
        "fn f<const N:usize>()->(&'static usize,&'static[u8;N],&'static u8){(&N,&[3;N],&7)}";
    rustc::qualify("deferred-promotion-facts", source, true)?;
    let (report, mut facts) = semantic::analyze(source)?;
    let mut templates = body::templates::build(&report, &facts)?;
    body::promote::analyze_templates(&report, &mut facts, &mut templates)?;
    let promotion = templates.bodies[0]
        .body
        .promotions
        .as_ref()
        .ok_or("missing promotions")?;
    let deferred = promotion.deferred().collect::<Vec<_>>();
    assert_eq!(deferred.len(), 2);
    assert_eq!(promotion.entries().count(), 1);
    let steps = promotion.steps;
    for &root in &deferred {
        assert!(promotion.contains(root));
        assert!(promotion.get(root).is_none());
        assert!(!templates.bodies[0].destruction.values[root.0 as usize]);
    }
    for (root, value) in promotion.entries() {
        assert!(promotion.contains(root));
        assert_eq!(value.as_scalar()?, rustgm::sema::model::Scalar::Integer(7));
    }
    body::promote::analyze_templates(&report, &mut facts, &mut templates)?;
    let promotion = templates.bodies[0]
        .body
        .promotions
        .as_ref()
        .ok_or("lost promotions")?;
    assert_eq!(promotion.steps, steps);
    assert_eq!(promotion.deferred().collect::<Vec<_>>(), deferred);
    assert_eq!(promotion.entries().count(), 1);
    Ok(())
}

#[test]
fn deferred_promotion_preserves_runtime_effects_and_bounds_checks()
-> Result<(), Box<dyn std::error::Error>> {
    use body::ir::{Action, ProjectionKind, Rvalue};
    let source =
        "fn effect(){}fn f<const N:usize>(i:usize)->&'static usize{&[{effect();N+1};N][i]}";
    rustc::qualify("deferred-promotion-effects-and-bounds", source, true)?;
    let (report, mut facts) = semantic::analyze(source)?;
    let mut templates = body::templates::build(&report, &facts)?;
    let statements = |body: &body::ir::Body| {
        body.statements
            .iter()
            .map(|s| format!("{:?}", s.action))
            .collect::<Vec<_>>()
    };
    let before = statements(&templates.bodies[0].body);
    body::promote::analyze_templates(&report, &mut facts, &mut templates)?;
    let body = &templates.bodies[0].body;
    assert_eq!(statements(body), before);
    assert!(
        body.values
            .iter()
            .any(|v| matches!(v.kind, Rvalue::Call { .. }))
    );
    assert!(
        body.values
            .iter()
            .any(|v| matches!(v.kind, Rvalue::Binary { .. }))
    );
    assert!(
        body.statements
            .iter()
            .any(|s| matches!(s.action, Action::Project(_)))
    );
    assert!(
        body.projections
            .iter()
            .any(|p| matches!(p.kind, ProjectionKind::IndexParameter { .. }))
    );
    assert_eq!(
        body.promotions
            .as_ref()
            .ok_or("missing promotion")?
            .deferred()
            .count(),
        1
    );
    Ok(())
}

#[test]
fn deferred_queries_scale_with_retained_operands_not_unknown_array_lengths()
-> Result<(), Box<dyn std::error::Error>> {
    for count in [128, 512, 2048] {
        let mut source = String::from("fn f<const N:usize>(){");
        for i in 0..count {
            use std::fmt::Write;
            write!(source, "let r{i}:&'static[usize;N]=&[N+{i};N];")?;
        }
        source.push('}');
        rustc::qualify(&format!("deferred-promotion-wide-{count}"), &source, true)?;
        let (report, mut facts) = semantic::analyze(&source)?;
        let mut templates = body::templates::build(&report, &facts)?;
        body::promote::analyze_templates(&report, &mut facts, &mut templates)?;
        let promotion = templates.bodies[0]
            .body
            .promotions
            .as_ref()
            .ok_or("missing promotion")?;
        assert_eq!(promotion.deferred().count(), count);
        assert_eq!(promotion.entries().count(), 0);
        assert!(promotion.steps <= count * 32);
    }
    Ok(())
}

#[test]
fn deferred_proofs_cannot_authorize_concrete_execution() -> Result<(), Box<dyn std::error::Error>> {
    let (report, mut facts) = semantic::analyze("fn f<const N:usize>()->&'static usize{&N}")?;
    let mut templates = body::templates::build(&report, &facts)?;
    body::promote::analyze_templates(&report, &mut facts, &mut templates)?;
    let contracts = body::regions::collect(&facts)?;
    let contracts = borrow::regions::check_templates(
        &facts.types,
        &facts.lifetimes.variances,
        &contracts,
        &templates,
    )?;
    template::verify(&facts, &templates, &contracts)?;
    let program = body::ir::Program {
        bodies: templates.bodies.into_iter().map(|t| t.body).collect(),
        drops: None,
        region_equalities: Vec::new(),
        coercions: Vec::new(),
        outlives: Vec::new(),
        body_regions: Vec::new(),
    };
    let mut copies = Copies::default();
    let mut environment = borrow::Environment {
        types: &facts.types,
        definitions: &facts.aggregates,
        derives: &facts.derives,
        copies: &mut copies,
    };
    let error = borrow::verify(&mut environment, &contracts, &program)
        .err()
        .ok_or("unresolved promotion authorized execution")?;
    assert!(
        error
            .message
            .contains("deferred promotion values require concrete instantiation")
    );
    let catalog =
        body::drops::glue::Catalog::new(&facts.types, &facts.aggregates, &facts.members, &[])?;
    let error = borrow::verify_destruction(&mut environment, &contracts, &program, catalog)
        .err()
        .ok_or("destruction proof authorized unresolved promotion")?;
    assert!(
        error
            .message
            .contains("deferred promotion values require concrete instantiation")
    );
    Ok(())
}
