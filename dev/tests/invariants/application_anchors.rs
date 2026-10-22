//! Symbolic applications use only their selected proof's retained equalities.
use super::restore;
use crate::types::{
    ItemId, Kind, Primitive, RegionId, Store, TypeId,
    arguments::Argument,
    lifetime_proof::Obligations,
    projection::{Application, Schema},
    substitute::Alias,
    traits,
};

fn application(types: &mut Store, region: RegionId, nested: bool) -> Result<Alias, &'static str> {
    types.declare_projection(
        ItemId(11),
        Schema {
            trait_item: ItemId(10),
            sized: true,
            copies: crate::types::copy::Properties::default(),
        },
    )?;
    let mut arguments = vec![Argument::Lifetime(region)];
    if nested {
        let reference = types.intern(Kind::Reference {
            region: RegionId::bound(0, 0)?,
            mutable: false,
            referent: TypeId::primitive(Primitive::U8),
        })?;
        let parameters = types.intern_list(&[reference])?;
        let callable = types.intern(Kind::Function {
            bound_regions: 1,
            parameters,
            result: reference,
            unsafe_: false,
            convention: crate::types::callable::Convention::RUST,
        })?;
        arguments.push(Argument::Type(callable));
    }
    let arguments = types.intern_arguments(&arguments)?;
    let trait_application = types.intern_trait_application(traits::Application {
        item: ItemId(10),
        arguments,
    })?;
    Ok(Alias::Associated(Application {
        item: ItemId(11),
        subject: TypeId::UNIT,
        trait_application,
        arguments: crate::types::GenericArgs::EMPTY,
    }))
}

#[test]
fn selected_aliases_restore_without_consuming_their_debts() -> Result<(), &'static str> {
    let mut types = Store::default();
    let input = application(&mut types, RegionId(3), true)?;
    let caller = application(&mut types, RegionId(1), true)?;
    let distinct = application(&mut types, RegionId(2), true)?;
    let mut proof = Obligations::new(10);
    proof.regions.end = 12;
    proof.equalities.extend([
        (RegionId(10), RegionId(3)),
        (RegionId(10), RegionId(1)),
        (RegionId(11), RegionId(2)),
    ]);
    let debts = proof.equalities.clone();
    let mut work = 0;
    let result = restore(&mut types, &proof, input, &mut work)?.ok_or("missing restored input")?;
    assert_eq!(types.kind(result), Some(caller.kind()));
    assert!(work > 0);
    assert!(restore(&mut types, &proof, caller, &mut work)?.is_none());
    assert!(restore(&mut types, &proof, distinct, &mut work)?.is_none());
    assert_eq!(proof.equalities, debts);
    assert_eq!(proof.regions, 10..12);
    let mut missing = Obligations::new(10);
    missing.regions.end = 11;
    missing.equalities.push((RegionId(10), RegionId(3)));
    assert!(restore(&mut types, &missing, input, &mut work)?.is_none());
    let foreign = application(&mut types, RegionId(99), false)?;
    let before = work;
    assert!(restore(&mut types, &proof, foreign, &mut work).is_err());
    assert!(work > before);
    Ok(())
}

#[test]
fn repeated_normalization_charges_each_selected_proof_and_terminates() -> Result<(), &'static str> {
    let mut per_call = None;
    for count in [8, 32, 128] {
        let mut types = Store::default();
        let input = application(&mut types, RegionId(3), true)?;
        let mut proof = Obligations::new(10);
        proof.regions.end = 11;
        proof
            .equalities
            .extend([(RegionId(10), RegionId(3)), (RegionId(10), RegionId(1))]);
        let mut work = 0;
        for _ in 0..count {
            let restored = restore(&mut types, &proof, input, &mut work)?.ok_or("missing input")?;
            let Some(Kind::Projection(application)) = types.kind(restored) else {
                return Err("restored result lost its associated identity");
            };
            assert!(
                restore(
                    &mut types,
                    &proof,
                    Alias::Associated(application),
                    &mut work
                )?
                .is_none()
            );
        }
        let each = work / count;
        eprintln!("normalizations={count} work={work} each={each}");
        assert_eq!(work, each * count);
        if let Some(previous) = per_call {
            assert_eq!(each, previous);
        }
        per_call = Some(each);
    }
    Ok(())
}

#[test]
fn unused_generic_initializers_retain_checked_symbolic_results()
-> Result<(), Box<dyn std::error::Error>> {
    for (result, value) in [("&'a u8", "&7"), ("(&'a u8,u8)", "(&7,9)")] {
        let source = format!(
            "trait Pick<'a>{{type Out;const V:Self::Out;}}struct Tag;impl<'a>Pick<'a> for Tag{{type Out={result};const V:Self::Out={value};}}struct S<'a,T>(&'a u8,T);impl<'a,T:Pick<'a>>S<'a,T>{{const C:T::Out={{let n=T::V;n}};}}fn main(){{}}"
        );
        let mut report = crate::test_source::resolve(&source, true)?;
        let analysis = crate::pipeline::analyze_source(&report)?;
        let mut facts = analysis.facts;
        let entry = crate::sema::entry::select(&mut report, &facts)?;
        crate::lower::instances::build_prepared(&report, &mut facts, entry, analysis.prepared)?;
        assert!(!facts.region_equalities.is_empty());
    }
    Ok(())
}

#[test]
fn unchanged_opaque_family_uses_original_request_arguments()
-> Result<(), Box<dyn std::error::Error>> {
    let source =
        "trait Pick{type Out<'a>;}fn value<'a,T:Pick>(x:T::Out<'a>)->T::Out<'a>{x}fn main(){}";
    let mut report = crate::test_source::resolve(source, true)?;
    let analysis = crate::pipeline::analyze_source(&report)?;
    let mut facts = analysis.facts;
    let entry = crate::sema::entry::select(&mut report, &facts)?;
    crate::lower::instances::build_prepared(&report, &mut facts, entry, analysis.prepared)?;
    Ok(())
}
