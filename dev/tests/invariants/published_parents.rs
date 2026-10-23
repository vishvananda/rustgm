//! Failed component publication preserves exact candidate and context partitions.
use super::*;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
#[test]
fn duplicate_handoff_is_transactional_and_foreign_headers_fail() -> Result {
    let text = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../pa25/tests/spec/inline-query-context/paid-lifetime.rs"),
    )?;
    let report = crate::test_source::resolve(&text, true)?;
    let mut analysis = crate::pipeline::analyze_source(&report)?;
    let originals = analysis
        .prepared
        .original_contexts
        .take()
        .ok_or("retained originals")?;
    let templates =
        std::mem::take(&mut analysis.prepared.templates).finish(&report, &analysis.facts)?;
    let child = templates
        .bodies
        .iter()
        .find(|t| {
            matches!(t.body.key(), BodyKey::Anonymous(_))
                && originals.contracts.contains_key(&t.body.key())
        })
        .ok_or("retained child template")?;
    let mut prepared = crate::body::templates::Prepared::default();
    prepared.retain(child.clone())?;
    let rows = templates
        .bodies
        .iter()
        .chain(&originals.contexts)
        .filter_map(|template| {
            originals
                .contracts
                .get(&template.body.key())
                .map(|contract| Original {
                    template: template.clone(),
                    contract: contract.clone(),
                })
        })
        .collect();
    let mut rejected = Originals::default();
    assert!(
        rejected
            .publish(&mut analysis.facts, &mut prepared, rows)
            .is_err()
    );
    assert!(rejected.contexts.is_empty());
    assert!(rejected.contracts.is_empty());
    assert_eq!(rejected.bytes, 0);
    let template = originals
        .contexts
        .iter()
        .find(|t| matches!(t.body.owner, Owner::Function { .. }))
        .ok_or("context-only parent")?;
    let contract = originals
        .contracts
        .get(&template.body.key())
        .ok_or("parent contract")?;
    Original {
        template: template.clone(),
        contract: contract.clone(),
    }
    .validate(&analysis.facts)?;
    for changed in 0..5 {
        let mut row = Original {
            template: template.clone(),
            contract: contract.clone(),
        };
        match changed {
            0 => row.contract.source = child.body.source,
            1 => row.contract.universals += 1,
            2 => row.contract.signature = None,
            3 => {
                // Preserve the node number while changing its actual source unit.
                row.template.body.source = crate::resolve::unit::NodeKey {
                    file: crate::resolve::unit::FileId(u32::MAX),
                    ..crate::test_sites::node(row.template.body.source)?
                }
                .into();
                row.contract.source = row.template.body.source;
            }
            _ => row.contract.key = child.body.key(),
        }
        assert!(row.validate(&analysis.facts).is_err());
    }
    Ok(())
}
