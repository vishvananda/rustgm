//! Caller namespaces restore the same folder after failure without leaking foreign ownership.
use super::*;
use crate::types::{RegionId, instance::Instance};
#[test]
fn caller_folder_restores_after_errors_and_rejects_foreign_contexts()
-> Result<(), Box<dyn std::error::Error>> {
    let report = crate::test_source::resolve(
        "struct S<'a>(&'a u8);impl<'a>S<'a>{const C:u8={7};}fn f<'a>(_:&'a u8)->u8{S::<'a>::C}fn g()->u8{0}fn main(){let n=7;let _=f(&n);let _=g();}",
        true,
    )?;
    let mut facts = crate::pipeline::analyze_source(&report)?.facts;
    let f = facts
        .functions
        .iter()
        .find(|f| f.body.is_some_and(|s| published(&facts, s)))
        .ok_or("missing caller")?;
    let source = f.body.ok_or("missing caller source")?;
    let ty = facts.definitions[f.definition.0 as usize].ok_or("missing caller type")?;
    let instance = CallerInstance::Callable(Instance::new(&facts.types, ty)?);
    let g = facts
        .functions
        .iter()
        .find(|f| f.body.is_some_and(|s| s != source))
        .ok_or("missing foreign caller")?;
    let foreign = CallerInstance::Callable(Instance::new(
        &facts.types,
        facts.definitions[g.definition.0 as usize].ok_or("missing foreign type")?,
    )?);
    let (_, publication) = facts
        .source_computations
        .as_deref()
        .ok_or("missing catalog")?
        .0
        .original(source)?;
    publication.validate(&facts, source)?;
    let slot = RegionId::bound(0, 0)?;
    assert!(
        with_namespace(&mut facts, source, instance, 1, &[], usize::MAX, |_, _| Ok(
            ()
        ))
        .is_err()
    );
    assert!(
        facts
            .source_computations
            .as_ref()
            .is_some_and(|c| c.0.namespaces.is_none())
    );
    with_namespace(&mut facts, source, instance, 1, &[], 0, |_, t| {
        assert_eq!(t.fold.region(slot, 0)?, slot);
        assert!(t.fold.region(RegionId(1), 0).is_err());
        Ok(())
    })?;
    let bytes = facts.constant_work.source_context_bytes;
    let error = with_namespace(
        &mut facts,
        source,
        instance,
        1,
        &[],
        0,
        |facts, t| -> Result<(), &'static str> {
            assert!(with_registered_namespace(facts, source, instance, |_, _| Ok(())).is_err());
            assert_eq!(t.fold.region(slot, 0)?, slot);
            Err("deliberate caller operation failure")
        },
    )
    .err()
    .ok_or("error disappeared")?;
    assert_eq!(error, "deliberate caller operation failure");
    with_registered_namespace(&mut facts, source, instance, |_, t| {
        assert_eq!(t.fold.region(slot, 0)?, slot);
        assert!(t.fold.region(RegionId(1), 0).is_err());
        Ok(())
    })?;
    assert_eq!(facts.constant_work.source_context_bytes, bytes);
    assert!(
        with_namespace(
            &mut facts,
            source,
            instance,
            1,
            std::slice::from_ref(&(1..2)),
            0,
            |_, _| Ok(())
        )
        .is_err()
    );
    assert!(with_namespace(&mut facts, source, instance, 2, &[], 0, |_, _| Ok(())).is_err());
    assert!(with_namespace(&mut facts, source, foreign, 1, &[], 0, |_, _| Ok(())).is_err());
    assert!(
        with_registered_namespace(&mut facts, source, instance, |_, t| {
            t.fold = crate::types::substitute::Bound::new(Vec::new());
            Ok(())
        })
        .is_err()
    );
    assert!(with_registered_namespace(&mut facts, source, instance, |_, _| Ok(())).is_err());
    Ok(())
}

