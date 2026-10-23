//! Deferred registration and wakeup charge touched applications, never all owners.
use super::*;

#[test]
fn deferred_preparation_keeps_the_existing_cold_record_slot() {
    assert_eq!(size_of::<crate::sema::opaque::Opaques>(), 144);
    assert_eq!(
        size_of::<Option<Box<Preparation>>>(),
        size_of::<Option<Box<crate::abi::templates::Templates>>>()
    );
    eprintln!(
        "opaque-record={} preparation={} former-layouts={} cold-slot={} failure={}",
        size_of::<crate::sema::opaque::Opaques>(),
        size_of::<Preparation>(),
        size_of::<crate::abi::templates::Templates>(),
        size_of::<Option<Box<Preparation>>>(),
        size_of::<StorageFailure>()
    );
}

#[test]
fn storage_failure_keeps_the_actual_defining_owner_on_cold_and_warm_requests()
-> Result<(), Box<dyn std::error::Error>> {
    for source in [
        "fn f<T:Copy>(x:T)->impl Copy{(x,f(x))}fn main(){}",
        "fn unrelated(){}fn f()->impl Copy{(f(),)}fn main(){}",
    ] {
        let report = crate::test_source::resolve(source, true)?;
        let crate::pipeline::Analysis {
            mut facts,
            prepared,
        } = crate::pipeline::analyze_source(&report)?;
        assert_eq!(facts.opaques.owners.len(), 1);
        let owner = *facts.opaques.owners.keys().next().ok_or("missing owner")?;
        let node =
            facts.functions[*facts.function_indices.get(&owner).ok_or("function index")?].node;
        let offset = report
            .package
            .unit(node.file)
            .configured
            .ast
            .node(node.node)
            .span
            .start;
        let error =
            crate::lower::instances::inspect_prepared(&report, &mut facts, prepared, |_, _| ())
                .err()
                .ok_or("recursive opaque storage was accepted")?;
        assert_eq!(error.file.source(), Some(node.file));
        assert_eq!(error.span.start, offset);
        let failure = facts
            .prepare_opaque_storage_located()
            .err()
            .ok_or("lost sticky failure")?;
        assert_eq!(failure.owner, Some(owner));
        assert_eq!(failure.message, "recursive type has infinite size");
        for _ in 0..8 {
            assert_eq!(facts.prepare_opaque_storage_located(), Err(failure));
            assert_eq!(facts.prepare_opaque_storage(), Err(failure.message));
        }
    }
    Ok(())
}

#[test]
fn wakeup_preserves_registration_order_and_ignores_other_identities() -> Result<(), &'static str> {
    for width in [8_u32, 32, 128] {
        for noise in [0_u32, 32, 128] {
            let mut preparation = Preparation::default();
            for id in 0..width {
                preparation.wait(OpaqueId(1), TypeId(id))?;
            }
            for id in 0..noise {
                preparation.wait(OpaqueId(2), TypeId(width + id))?;
            }
            let before = preparation.steps;
            preparation.wake(OpaqueId(1))?;
            assert_eq!(
                preparation.ready,
                (0..width).map(TypeId).collect::<Vec<_>>()
            );
            assert_eq!(preparation.steps - before, width as usize + 1);
            assert_eq!(preparation.waiting_entries, noise as usize);
            let after = preparation.steps;
            preparation.wake(OpaqueId(1))?;
            preparation.wake(OpaqueId(7))?;
            assert_eq!(preparation.steps, after);
            assert_eq!(preparation.ready.len(), width as usize);
        }
    }
    Ok(())
}

#[test]
fn preparation_limits_reject_before_queue_allocation() {
    let mut full = Preparation {
        waiting_entries: APPLICATIONS,
        ..Preparation::default()
    };
    assert!(full.wait(OpaqueId(0), TypeId(0)).is_err());
    assert_eq!(full.waiting.capacity(), 0);
    assert_eq!(full.ready.capacity(), 0);
    let mut exhausted = Preparation {
        steps: 32_000_000,
        ..Preparation::default()
    };
    assert!(exhausted.wait(OpaqueId(0), TypeId(0)).is_err());
    assert_eq!(exhausted.waiting.capacity(), 0);
    assert_eq!(exhausted.ready.capacity(), 0);
    assert!(exhausted.scratch(APPLICATIONS + 1, 0).is_err());
}

