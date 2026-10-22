use crate::qualify as rustc;
use crate::semantic;
use rustgm::{
    body,
    types::{Kind, aggregate, destruction::regions::Regions},
};

#[test]
fn drop_regions_follow_owned_fields_and_actual_nominal_arguments()
-> Result<(), Box<dyn std::error::Error>> {
    let source = "struct D<'a>(&'a u8);impl Drop for D<'_>{fn drop(&mut self){}}struct W<'a,'b>{owned:D<'b>,ignored:&'a u8}struct X<'a,'b>(W<'b,'a>);struct C<'a,'b>(D<'b>,&'a u8);impl Drop for C<'_,'_>{fn drop(&mut self){}}enum E<'a,'b>{A(W<'a,'b>),B(D<'a>)}fn f<'a,'b>(a:D<'a>,b:D<'b>,w:W<'a,'b>,x:X<'a,'b>,c:C<'a,'b>,e:E<'a,'b>,r:&'a D<'b>,z:[D<'b>;0],array:[D<'b>;1000000000],pair:(D<'a>,W<'a,'b>),ptr:*mut D<'a>,fun:fn(D<'a>),s:D<'static>){}";
    rustc::qualify("drop-region-components", source, true)?;
    let (report, facts) = semantic::analyze(source)?;
    let function = facts
        .functions
        .iter()
        .find(|f| {
            report
                .package
                .names
                .get(report.resolution.definitions[f.definition.0 as usize].name)
                == "f"
        })
        .ok_or("f")?;
    let Some(Kind::Function { parameters, .. }) = facts.types.kind(function.signature) else {
        return Err("signature".into());
    };
    let parameters = facts.types.list(parameters).ok_or("parameters")?;
    let mut query = Regions::new(facts.destructors.implementations().iter().map(|i| i.item))?;
    let a = query
        .get(&facts.types, &facts.aggregates, parameters[0])?
        .to_vec();
    let b = query
        .get(&facts.types, &facts.aggregates, parameters[1])?
        .to_vec();
    assert_eq!(a.len(), 1);
    assert_eq!(b.len(), 1);
    assert_ne!(a, b);
    let mut both = vec![a[0], b[0]];
    both.sort_unstable_by_key(|r| r.0);
    let expected = [
        a.clone(),
        b.clone(),
        b.clone(),
        a.clone(),
        both.clone(),
        both.clone(),
        vec![],
        vec![],
        b,
        both,
        vec![],
        vec![],
        vec![],
    ];
    for (&ty, expected) in parameters.iter().zip(expected) {
        assert_eq!(query.get(&facts.types, &facts.aggregates, ty)?, expected);
    }
    let steps = query.steps;
    for &ty in parameters {
        query.get(&facts.types, &facts.aggregates, ty)?;
    }
    assert_eq!(query.steps, steps);
    Ok(())
}

#[test]
fn drop_region_selection_matches_independent_borrow_acceptance()
-> Result<(), Box<dyn std::error::Error>> {
    let base = "struct D<'a>(&'a mut u8);impl Drop for D<'_>{fn drop(&mut self){*self.0+=1;}}struct W<'a,'b>{d:D<'a>,r:&'b mut u8}";
    for (name, custom, statement, accepted, count) in [
        ("plain-reference", false, "b=2;", true, 1),
        ("drop-field", false, "a=2;", false, 1),
        ("custom-wrapper", true, "b=2;", false, 2),
        ("moved-field", false, "{let _d=w.d;}a=2;", false, 1),
    ] {
        let implementation = if custom {
            "impl Drop for W<'_,'_>{fn drop(&mut self){}}"
        } else {
            ""
        };
        let source = format!(
            "{base}{implementation}fn f(){{let mut a=0;let mut b=0;let w=W{{d:D(&mut a),r:&mut b}};{statement}}}"
        );
        rustc::qualify(&format!("drop-region-{name}"), &source, accepted)?;
        let (report, facts) = semantic::analyze(&source)?;
        let program = body::build(&report, &facts)?;
        let local = program
            .bodies
            .iter()
            .flat_map(|b| &b.locals)
            .find(|l| {
                l.definition.is_some_and(|id| {
                    report
                        .package
                        .names
                        .get(report.resolution.definitions[id.0 as usize].name)
                        == "w"
                })
            })
            .ok_or("w")?;
        let mut query = Regions::new(facts.destructors.implementations().iter().map(|i| i.item))?;
        assert_eq!(
            query.get(&facts.types, &facts.aggregates, local.ty)?.len(),
            count
        );
    }
    Ok(())
}

#[test]
fn region_queries_are_iterative_shared_and_fail_without_repeating_work()
-> Result<(), Box<dyn std::error::Error>> {
    let (report, mut facts) = semantic::analyze(
        "struct D<'a>(&'a u8);impl Drop for D<'_>{fn drop(&mut self){}}fn f(d:D<'_>){}",
    )?;
    let program = body::build(&report, &facts)?;
    let method = &program.drops.as_ref().ok_or("method")?.methods[0];
    let mut root = method.target;
    for _ in 0..20_000 {
        let fields = facts.types.intern_list(&[root, root])?;
        root = facts.types.intern(Kind::Tuple(fields))?;
    }
    let mut query = Regions::new([method.item])?;
    assert_eq!(query.get(&facts.types, &facts.aggregates, root)?.len(), 1);
    assert!(query.steps < 400_000);
    let steps = query.steps;
    query.get(&facts.types, &facts.aggregates, root)?;
    assert_eq!(query.steps, steps);

    let fields = facts.types.intern_list(&[method.target])?;
    let mut cyclic = aggregate::Definitions::default();
    cyclic.insert(
        method.item,
        aggregate::Kind::Struct,
        aggregate::Repr::default(),
        &[aggregate::Variant {
            fields,
            discriminant: 0,
        }],
    )?;
    let mut query = Regions::new([])?;
    assert_eq!(
        query.get(&facts.types, &cyclic, method.target),
        Err("recursive owned drop region requirement")
    );
    let steps = query.steps;
    assert_eq!(
        query.get(&facts.types, &cyclic, method.target),
        Err("recursive owned drop region requirement")
    );
    assert_eq!(query.steps, steps);
    Ok(())
}
