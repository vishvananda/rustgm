//! Batched questions return independent answers and cannot turn proof debt into assumptions.
use super::Constraints;
use crate::types::{
    ItemId, Kind, RegionId, Store,
    arguments::Argument,
    callable::Templates,
    outlives::{Bounds, Queries},
    parameters::{Parameter, ParameterKind},
    projection::{Application, Schema},
    region_graph::{Error, Graph, Kind as RegionKind},
    traits,
    variance::Variances,
};
type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn independent_outlives_answers_preserve_opaque_inputs_and_failed_queries() -> TestResult {
    let mut store = Store::default();
    let id = store.intern_parameter(Parameter {
        owner: ItemId(1),
        index: 0,
        kind: ParameterKind::Type { sized: true },
    })?;
    let subject = store.intern(Kind::Parameter(id))?;
    store.declare_projection(
        ItemId(3),
        Schema {
            trait_item: ItemId(2),
            sized: true,
            copies: crate::types::copy::Properties::default(),
        },
    )?;
    let a = RegionId::bound(0, 0)?;
    let b = RegionId::bound(0, 1)?;
    let mut projections = Vec::new();
    for region in [a, b] {
        let arguments = store.intern_arguments(&[Argument::Lifetime(region)])?;
        let trait_application = store.intern_trait_application(traits::Application {
            item: ItemId(2),
            arguments,
        })?;
        projections.push(store.intern(Kind::Projection(Application {
            item: ItemId(3),
            subject,
            trait_application,
            arguments: crate::types::GenericArgs::EMPTY,
        }))?);
    }
    let bounds = Bounds::default();
    let callables = Templates::default();
    let variances = Variances::default();
    let mut queries = Queries::new(&store, &bounds, &callables);
    let mut proof = Constraints::new(&store, &variances, 2, 10_000)?;
    proof.parameter(id)?;
    proof.assume_type_outlives(&mut queries, projections[0], a)?;
    let mut tests = Vec::new();
    for ty in [subject, projections[0], projections[1], subject] {
        tests.push(proof.test_type(&mut queries, ty, a)?);
    }
    tests.push(proof.test_region(a, b)?);
    let answers = proof.test_answers(&tests)?;
    assert_eq!(answers.values, [false, true, false, false, false]);
    Ok(())
}

#[test]
fn batched_assumption_reachability_reuses_one_walk_per_longer_region() -> TestResult {
    for width in [16, 64, 256, 1024] {
        let mut graph = Graph::new(100_000);
        let mut regions = Vec::new();
        for _ in 0..width {
            regions.push(graph.region(RegionKind::Universal { universe: 0 })?);
        }
        for pair in regions.windows(2) {
            graph.assume(pair[0], pair[1])?;
        }
        let mut roots = Vec::new();
        for &shorter in &regions {
            roots.push(graph.verify_outlives(regions[0], shorter)?);
        }
        let unknown = graph.region(RegionKind::Universal { universe: 0 })?;
        roots.push(graph.verify_outlives(regions[0], unknown)?);
        let answers = graph.test_verifications(&roots)?;
        assert!(answers.values[..width].iter().all(|v| *v));
        assert!(!answers.values[width]);
        println!("outlives answers width={width} steps={}", answers.steps);
        assert!(answers.steps <= 12 * width + 32);
        graph.require(regions[0], unknown, 0)?;
        assert!(matches!(
            graph.test_verifications(&roots),
            Err(Error::Identity)
        ));
    }
    Ok(())
}
