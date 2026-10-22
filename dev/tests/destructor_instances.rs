//! Concrete destructor selection uses the retained impl-to-nominal argument permutation.
use crate::qualify as rustc;
use crate::semantic;
use rustgm::{
    body, sema,
    types::{Kind, arguments::Argument, instance::Instance},
};

struct Selected {
    report: rustgm::resolve::expansion::Report,
    facts: sema::model::Declarations,
    items: Vec<Instance>,
}

fn selected(source: &str, name: &str) -> Result<Selected, Box<dyn std::error::Error>> {
    rustc::qualify(name, source, true)?;
    let (report, mut facts) = semantic::analyze(source)?;
    let targets = facts.initializers.iter().map(|i| i.ty).collect::<Vec<_>>();
    let mut selected = Vec::new();
    for ty in targets {
        let instance = sema::destructors::select(&mut facts, ty)?.ok_or("missing selected Drop")?;
        assert_eq!(sema::destructors::select(&mut facts, ty)?, Some(instance));
        selected.push(instance);
    }
    Ok(Selected {
        report,
        facts,
        items: selected,
    })
}
#[test]
fn destructor_items_preserve_permuted_const_kinds_and_type_arguments()
-> Result<(), Box<dyn std::error::Error>> {
    let source = "struct S<T,U,const B:bool,const C:char,const N:i8>(T,U);impl<A,Z,const K:i8,const H:char,const V:bool> Drop for S<Z,A,V,H,K>{fn drop(&mut self){}}const X:S<u8,u16,true,'A',{-7}>=S(1,2);const Y:S<u16,u8,false,'B',3>=S(1,2);";
    let Selected { facts, items, .. } = selected(source, "permuted-drop-items")?;
    assert_eq!(items.len(), 2);
    assert_ne!(items[0], items[1]);
    for (index, instance) in items.iter().enumerate() {
        let Some(Kind::FunctionItem { arguments, .. }) = facts.types.kind(instance.ty()) else {
            return Err("missing item".into());
        };
        let arguments = facts
            .types
            .arguments(arguments)
            .ok_or("missing args")?
            .collect::<Vec<_>>();
        assert_eq!(arguments.len(), 5);
        let Some(Kind::Adt(_, nominal)) = facts.types.kind(facts.initializers[index].ty) else {
            return Err("missing nominal".into());
        };
        let nominal = facts
            .types
            .arguments(nominal)
            .ok_or("missing nominal args")?
            .collect::<Vec<_>>();
        assert_eq!(
            arguments,
            vec![nominal[1], nominal[0], nominal[4], nominal[3], nominal[2]]
        );
        assert!(matches!(arguments[2], Argument::Const(_)));
    }
    Ok(())
}
#[test]
fn selecting_a_method_keeps_unused_lifetime_proof_slots() -> Result<(), Box<dyn std::error::Error>>
{
    let source = "struct S<'a,'b,T>(&'a u8,&'b u8,T)where 'a:'b;impl<'x,'y,'u,T> Drop for S<'y,'x,T>where 'y:'u,'u:'x{fn drop(&mut self){}}const X:S<'static,'static,u8>=S(&1,&2,3);";
    let Selected {
        report,
        mut facts,
        items,
    } = selected(source, "unused-drop-lifetime")?;
    let templates = body::templates::build(&report, &facts)?;
    let original = templates.bodies.first().ok_or("missing method template")?;
    let inputs = body::regions::collect(&facts)?;
    let input = inputs
        .bodies
        .iter()
        .find(|c| c.source == original.body.source)
        .ok_or("missing input")?
        .clone();
    assert_eq!(input.universals, 4);
    let assumptions = input.assumptions.clone();
    let value = body::instantiate::with_contract(&mut facts, original, items[0], &input)?;
    assert_eq!(value.contract.universals, 4);
    assert_eq!(value.contract.assumptions, assumptions);
    assert!(value.contract.parameters.is_empty());
    Ok(())
}

