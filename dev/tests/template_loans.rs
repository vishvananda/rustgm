use crate::qualify as rustc;
use crate::semantic;

use rustgm::{body, borrow, types::outlives::LiveRegions};

const CASES: &[(&str, &str, bool)] = &[
    (
        "generic-call-shared",
        "fn id<T:Copy>(x:T)->T{x}fn f<T:Copy>(x:T){let r=&x;let y=id(x);let z=r;}",
        true,
    ),
    (
        "generic-call-mutable",
        "fn id<T:Copy>(x:T)->T{x}fn f<T:Copy>(mut x:T){let r=&mut x;let y=id(x);let z=r;}",
        false,
    ),
    (
        "separate-owners",
        "fn f<T>(x:&mut T)->&T{&*x}fn g<U>(x:&mut U)->&U{&*x}",
        true,
    ),
    ("identity", "fn f<T>(x:&T)->&T{x}", true),
    ("shared-reborrow", "fn f<T>(x:&mut T)->&T{&*x}", true),
    (
        "mutable-reborrow",
        "fn f<T>(x:&mut T)->&mut T{&mut*x}",
        true,
    ),
    ("shared-pair", "fn f<T>(x:&T)->(&T,&T){(x,x)}", true),
    (
        "mutable-pair",
        "fn f<T>(x:&mut T)->(&mut T,&mut T){(x,x)}",
        false,
    ),
    (
        "copy-reference-pair",
        "fn f<'a,T:'a>(x:&'a mut T)->(&'a mut T,&'a mut T)where &'a mut T:Copy{(x,x)}",
        false,
    ),
    (
        "copy-reference-calls",
        "fn take<'a,T:'a>(x:&'a mut T)where &'a mut T:Copy{}fn f<'a,T:'a>(x:&'a mut T)where &'a mut T:Copy{take::<'a,T>(x);take::<'a,T>(x);}",
        false,
    ),
    (
        "reborrow-last-use",
        "fn f<T>(x:&mut T){let r=&mut*x;let s=&*r;let use_s=s;let again=&mut*x;}",
        true,
    ),
    (
        "reborrow-conflict",
        "fn f<T>(x:&mut T){let r=&mut*x;let again=&mut*x;let s=&*r;}",
        false,
    ),
    (
        "replace-live",
        "fn f<T:Copy>(mut x:T,y:T){let r=&x;x=y;let z=r;}",
        false,
    ),
    (
        "replace-ended",
        "fn f<T:Copy>(mut x:T,y:T){let r=&x;let z=r;x=y;}",
        true,
    ),
    (
        "read-shared",
        "fn f<T:Copy>(x:T){let r=&x;let y=x;let z=r;}",
        true,
    ),
    (
        "read-mutable",
        "fn f<T:Copy>(mut x:T){let r=&mut x;let y=x;let z=r;}",
        false,
    ),
    (
        "read-after-use",
        "fn f<T:Copy>(mut x:T){let r=&mut x;let z=r;let y=x;}",
        true,
    ),
    (
        "scope-ended",
        "fn f<T:Copy>(mut x:T,y:T){{let r=&mut x;let z=r;}x=y;}",
        true,
    ),
    (
        "copy-local-escape",
        "fn f<'a,T:Copy>(x:T)->&'a T{&x}",
        false,
    ),
    (
        "bound-is-directed",
        "fn f<'a,T:Copy+'a>(mut x:T,y:T){let r=&x;let z=r;x=y;}",
        true,
    ),
    (
        "static-bound-is-directed",
        "fn f<T:Copy+'static>(mut x:T,y:T){let r=&x;let z=r;x=y;}",
        true,
    ),
    (
        "array-reborrow",
        "fn f<T,const N:usize>(x:&[T;N])->&[T;N]{&*x}",
        true,
    ),
    (
        "empty-promotion",
        "fn f<T:'static>()->&'static[T;0]{&[]}",
        true,
    ),
    (
        "empty-mutable-promotion",
        "fn f<T:'static>()->&'static mut[T;0]{&mut[]}",
        true,
    ),
    ("constant-promotion", "fn f<T>()->&'static u8{&42u8}", true),
    (
        "branch-exclusive",
        "fn f<T>(x:&mut T,b:bool)->&mut T{if b{&mut*x}else{&mut*x}}",
        true,
    ),
    (
        "loop-last-use",
        "fn f<T>(x:&mut T,b:bool){while b{let r=&mut*x;let use_r=r;}}",
        true,
    ),
    (
        "loop-conflict",
        "fn f<T>(x:&mut T,b:bool){let r=&mut*x;while b{let s=&mut*x;let use_r=&*r;}}",
        false,
    ),
];

