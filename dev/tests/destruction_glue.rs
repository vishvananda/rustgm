use rustgm::{
    body,
    borrow::destruction::glue::{Catalog, Shape},
    sema::members::Selection,
    types::{Kind, aggregate},
};
#[path = "helpers/qualify.rs"]
mod rustc;
#[path = "support/semantic.rs"]
mod semantic;

#[test]
fn shared_recipes_preserve_custom_methods_and_declared_field_order()
-> Result<(), Box<dyn std::error::Error>> {
    let source = "struct D<'a>(&'a u8);impl Drop for D<'_>{fn drop(&mut self){}}struct W<'a>(u8,D<'a>,D<'a>);impl Drop for W<'_>{fn drop(&mut self){}}enum E<'a>{Empty,Full(u8,W<'a>,D<'a>)}fn f<'a,'b>(a:D<'a>,b:D<'b>,s:D<'static>,w:W<'a>,e:E<'a>,z:[D<'a>;0],r:&D<'a>,big:[D<'a>;1000000000]){}";
    rustc::qualify("shared-drop-glue", source, true)?;
    let (report, facts) = semantic::analyze(source)?;
    let program = body::build(&report, &facts)?;
    let methods = &program.drops.as_ref().ok_or("pending methods")?.methods;
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
    let mut catalog = Catalog::new(&facts.types, &facts.aggregates, &facts.members, methods)?;
    let ids = parameters
        .iter()
        .map(|&ty| catalog.demand(ty))
        .collect::<Result<Vec<_>, _>>()?;
    assert_eq!(ids[0], ids[1]);
    assert_eq!(ids[1], ids[2]);
    assert_eq!(ids[5], None);
    assert_eq!(ids[6], None);
    let d = catalog.get(ids[0].ok_or("D")?).ok_or("D recipe")?;
    assert!(d.method.is_some());
    let Shape::Fields(fields) = d.kind else {
        return Err("D fields".into());
    };
    assert!(catalog.fields(fields).ok_or("D field list")?.is_empty());
    let w = catalog.get(ids[3].ok_or("W")?).ok_or("W recipe")?;
    assert!(w.method.is_some());
    assert_ne!(d.method, w.method);
    let Shape::Fields(fields) = w.kind else {
        return Err("W fields".into());
    };
    let fields = catalog.fields(fields).ok_or("W field list")?;
    assert_eq!(fields.len(), 2);
    for (field, ordinal) in fields.iter().zip([1, 2]) {
        assert_eq!(Some(field.recipe), ids[0]);
        let Selection::Nominal(id) = field.selection else {
            return Err("nominal field".into());
        };
        assert_eq!(facts.members.ordinal(id), Some(ordinal));
    }
    let e = catalog.get(ids[4].ok_or("E")?).ok_or("E recipe")?;
    assert!(e.method.is_none());
    let Shape::Variants(variants) = e.kind else {
        return Err("E variants".into());
    };
    let variants = catalog.variants(variants).ok_or("variant list")?;
    assert_eq!(variants.len(), 2);
    assert!(
        catalog
            .fields(variants[0].fields)
            .ok_or("empty variant")?
            .is_empty()
    );
    assert_eq!(
        catalog
            .fields(variants[1].fields)
            .ok_or("full variant")?
            .iter()
            .map(|f| Some(f.recipe))
            .collect::<Vec<_>>(),
        [ids[3], ids[0]]
    );
    let array = catalog.get(ids[7].ok_or("array")?).ok_or("array recipe")?;
    assert!(
        matches!(array.kind, Shape::Array { element, length: 1_000_000_000 } if Some(element)==ids[0])
    );
    assert_eq!(catalog.recipes().len(), 4);
    let steps = catalog.steps;
    for &ty in parameters {
        catalog.demand(ty)?;
    }
    assert_eq!(catalog.steps, steps);
    Ok(())
}

#[test]
fn deeply_shared_recipes_use_a_worklist_and_memoize_failures()
-> Result<(), Box<dyn std::error::Error>> {
    let (report, mut facts) =
        semantic::analyze("struct D;impl Drop for D{fn drop(&mut self){}}fn f(d:D){}")?;
    let program = body::build(&report, &facts)?;
    let methods = &program.drops.as_ref().ok_or("methods")?.methods;
    let mut root = methods[0].target;
    for _ in 0..20_000 {
        let fields = facts.types.intern_list(&[root, root])?;
        root = facts.types.intern(Kind::Tuple(fields))?;
    }
    let mut catalog = Catalog::new(&facts.types, &facts.aggregates, &facts.members, methods)?;
    let recipe = catalog.demand(root)?.ok_or("root recipe")?;
    assert_eq!(catalog.recipes().len(), 20_001);
    assert!(catalog.steps < 200_000);
    let steps = catalog.steps;
    assert_eq!(catalog.demand(root)?, Some(recipe));
    assert_eq!(catalog.steps, steps);

    let (report, mut facts) =
        semantic::analyze("struct D(u8);impl Drop for D{fn drop(&mut self){}}fn f(d:D){}")?;
    let program = body::build(&report, &facts)?;
    let methods = &program.drops.as_ref().ok_or("methods")?.methods;
    let method = &methods[0];
    let fields = facts.types.intern_list(&[method.target])?;
    let mut definitions = aggregate::Definitions::default();
    definitions.insert(
        method.item,
        aggregate::Kind::Struct,
        aggregate::Repr::default(),
        &[aggregate::Variant {
            fields,
            discriminant: 0,
        }],
    )?;
    facts.members.fields[0].ty = method.target;
    let mut catalog = Catalog::new(&facts.types, &definitions, &facts.members, methods)?;
    assert_eq!(
        catalog.demand(method.target),
        Err("recursive by-value destruction recipe")
    );
    let steps = catalog.steps;
    assert_eq!(
        catalog.demand(method.target),
        Err("recursive by-value destruction recipe")
    );
    assert_eq!(catalog.steps, steps);
    Ok(())
}
