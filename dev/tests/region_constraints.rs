#[path = "support/semantic.rs"]
mod semantic;
use rustgm::{
    borrow::regions::{constraints::Constraints, graph::Error},
    sema::model::{AdjustmentKind, Cast, Operation},
    types::{Kind, Primitive, RegionId, Store, TypeId, variance::Variances},
};

fn reference(
    store: &mut Store,
    region: RegionId,
    mutable: bool,
    referent: TypeId,
) -> Result<TypeId, &'static str> {
    store.intern(Kind::Reference {
        region,
        mutable,
        referent,
    })
}

#[test]
fn retained_identities_include_only_root_parameters_and_registered_variables()
-> Result<(), Box<dyn std::error::Error>> {
    let mut store = Store::default();
    let a = RegionId::bound(0, 0)?;
    let bound = reference(&mut store, a, false, TypeId::UNIT)?;
    let static_ = reference(&mut store, RegionId::STATIC, false, TypeId::UNIT)?;
    let poly = function(&mut store, 1, &[bound], bound)?;
    let mono = function(&mut store, 0, &[static_], static_)?;
    let variances = Variances::default();
    let mut constraints = Constraints::new(&store, &variances, 1, 10_000)?;
    constraints.variable(RegionId(17))?;
    constraints.coerce(poly, mono, false, 23)?;
    let proof = constraints.prove()?;
    for (id, kind) in [
        (
            a,
            rustgm::borrow::regions::graph::Kind::Universal { universe: 0 },
        ),
        (
            RegionId(17),
            rustgm::borrow::regions::graph::Kind::Variable { universe: 0 },
        ),
        (
            RegionId::STATIC,
            rustgm::borrow::regions::graph::Kind::Static,
        ),
    ] {
        let region = proof.identities.get(id).ok_or("missing body identity")?;
        assert_eq!(proof.solution.kind(region), Some(kind));
    }
    for absent in [RegionId(18), RegionId::bound(0, 1)?, RegionId::bound(1, 0)?] {
        assert_eq!(proof.identities.get(absent), None);
    }
    Ok(())
}
fn function(
    store: &mut Store,
    regions: u32,
    parameters: &[TypeId],
    result: TypeId,
) -> Result<TypeId, &'static str> {
    let parameters = store.intern_list(parameters)?;
    store.intern(Kind::Function {
        bound_regions: regions,
        parameters,
        result,
        unsafe_: false,
    })
}

#[test]
fn shared_referents_shorten_but_mutable_referents_remain_invariant()
-> Result<(), Box<dyn std::error::Error>> {
    for mutable in [false, true] {
        let mut store = Store::default();
        let a = RegionId::bound(0, 0)?;
        let b = RegionId::bound(0, 1)?;
        let left = reference(&mut store, a, false, TypeId::UNIT)?;
        let right = reference(&mut store, b, false, TypeId::UNIT)?;
        let left = reference(&mut store, a, mutable, left)?;
        let right = reference(&mut store, b, mutable, right)?;
        let variances = Variances::default();
        let mut constraints = Constraints::new(&store, &variances, 2, 10_000)?;
        constraints.assume(a, b)?;
        constraints.coerce(left, right, false, 0)?;
        assert_eq!(constraints.solve().is_ok(), !mutable);
    }
    Ok(())
}

#[test]
fn callable_subsumption_instantiates_source_binders_without_capturing_the_target()
-> Result<(), Box<dyn std::error::Error>> {
    let mut store = Store::default();
    let byte = TypeId::primitive(Primitive::U8);
    let bound = reference(&mut store, RegionId::bound(0, 0)?, false, byte)?;
    let static_ = reference(&mut store, RegionId::STATIC, false, byte)?;
    let variable = reference(&mut store, RegionId(1), false, byte)?;
    let poly = function(&mut store, 1, &[bound], bound)?;
    let mono = function(&mut store, 0, &[static_], static_)?;
    let early = function(&mut store, 0, &[variable], variable)?;
    let variances = Variances::default();
    for (actual, expected, accepted) in [
        (poly, mono, true),
        (mono, poly, false),
        (early, poly, false),
    ] {
        let mut constraints = Constraints::new(&store, &variances, 0, 10_000)?;
        constraints.variable(RegionId(1))?;
        constraints.coerce(actual, expected, false, 0)?;
        assert_eq!(constraints.solve().is_ok(), accepted);
    }
    Ok(())
}

