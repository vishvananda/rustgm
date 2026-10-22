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
