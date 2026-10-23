/// Source-only invariants must never reinterpret an imported body as syntax.
pub(crate) fn node(
    site: impl Into<crate::body::ir::ProofSite>,
) -> Result<crate::resolve::unit::NodeKey, &'static str> {
    site.into()
        .source()
        .ok_or("source-only fixture received imported provenance")
}
pub(crate) fn next_node(
    site: impl Into<crate::body::ir::ProofSite>,
) -> Result<crate::resolve::unit::NodeKey, &'static str> {
    let mut source = node(site)?;
    source.node = crate::syntax::ast::NodeId::checked(
        source
            .node
            .get()
            .checked_add(1)
            .ok_or("test node overflow")?,
    )
    .ok_or("test node overflow")?;
    Ok(source)
}