#[test]
fn callable_inputs_are_contravariant_and_captured_lifetimes_use_parent_scopes()
-> Result<(), Box<dyn std::error::Error>> {
    let mut store = Store::default();
    let a = RegionId::bound(0, 0)?;
    let b = RegionId::bound(0, 1)?;
    let captured_a = reference(&mut store, RegionId::bound(1, 0)?, false, TypeId::UNIT)?;
    let captured_b = reference(&mut store, RegionId::bound(1, 1)?, false, TypeId::UNIT)?;
    let actual = function(&mut store, 0, &[captured_b], captured_a)?;
    let expected = function(&mut store, 0, &[captured_a], captured_b)?;
    let variances = Variances::default();
    let mut constraints = Constraints::new(&store, &variances, 2, 10_000)?;
    constraints.assume(a, b)?;
    constraints.coerce(actual, expected, false, 0)?;
    assert!(constraints.solve().is_ok());
    let mut reverse = Constraints::new(&store, &variances, 2, 10_000)?;
    reverse.assume(a, b)?;
    reverse.coerce(expected, actual, false, 0)?;
    assert!(matches!(reverse.solve(), Err(Error::Unsatisfied { .. })));
    Ok(())
}

fn contract(source: &str) -> Result<(), String> {
    let (_, facts) = semantic::analyze(source)?;
    let contracts = rustgm::body::regions::collect(&facts)?;
    rustgm::borrow::regions::check(&facts.types, &facts.lifetimes.variances, &contracts)
        .map(|_| ())
        .map_err(|e| e.to_string())
}

fn check_contracts(cases: &[(&str, bool)]) -> Result<(), Box<dyn std::error::Error>> {
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../obj/region-contracts-{}-{}",
        std::process::id(),
        std::thread::current().name().unwrap_or("unnamed")
    ));
    std::fs::create_dir_all(&directory)?;
    for (index, &(source, accepted)) in cases.iter().enumerate() {
        let source_path = directory.join(format!("case-{index}.rs"));
        std::fs::write(&source_path, source)?;
        let rustc = std::process::Command::new("rustc")
            .args([
                "+1.98.0",
                "--edition=2024",
                "--crate-type=lib",
                "--crate-name=probe",
                "--emit=metadata",
                "--cap-lints=allow",
                "-o",
            ])
            .arg(directory.join("probe.rmeta"))
            .arg(source_path)
            .output()?;
        std::fs::write(
            directory.join(format!("case-{index}.stderr")),
            &rustc.stderr,
        )?;
        assert_eq!(
            rustc.status.code(),
            Some(i32::from(!accepted)),
            "rustc: {source}: {}",
            String::from_utf8_lossy(&rustc.stderr)
        );
        let result = contract(source);
        assert_eq!(result.is_ok(), accepted, "RustGM: {source}: {result:?}");
        if let Err(message) = result {
            assert!(
                message.contains("required lifetime does not outlive its target"),
                "{message}"
            );
        }
    }
    Ok(())
}

#[test]
fn source_contracts_apply_explicit_bounds_variance_and_call_constraints()
-> Result<(), Box<dyn std::error::Error>> {
    check_contracts(&[
        ("fn f<'a,'b>(x:&'a u8)->&'b u8{x}", false),
        ("fn f<'a:'b,'b>(x:&'a u8)->&'b u8{x}", true),
        ("fn f<'a:'b,'b:'c,'c>(x:&'a u8)->&'c u8{x}", true),
        ("fn f<'a:'b,'b>(x:&'b u8)->&'a u8{x}", false),
        (
            "fn f<'a,'b>(x:&'a u8,y:&'b u8){let mut z=x;z=y;let _=z;}",
            true,
        ),
        (
            "fn f<'a,'b>(x:&'a u8,y:&'b u8)->&'a u8{let mut z=x;z=y;z}",
            false,
        ),
        (
            "fn id(x:&u8)->&u8{x}fn f<'a>(x:&'a u8)->&'a u8{id(x)}",
            true,
        ),
        (
            "fn id(x:&u8)->&u8{x}fn f<'a,'b>(x:&'a u8)->&'b u8{id(x)}",
            false,
        ),
        (
            "struct S<'a>(&'a u8);fn f<'a:'b,'b>(x:S<'a>)->S<'b>{x}",
            true,
        ),
        (
            "struct S<'a>(&'a u8);fn f<'a:'b,'b>(x:S<'b>)->S<'a>{x}",
            false,
        ),
        (
            "struct S<'a>(fn(&'a u8));fn f<'a:'b,'b>(x:S<'b>)->S<'a>{x}",
            true,
        ),
        (
            "struct S<'a>(fn(&'a u8));fn f<'a:'b,'b>(x:S<'a>)->S<'b>{x}",
            false,
        ),
        (
            "struct S<'a>(*mut &'a u8);fn f<'a:'b,'b>(x:S<'a>)->S<'b>{x}",
            false,
        ),
        (
            "fn required<'a:'static>(){}fn invalid<'a>(){required::<'a>();}",
            false,
        ),
    ])
}