fn ownership(
    facts: &rustgm::sema::model::Declarations,
    templates: &body::templates::Templates,
) -> Result<Vec<borrow::ownership::BodyFacts>, String> {
    let custom = (!facts.destructors.implementations().is_empty())
        .then_some(|item| facts.destructors.item(item).is_some());
    borrow::ownership::templates::analyze(
        &facts.types,
        &facts.aggregates,
        &facts.derives,
        custom.as_ref(),
        templates,
    )
    .map_err(|e| e.message.to_owned())
}

fn provenance(
    facts: &rustgm::sema::model::Declarations,
    templates: &body::templates::Templates,
) -> Result<borrow::regions::Analysis, Box<dyn std::error::Error>> {
    let contracts = body::regions::collect(facts)?;
    Ok(borrow::regions::check_templates(
        &facts.types,
        &facts.lifetimes.variances,
        &contracts,
        templates,
    )?)
}

fn check(source: &str) -> Result<Vec<borrow::loans::Analysis>, Box<dyn std::error::Error>> {
    let (report, mut facts) = semantic::analyze(source)?;
    let mut templates = body::templates::build(&report, &facts)?;
    body::promote::analyze_templates(&report, &mut facts, &mut templates)?;
    let owned = ownership(&facts, &templates)?;
    let contracts = provenance(&facts, &templates)?;
    Ok(borrow::loans::templates::check(
        &facts.types,
        &mut LiveRegions::default(),
        &templates,
        &contracts,
        &owned,
    )?)
}

#[test]
fn declared_template_loan_outcomes_match_rustc() -> Result<(), Box<dyn std::error::Error>> {
    for (name, source, accepted) in CASES {
        rustc::qualify(&format!("template-loan-{name}"), source, *accepted)?;
    }
    Ok(())
}

#[test]
fn reference_and_copy_templates_obey_last_use_and_exclusive_access() -> Result<(), String> {
    let mut failures = Vec::new();
    for (name, source, accepted) in CASES {
        let result = check(source);
        if matches!(
            *name,
            "mutable-pair" | "copy-reference-pair" | "copy-reference-calls"
        ) && result.as_ref().err().is_some_and(|error| {
            !error
                .to_string()
                .contains("access conflicts with a live reference loan")
        }) {
            failures.push(format!("{name}: expected a completed loan conflict check"));
        }
        if result.is_ok() != *accepted {
            failures.push(format!(
                "{name}: wanted {accepted}; {}",
                result.err().map_or_else(String::new, |e| e.to_string())
            ));
        }
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures.join("\n"))
    }
}

#[test]
fn opaque_components_are_retained_and_must_belong_to_the_contract()
-> Result<(), Box<dyn std::error::Error>> {
    let source = "fn f<T>(x:&mut T)->&T{&*x}";
    let (report, mut facts) = semantic::analyze(source)?;
    let mut templates = body::templates::build(&report, &facts)?;
    body::promote::analyze_templates(&report, &mut facts, &mut templates)?;
    let owned = ownership(&facts, &templates)?;
    let mut regions = LiveRegions::default();
    let uses = borrow::liveness::analyze(&facts.types, &mut regions, &templates.bodies[0].body)?;
    assert!(!uses.seeds.is_empty());
    assert!(!uses.parameter_seeds.is_empty());
    let mut contracts = body::regions::collect(&facts)?;
    contracts.bodies[0].parameters.clear();
    contracts.bodies[0].signature = None;
    // This malformed proof has the right source but omits opaque identities.
    contracts.bodies[0]
        .obligations
        .retain(|o| !matches!(o.kind, body::regions::Kind::WellFormed(_)));
    let missing = borrow::regions::check_templates(
        &facts.types,
        &facts.lifetimes.variances,
        &contracts,
        &templates,
    )?;
    let error =
        borrow::loans::templates::check(&facts.types, &mut regions, &templates, &missing, &owned)
            .err()
            .ok_or("missing opaque identity was ignored")?;
    assert!(
        error
            .message
            .contains("missing live type-parameter identity"),
        "{error}"
    );
    Ok(())
}

