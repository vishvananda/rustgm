//! Mutable iteration preserves ordinary source, original and split-loan authority.
use super::*;
use crate::support::{intrinsics::Intrinsic, iteration::Collection, library_types::LibraryType};
const MUTABLE: &str =
    "struct V(u8);fn main(){let mut a=[V(1),V(2)];let mut i=a.iter_mut();let _=i.next();}";
fn original(facts: &Declarations) -> Result<sema::iteration::collections::Original> {
    facts
        .iteration
        .as_deref()
        .and_then(|r| r.collections.as_deref())
        .and_then(|r| r.mutable.as_deref())
        .and_then(|m| m.originals[Collection::SliceMutableIterator.index()])
        .ok_or_else(|| "mutable original".into())
}
#[test]
fn mutable_original_keeps_selected_owner_and_independent_lifetime_slots() -> Result {
    let (_, mut facts) = fixture(MUTABLE)?;
    let original = original(&facts)?;
    original.validate(&facts)?;
    for index in 0..4 {
        let mut forged = original;
        match index {
            0 => forged.declaration.definition = original.declaration.method,
            1 => forged.parameter = types::TypeId::UNIT,
            2 => forged.subject = types::TypeId::UNIT,
            _ => forged.item = types::TypeId::UNIT,
        }
        assert!(forged.validate(&facts).is_err());
    }
    let scheme = facts
        .lifetimes
        .functions
        .get_mut(&original.declaration.method)
        .ok_or("scheme")?;
    assert_eq!(
        (scheme.total, scheme.captures, scheme.late.as_slice()),
        (2, 1, &[1][..])
    );
    scheme.late[0] = 0;
    assert!(original.validate(&facts).is_err());
    facts
        .lifetimes
        .functions
        .get_mut(&original.declaration.method)
        .ok_or("scheme")?
        .late[0] = 1;
    facts
        .iteration
        .as_deref_mut()
        .and_then(|r| r.collections.as_deref_mut())
        .and_then(|r| r.mutable.as_deref_mut())
        .ok_or("mutable registry")?
        .status = sema::iteration::collections::Status::Prepared;
    assert!(original.validate(&facts).is_err());
    Ok(())
}
#[test]
fn mutable_kernel_enters_both_original_proofs_and_rejects_overlapping_split() -> Result {
    let (report, mut facts) = fixture(MUTABLE)?;
    for incremental in [false, true] {
        let mut contracts = if incremental {
            body::regions::Collector::default().finish(&facts)?.bodies
        } else {
            body::regions::collect(&facts)?.bodies
        };
        let mut templates = Templates { bodies: Vec::new() };
        append(&report, &mut facts, &mut templates, &mut contracts)?;
        assert_eq!(templates.bodies.len(), 10);
        body::promote::analyze_templates(&report, &mut facts, &mut templates)?;
        verify(&mut facts, &templates, contracts.clone())?;
        let method = original(&facts)?.declaration.method;
        let template = templates
            .bodies
            .iter_mut()
            .find(|t| matches!(t.body.owner,Owner::Function{definition,..}if definition==method))
            .ok_or("next body")?;
        assert!(template.body.values.iter().any(|v| matches!(
            v.kind,
            Rvalue::Intrinsic {
                intrinsic: Intrinsic::Read,
                ..
            }
        )));
        let rest = template
            .body
            .projections
            .iter_mut()
            .find(|p| {
                matches!(
                    p.kind,
                    ProjectionKind::Subslice {
                        prefix: 1,
                        suffix: 0
                    }
                )
            })
            .ok_or("rest projection")?;
        rest.kind = ProjectionKind::Subslice {
            prefix: 0,
            suffix: 0,
        };
        assert!(verify(&mut facts, &templates, contracts).is_err());
    }
    Ok(())
}
#[test]
fn mutable_read_identity_is_independently_checked_by_factory() -> Result {
    let (report, mut facts) = fixture(MUTABLE)?;
    facts
        .iteration
        .as_deref_mut()
        .and_then(|r| r.collections.as_deref_mut())
        .and_then(|r| r.mutable.as_deref_mut())
        .ok_or("registry")?
        .canonical
        .read
        .kind = Intrinsic::Write;
    let mut templates = Templates { bodies: Vec::new() };
    let mut contracts = Vec::new();
    assert!(append(&report, &mut facts, &mut templates, &mut contracts).is_err());
    Ok(())
}
#[test]
fn shared_custom_and_source_winners_keep_mutable_contracts_and_read_cold() -> Result {
    for source in [
        "fn main(){let _=1u8;}",
        "fn main(){let a=[1u8];for _ in &a{}}",
        "struct I;impl core::iter::Iterator for I{type Item=u8;fn next(&mut self)->Option<u8>{None}}fn main(){for _ in I{}}",
        "trait X{fn iter_mut(&mut self)->u8;}impl X for [u8;2]{fn iter_mut(&mut self)->u8{9}}fn main(){let mut a=[1u8,2];let _=a.iter_mut();}",
    ] {
        let (report, facts) = fixture(source)?;
        assert!(
            facts
                .iteration
                .as_deref()
                .and_then(|r| r.collections.as_deref())
                .is_none_or(|r| r.mutable.is_none())
        );
        if let Some(nominal) = report
            .resolution
            .library_types
            .get(LibraryType::SliceIterMut)
        {
            assert!(
                facts
                    .aggregates
                    .get(types::ItemId(nominal.definition.0))
                    .is_none()
            );
            assert!(
                !facts
                    .derives
                    .contains_key(&types::ItemId(nominal.definition.0))
            );
        }
        if let Some(read) = report.resolution.intrinsics.get(Intrinsic::Read) {
            assert!(!facts.function_indices.contains_key(&read.definition));
        }
    }
    Ok(())
}
#[test]
fn mutable_group_activation_and_unrelated_source_work_stay_bounded() -> Result {
    let mut observations = Vec::new();
    for (loops, unrelated) in [(8, 0), (32, 0), (128, 0), (1, 8), (1, 32), (1, 128)] {
        let mut source = String::new();
        for index in 0..unrelated {
            write!(source, "struct U{index};fn unused{index}(_:U{index}){{}}")?;
        }
        source.push_str("fn main(){let mut a=[1u8,2];");
        for _ in 0..loops {
            source.push_str("for x in &mut a{*x+=1u8;}");
        }
        source.push('}');
        let report = crate::test_source::resolve(&source, true)?;
        let facts = sema::declarations::analyze(&report)?;
        let before = (
            facts.lifetimes.nominal_requirements.steps,
            facts.lifetimes.variances.steps,
        );
        let facts = sema::bodies::analyze(&report, facts)?;
        let selected = original(&facts)?;
        selected.validate(&facts)?;
        assert_eq!(
            facts
                .generics
                .implementations()
                .filter(|i| i.owner == selected.declaration.definition)
                .count(),
            1
        );
        let work = (
            facts.lifetimes.nominal_requirements.steps - before.0,
            facts.lifetimes.variances.steps - before.1,
        );
        observations.push(work);
        eprintln!(
            "mutable loops={loops} unrelated={unrelated} inference={} selections={} metadata={} variances={}",
            facts.body_work.inference_steps, facts.body_work.trait_selections, work.0, work.1
        );
    }
    assert!(observations.iter().all(|o| *o == observations[0]));
    Ok(())
}
#[test]
fn mutable_iteration_keeps_ordinary_record_sizes_and_records_lazy_catalog_cost() {
    eprintln!(
        "mutable sizes declarations={} resolution={} collections={} mutable_registry={} canonical_mutable={} kind={} context={} store={}",
        size_of::<Declarations>(),
        size_of::<crate::resolve::model::Resolution>(),
        size_of::<sema::iteration::collections::Registry>(),
        size_of::<sema::iteration::collections::MutableRegistry>(),
        size_of::<crate::resolve::model::BuiltinMutableCollections>(),
        size_of::<types::Kind>(),
        size_of::<types::infer::Context>(),
        size_of::<types::Store>()
    );
    // Nullable policy/layout caches add 16 bytes; source bindings add one cold pointer.
    assert_eq!(size_of::<Declarations>(), 7048);
    assert_eq!(size_of::<types::Kind>(), 24);
    assert_eq!(size_of::<types::infer::Context>(), 400);
    assert_eq!(size_of::<types::Store>(), 464);
    assert_eq!(size_of::<body::ir::Value>(), 24);
}
