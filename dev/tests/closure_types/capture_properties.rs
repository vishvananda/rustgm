use super::{closure, signature};
use rustgm::support::hash::HashMap;
use rustgm::{
    abi::layout::Layouts,
    types::{
        GenericArgs, ItemId, Kind, Primitive, RegionId, Store, TypeId,
        aggregate::{self, Definitions, Repr, Variant},
        auto_traits::{AutoTrait, Obligation, Query, Rule, Step},
        callable::Templates,
        closure::CallKind,
        copy::Copies,
        destruction::{NeedsDrop, regions::Regions},
        equality::Equality,
        outlives::{Bounds, LiveRegions, Queries},
    },
};

#[test]
fn storage_properties_ignore_signature_and_unused_parent_arguments()
-> Result<(), Box<dyn std::error::Error>> {
    let mut store = Store::default();
    let mut definitions = Definitions::default();
    let pointer = store.intern(Kind::Pointer {
        mutable: false,
        pointee: TypeId::UNIT,
    })?;
    let mutable = store.intern(Kind::Reference {
        mutable: true,
        region: RegionId(4),
        referent: TypeId::UNIT,
    })?;
    let shared = store.intern(Kind::Reference {
        mutable: false,
        region: RegionId(5),
        referent: TypeId::primitive(Primitive::U64),
    })?;
    let signature = signature(&mut store, mutable)?;
    let args = store.intern_arguments(&[rustgm::types::arguments::Argument::Type(pointer)])?;
    let id = store.declare_closure(CallKind::Shared)?;
    let mut copies = Copies::default();
    let mut equality = Equality::default();
    let mut layouts = Layouts::default();
    let mut live = LiveRegions::default();
    for (fields, copy, send, size, regions) in [
        (vec![], true, true, 0, vec![]),
        (vec![shared], true, true, 8, vec![RegionId(5)]),
        (vec![mutable], false, true, 8, vec![RegionId(4)]),
        (vec![pointer], true, false, 8, vec![]),
    ] {
        let ty = closure(&mut store, id, signature, args, &fields)?;
        assert_eq!(
            copies
                .demand(&store, &definitions, &HashMap::default(), ty)?
                .copy,
            copy
        );
        assert!(
            !equality
                .demand(&store, &definitions, &HashMap::default(), ty)?
                .partial_eq
        );
        let layout = layouts.demand(&store, &definitions, ty)?;
        assert_eq!(layouts.at(layout).ok_or("layout")?.size, size);
        assert_eq!(live.get(&store, ty)?, regions);
        let bounds = Bounds::default();
        let callables = Templates::default();
        let mut outlives = Queries::new(&store, &bounds, &callables);
        assert_eq!(outlives.components(ty)?.scalars.regions, regions);
        let mut query = Query::new(Obligation {
            ty,
            marker: AutoTrait::Send,
        });
        assert_eq!(
            query.advance(&mut store, &mut definitions, |_, _| Ok(Rule::Structural))?,
            Step::Ready(send)
        );
    }
    Ok(())
}

#[test]
fn owned_destructors_and_drop_regions_follow_capture_fields()
-> Result<(), Box<dyn std::error::Error>> {
    let mut store = Store::default();
    let mut definitions = Definitions::default();
    let item = ItemId(1);
    let field = store.intern(Kind::Reference {
        mutable: false,
        region: RegionId::bound(0, 0)?,
        referent: TypeId::UNIT,
    })?;
    let fields = store.intern_list(&[field])?;
    definitions.insert(
        item,
        aggregate::Kind::Struct,
        Repr::default(),
        &[Variant {
            fields,
            discriminant: 0,
        }],
    )?;
    let arguments = store.intern_regions(&[RegionId(11)])?;
    let drop_type = store.intern(Kind::Adt(item, arguments))?;
    let signature = signature(&mut store, drop_type)?;
    let id = store.declare_closure(CallKind::Once)?;
    let empty = closure(&mut store, id, signature, arguments, &[])?;
    let owned = closure(&mut store, id, signature, arguments, &[drop_type])?;
    let mut needs = NeedsDrop::new([item]);
    assert!(!needs.demand(&store, &definitions, empty)?);
    assert!(needs.demand(&store, &definitions, owned)?);
    let mut regions = Regions::new([item])?;
    assert!(regions.get(&store, &definitions, empty)?.is_empty());
    assert_eq!(regions.get(&store, &definitions, owned)?, [RegionId(11)]);
    Ok(())
}

#[test]
fn shared_nested_environments_are_queried_once_and_drop_iteratively()
-> Result<(), Box<dyn std::error::Error>> {
    for depth in [32, 128, 512, 8192] {
        let mut store = Store::default();
        let signature = signature(&mut store, TypeId::UNIT)?;
        let mut ty = TypeId::primitive(Primitive::U8);
        for _ in 0..depth {
            let id = store.declare_closure(CallKind::Shared)?;
            ty = closure(&mut store, id, signature, GenericArgs::EMPTY, &[ty, ty])?;
        }
        let definitions = Definitions::default();
        let selected = HashMap::default();
        let mut copies = Copies::default();
        assert!(copies.demand(&store, &definitions, &selected, ty)?.copy);
        assert!(copies.steps <= depth * 3 + 2, "{}", copies.steps);
        let before = copies.steps;
        copies.demand(&store, &definitions, &selected, ty)?;
        assert_eq!(copies.steps, before);
        let mut needs = NeedsDrop::new([]);
        assert!(!needs.demand(&store, &definitions, ty)?);
        assert!(needs.steps < depth * 12 + 10, "{}", needs.steps);
    }
    Ok(())
}

#[test]
fn an_unused_rigid_parameter_does_not_require_storage_proofs()
-> Result<(), Box<dyn std::error::Error>> {
    use rustgm::types::{
        arguments::Argument,
        parameters::{Parameter, ParameterKind},
    };
    let mut store = Store::default();
    let parameter = store.intern_parameter(Parameter {
        owner: ItemId(1),
        index: 0,
        kind: ParameterKind::Type { sized: false },
    })?;
    let parameter = store.intern(Kind::Parameter(parameter))?;
    let pointer = store.intern(Kind::Pointer {
        mutable: false,
        pointee: parameter,
    })?;
    let signature = signature(&mut store, pointer)?;
    let arguments = store.intern_arguments(&[Argument::Type(parameter)])?;
    let id = store.declare_closure(CallKind::Shared)?;
    let ty = closure(&mut store, id, signature, arguments, &[])?;
    assert_eq!(store.contains_parameters(ty), Some(true));
    let mut definitions = Definitions::default();
    assert!(
        Copies::default()
            .demand(&store, &definitions, &HashMap::default(), ty)?
            .copy
    );
    let mut layouts = Layouts::default();
    let layout = layouts.demand(&store, &definitions, ty)?;
    assert_eq!(layouts.at(layout).ok_or("layout")?.size, 0);
    let mut query = Query::new(Obligation {
        ty,
        marker: AutoTrait::Send,
    });
    assert_eq!(
        query.advance(&mut store, &mut definitions, |_, _| Ok(Rule::Structural))?,
        Step::Ready(true)
    );
    Ok(())
}
