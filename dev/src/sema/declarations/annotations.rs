//! Partial local annotations reuse the declaration binder and elision machinery.
use super::{Checker, Error, Kind, List, NodeId, NodeKey, TypeFlavor};

impl Checker<'_> {
    pub(super) fn partial_compound(&self, key: NodeKey) -> bool {
        let partial = |node| {
            self.facts
                .partial_annotations
                .contains_key(&NodeKey { node, ..key })
        };
        match self.kind(key) {
            Kind::Group(node) | Kind::Attributed { value: node, .. } => partial(node),
            Kind::Type { flavor, elements } => {
                let children = self
                    .report
                    .package
                    .unit(key.file)
                    .configured
                    .ast
                    .list(elements);
                let count = children
                    .len()
                    .saturating_sub(usize::from(flavor == TypeFlavor::Array));
                children[..count].iter().any(|node| partial(*node))
            }
            _ => false,
        }
    }
    pub(super) fn partial_signature(
        &self,
        key: NodeKey,
        parameters: List,
        result: Option<NodeId>,
    ) -> Result<bool, Error> {
        if result.is_some_and(|node| {
            self.facts
                .partial_annotations
                .contains_key(&NodeKey { node, ..key })
        }) {
            return Ok(true);
        }
        let unit = self.report.package.unit(key.file);
        for parameter in unit.configured.ast.list(parameters) {
            if !unit.configured.active[parameter.0 as usize] {
                continue;
            }
            let node = super::parameter_type(
                self.report,
                NodeKey {
                    node: *parameter,
                    ..key
                },
            )?;
            if self
                .facts
                .partial_annotations
                .contains_key(&NodeKey { node, ..key })
            {
                return Ok(true);
            }
        }
        Ok(false)
    }
}
