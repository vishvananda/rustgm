//! Range originals require exact typed headers, ordinary proof and bounded demand.
use super::*;
use crate::sema::iteration::range_originals::Original as RangeOriginal;
const RANGE: &str = "fn main(){for _ in 1u8..4u8{}}";
fn ranges(facts: &Declarations) -> Result<Vec<RangeOriginal>> {
    Ok(facts
        .iteration
        .as_deref()
        .and_then(|r| r.ranges.as_deref())
        .ok_or("range registry")?
        .originals
        .clone())
}
#[test]
fn range_originals_use_both_collectors_and_ordinary_body_proof() -> Result {
    let (report, mut facts) = fixture(RANGE)?;
    for incremental in [false, true] {
        let mut contracts = if incremental {
            body::regions::Collector::default().finish(&facts)?.bodies
        } else {
            body::regions::collect(&facts)?.bodies
        };
        let mut templates = Templates::source(Vec::new());
        append(&report, &mut facts, &mut templates, &mut contracts)?;
        assert_eq!(templates.bodies.len(), 14);
        body::promote::analyze_templates(&report, &mut facts, &mut templates)?;
        verify(&mut facts, &templates, contracts.clone())?;
        let method = ranges(&facts)?[0].declaration.method;
        let template = templates.bodies.iter_mut().find(|t| matches!(t.body.owner, body::ir::Owner::Function { definition, .. } if definition == method)).ok_or("range next original")?;
        template.body.locals[0].kind = body::ir::LocalKind::Binding;
        assert!(verify(&mut facts, &templates, contracts).is_err());
    }
    Ok(())
}
#[test]
fn range_factory_rejects_wrong_registered_owner_item_binder_and_completion() -> Result {
    let (_, mut facts) = fixture(RANGE)?;
    let original = ranges(&facts)?[0];
    original.validate(&facts)?;
    for which in 0..4 {
        let mut forged = original;
        match which {
            0 => forged.declaration.method = original.declaration.item,
            1 => forged.element = types::TypeId::UNIT,
            2 => forged.subject = types::TypeId::UNIT,
            _ => forged.predicate.ty = types::TypeId::UNIT,
        }
        facts
            .iteration
            .as_deref_mut()
            .and_then(|r| r.ranges.as_deref_mut())
            .ok_or("registry")?
            .originals[0] = forged;
        assert!(forged.validate(&facts).is_err());
        facts
            .iteration
            .as_deref_mut()
            .and_then(|r| r.ranges.as_deref_mut())
            .ok_or("registry")?
            .originals[0] = original;
    }
    let stored = facts.definitions[original.declaration.item.0 as usize].take();
    assert!(original.validate(&facts).is_err());
    facts.definitions[original.declaration.item.0 as usize] = stored;
    let scheme = facts
        .lifetimes
        .functions
        .get_mut(&original.declaration.method)
        .ok_or("scheme")?;
    scheme.late[0] = 1;
    assert!(original.validate(&facts).is_err());
    facts
        .lifetimes
        .functions
        .get_mut(&original.declaration.method)
        .ok_or("scheme")?
        .late[0] = 0;
    facts
        .iteration
        .as_deref_mut()
        .and_then(|r| r.ranges.as_deref_mut())
        .ok_or("registry")?
        .complete = false;
    assert!(original.validate(&facts).is_err());
    Ok(())
}
#[test]
fn scalar_record_only_and_source_lookalikes_keep_range_bodies_cold() -> Result {
    let (report, facts) = fixture("fn main(){let _=1u8+2u8;}")?;
    assert!(report.resolution.builtin_traits.ranges().is_none());
    assert!(facts.iteration.is_none());
    let (_, facts) = fixture("fn main(){let r=1u8..4u8;let _=r.start;}")?;
    assert!(
        facts
            .iteration
            .as_deref()
            .and_then(|r| r.ranges.as_deref())
            .is_none()
    );
    let (report, facts) = fixture(
        "struct Range;impl Iterator for Range{type Item=u8;fn next(&mut self)->Option<u8>{None}}fn main(){for _ in Range{}}",
    )?;
    assert!(report.resolution.builtin_traits.ranges().is_none());
    assert!(
        facts
            .iteration
            .as_deref()
            .and_then(|r| r.ranges.as_deref())
            .is_none()
    );
    Ok(())
}
#[test]
fn shared_and_range_catalogs_activate_once_without_changing_old_identities() -> Result {
    let (report, facts) = fixture("fn main(){let a=[1u8,2];for _ in &a{}for _ in 1u8..3u8{}}")?;
    let registry = facts.iteration.as_deref().ok_or("registry")?;
    assert!(registry.collections.is_some());
    let ranges = registry.ranges.as_deref().ok_or("ranges")?;
    assert!(ranges.complete);
    assert_eq!(ranges.originals.len(), 12);
    assert_eq!(
        ranges.canonical.nominal,
        report
            .resolution
            .library_types
            .get(crate::support::library_types::LibraryType::Range)
            .ok_or("nominal")?
            .definition
    );
    for original in &ranges.originals {
        original.validate(&facts)?;
    }
    Ok(())
}
#[test]
fn range_activation_and_unrelated_source_work_remain_linear() -> Result {
    let mut observations = Vec::new();
    for (loops, unrelated) in [(8, 0), (32, 0), (128, 0), (1, 8), (1, 32), (1, 128)] {
        let mut source = String::new();
        for index in 0..unrelated {
            write!(source, "struct U{index};fn unused{index}(_:U{index}){{}}")?;
        }
        source.push_str("fn main(){");
        for _ in 0..loops {
            source.push_str("for _ in 1u8..4u8{}");
        }
        source.push('}');
        let report = crate::test_source::resolve(&source, true)?;
        let facts = sema::declarations::analyze(&report)?;
        let before = (
            facts.lifetimes.nominal_requirements.steps,
            facts.lifetimes.variances.steps,
        );
        let facts = sema::bodies::analyze(&report, facts)?;
        let originals = ranges(&facts)?;
        assert_eq!(originals.len(), 12);
        let work = (
            facts.lifetimes.nominal_requirements.steps - before.0,
            facts.lifetimes.variances.steps - before.1,
        );
        observations.push(work);
        eprintln!(
            "range loops={loops} unrelated={unrelated} inference={} selections={} metadata={} variances={}",
            facts.body_work.inference_steps, facts.body_work.trait_selections, work.0, work.1
        );
    }
    assert!(observations.iter().all(|o| *o == observations[0]));
    Ok(())
}
#[test]
fn range_catalog_preserves_ordinary_record_sizes() {
    eprintln!(
        "range sizes declarations={} resolution={} registry={} range_registry={} canonical_ranges={} original={} kind={} value={}",
        size_of::<Declarations>(),
        size_of::<crate::resolve::model::Resolution>(),
        size_of::<sema::iteration::Registry>(),
        size_of::<sema::iteration::range_originals::Registry>(),
        size_of::<crate::resolve::model::BuiltinRanges>(),
        size_of::<RangeOriginal>(),
        size_of::<types::Kind>(),
        size_of::<body::ir::Value>()
    );
    // The public nullable policy/layout caches add 16 bytes; source bindings add 8.
    assert_eq!(size_of::<Declarations>(), 7048);
    assert_eq!(size_of::<crate::resolve::model::Resolution>(), 2032);
    assert_eq!(size_of::<sema::iteration::Registry>(), 304);
    assert_eq!(size_of::<types::Kind>(), 24);
    assert_eq!(size_of::<body::ir::Value>(), 24);
    assert_eq!(size_of::<types::infer::Context>(), 400);
    assert_eq!(size_of::<types::Store>(), 464);
}

#[test]
fn range_factory_requires_immutable_canonical_catalog() -> Result {
    let (report, mut facts) = fixture(RANGE)?;
    let original = ranges(&facts)?[0];
    facts
        .iteration
        .as_deref_mut()
        .and_then(|r| r.ranges.as_deref_mut())
        .ok_or("registry")?
        .canonical
        .nominal = original.declaration.definition;
    let mut contracts = body::regions::collect(&facts)?.bodies;
    let mut templates = Templates::source(Vec::new());
    assert!(append(&report, &mut facts, &mut templates, &mut contracts).is_err());
    Ok(())
}
