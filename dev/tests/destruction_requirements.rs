use rustgm::{
    body::{self, drops::Trigger, ir::Owner},
    types::{ItemId, Kind, RegionArgs, Store, aggregate, destruction::NeedsDrop},
};
#[path = "helpers/qualify.rs"]
mod rustc;
#[path = "support/semantic.rs"]
mod semantic;

#[test]
fn closed_type_requirements_agree_with_rustc_and_cache_shared_shapes()
-> Result<(), Box<dyn std::error::Error>> {
    let declarations = "struct D;impl Drop for D{fn drop(&mut self){}}struct W(D);enum E{Empty,Full(W)}enum Z{A([D;0]),B}struct R<'a>(&'a D);";
    let cases = [
        ("D", true),
        ("W", true),
        ("E", true),
        ("Z", false),
        ("R<'static>", false),
        ("[D;0]", false),
        ("[D;1000000000]", true),
        ("&'static D", false),
        ("*mut D", false),
        ("fn(D)", false),
        ("(D,u8)", true),
        ("()", false),
    ];
    let mut independent = declarations.to_owned();
    for &(ty, expected) in &cases {
        use std::fmt::Write;
        write!(
            independent,
            "const _: [(); {}] = [(); std::mem::needs_drop::<{ty}>() as usize];",
            usize::from(expected)
        )?;
    }
    rustc::qualify("drop-requirements", &independent, true)?;
    let parameters = cases
        .iter()
        .enumerate()
        .map(|(i, (ty, _))| format!("_p{i}:{ty}"))
        .collect::<Vec<_>>()
        .join(",");
    let (report, facts) = semantic::analyze(&format!("{declarations}fn probe({parameters}){{}}"))?;
    let function = facts
        .functions
        .iter()
        .find(|f| {
            report
                .package
                .names
                .get(report.resolution.definitions[f.definition.0 as usize].name)
                == "probe"
        })
        .ok_or("missing probe")?;
    let Some(Kind::Function { parameters, .. }) = facts.types.kind(function.signature) else {
        return Err("missing signature".into());
    };
    let parameters = facts
        .types
        .list(parameters)
        .ok_or("missing parameter list")?;
    assert_eq!(parameters.len(), cases.len());
    let mut query = NeedsDrop::new(facts.destructors.implementations().iter().map(|i| i.item));
    for (&ty, &(name, expected)) in parameters.iter().zip(&cases) {
        assert_eq!(
            query.demand(&facts.types, &facts.aggregates, ty)?,
            expected,
            "{name}"
        );
    }
    let steps = query.steps;
    for &ty in parameters {
        query.demand(&facts.types, &facts.aggregates, ty)?;
    }
    assert_eq!(query.steps, steps);
    assert!(
        steps < 150,
        "array lengths must not enumerate elements: {steps}"
    );
    Ok(())
}

#[test]
fn destruction_queries_are_iterative_and_diagnose_by_value_cycles() -> Result<(), &'static str> {
    let mut types = Store::default();
    let item = ItemId(1);
    let nominal = types.intern(Kind::Adt(item, RegionArgs::EMPTY))?;
    let mut definitions = aggregate::Definitions::default();
    let fields = types.intern_list(&[])?;
    definitions.insert(
        item,
        aggregate::Kind::Struct,
        aggregate::Repr::default(),
        &[aggregate::Variant {
            fields,
            discriminant: 0,
        }],
    )?;
    let mut root = nominal;
    for _ in 0..20_000 {
        let pair = types.intern_list(&[root, root])?;
        root = types.intern(Kind::Tuple(pair))?;
    }
    let mut query = NeedsDrop::new([item]);
    assert!(query.demand(&types, &definitions, root)?);
    assert!(query.steps < types.len() * 10);
    let steps = query.steps;
    assert!(query.demand(&types, &definitions, root)?);
    assert_eq!(query.steps, steps);

    let mut cyclic = aggregate::Definitions::default();
    let fields = types.intern_list(&[nominal])?;
    cyclic.insert(
        item,
        aggregate::Kind::Struct,
        aggregate::Repr::default(),
        &[aggregate::Variant {
            fields,
            discriminant: 0,
        }],
    )?;
    let mut query = NeedsDrop::new([]);
    for _ in 0..2 {
        assert_eq!(
            query.demand(&types, &cyclic, nominal),
            Err("recursive by-value destruction requirement")
        );
    }
    Ok(())
}