#[test]
fn implied_bounds_come_from_inputs_and_outputs_but_not_local_annotations()
-> Result<(), Box<dyn std::error::Error>> {
    check_contracts(&[
        ("fn f<'a,'b>(x:&'a &'b u8,y:&'b u8)->&'a u8{y}", true),
        ("fn f<'a,'b>(x:&'a &'b u8,y:&'a u8)->&'b u8{y}", false),
        ("fn f<'a,'b>(x:&'a fn(&'b u8),y:&'b u8)->&'a u8{y}", true),
        ("fn f<'a,'b>(x:&'a fn()->&'b u8,y:&'b u8)->&'a u8{y}", true),
        ("fn f<'a,'b>(x:*const &'b u8,y:&'b u8)->&'a u8{y}", false),
        ("fn f<'a,'b>(x:*const &'a &'b u8,y:&'b u8)->&'a u8{y}", true),
        ("fn f<'a,'b>(x:fn(&'a &'b u8),y:&'b u8)->&'a u8{y}", true),
        ("fn f<'a,'b>(x:fn()->&'a &'b u8,y:&'b u8)->&'a u8{y}", true),
        (
            "fn f<'a,'b>(x:for<'c>fn(&'c &'b u8),y:&'b u8)->&'a u8{y}",
            false,
        ),
        (
            "fn f<'a,'b>(x:&'b u8)->&'a &'b u8{let y:&'a u8=x;loop{}}",
            true,
        ),
        (
            "fn f<'a,'b>(x:&'b u8){let y:&'a &'b u8;let z:&'a u8=x;}",
            false,
        ),
        ("fn f<'a,'b>(x:&'a *const &'b u8,y:&'b u8)->&'a u8{y}", true),
        (
            "struct S<'a,'b>(&'a &'b u8);fn f<'a,'b>(x:S<'a,'b>,y:&'b u8)->&'a u8{y}",
            true,
        ),
        (
            "struct S<'a,'b>(&'a &'b u8);fn f<'a,'b>(x:&'b u8)->S<'a,'b>{let y:&'a u8=x;loop{}}",
            true,
        ),
        (
            "struct S<'a,'b>(fn(&'a &'b u8));fn f<'a,'b>(x:S<'a,'b>,y:&'b u8)->&'a u8{y}",
            true,
        ),
        (
            "struct C<'b>(fn(&'b u8));fn f<'a,'b>(x:&'a C<'b>,y:&'b u8)->&'a u8{y}",
            true,
        ),
        (
            "struct S<'a,'b>(&'a &'b u8);fn f<'a,'b>(x:&'b u8){let y:S<'a,'b>;let z:&'a u8=x;}",
            false,
        ),
        (
            "enum E<'a,'b>{A(&'a &'b u8),B}fn f<'a,'b>(x:E<'a,'b>,y:&'b u8)->&'a u8{y}",
            true,
        ),
        (
            "enum E<'a,'b>{A(&'a &'b u8),B}fn f<'a,'b>(){let x:E<'a,'b>=E::B;}",
            false,
        ),
        (
            "struct S<'a,'b>(&'a &'b u8);struct W<'a,'b>(S<'b,'a>);fn f<'a,'b>(x:W<'a,'b>,y:&'a u8)->&'b u8{y}",
            true,
        ),
        (
            "struct S<'a,'b>(&'a &'b u8);struct W<'a,'b>(S<'b,'a>);fn f<'a,'b>(x:W<'a,'b>,y:&'b u8)->&'a u8{y}",
            false,
        ),
    ])
}

