//! Structural warmth never replaces exact source registration or const permission.
use super::*;
use crate::{pipeline, test_source};
use std::path::Path;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn source_glue_selection_rechecks_complete_registration_and_const_policy() -> Result {
    let source = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../pa31/tests/spec/drop-glue/primitive-dropless.rs"),
    )?;
    let report = test_source::source_provider(&source, "glue_registration")?;
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = pipeline::analyze_source(&report)?;
    let target = crate::lower::instances::inspect_prepared(
        &report,
        &mut facts,
        prepared,
        |facts, proof| -> Result<Instance> {
            for body in &proof.program().bodies {
                if let Some(instance) = body.instance
                    && source_drop_glue(facts, instance)?.is_some()
                {
                    return Ok(instance);
                }
            }
            Err("closed actual helper target".into())
        },
    )??;
    let expected = source_drop_glue(&facts, target)?.ok_or("selection")?;
    assert_eq!(
        expected.element,
        TypeId::primitive(crate::types::Primitive::U8)
    );
    assert!(expected.empty_registry);
    for _ in 0..128 {
        assert_eq!(
            source_drop_glue(&facts, target)?.ok_or("warm")?.element,
            expected.element
        );
        assert!(check_const_drop_glue(&facts, target).is_err());
    }
    let marker = facts
        .source_bindings
        .as_deref()
        .ok_or("catalog")?
        .destruction_registration;
    for forged in [None, Some(DefId(u32::MAX))] {
        facts
            .source_bindings
            .as_deref_mut()
            .ok_or("catalog")?
            .destruction_registration = forged;
        assert!(source_drop_glue(&facts, target).is_err());
    }
    facts
        .source_bindings
        .as_deref_mut()
        .ok_or("catalog")?
        .destruction_registration = marker;
    assert_eq!(
        source_drop_glue(&facts, target)?.ok_or("restored")?.element,
        expected.element
    );
    Ok(())
}
