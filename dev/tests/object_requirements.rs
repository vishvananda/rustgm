//! Object proof construction shares graph folds and preserves binder ownership.
use rustgm::types::{
    GenericArgs, ItemId, Kind, RegionId, Store, TypeId, arguments::Argument, dynamic::Bounds,
    infer::Context, parameters::predicates::Bound,
};

#[test]
fn empty_object_binder_is_opened_once_across_shared_arguments()
-> Result<(), Box<dyn std::error::Error>> {
    for size in [8_u32, 32, 128] {
        let mut types = Store::default();
        let mut input = types.intern(Kind::Reference {
            mutable: false,
            region: RegionId::bound(1, 0)?,
            referent: TypeId::UNIT,
        })?;
        for _ in 0..size {
            let fields = types.intern_list(&[input, input])?;
            input = types.intern(Kind::Tuple(fields))?;
        }
        let shape = types.intern_dynamic_shape(Bounds {
            principal: Some(ItemId(10)),
            principal_arguments: size,
            bindings: &[],
            auto_traits: &[],
        })?;
        let arguments = types.intern_arguments(&vec![Argument::Type(input); size as usize])?;
        let object = types.intern(Kind::Dynamic {
            shape,
            arguments,
            bound_regions: 0,
            region: RegionId::STATIC,
        })?;
        let mut context = Context::default();
        let target = context.known(&types, object)?;
        let concrete = context.known(&types, TypeId::UNIT)?;
        let start = context.steps;
        let goals = context.object_requirements(&mut types, concrete, target)?;
        let steps = context.steps - start;
        assert_eq!(goals.len(), 3);
        assert!(goals.iter().all(|goal| goal.bound_regions.is_none()));
        assert!(
            steps <= 100 * size as usize + 100,
            "size={size}, steps={steps}"
        );
        let goal = context.resolved_predicate(&mut types, &goals[2].predicate)?;
        let Bound::Trait(application) = goal.bound else {
            return Err("missing principal goal".into());
        };
        let application = types
            .trait_application(application)
            .ok_or("missing trait inputs")?;
        let inputs = types
            .arguments(application.arguments)
            .ok_or("invalid trait arguments")?
            .collect::<Vec<_>>();
        assert_eq!(inputs.len(), size as usize);
        assert!(inputs.windows(2).all(|pair| pair[0] == pair[1]));
    }
    Ok(())
}

#[test]
fn object_quantifier_does_not_capture_source_or_object_lifetime()
-> Result<(), Box<dyn std::error::Error>> {
    let mut types = Store::default();
    let region = RegionId::bound(0, 0)?;
    let concrete = types.intern(Kind::Reference {
        mutable: false,
        region,
        referent: TypeId::UNIT,
    })?;
    let shape = types.intern_dynamic_shape(Bounds {
        principal: Some(ItemId(10)),
        principal_arguments: 1,
        bindings: &[],
        auto_traits: &[],
    })?;
    let arguments = types.intern_arguments(&[Argument::Lifetime(region)])?;
    let object = types.intern(Kind::Dynamic {
        shape,
        arguments,
        bound_regions: 1,
        region,
    })?;
    let mut context = Context::default();
    let target = context.known(&types, object)?;
    let source = context.known(&types, concrete)?;
    let goals = context.object_requirements(&mut types, source, target)?;
    assert_eq!(goals.len(), 3);
    assert_eq!(goals[1].bound_regions, None);
    let lifetime = context.resolved_predicate(&mut types, &goals[1].predicate)?;
    assert_eq!(lifetime.ty, concrete);
    assert_eq!(lifetime.bound, Bound::Outlives(region));
    assert_eq!(goals[2].bound_regions, Some(1));
    let principal = context.resolved_predicate(&mut types, &goals[2].predicate)?;
    assert!(
        matches!(types.kind(principal.ty), Some(Kind::Reference { region, .. }) if region.bound_parts() == Some((1, 0)))
    );
    let Bound::Trait(application) = principal.bound else {
        return Err("missing object principal".into());
    };
    let args = types
        .trait_application(application)
        .ok_or("invalid principal")?
        .arguments;
    assert_ne!(args, GenericArgs::EMPTY);
    assert_eq!(
        types.arguments(args).and_then(|mut args| args.next()),
        Some(Argument::Lifetime(region))
    );
    Ok(())
}

#[test]
fn projected_inputs_normalize_inside_the_object_binder() -> Result<(), Box<dyn std::error::Error>> {
    use rustgm::types::{
        projection,
        substitute::{Normalizer, Resolution, Step},
        traits,
    };
    for count in [0, 1] {
        let mut types = Store::default();
        types.declare_projection(
            ItemId(21),
            projection::Schema {
                trait_item: ItemId(20),
                sized: true,
                copies: rustgm::types::copy::Properties::default(),
            },
        )?;
        let region = RegionId::bound(u16::from(count == 0), 0)?;
        let inputs = types.intern_arguments(&[Argument::Lifetime(region)])?;
        let application = types.intern_trait_application(traits::Application {
            item: ItemId(20),
            arguments: inputs,
        })?;
        let projection = types.intern(Kind::Projection(projection::Application {
            item: ItemId(21),
            subject: TypeId::UNIT,
            trait_application: application,
            arguments: GenericArgs::EMPTY,
        }))?;
        let shape = types.intern_dynamic_shape(Bounds {
            principal: Some(ItemId(10)),
            principal_arguments: 1,
            bindings: &[],
            auto_traits: &[],
        })?;
        let arguments = types.intern_arguments(&[Argument::Type(projection)])?;
        let object = types.intern(Kind::Dynamic {
            shape,
            arguments,
            bound_regions: count,
            region: RegionId(7),
        })?;
        let mut fold = Normalizer::default();
        let Step::Resolve(request) = fold.start(&mut types, object)? else {
            return Err("object projection did not suspend normalization".into());
        };
        assert_eq!(request.depth, 1);
        assert_eq!(
            fold.binder(request.scope),
            Some((rustgm::types::lifetime_proof::ScopeId::ROOT, count))
        );
        let result = types.intern(Kind::Reference {
            mutable: false,
            region,
            referent: TypeId::UNIT,
        })?;
        let Step::Complete(complete) =
            fold.resume(&mut types, request, Resolution::Type(result))?
        else {
            return Err("object normalization did not complete".into());
        };
        let Some(Kind::Dynamic {
            arguments,
            bound_regions,
            region,
            ..
        }) = types.kind(complete)
        else {
            return Err("normalization lost the object".into());
        };
        assert_eq!(bound_regions, count);
        assert_eq!(region, RegionId(7));
        assert_eq!(
            types.arguments(arguments).and_then(|mut args| args.next()),
            Some(Argument::Type(result))
        );
    }
    Ok(())
}