#[test]
fn mutable_unsizing_keeps_the_element_type_invariant() -> Result<(), Box<dyn std::error::Error>> {
    check_contracts(&[
        ("fn f<'a:'b,'b>(x:&'b [&'a u8;1])->&'b [&'b u8]{x}", true),
        (
            "fn f<'a:'b,'b>(x:&'b mut [&'a u8;1])->&'b mut [&'b u8]{x}",
            false,
        ),
        (
            "fn f<'a:'b,'b>(x:&'b mut [&'a u8;1])->&'b mut [&'a u8]{x}",
            true,
        ),
    ])
}

#[test]
fn callable_specialization_checks_binding_time_in_the_region_proof()
-> Result<(), Box<dyn std::error::Error>> {
    check_contracts(&[
        (
            "fn f<'b>(p:for<'a>fn(&'a &'b u8,&'a u8)->&'a u8){let q:for<'a>fn(&'a &'b u8,&'b u8)->&'a u8=p;}",
            false,
        ),
        (
            "fn f<'b>(p:for<'a>fn(&'a &'b u8,&'b u8)->&'a u8){let q:for<'a>fn(&'a &'b u8,&'a u8)->&'a u8=p;}",
            false,
        ),
        (
            "fn f<'b:'static>(p:for<'a>fn(&'a &'b u8,&'a u8)->&'a u8){let q:for<'a>fn(&'a &'b u8,&'b u8)->&'a u8=p;}",
            true,
        ),
        (
            "fn id(x:&u8)->&u8{x}fn f(){let _:fn(&'static u8)->&'static u8=id;}",
            true,
        ),
        (
            "fn id<'a:'a>(x:&'a u8)->&'a u8{x}fn f(){let _:for<'b>fn(&'b u8)->&'b u8=id;}",
            false,
        ),
        (
            "fn f(p:for<'a>fn(&'a u8)->&'a u8){let _:fn(&'static u8)->&'static u8=p;}",
            true,
        ),
        (
            "fn f(p:fn(&'static u8)->&'static u8){let _:for<'a>fn(&'a u8)->&'a u8=p;}",
            false,
        ),
        (
            "fn f<'a>(p:for<'b>fn(&'b u8)->&'b u8){let _:fn(&'a u8)->&'a u8=p;}",
            true,
        ),
        (
            "fn f<'a>(p:fn(&'a u8)->&'a u8){let _:for<'b>fn(&'b u8)->&'b u8=p;}",
            false,
        ),
        (
            "fn f<'a:'b,'b>(p:fn(&'b u8)->&'a u8){let _:fn(&'a u8)->&'b u8=p;}",
            true,
        ),
        (
            "fn f<'a:'b,'b>(p:fn(&'a u8)->&'b u8){let _:fn(&'b u8)->&'a u8=p;}",
            false,
        ),
    ])
}

#[test]
fn joins_preserve_a_function_items_coupled_early_lifetimes()
-> Result<(), Box<dyn std::error::Error>> {
    check_contracts(&[
        (
            "fn g<'a:'a>(x:&'a u8)->&'a u8{x}fn f<'a>(){let mut p=g::<'a>;p=g::<'static>;}",
            false,
        ),
        (
            "fn g<'a:'a>(x:&'a u8)->&'a u8{x}fn f<'a>(x:&'a u8,c:bool)->&'a u8{let p=if c{g::<'static>}else{g::<'a>};p(x)}",
            false,
        ),
        (
            "fn g<'a:'a>(x:&'a u8)->&'a u8{x}fn f<'a>(x:&'a u8,c:bool)->&'a u8{let p=if c{g::<'a>}else{g::<'a>};p(x)}",
            true,
        ),
        (
            "fn g<'a:'a>(x:&'a u8)->&'a u8{x}fn f<'a>(c:bool){let p=if c{g::<'a>}else{g::<'static>};}",
            false,
        ),
        (
            "fn a(x:&u8)->&u8{x}fn b(x:&u8)->&u8{x}fn f(x:&u8,c:bool)->&u8{let p=if c{a}else{b};p(x)}",
            true,
        ),
    ])
}

