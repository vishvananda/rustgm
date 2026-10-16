//! Opaque live values retain sufficient input lifetimes without claiming output contracts.
use rustgm::types::{
    ItemId, Kind, RegionId, Store, TypeId,
    arguments::Argument,
    outlives::{Bounds, LiveRegions, Queries},
    parameters::{Parameter, ParameterKind},
    projection::{Application, Schema},
    traits,
};

#[test]
fn projected_value_inputs_retain_regions_and_rigid_types_without_output_assumptions()
-> Result<(), Box<dyn std::error::Error>> {
    let mut store = Store::default();
    let parameter = store.intern_parameter(Parameter {
        owner: ItemId(20),
        index: 0,
        kind: ParameterKind::Type { sized: true },
    })?;
    let ty = store.intern(Kind::Parameter(parameter))?;
    let subject = store.intern(Kind::Reference {
        region: RegionId(2),
        mutable: false,
        referent: ty,
    })?;
    let argument = store.intern(Kind::Reference {
        region: RegionId(4),
        mutable: false,
        referent: ty,
    })?;
    store.declare_projection(
        ItemId(11),
        Schema {
            trait_item: ItemId(10),
            sized: true,
            copies: rustgm::types::copy::Properties::default(),
        },
    )?;
    let arguments =
        store.intern_arguments(&[Argument::Lifetime(RegionId(3)), Argument::Type(argument)])?;
    let trait_application = store.intern_trait_application(traits::Application {
        item: ItemId(10),
        arguments,
    })?;
    let arguments =
        store.intern_arguments(&[Argument::Lifetime(RegionId(5)), Argument::Type(ty)])?;
    let projected = store.intern(Kind::Projection(Application {
        item: ItemId(11),
        subject,
        trait_application,
        arguments,
    }))?;
    let mut live = LiveRegions::default();
    let parts = live.symbolic(&store, projected)?;
    let mut regions = parts.regions.to_vec();
    regions.sort_unstable_by_key(|region| region.0);
    assert_eq!(
        regions,
        [RegionId(2), RegionId(3), RegionId(4), RegionId(5)]
    );
    assert_eq!(parts.parameters, [parameter]);
    assert!(live.get(&store, projected).is_err());
    let bounds = Bounds::default();
    let callables = rustgm::types::callable::Templates::default();
    let mut queries = Queries::new(&store, &bounds, &callables);
    let bound = queries.components(projected)?;
    assert!(bound.scalars.is_empty());
    assert_eq!(
        bound.projections,
        &[rustgm::types::outlives::Projection {
            ty: projected,
            depth: 0
        }]
    );
    let empty = store.intern(Kind::Array {
        element: projected,
        length: 0,
    })?;
    assert!(live.symbolic(&store, empty)?.is_empty());
    let list = store.intern_list(&vec![projected; 1024])?;
    let tuple = store.intern(Kind::Tuple(list))?;
    assert_eq!(live.symbolic(&store, tuple)?.parameters, [parameter]);
    assert!(live.steps() < 1100, "{}", live.steps());
    let steps = live.steps();
    for _ in 0..100 {
        assert_eq!(live.symbolic(&store, tuple)?.len(), 5);
    }
    assert_eq!(live.steps(), steps);
    assert!(live.symbolic(&store, TypeId::UNIT)?.is_empty());
    Ok(())
}