#[test]
fn empty_transport_reserves_the_observed_actual_floor_without_owning_it()
-> Result<(), Box<dyn std::error::Error>> {
    use crate::{
        sema::model::{BodyRegion, BodyRegionOrigin},
        types::{Kind, arguments::Argument, lifetime_proof::Obligations},
    };

    let report = crate::test_source::resolve(
        "struct S<'a>(&'a u8);impl<'a>S<'a>{const C:u8={7};}fn f<'a>(_:&'a u8)->u8{S::<'a>::C}fn g()->u8{0}fn main(){let n=7;let _=f(&n);let _=g();}",
        true,
    )?;
    let mut facts = crate::pipeline::analyze_source(&report)?.facts;
    let function = |name| {
        facts.functions.iter().find(|function| {
            let definition = &report.resolution.definitions[function.definition.0 as usize];
            report.package.names.get(definition.name) == name
        })
    };
    let owner = function("f").ok_or("missing receiving function")?;
    let source = owner.body.ok_or("missing receiving body")?;
    let donor = function("g")
        .and_then(|function| function.body)
        .ok_or("missing donor body")?;
    let ty = facts.definitions[owner.definition.0 as usize].ok_or("missing caller type")?;
    let Some(Kind::FunctionItem {
        item, signature, ..
    }) = facts.types.kind(ty)
    else {
        return Err("missing genuine function item".into());
    };
    let before = facts.body_regions.frontier(source)?;
    let donor_first = facts.body_regions.frontier(donor)?;
    assert!(donor_first <= before);
    facts.body_regions.allocate_reserved(
        BodyRegion {
            body: donor,
            source: donor,
            origin: BodyRegionOrigin::Inference,
        },
        donor_first,
        (before - donor_first + 1) as usize,
    )?;
    let actual = RegionId(before);
    let arguments = facts
        .types
        .intern_arguments(&[Argument::Lifetime(actual)])?;
    let selected = facts.types.intern(Kind::FunctionItem {
        item,
        arguments,
        signature,
    })?;
    let enclosing = CallerInstance::Callable(Instance::new(&facts.types, selected)?);
    let donor_frontier = facts.body_regions.frontier(donor)?;
    assert!(facts.body_regions.get(source, actual).is_none());
    assert!(with_namespace(&mut facts, donor, enclosing, 1, &[], 0, |_, _| Ok(())).is_err());
    assert_eq!(facts.body_regions.frontier(source)?, before);
    with_namespace(
        &mut facts,
        source,
        enclosing,
        1,
        &[],
        0,
        |facts, transport| {
            let floor = before + 1;
            assert_eq!(transport.interval_range(), floor..floor);
            assert_eq!(facts.body_regions.frontier(source)?, floor);
            let mut proof = Obligations::new(before);
            transport.proof(&mut facts.types, &mut proof)?;
            assert_eq!(proof.regions, floor..floor);
            assert_eq!(proof.owned_regions().next(), None);
            assert!(transport.fold.region(actual, 0).is_err());
            Ok(())
        },
    )?;

    let reserved = facts
        .body_regions
        .get(source, actual)
        .ok_or("missing receiver slot")?;
    assert_eq!(reserved.body, source);
    assert!(matches!(reserved.origin, BodyRegionOrigin::QuantifiedProof));
    let foreign = facts
        .body_regions
        .get(donor, actual)
        .ok_or("missing donor slot")?;
    assert_eq!(foreign.body, donor);
    assert!(matches!(foreign.origin, BodyRegionOrigin::Inference));
    assert_eq!(facts.body_regions.frontier(donor)?, donor_frontier);

    let floor = facts.body_regions.frontier(source)?;
    let mut next = Reservation::default();
    next.reserve(&mut facts, source, source, floor..floor + 2)?;
    let after = facts.body_regions.frontier(source)?;
    assert_eq!(after, floor + 2);
    with_registered_namespace(&mut facts, source, enclosing, |facts, transport| {
        assert_eq!(transport.interval_range(), floor..floor);
        assert_eq!(facts.body_regions.frontier(source)?, after);
        Ok(())
    })?;
    assert_eq!(facts.body_regions.frontier(source)?, after);
    Ok(())
}
