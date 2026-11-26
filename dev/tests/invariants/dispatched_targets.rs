//! Publication retains the exact normalized callable of each proved source original.
use super::*;
use crate::{lower::instances::SourcePublication, resolve::unit::FileId};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn source_originals() -> Result<(
    crate::sema::model::Declarations,
    crate::support::crates::Interface,
)> {
    let foundation =
        crate::test_source::fixture("pa31/tests/spec/source-enum-role-transport/provider.rs")?;
    let prefix = foundation
        .split_once("#[rustc_diagnostic_item = \"Option\"]")
        .ok_or("actual foundation boundary")?
        .0;
    let tail = crate::test_source::fixture(
        "pa31/tests/spec/source-selected-projection-calls/provider.rs",
    )?;
    let report = crate::test_source::source_provider(
        &format!("#![feature(auto_traits)]\n{prefix}{tail}"),
        "retained_dispatch",
    )?;
    let identity = report
        .package
        .source_core()
        .ok_or("actual provider")?
        .provider
        .clone();
    let crate::pipeline::Analysis {
        mut facts,
        prepared,
    } = crate::pipeline::analyze_source(&report)?;
    let (_, artifact) = crate::lower::instances::build_source_library_profile(
        &report,
        &mut facts,
        identity,
        prepared,
        SourcePublication::ModuleBodyOperations,
    )?;
    Ok((facts, artifact))
}

#[test]
fn original_dispatch_results_keep_owner_site_application_and_payment_boundaries() -> Result {
    let (mut facts, artifact) = source_originals()?;
    let rows = facts
        .methods
        .operands
        .as_deref()
        .and_then(|rows| rows.dispatched.as_deref())
        .ok_or("actual completed source dispatch table")?
        .iter()
        .map(|(&key, &target)| (key, target))
        .collect::<Vec<_>>();
    assert!(!rows.is_empty());
    let mut projection_difference = false;
    for ((owner, site, original), target) in rows {
        assert_eq!(
            facts.methods.dispatched(owner, site, original),
            Some(target)
        );
        let foreign = NodeKey {
            file: FileId(u32::MAX),
            ..owner
        };
        assert_eq!(facts.methods.dispatched(foreign, site, original), None);
        assert_eq!(facts.methods.dispatched(owner, foreign, original), None);
        assert_eq!(facts.methods.dispatched(owner, site, TypeId::UNIT), None);
        let raw = crate::sema::dispatch::selected_callable_with_projections(
            &mut facts,
            site,
            original,
            &|_, _| Ok(()),
        )?;
        if let Some(raw) = raw
            && raw != target
            && facts.types.contains_projections(raw) == Some(true)
        {
            projection_difference = true;
            let mut dispatcher = crate::sema::dispatch::Dispatcher::new(0, 1)?;
            assert_eq!(dispatcher.select(&mut facts, original)?, target);
            let selections = facts.dispatch_dependencies.work.selections;
            assert_eq!(dispatcher.select(&mut facts, raw)?, target);
            assert!(facts.dispatch_dependencies.work.selections <= selections);
            let proof = dispatcher.finish();
            assert!(proof.self_contained_in(&facts.types, 0)?);
            crate::sema::discharge::quantified(&facts, &proof, 0, "invalid projection query")?;
        }
        let denied = |_, _| Err("retention denied");
        facts
            .methods
            .retain_dispatched(owner, site, original, target, &denied)?;
        assert!(
            facts
                .methods
                .retain_dispatched(owner, site, original, TypeId::UNIT, &|_, _| Ok(()))
                .is_err()
        );
        assert_eq!(
            facts.methods.dispatched(owner, site, original),
            Some(target)
        );
        assert_eq!(
            facts
                .methods
                .retain_dispatched(foreign, site, original, target, &denied)
                .err(),
            Some("retention denied")
        );
        assert_eq!(facts.methods.dispatched(foreign, site, original), None);
    }
    assert!(
        projection_difference,
        "the actual source must require its normalized target"
    );
    let fresh = crate::support::crates::decode(&crate::support::crates::encode(&artifact)?)?;
    crate::source_packets::client_analysis(
        fresh,
        "#![feature(no_core)]#![no_core]extern crate base;pub fn observe()->u32{base::direct(11)}",
    )?;
    Ok(())
}

#[test]
fn query_proofs_reject_foreign_variables_and_unproved_universal_relations() -> Result {
    use crate::types::{RegionId, lifetime_proof::Obligations};
    let report = crate::test_source::resolve("fn main(){}", true)?;
    let crate::pipeline::Analysis { facts, .. } = crate::pipeline::analyze_source(&report)?;
    let a = RegionId::bound(0, 0)?;
    let b = RegionId::bound(0, 1)?;
    let mut proof = Obligations::new(7);
    proof.outlives.push((a, a));
    assert!(!proof.self_contained(&facts.types)?);
    assert!(proof.self_contained_in(&facts.types, 1)?);
    crate::sema::discharge::quantified(&facts, &proof, 1, "invalid universal relation")?;
    proof.outlives.push((a, b));
    assert!(!proof.self_contained_in(&facts.types, 1)?);
    assert!(proof.self_contained_in(&facts.types, 2)?);
    assert!(
        crate::sema::discharge::quantified(&facts, &proof, 2, "invalid universal relation")
            .is_err()
    );
    proof.outlives.clear();
    proof.outlives.push((RegionId(6), a));
    assert!(!proof.self_contained_in(&facts.types, 1)?);
    Ok(())
}