#[test]
fn possible_cleanup_keeps_an_explicit_boundary_even_without_declared_drop()
-> Result<(), Box<dyn std::error::Error>> {
    let sources = [
        ("fn f<T>(x:T){let r=&x;let z=r;}", true),
        ("fn f<T>(mut x:T,y:T){let r=&x;x=y;let z=r;}", false),
        (
            "struct D<'a>(&'a mut u8);impl Drop for D<'_>{fn drop(&mut self){}}fn f<T>(x:T){let mut n=0u8;let d=D(&mut n);n=1;}",
            false,
        ),
    ];
    for (index, (source, accepted)) in sources.iter().enumerate() {
        rustc::qualify(
            &format!("template-loan-pending-drop-{index}"),
            source,
            *accepted,
        )?;
        let error = check(source)
            .err()
            .ok_or("possible cleanup was not proved")?;
        assert!(
            error.to_string().contains(
                "generic loan checking requires abstract cleanup for possible destruction"
            )
        );
    }
    Ok(())
}

#[test]
fn symbolic_live_components_keep_zero_arrays_binders_and_shared_shapes_distinct()
-> Result<(), Box<dyn std::error::Error>> {
    use rustgm::types::{
        ItemId, Kind, Primitive, RegionId, Store, TypeId,
        arguments::ConstKind,
        parameters::{Parameter, ParameterKind},
    };
    let mut types = Store::default();
    let id = types.intern_parameter(Parameter {
        owner: ItemId(0),
        index: 0,
        kind: ParameterKind::Type { sized: true },
    })?;
    let ty = types.intern(Kind::Parameter(id))?;
    let count = types.intern_parameter(Parameter {
        owner: ItemId(0),
        index: 1,
        kind: ParameterKind::Const {
            ty: TypeId::primitive(Primitive::Usize),
        },
    })?;
    let count = types.intern_const(ConstKind::Parameter(count))?;
    let region = RegionId::bound(0, 0)?;
    let borrowed = types.intern(Kind::Reference {
        mutable: false,
        region,
        referent: ty,
    })?;
    let empty = types.intern(Kind::Array {
        element: borrowed,
        length: 0,
    })?;
    let array = types.intern(Kind::ArrayTerm {
        element: borrowed,
        length: count,
    })?;
    let mut query = LiveRegions::default();
    let parts = query.symbolic(&types, empty)?;
    assert!(parts.regions.is_empty() && parts.parameters.is_empty());
    let parts = query.symbolic(&types, array)?;
    assert_eq!(parts.regions, &[region]);
    assert_eq!(parts.parameters, &[id]);
    assert!(query.get(&types, array).is_err());
    let inputs = types.intern_list(&[borrowed])?;
    let function = types.intern(Kind::Function {
        bound_regions: 1,
        parameters: inputs,
        result: TypeId::UNIT,
        unsafe_: false,
        convention: rustgm::types::callable::Convention::RUST,
    })?;
    let parts = query.symbolic(&types, function)?;
    assert!(parts.regions.is_empty());
    assert_eq!(parts.parameters, &[id]);
    let mut root = array;
    for _ in 0..20_000 {
        let list = types.intern_list(&[root, root])?;
        root = types.intern(Kind::Tuple(list))?;
    }
    let parts = query.symbolic(&types, root)?;
    assert_eq!(parts.regions, &[region]);
    assert_eq!(parts.parameters, &[id]);
    let steps = query.steps();
    for _ in 0..100 {
        assert_eq!(query.symbolic(&types, root)?.parameters, &[id]);
    }
    assert_eq!(query.steps(), steps);
    assert!(steps < 50_000);
    Ok(())
}

#[test]
fn template_loan_inputs_require_complete_unique_body_coverage()
-> Result<(), Box<dyn std::error::Error>> {
    let source = "fn f<T>(x:&mut T)->&T{&*x}fn g<U>(x:&mut U)->&U{&*x}";
    let (report, mut facts) = semantic::analyze(source)?;
    let mut templates = body::templates::build(&report, &facts)?;
    body::promote::analyze_templates(&report, &mut facts, &mut templates)?;
    let owned = ownership(&facts, &templates)?;
    let mut contracts = provenance(&facts, &templates)?;
    let check = |templates: &body::templates::Templates,
                 contracts: &borrow::regions::Analysis,
                 owned: &[borrow::ownership::BodyFacts]| {
        borrow::loans::templates::check(
            &facts.types,
            &mut LiveRegions::default(),
            templates,
            contracts,
            owned,
        )
    };
    assert!(check(&templates, &contracts, &owned[..1]).is_err());
    assert!(
        check(
            &body::templates::Templates { bodies: Vec::new() },
            &contracts,
            &owned
        )
        .is_err()
    );
    let source = contracts.bodies[0].source;
    contracts.bodies[1].source = source;
    assert!(check(&templates, &contracts, &owned).is_err());
    let contracts = provenance(&facts, &templates)?;
    templates.bodies[0].destruction.values.clear();
    assert!(check(&templates, &contracts, &owned).is_err());
    Ok(())
}