fn first_parameter(
    report: &rustgm::resolve::expansion::Report,
    facts: &sema::model::Declarations,
    name: &str,
) -> Result<rustgm::types::TypeId, &'static str> {
    let function = facts
        .functions
        .iter()
        .find(|f| {
            report
                .package
                .names
                .get(report.resolution.definitions[f.definition.0 as usize].name)
                == name
        })
        .ok_or("missing function")?;
    let Some(Kind::Function { parameters, .. }) = facts.types.kind(function.signature) else {
        return Err("missing function signature");
    };
    facts
        .types
        .list(parameters)
        .and_then(|p| p.first())
        .copied()
        .ok_or("target")
}

const PERMUTED_LIFETIMES: &str = "struct S<'a,'b,T>(&'a u8,&'b u8,T)where 'a:'b;impl<'x,'y,'u,T> Drop for S<'y,'x,T>where 'y:'u,'u:'x{fn drop(&mut self){}}fn consume<'a,'b>(s:S<'a,'b,&'b u8>)where 'a:'b{}";

#[test]
fn selected_receiver_keeps_distinct_target_lifetimes_through_permuted_capture_slots()
-> Result<(), Box<dyn std::error::Error>> {
    use rustgm::types::{Primitive, RegionId, TypeId};
    let source = PERMUTED_LIFETIMES;
    rustc::qualify("distinct-drop-lifetimes", source, true)?;
    let (report, mut facts) = semantic::analyze(source)?;
    let target = first_parameter(&report, &facts, "consume")?;
    let Some(Kind::Adt(_, original)) = facts.types.kind(target) else {
        return Err("target nominal type".into());
    };
    let original = facts
        .types
        .arguments(original)
        .ok_or("target arguments")?
        .collect::<Vec<_>>();
    let [Argument::Lifetime(a), Argument::Lifetime(b), _] = original.as_slice() else {
        return Err("target lifetime arguments".into());
    };
    let (a, b) = (*a, *b);
    assert_ne!(a, b);
    let Some((0, a_index)) = a.bound_parts() else {
        return Err("first binder".into());
    };
    let Some((0, b_index)) = b.bound_parts() else {
        return Err("second binder".into());
    };
    let instance = sema::destructors::select(&mut facts, target)?.ok_or("selected destructor")?;
    let Some(Kind::FunctionItem {
        signature,
        arguments,
        ..
    }) = facts.types.kind(instance.ty())
    else {
        return Err("selected callable".into());
    };
    let arguments = facts
        .types
        .arguments(arguments)
        .ok_or("arguments")?
        .collect::<Vec<_>>();
    assert_eq!(arguments[0], Argument::Lifetime(a));
    assert_eq!(arguments[1], Argument::Lifetime(b));
    let Some(Kind::Function {
        parameters,
        bound_regions: 1,
        ..
    }) = facts.types.kind(signature)
    else {
        return Err("late receiver binder".into());
    };
    let receiver = facts.types.list(parameters).ok_or("receiver")?[0];
    let Some(Kind::Reference {
        referent,
        region,
        mutable: true,
    }) = facts.types.kind(receiver)
    else {
        return Err("receiver reference".into());
    };
    assert_eq!(region, RegionId::bound(0, 0)?);
    let Some(Kind::Adt(_, arguments)) = facts.types.kind(referent) else {
        return Err("receiver nominal type".into());
    };
    let arguments = facts
        .types
        .arguments(arguments)
        .ok_or("receiver arguments")?
        .collect::<Vec<_>>();
    assert_eq!(
        arguments[0],
        Argument::Lifetime(RegionId::bound(1, a_index)?)
    );
    assert_eq!(
        arguments[1],
        Argument::Lifetime(RegionId::bound(1, b_index)?)
    );
    let Argument::Type(nested) = arguments[2] else {
        return Err("nested reference argument".into());
    };
    assert_eq!(
        facts.types.kind(nested),
        Some(Kind::Reference {
            mutable: false,
            region: RegionId::bound(1, b_index)?,
            referent: TypeId::primitive(Primitive::U8),
        })
    );
    Ok(())
}

