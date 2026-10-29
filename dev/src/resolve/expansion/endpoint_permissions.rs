//! `PatTyRange` permission borrows one actual builtin invocation, never a name.
use super::{Body, Builtin, Error, ExpansionId, Report, permission_frames::FrameView};
use crate::{
    resolve::unit::NodeKey,
    syntax::{
        ast::{Kind, NodeId},
        gate_sites::{Endpoint, Helper},
    },
};

pub(crate) struct PatTyRangeSite<'a> {
    report: &'a Report,
    key: NodeKey,
    row: Endpoint,
    base: NodeId,
}
impl PatTyRangeSite<'_> {
    pub(crate) fn helper(&self) -> Helper {
        self.row.helper
    }
    pub(crate) fn base(&self) -> NodeKey {
        NodeKey {
            node: self.base,
            ..self.key
        }
    }
    pub(crate) fn allows(&self, report: &Report, key: NodeKey, feature: &str) -> bool {
        std::ptr::eq(report, self.report)
            && key == self.key
            && feature == "pattern_type_range_trait"
    }
}
impl Report {
    pub(crate) fn pattern_endpoint(&self, key: NodeKey) -> Result<PatTyRangeSite<'_>, Error> {
        self.lint_producer()
            .ok_or_else(|| Error::unlocated("pattern endpoint source payer missing"))?
            .diagnostic_payload(key, 24, 0)?;
        let unit = self
            .package
            .units
            .get(key.file.0 as usize)
            .ok_or_else(|| Error::unlocated("pattern endpoint file missing"))?;
        let ast = &unit.configured.ast;
        let invalid = || {
            Error::at(
                key.file,
                ast.nodes
                    .get(key.node.get() as usize)
                    .map_or(0, |n| n.span.start),
                "pattern endpoint source invocation or ownership changed",
            )
        };
        ast.validate_gate_sites_source(unit.source.original_image())
            .map_err(|e| Error::at(key.file, e.offset, e.message))?;
        let Some(Kind::PatternEndpoint { gate }) =
            ast.nodes.get(key.node.get() as usize).map(|n| n.kind)
        else {
            return Err(invalid());
        };
        let row = *ast.endpoint(gate).ok_or_else(invalid)?;
        let base = row
            .validate(ast, gate)
            .map_err(|e| Error::at(key.file, e.offset, e.message))?;
        let context = row.invocation.ok_or_else(invalid)?;
        let invocation = self
            .invocations
            .get(context.get() as usize - 1)
            .ok_or_else(invalid)?;
        if row.node != key.node
            || invocation.context.0 != context.get()
            || invocation.node
                != (NodeKey {
                    node: row.root,
                    ..key
                })
            || invocation.generated != ast.node(row.root).span
            || [row.node, row.expression, row.root, row.range]
                .into_iter()
                .any(|n| unit.configured.active.get(n.get() as usize) != Some(&true))
        {
            return Err(invalid());
        }
        let view = FrameView {
            package: &self.package,
            definitions: &self.definitions,
            invocations: &self.invocations,
            facts: &self.resolution,
        };
        let frame = view.frame(ExpansionId(context.get()))?;
        if !view.matches(frame)
            || !matches!(
                view.definition(invocation.definition)?.body,
                Body::Builtin(Builtin::PatternType)
            )
        {
            return Err(invalid());
        }
        Ok(PatTyRangeSite {
            report: self,
            key,
            row,
            base,
        })
    }
}

#[cfg(test)]
#[path = "../../../tests/invariants/endpoint_origins.rs"]
mod tests;
