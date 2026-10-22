//! Input scopes preserve ownership, repeated scopes and lazy retained storage.
use super::*;
use crate::resolve::unit::FileId;
use crate::syntax::ast::NodeId;
use std::mem::size_of;

fn key(node: u32) -> NodeKey {
    NodeKey {
        file: FileId(0),
        node: NodeId::new(node),
    }
}

#[test]
fn input_scopes_are_lazy_owned_and_keep_nested_scope_results() -> Result<(), &'static str> {
    let mut pending = Inputs::default();
    assert!(pending.entries.is_none());
    assert!(pending.finish(key(1), 0).is_err());
    pending.record(key(1), RegionId(100));
    assert!(pending.entries.is_none());
    let first = pending.begin(key(1))?;
    pending.record(key(1), RegionId(1));
    pending.record(key(2), RegionId(99));
    let nested = pending.begin(key(1))?;
    pending.record(key(1), RegionId(2));
    assert!(pending.finish(key(2), 0).is_err());
    assert!(pending.finish(key(1), usize::MAX).is_err());
    assert_eq!(pending.finish(key(1), nested)?, [RegionId(2)]);
    assert_eq!(pending.finish(key(1), first)?, [RegionId(1), RegionId(2)]);
    assert!(pending.finish(key(1), 0).is_err());
    let repeated = pending.begin(key(1))?;
    assert!(pending.finish(key(1), repeated)?.is_empty());
    assert_eq!(pending.entries.as_deref().map(HashMap::len), Some(0));
    println!(
        "annotation Inputs={} Input={} Entry={} Map={}",
        size_of::<Inputs>(),
        size_of::<Input>(),
        size_of::<(NodeKey, Input)>(),
        size_of::<HashMap<NodeKey, Input>>()
    );
    assert_eq!(size_of::<Inputs>(), 8);
    Ok(())
}
