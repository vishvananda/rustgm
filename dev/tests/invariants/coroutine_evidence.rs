//! Selected language proof cannot authorize a different projection or stale schema.
use super::*;
use crate::{pipeline, sema, test_source};
#[path = "../helpers/asynccheck.rs"]
mod checks;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn actual_future_output_rejects_foreign_full_projection_inputs_on_cold_and_warm_reads() -> Result {
    let text = test_source::fixture("pa43/tests/spec/future-runtime-v1/selection-source.rs")?;
    let report = test_source::source_provider(&text, "future_output_provenance")?;
    let mut facts = sema::bodies::analyze(&report, sema::declarations::analyze(&report)?)?;
    let (templates, inputs) = checks::prepare(&report, &mut facts)?;
    let checked = checks::verify(&facts, &templates, inputs)?;
    let future = checks::future(&report, &mut facts, "value", &[])?;
    let (trait_, member) = schema::require(&facts)?;
    let trait_application =
        facts
            .types
            .intern_trait_application(crate::types::traits::Application {
                item: ItemId(trait_.0),
                arguments: GenericArgs::EMPTY,
            })?;
    let application = Application {
        subject: future,
        item: ItemId(member.0),
        trait_application,
        arguments: GenericArgs::EMPTY,
    };
    assert!(output(&facts, application, future).is_err());
    let _ = pipeline::future_application(&mut facts, &checked, future)?;
    let wrong_subject = checks::future(&report, &mut facts, "wide", &[])?;
    let _ = pipeline::future_application(&mut facts, &checked, wrong_subject)?;
    let clone = facts
        .source_bindings
        .as_deref()
        .and_then(|catalog| catalog.language(LangItem::Clone))
        .ok_or("actual Clone")?;
    let wrong_trait = facts
        .types
        .intern_trait_application(crate::types::traits::Application {
            item: ItemId(clone.definition.0),
            arguments: GenericArgs::EMPTY,
        })?;
    let own = facts
        .types
        .intern_regions(&[crate::types::RegionId::STATIC])?;
    for _ in 0..32 {
        assert_eq!(
            output(&facts, application, future)?,
            TypeId::primitive(crate::types::Primitive::U8)
        );
        for forged in [
            Application {
                subject: wrong_subject,
                ..application
            },
            Application {
                trait_application: wrong_trait,
                ..application
            },
            Application {
                item: ItemId(clone.definition.0),
                ..application
            },
            Application {
                arguments: own,
                ..application
            },
        ] {
            assert!(output(&facts, forged, future).is_err());
        }
    }
    Ok(())
}

#[test]
fn output_schema_requires_actual_bounds_and_completed_own_family_after_warm_selection() -> Result {
    use crate::sema::associated::Family;
    use crate::types::parameters::predicates::{Bound as PredicateBound, Predicate};
    let text = test_source::fixture("pa43/tests/spec/future-runtime-v1/selection-source.rs")?;
    let report = test_source::source_provider(&text, "future_schema_provenance")?;
    let mut facts = sema::bodies::analyze(&report, sema::declarations::analyze(&report)?)?;
    let (templates, inputs) = checks::prepare(&report, &mut facts)?;
    let checked = checks::verify(&facts, &templates, inputs)?;
    let future = checks::future(&report, &mut facts, "value", &[])?;
    let _ = pipeline::future_application(&mut facts, &checked, future)?;
    let (trait_, member) = schema::require(&facts)?;
    let trait_application =
        facts
            .types
            .intern_trait_application(crate::types::traits::Application {
                item: ItemId(trait_.0),
                arguments: GenericArgs::EMPTY,
            })?;
    let application = Application {
        subject: future,
        item: ItemId(member.0),
        trait_application,
        arguments: GenericArgs::EMPTY,
    };
    let _ = output(&facts, application, future)?;
    let contract = facts
        .associated
        .types
        .get_mut(&member)
        .ok_or("actual Output contract")?;
    let bounds = std::mem::take(&mut contract.bounds);
    assert!(
        output(&facts, application, future)
            .err()
            .is_some_and(|message| message.contains("canonical bound"))
    );
    let contract = facts
        .associated
        .types
        .get_mut(&member)
        .ok_or("actual Output contract")?;
    contract.bounds = bounds;
    contract.bounds.push(
        Predicate {
            ty: contract.projection,
            bound: PredicateBound::Copy,
        }
        .into(),
    );
    assert!(
        output(&facts, application, future)
            .err()
            .is_some_and(|message| message.contains("additional result"))
    );
    let _ = facts
        .associated
        .types
        .get_mut(&member)
        .ok_or("actual Output contract")?
        .bounds
        .pop();
    let previous = facts.associated.families.remove(&member);
    facts.associated.families.insert(
        member,
        Family {
            captures: 0,
            total: 0,
            regions: Vec::new(),
            requirements: None,
        },
    );
    assert!(output(&facts, application, future).is_err());
    facts.associated.families.insert(
        member,
        Family {
            captures: 0,
            total: 1,
            regions: Vec::new(),
            requirements: Some(Vec::new()),
        },
    );
    assert!(output(&facts, application, future).is_err());
    facts.associated.families.remove(&member);
    if let Some(previous) = previous {
        facts.associated.families.insert(member, previous);
    }
    assert_eq!(
        output(&facts, application, future)?,
        TypeId::primitive(crate::types::Primitive::U8)
    );
    Ok(())
}