#[test]
fn unresolved_applications_register_once_without_revealing_inferred_heads() -> Result<(), String> {
    use crate::resolve::{
        model::DefId,
        unit::{FileId, NodeKey},
    };
    use crate::types::{GenericArgs, Kind, Primitive, copy, opaque};

    let report = crate::test_source::resolve("", true)?;
    let mut facts = crate::sema::declarations::analyze(&report).map_err(|e| e.to_string())?;
    facts.prepare_opaque_storage()?;
    assert!(facts.opaques.storage_preparation.is_none());
    let id = facts.types.declare_opaque(opaque::Declaration {
        trait_result: false,
        sized: true,
        copies: copy::Properties::default(),
    })?;
    let application = facts.types.intern(Kind::Opaque(id, GenericArgs::EMPTY))?;
    facts.opaques.declarations.insert(
        id,
        super::super::Declaration {
            owner: DefId(7),
            source: NodeKey {
                file: FileId(0),
                node: report.package.unit(FileId(0)).configured.ast.root,
            },
            application,
            bounds: Vec::new(),
            hidden: None,
            inferred: Some(TypeId::primitive(Primitive::U8)),
            free_regions: Vec::new(),
        },
    );
    facts.prepare_opaque_storage()?;
    let steps = facts
        .opaques
        .storage_preparation
        .as_ref()
        .ok_or("preparation")?
        .steps;
    for _ in 0..128 {
        facts.prepare_opaque_storage()?;
    }
    let preparation = facts
        .opaques
        .storage_preparation
        .as_ref()
        .ok_or("preparation")?;
    assert_eq!(preparation.steps, steps);
    assert_eq!(preparation.waiting_entries, 1);
    assert!(preparation.ready.is_empty());
    assert!(facts.types.storage_head(application).is_err());
    assert_eq!(
        facts
            .opaques
            .declarations
            .get(&id)
            .ok_or("declaration")?
            .hidden,
        None
    );
    // Exercise successful queue replay separately from proof authority: this
    // private fixture supplies its scalar defining result directly.
    facts.opaques.owners.insert(DefId(7), vec![id]);
    facts.close_opaque_results(DefId(7), &[])?;
    facts.prepare_opaque_storage()?;
    assert_eq!(
        facts.types.storage_head(application)?,
        TypeId::primitive(Primitive::U8)
    );
    let steps = facts
        .opaques
        .storage_preparation
        .as_ref()
        .ok_or("preparation")?
        .steps;
    for _ in 0..128 {
        facts.close_opaque_results(DefId(7), &[])?;
        facts.prepare_opaque_storage()?;
    }
    let preparation = facts
        .opaques
        .storage_preparation
        .as_ref()
        .ok_or("preparation")?;
    assert_eq!(preparation.steps, steps);
    assert_eq!(preparation.waiting_entries, 0);
    assert!(preparation.ready.is_empty());
    Ok(())
}

#[test]
fn failed_wakeup_remains_an_error_after_the_checked_head_closes() -> Result<(), String> {
    use crate::resolve::{
        model::DefId,
        unit::{FileId, NodeKey},
    };
    use crate::types::{GenericArgs, Kind, Primitive, copy, opaque};
    let report = crate::test_source::resolve("", true)?;
    let mut facts = crate::sema::declarations::analyze(&report).map_err(|e| e.to_string())?;
    let id = facts.types.declare_opaque(opaque::Declaration {
        trait_result: false,
        sized: true,
        copies: copy::Properties::default(),
    })?;
    let application = facts.types.intern(Kind::Opaque(id, GenericArgs::EMPTY))?;
    let owner = DefId(7);
    facts.opaques.owners.insert(owner, vec![id]);
    facts.opaques.declarations.insert(
        id,
        super::super::Declaration {
            owner,
            source: NodeKey {
                file: FileId(0),
                node: report.package.unit(FileId(0)).configured.ast.root,
            },
            application,
            bounds: Vec::new(),
            hidden: None,
            inferred: Some(TypeId::primitive(Primitive::U8)),
            free_regions: Vec::new(),
        },
    );
    facts.prepare_opaque_storage()?;
    facts
        .opaques
        .storage_preparation
        .as_mut()
        .ok_or("preparation")?
        .steps = 32_000_000;
    let error = facts
        .close_opaque_results(owner, &[])
        .err()
        .ok_or("missing wakeup budget rejection")?;
    assert!(
        facts
            .opaques
            .declarations
            .get(&id)
            .ok_or("declaration")?
            .hidden
            .is_some()
    );
    let preparation = facts
        .opaques
        .storage_preparation
        .as_mut()
        .ok_or("preparation")?;
    assert_eq!(
        preparation.failed.map(|failure| failure.message),
        Some(error)
    );
    assert_eq!(preparation.waiting_entries, 1);
    assert!(preparation.ready.is_empty());
    // Even a later cheap no-op cannot make this preparation usable again.
    assert_eq!(preparation.wake(OpaqueId(99)), Err(error));
    assert_eq!(facts.close_opaque_results(owner, &[]), Err(error));
    assert_eq!(facts.prepare_opaque_storage(), Err(error));
    assert!(facts.types.storage_head(application).is_err());
    Ok(())
}
