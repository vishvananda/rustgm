//! Each specialization has a fresh lifetime context even when source keys coincide.
use crate::qualify as rustc;
use crate::semantic;
use rustgm::{
    body::{self, ir::Program, regions::Contracts},
    borrow,
    types::{Kind, copy::Copies, instance::Instance},
};

fn verify(source: &str, name: &str) -> Result<usize, Box<dyn std::error::Error>> {
    rustc::qualify(name, source, true)?;
    let (report, mut facts) = semantic::analyze(source)?;
    let mut templates = body::templates::build(&report, &facts)?;
    body::promote::analyze_templates(&report, &mut facts, &mut templates)?;
    let contracts = body::regions::collect(&facts)?;
    let mut copies = Copies::default();
    let mut environment = borrow::Environment {
        types: &facts.types,
        definitions: &facts.aggregates,
        derives: &facts.derives,
        source_copy: facts.uses_source_copy(),
        copies: &mut copies,
    };
    let checked = borrow::templates::verify(
        &mut environment,
        &facts.lifetimes.variances,
        &contracts,
        &templates,
        &[],
        |_, _| Err("unexpected nominal selection"),
    )?;
    let ordinary = body::build_concrete(&report, &facts)?;
    let mut instances = Vec::new();
    for value in ordinary.bodies.iter().flat_map(|b| &b.values) {
        if let Some(Kind::FunctionItem { item, .. }) = facts.types.kind(value.ty)
            && checked.get(item).is_some()
        {
            let instance = Instance::new(&facts.types, value.ty)?;
            if !instances.contains(&instance) {
                instances.push(instance);
            }
        }
    }
    let mut program = Program {
        bodies: vec![],
        drops: None,
        region_equalities: vec![],
        coercions: vec![],
        outlives: vec![],
        body_regions: vec![],
    };
    let mut inputs = Vec::new();
    for instance in instances {
        let Some(Kind::FunctionItem { item, .. }) = facts.types.kind(instance.ty()) else {
            return Err("missing selected function".into());
        };
        let materialized = body::instantiate::with_contract(
            &mut facts,
            checked.get(item).ok_or("missing template")?,
            instance,
            checked.input(item).ok_or("missing input contract")?,
        );
        if materialized.is_err() {
            let mut dump = Vec::new();
            rustgm::sema::type_view::types(&facts.types, &mut dump)?;
            eprintln!(
                "{name} selected {instance:?}\n{}",
                String::from_utf8_lossy(&dump)
            );
        }
        let mut materialized = materialized?;
        body::promote::instance(
            &report,
            &mut facts,
            &mut materialized.body,
            &materialized.destruction,
        )?;
        inputs.push(materialized.contract);
        program.bodies.push(materialized.body);
    }
    assert!(!program.bodies.is_empty(), "{name}");
    let contracts = Contracts {
        bodies: inputs,
        callables: &facts.lifetimes.callables,
        nominal_requirements: &facts.lifetimes.nominal_requirements,
    };
    let mut regions = borrow::regions::check_ordered(
        &facts.types,
        &facts.lifetimes.variances,
        &contracts,
        &program,
    )?;
    let mut environment = borrow::Environment {
        types: &facts.types,
        definitions: &facts.aggregates,
        derives: &facts.derives,
        source_copy: facts.uses_source_copy(),
        copies: &mut copies,
    };
    // A normal source-keyed proof cannot authorize specialized bodies.
    assert!(borrow::verify(&mut environment, &regions, &program).is_err());
    if regions.bodies.len() > 1 {
        regions.bodies.swap(0, 1);
        assert!(borrow::verify_instances(&mut environment, &regions, &program, &checked).is_err());
        regions.bodies.swap(0, 1);
    }
    borrow::verify_instances(&mut environment, &regions, &program, &checked)?;
    Ok(program.bodies.len())
}

#[test]
fn argument_lifetimes_are_checked_in_distinct_instance_contexts()
-> Result<(), Box<dyn std::error::Error>> {
    let cases = [
        (
            "projected-early-identity",
            "trait A<'a>{type Out;}impl<'a,'b:'a>A<'a>for &'b u8{type Out=u16;}fn f<'a,T:A<'a>>(x:T::Out)->T::Out{x}fn use_<'a>(_:&'a u8)->u16{f::<'a,&'a u8>(7)}",
            1,
        ),
        (
            "identity",
            "fn id<T>(x:T)->T{x}fn use_<'a,'b>(x:&'a u8,y:&'b u16)->(&'a u8,&'b u16){(id(x),id(y))}",
            2,
        ),
        (
            "late-slot",
            "fn f<'a,T>(x:&'a T)->&'a T{x}fn use_<'a,'b>(x:&'a &'b u8)->&'a &'b u8{f(x)}",
            1,
        ),
        (
            "early-slot",
            "fn f<'a,T:'a>(x:&'a T)->&'a T{x}fn use_<'a,'b>(x:&'a &'b u8)->&'a &'b u8{f(x)}",
            1,
        ),
        (
            "nested",
            "fn id<T>(x:T)->T{x}fn use_<'a,'b>(x:&'a &'b u8)->&'a &'b u8{id(x)}",
            1,
        ),
        (
            "local-borrow",
            "fn id<T>(x:T)->T{let r=&x;let s=r;x}fn use_<'a>(x:&'a u8)->&'a u8{id(x)}",
            1,
        ),
        (
            "callable",
            "fn f<T>(g:fn(T)->T,x:T)->T{g(x)}fn use_<'a>(g:fn(&'a u8)->&'a u8,x:&'a u8)->&'a u8{f(g,x)}",
            1,
        ),
        (
            "static-bound",
            "fn f<T:'static>(x:T)->T{x}fn use_(x:&'static u8)->&'static u8{f(x)}",
            1,
        ),
    ];
    for (name, source, count) in cases {
        assert_eq!(
            verify(source, name).map_err(|e| format!("{name}: {e}"))?,
            count
        );
    }
    Ok(())
}