#[test]
fn imported_drop_arguments_reconnect_only_their_original_body_slots()
-> Result<(), Box<dyn std::error::Error>> {
    use rustgm::types::RegionId;
    rustc::qualify("permuted-drop-lifetime-import", PERMUTED_LIFETIMES, true)?;
    let (report, mut facts) = semantic::analyze(PERMUTED_LIFETIMES)?;
    let target = first_parameter(&report, &facts, "consume")?;
    let instance = sema::destructors::select(&mut facts, target)?.ok_or("selected destructor")?;
    let templates = body::templates::build(&report, &facts)?;
    let template = templates.bodies.first().ok_or("missing Drop template")?;
    let contracts = body::regions::collect(&facts)?;
    let input = contracts
        .bodies
        .iter()
        .find(|c| c.source == template.body.source)
        .ok_or("missing Drop input")?
        .clone();
    assert_eq!(input.universals, 4);
    let materialized = body::instantiate::with_contract(&mut facts, template, instance, &input)?;
    // The type argument imports only 'b, which is the original impl's 'x slot.
    // Its permuted 'y and unused 'u retain the original proof identities.
    assert_eq!(materialized.contract.universals, 5);
    assert_eq!(
        &materialized.contract.assumptions[input.assumptions.len()..],
        &[
            (RegionId::bound(0, 0)?, RegionId::bound(0, 4)?),
            (RegionId::bound(0, 4)?, RegionId::bound(0, 0)?)
        ],
    );
    Ok(())
}

#[test]
fn generic_recipes_use_concrete_fields_and_keep_their_cache_between_selections()
-> Result<(), Box<dyn std::error::Error>> {
    use body::drops::glue::{Catalog, Method, Shape};
    let source = "struct D<T>(T);impl<T> Drop for D<T>{fn drop(&mut self){}}struct W<T,const N:usize>([T;N]);const A:W<u8,2>=W([1,2]);const B:W<D<u8>,2>=W([D(3),D(4)]);const C:W<D<u16>,1>=W([D(5)]);const Z:W<D<u8>,0>=W([]);";
    rustc::qualify("generic-drop-recipes", source, true)?;
    let (report, mut facts) = semantic::analyze(source)?;
    let program = body::build_concrete(&report, &facts)?;
    let methods = &program.drops.as_ref().ok_or("missing methods")?.methods;
    let mut catalog = Catalog::new(&facts.types, &facts.aggregates, &facts.members, methods)?;
    let targets = facts.initializers.iter().map(|i| i.ty).collect::<Vec<_>>();
    let recipes = targets
        .iter()
        .map(|&ty| catalog.demand(ty))
        .collect::<Result<Vec<_>, _>>()?;
    assert_eq!(recipes.len(), 4);
    assert_eq!(recipes[0], None);
    assert_eq!(recipes[3], None);
    assert_ne!(recipes[1], recipes[2]);
    let mut selected = Vec::new();
    for (index, length) in [(1, 2), (2, 1)] {
        let recipe = catalog
            .get(recipes[index].ok_or("missing wrapper recipe")?)
            .ok_or("wrapper")?;
        assert_eq!(recipe.ty, targets[index]);
        assert!(recipe.method.is_none());
        let Shape::Fields(fields) = recipe.kind else {
            return Err("wrapper fields".into());
        };
        let [field] = catalog.fields(fields).ok_or("wrapper field")? else {
            return Err("wrapper arity".into());
        };
        let array = catalog.get(field.recipe).ok_or("array recipe")?;
        let Shape::Array {
            element,
            length: actual,
        } = array.kind
        else {
            return Err("array shape".into());
        };
        assert_eq!(actual, length);
        let element = catalog.get(element).ok_or("element recipe")?;
        assert!(matches!(element.method, Some(Method::Unresolved(_))));
        selected.push(element.ty);
    }
    assert_ne!(selected[0], selected[1]);
    let steps = catalog.steps();
    let count = catalog.recipes().len();
    let cache = catalog.into_cache();
    let first = sema::destructors::select(&mut facts, selected[0])?.ok_or("first method")?;
    let second = sema::destructors::select(&mut facts, selected[1])?.ok_or("second method")?;
    assert_ne!(first, second);
    let mut catalog = cache.attach(&facts.types, &facts.aggregates, &facts.members);
    for (target, recipe) in targets.into_iter().zip(recipes) {
        assert_eq!(catalog.demand(target)?, recipe);
    }
    assert_eq!(catalog.steps(), steps);
    assert_eq!(catalog.recipes().len(), count);
    Ok(())
}
