//! Collect owned source lifetimes under a scratch budget proportional to runs.
use super::{Contract, Kind, segments};
use crate::{
    body::{
        ir::{BodyKey, NodeKey},
        regions::Obligation,
    },
    support::source::FileId,
    syntax::ast::NodeId,
    types::RegionId,
};
use std::ops::Range;

#[test]
fn adjacent_variables_share_scratch_and_keep_foreign_gaps() -> Result<(), &'static str> {
    let source = NodeKey {
        file: FileId(0),
        node: NodeId::new(3),
    };
    let obligations = [100..160, 1..40, 36..70, 66..80]
        .into_iter()
        .flatten()
        .map(|id| Obligation {
            source: source.into(),
            kind: Kind::Variable {
                id: RegionId(id),
                needs_provenance: false,
            },
        })
        .collect();
    let input = Contract {
        key: BodyKey::Anonymous(source),
        source: source.into(),
        signature: None,
        universals: 0,
        assumptions: Vec::new(),
        parameters: Vec::new(),
        type_assumptions: Vec::new(),
        obligations,
        proofs: Vec::new(),
        builtins: None,
        output: None,
        require_result_formation: false,
    };
    let (owned, _) = segments(&input, 4 * size_of::<Range<u32>>())?;
    assert_eq!(owned, [1..80, 100..160]);
    Ok(())
}