#[test]
fn explicit_function_casts_keep_quantifier_obligations_and_one_selected_operation()
-> Result<(), Box<dyn std::error::Error>> {
    check_contracts(&[
        (
            "fn id(x:&u8)->&u8{x}fn f(){let p=id as fn(&'static u8)->&'static u8;}",
            true,
        ),
        (
            "fn id(x:&u8)->&u8{x}fn f<'a>(){let p=id as fn(&'a u8)->&'a u8;}",
            true,
        ),
        (
            "fn id<'a:'a>(x:&'a u8)->&'a u8{x}fn f(){let p=id as for<'b>fn(&'b u8)->&'b u8;}",
            false,
        ),
        (
            "fn id(x:&'static u8)->&'static u8{x}fn f(){let p=id as for<'b>fn(&'b u8)->&'b u8;}",
            false,
        ),
        (
            "fn id<'a:'a>(x:&'a u8)->&'a u8{x}fn f(){let p=id::<'static> as fn(&'static u8)->&'static u8;}",
            true,
        ),
    ])?;
    let (_, facts) =
        semantic::analyze("fn id(x:&u8)->&u8{x}fn f(){let _=id as fn(&'static u8)->&'static u8;}")?;
    let casts: Vec<_> = facts
        .nodes
        .iter()
        .flatten()
        .filter(|f| {
            matches!(
                f.operation,
                Some(Operation::Cast {
                    kind: Cast::FunctionPointer,
                    ..
                })
            )
        })
        .collect();
    assert_eq!(casts.len(), 1);
    let conversions: Vec<_> = facts
        .coercions
        .iter()
        .filter(|c| {
            c.kind == AdjustmentKind::FunctionPointer
                && matches!(
                    facts.node(c.source).and_then(|f| f.operation),
                    Some(Operation::Cast { .. })
                )
        })
        .collect();
    assert_eq!(conversions.len(), 1);
    let operand = facts
        .nodes
        .iter()
        .flatten()
        .find(|f| f.ty == Some(conversions[0].actual) && f.reference.is_some())
        .ok_or("missing function item operand")?;
    assert!(operand.adjustment.is_none());
    Ok(())
}

#[test]
fn inferred_item_arguments_obey_declared_variance_without_coupling_distinct_formals()
-> Result<(), Box<dyn std::error::Error>> {
    check_contracts(&[
        (
            "fn g<'a:'a>(_:&'a u8){}fn f<'a>(){let mut p=g::<'a>;p=g::<'static>;}",
            true,
        ),
        (
            "fn g<'a:'a>(_:&'a u8){}fn f<'a>(){let mut p=g::<'static>;p=g::<'a>;}",
            true,
        ),
        (
            "fn g<'a:'a>(_:&'a u8){}fn f<'a>(x:&'a u8){let mut p=g::<'a>;p=g::<'static>;p(x)}",
            false,
        ),
        (
            "fn g<'a:'a,'b:'b>(_:&'a u8)->&'b u8{loop{}}fn f<'a>(){let mut p=g::<'a,'a>;p=g::<'static,'a>;let _:fn(&'static u8)->&'a u8=p;}",
            true,
        ),
        (
            "fn g<'a:'a,'b>(_:&'a u8,x:&'b u8)->&'b u8{x}fn f<'a,'b>(s:&'static u8,x:&'b u8)->&'b u8{let mut p=g::<'a>;p=g::<'static>;p(s,x)}",
            true,
        ),
        (
            "struct S<'a>(&'a u8);fn f<'a>(){let mut c=S::<'a>;c=S::<'static>;}",
            false,
        ),
        (
            "fn g<'a:'static>(_:&'a u8){}fn f<'a>(){let mut p=g::<'static>;p=g::<'a>;}",
            false,
        ),
        (
            "fn g<'a>()->&'a u8{loop{}}fn f<'a>(){let mut p=g::<'a>;p=g::<'static>;}",
            true,
        ),
        (
            "fn g<'a>()->&'a u8{loop{}}fn f<'a>(){let mut p=g::<'static>;p=g::<'a>;}",
            true,
        ),
        (
            "fn g<'a>()->&'a u8{loop{}}fn f<'a>()->&'static u8{let mut p=g::<'static>;p=g::<'a>;p()}",
            false,
        ),
    ])
}