#[test]
fn body_candidates_preserve_conditional_stores_and_storage_ranges()
-> Result<(), Box<dyn std::error::Error>> {
    let source = "struct D;impl Drop for D{fn drop(&mut self){}}fn f(flag:bool){let mut d:D;if flag{d=D;}d=D;let _hold:&D=&D;let _zero:[D;0]=[];let _pair=(D,1u8);}";
    rustc::qualify("drop-body-sites", source, true)?;
    let (report, facts) = semantic::analyze(source)?;
    let program = body::build(&report, &facts)?;
    let pending = program.drops.as_ref().ok_or("missing drop requirements")?;
    let (index, cfg) = program.bodies.iter().enumerate().find(|(_, body)| matches!(body.owner, Owner::Function { definition, .. } if report.package.names.get(report.resolution.definitions[definition.0 as usize].name) == "f")).ok_or("missing f")?;
    let requirements = &pending.bodies[index];
    for (name, expected) in [
        ("d", true),
        ("_hold", false),
        ("_zero", false),
        ("_pair", true),
    ] {
        let local = cfg
            .locals
            .iter()
            .position(|l| {
                l.definition.is_some_and(|id| {
                    report
                        .package
                        .names
                        .get(report.resolution.definitions[id.0 as usize].name)
                        == name
                })
            })
            .ok_or("missing local")?;
        assert_eq!(requirements.locals[local], expected, "{name}");
    }
    assert!(
        requirements
            .sites
            .iter()
            .filter(|s| matches!(s.trigger, Trigger::Store(_)))
            .count()
            >= 3
    );
    assert!(
        requirements
            .sites
            .iter()
            .any(|s| matches!(s.trigger, Trigger::Locals { .. }))
    );
    assert!(
        requirements
            .sites
            .iter()
            .any(|s| matches!(s.trigger, Trigger::Exit(_)))
    );
    let contracts = body::regions::collect(&facts)?;
    let error = rustgm::borrow::regions::check_bodies(
        &facts.types,
        &facts.aggregates,
        &facts.lifetimes.variances,
        &contracts,
        &program,
    )
    .err()
    .ok_or("pending destructor received a region proof")?;
    assert_eq!(error.message, body::drops::PENDING_MESSAGE);
    Ok(())
}

#[test]
fn promotion_does_not_erase_destructors_but_keeps_empty_arrays_promotable()
-> Result<(), Box<dyn std::error::Error>> {
    let source = "struct D;impl Drop for D{fn drop(&mut self){}}fn f(){let _drop:&D=&D;let _empty:&[D;0]=&[];let _plain:&u8=&42u8;}";
    rustc::qualify("drop-promotion", source, true)?;
    let (report, mut facts) = semantic::analyze(source)?;
    let mut program = body::build(&report, &facts)?;
    body::promote::analyze(&report, &mut facts, &mut program)?;
    let pending = program
        .drops
        .as_ref()
        .ok_or("missing pending destruction")?;
    let mut checked = 0;
    for (index, cfg) in program.bodies.iter().enumerate() {
        for temporary in &cfg.temporaries {
            let ty = cfg.values[temporary.value.0 as usize].ty;
            let expected = match facts.types.kind(ty) {
                Some(Kind::Adt(..)) => false,
                Some(Kind::Array { length: 0, .. } | Kind::Primitive(_)) => true,
                _ => continue,
            };
            let promotion = cfg.promotions.as_ref().and_then(|p| p.get(temporary.value));
            assert_eq!(promotion.is_some(), expected);
            assert_eq!(
                pending.bodies[index].values[temporary.value.0 as usize],
                !expected
            );
            checked += 1;
        }
    }
    assert_eq!(checked, 3);
    let empty = rustgm::borrow::regions::Analysis { bodies: Vec::new() };
    let error = rustgm::borrow::verify(
        &mut rustgm::borrow::Environment {
            types: &facts.types,
            definitions: &facts.aggregates,
            derives: &facts.derives,
            copies: &mut facts.copies,
        },
        &empty,
        &program,
    )
    .err()
    .ok_or("pending destruction received an executable token")?;
    assert_eq!(error.message, body::drops::PENDING_MESSAGE);
    Ok(())
}
