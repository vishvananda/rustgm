//! Proved helper bodies retain full type, owner and both signature identities.
use super::*;
use crate::{pipeline, test_source};
use std::path::Path;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn source_cleanup_witness_rejects_changed_types_and_materialized_headers() -> Result {
    let source = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../pa31/tests/spec/drop-glue/primitive-dropless.rs"),
    )?;
    let report = test_source::source_provider(&source, "glue_witness")?;
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = pipeline::analyze_source(&report)?;
    crate::lower::instances::inspect_prepared(
        &report,
        &mut facts,
        prepared,
        |facts, proof| -> Result {
            let mut found = false;
            for body in &proof.program().bodies {
                let Some(instance) = body.instance else {
                    continue;
                };
                let Some(selection) =
                    crate::sema::compiler_items::source_drop_glue(facts, instance)?
                else {
                    continue;
                };
                checked(&facts.types, body, selection)?;
                retained_header(body, body)?;
                let mut wrong = selection;
                wrong.element = TypeId::primitive(crate::types::Primitive::Bool);
                assert!(checked(&facts.types, body, wrong).is_err());
                let mut wrong = selection;
                wrong.definition = DefId(u32::MAX);
                assert!(checked(&facts.types, body, wrong).is_err());
                let mut wrong = selection;
                wrong.signature = TypeId::UNIT;
                assert!(checked(&facts.types, body, wrong).is_err());
                let mut header = body.header();
                header.locals = body.locals.clone();
                if let Owner::Function { execution, .. } = &mut header.owner {
                    *execution = None;
                }
                assert!(checked(&facts.types, &header, selection).is_err());
                assert!(retained_header(body, &header).is_err());
                let mut header = body.header();
                header.instance = None;
                assert!(retained_header(body, &header).is_err());
                let other = proof
                    .program()
                    .bodies
                    .iter()
                    .find(|other| other.source != body.source)
                    .ok_or("distinct actual proved body")?;
                let mut header = body.header();
                header.source = other.source;
                assert!(retained_header(body, &header).is_err());
                let Owner::Function { signature, .. } = other.owner else {
                    return Err("other actual callable header".into());
                };
                let mut header = body.header();
                if let Owner::Function { execution, .. } = &mut header.owner {
                    assert_ne!(*execution, Some(signature));
                    *execution = Some(signature);
                }
                assert!(retained_header(body, &header).is_err());
                let mut header = body.header();
                header.locals = body.locals.clone();
                header.result = TypeId::primitive(crate::types::Primitive::U8);
                assert!(checked(&facts.types, &header, selection).is_err());
                let mut header = body.header();
                header.locals = body.locals.clone();
                header
                    .locals
                    .iter_mut()
                    .filter(|l| matches!(l.kind, LocalKind::Parameter(_)))
                    .for_each(|l| l.ty = TypeId::UNIT);
                assert!(checked(&facts.types, &header, selection).is_err());
                found = true;
            }
            assert!(found, "no actual proved helper instance");
            Ok(())
        },
    )??;
    Ok(())
}
