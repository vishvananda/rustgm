//! Real fresh immutable views reject scalar-pool mutations before original publication.
use super::super::{ImportedOriginals, rebuilt_operations};
use crate::{body::ir::Rvalue, sema::model::Scalar, support::crates};
type Result = std::result::Result<(), Box<dyn std::error::Error>>;
#[test]
fn fresh_scalar_storage_rejects_changed_missing_extra_and_foreign_slots() -> Result {
    let (report, analysis, _) = crate::source_packets::client_analysis(
        crate::encoded_literals::provider()?,
        &crate::encoded_literals::fixture("consumer")?,
    )?;
    let definition = report
        .resolution
        .definitions
        .iter()
        .enumerate()
        .find(|(_, d)| d.node.is_none() && report.package.names.get(d.name) == "byte")
        .map(|(i, _)| u32::try_from(i).map(crate::resolve::model::DefId))
        .transpose()?
        .ok_or("actual imported byte")?;
    let mut steps = 0;
    let view = crate::sema::crates::source_original(&analysis.facts, definition, &mut steps)?
        .ok_or("actual retained source view")?;
    let body = rebuilt_operations::build(&view, &mut ImportedOriginals::default())?;
    rebuilt_operations::validate(&view, &body, &mut steps)?;
    assert_eq!(body.scalars, [Scalar::Integer(7)]);
    let edits: [fn(&mut crate::body::ir::Body); 4] = [
        |b| b.scalars[0] = Scalar::Integer(8),
        |b| b.scalars.clear(),
        |b| b.scalars.push(Scalar::Integer(7)),
        |b| {
            for value in &mut b.values {
                if let Rvalue::Scalar(id) = &mut value.kind {
                    id.0 = 1;
                }
            }
        },
    ];
    for edit in edits {
        let mut changed = body.clone();
        edit(&mut changed);
        assert!(rebuilt_operations::validate(&view, &changed, &mut steps).is_err());
    }
    let mut limited = ImportedOriginals {
        steps: crates::MAX_COMPONENTS,
        ..ImportedOriginals::default()
    };
    assert!(rebuilt_operations::build(&view, &mut limited).is_err());
    assert!(limited.rows.is_empty() && limited.steps > crates::MAX_COMPONENTS);
    Ok(())
}