#[test]
fn nominal_bounds_are_enforced_and_implied_while_alias_bounds_remain_unenforced()
-> Result<(), Box<dyn std::error::Error>> {
    check_contracts(&[
        (
            "struct S<'a:'b,'b>(&'a u8,&'b u8);fn f<'a,'b>(s:S<'a,'b>,x:&'a u8)->&'b u8{x}",
            true,
        ),
        (
            "struct S<'a,'b>(&'a u8,&'b u8) where 'a:'b;fn f<'a,'b>(s:S<'a,'b>,x:&'a u8)->&'b u8{x}",
            true,
        ),
        (
            "struct S<'a:'b,'b>(&'a u8,&'b u8);fn f<'a,'b>(x:&'a u8,y:&'b u8){let s:S<'a,'b>=S(x,y);}",
            false,
        ),
        (
            "struct S<'a:'b,'b>(&'a u8,&'b u8);fn f<'a,'b>(x:&'a u8,y:&'b u8){let s=S(x,y);}",
            true,
        ),
        (
            "type R<'a:'static>=&'a u8;fn f<'a>(x:&'a u8)->R<'a>{x}",
            true,
        ),
        (
            "type R<'a> where 'a:'static = &'a u8;fn f<'a>(x:&'a u8)->R<'a>{x}",
            true,
        ),
        (
            "type R<'a:'b,'b>=(&'a u8,&'b u8);fn f<'a,'b>(x:R<'a,'b>)->&'b u8{x.0}",
            false,
        ),
        (
            "struct S<'a:'static>(&'a u8);type R<'a>=S<'a>;fn f<'a>(x:R<'a>)->&'static u8{x.0}",
            true,
        ),
        (
            "struct S<'a:'static>(&'a u8);type R<'a>=S<'a>;fn f<'a>(x:&'a u8){let y:R<'a>=S(x);}",
            false,
        ),
        ("struct S where 'static:'static;", true),
    ])
}

#[test]
fn partial_local_annotations_reuse_declared_binders_without_capturing_inferred_holes()
-> Result<(), Box<dyn std::error::Error>> {
    check_contracts(&[
        (
            "fn g(x:&u8){}fn f(){let p:fn(_)=g;let _:for<'a>fn(&'a u8)=p;}",
            false,
        ),
        (
            "fn f(p:fn()->for<'a>fn(&'a u8)->&'a u8){let q:fn()->fn(_)->_=p;}",
            true,
        ),
        ("fn f(p:fn(for<'a>fn(&'a u8))){let q:fn(fn(_))=p;}", false),
        (
            "fn f(p:&for<'a>fn(&'a u8)->&'a u8){let q:&fn(_)->_=p;}",
            true,
        ),
        (
            "fn f(p:for<'a>fn(&'a &'a u8)){let q:for<'a>fn(&'a _)=p;}",
            true,
        ),
        ("fn f<'a>(p:fn(&'a u8)){let q:fn(_)=p;}", true),
        ("fn f(x:&u8)->&u8{let y:&_=x;y}", true),
        ("fn f<'a>(x:&'a u8)->&'a u8{let y:&'a _=x;y}", true),
        ("fn f<'a,'b>(x:&'a u8)->&'a u8{let y:&'b _=x;y}", false),
        (
            "struct S<'a>(&'a u8);fn f<'a>(x:&'a u8){let y:(S<'a>,_)= (S(x),1u8);}",
            true,
        ),
        (
            "fn id(x:&u8)->&u8{x}fn f(x:&u8)->&u8{let p:fn(&_) -> &_=id;p(x)}",
            true,
        ),
        (
            "fn id(x:&u8)->&u8{x}fn f(x:&u8)->&u8{let p:for<'a>fn(&'a _) -> &'a _=id;p(x)}",
            true,
        ),
        (
            "fn f<'a>(p:fn(&'a u8)->&'a u8,x:&'a u8)->&'a u8{let q:fn(&'a _)->&'a _=p;q(x)}",
            true,
        ),
        (
            "fn id(x:&u8)->&u8{x}fn f(x:&u8)->&u8{let p:fn(_) -> _=id;p(x)}",
            true,
        ),
        (
            "fn f(p:for<'a,'b>fn(&'a u8,&'b u8)){let q:for<'a>fn(_,&'a _)=p;}",
            true,
        ),
        (
            "fn id(x:&u8)->&u8{x}fn f<'a,'b>(x:&'a u8,y:&'b u8)->&'a u8{let p:fn(_)->_=id;let _=p(y);p(x)}",
            false,
        ),
        (
            "fn id(x:&u8)->&u8{x}fn f<'a,'b>(x:&'a u8,y:&'b u8)->&'a u8{let p:fn(&_)->&_=id;let _=p(y);p(x)}",
            true,
        ),
        (
            "fn id(x:&u8)->&u8{x}fn f(){let p:fn(_)->_=id;let _:for<'a>fn(&'a u8)->&'a u8=p;}",
            false,
        ),
        (
            "fn id(x:&u8)->&u8{x}fn f<'a>(){let p:for<'b>fn(_)->&'a _=id;}",
            true,
        ),
    ])
}
