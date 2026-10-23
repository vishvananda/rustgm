//! Reached operations preserve both signature views and exact proved ownership.
use super::*;
use crate::{pipeline, test_source};
use std::path::Path;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn reached_header_keeps_actual_source_instance_and_both_callable_signatures() -> Result {
    let source = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../pa31/tests/spec/drop-glue/primitive-dropless.rs"),
    )?;
    let report = test_source::source_provider(&source, "reached_owner")?;
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = pipeline::analyze_source(&report)?;
    crate::lower::instances::inspect_prepared(
        &report,
        &mut facts,
        prepared,
        |_, proof| -> Result {
            let bodies = &proof.program().bodies;
            let body = bodies
                .iter()
                .find(|body| {
                    body.instance.is_some()
                        && matches!(
                            body.owner,
                            Owner::Function {
                                execution: Some(_),
                                ..
                            }
                        )
                })
                .ok_or("missing genuinely materialized function")?;
            matched_header(body, body)?;
            matched_header(&body.header(), body)?;
            let other = bodies
                .iter()
                .find(|other| other.source != body.source)
                .ok_or("missing distinct proved source")?;
            let mut wrong = body.header();
            wrong.source = other.source;
            assert!(matched_header(body, &wrong).is_err());
            let mut wrong = body.header();
            wrong.instance = None;
            assert!(matched_header(body, &wrong).is_err());
            let mut wrong = body.header();
            wrong.result = TypeId::primitive(crate::types::Primitive::U8);
            assert_ne!(wrong.result, body.result);
            assert!(matched_header(body, &wrong).is_err());
            let mut wrong = body.header();
            if let Owner::Function { definition, .. } = &mut wrong.owner {
                *definition = DefId(u32::MAX);
            }
            assert!(matched_header(body, &wrong).is_err());
            let mut wrong = body.header();
            if let Owner::Function { execution, .. } = &mut wrong.owner {
                *execution = None;
            }
            assert!(matched_header(body, &wrong).is_err());
            let mut wrong = body.header();
            if let Owner::Function { signature, .. } = &mut wrong.owner {
                *signature = TypeId::UNIT;
            }
            assert!(matched_header(body, &wrong).is_err());
            let mut wrong = body.header();
            wrong.owner = Owner::AnonymousConstant;
            assert!(matched_header(body, &wrong).is_err());
            assert!(matched_header(&wrong, &wrong).is_err());
            Ok(())
        },
    )??;
    Ok(())
}
