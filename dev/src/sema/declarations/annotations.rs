//! Partial local annotations reuse the declaration binder and elision machinery.
use super::{
    Checker, Error, Kind, List, NodeId, NodeKey, SignaturePhase, TypeFlavor, Work, regions,
};

impl Checker<'_> {
    pub(super) fn schedule_annotation(
        &mut self,
        key: NodeKey,
        kind: Kind,
        context: regions::Context,
    ) -> Result<(), Error> {
        if matches!(
            kind,
            Kind::Path { .. } | Kind::Applied { .. } | Kind::Qualified { .. }
        ) && self.schedule_implicit_projection(key, context)?
        {
            return Ok(());
        }
        if matches!(kind, Kind::Qualified { .. }) {
            return self.schedule_projection_path(key, context);
        }
        if let Kind::FunctionType(_) = kind {
            self.work
                .push(Work::Signature(key, None, SignaturePhase::Begin(context)));
            return Ok(());
        }
        if let Kind::Type {
            flavor: TypeFlavor::Ref | TypeFlavor::RefMut,
            elements,
        } = kind
        {
            self.schedule_reference(key, elements, context)?;
            return Ok(());
        }
        self.work.push(Work::Node(key, true, context));
        match kind {
            Kind::NeverType => {}
            Kind::Group(node) | Kind::Attributed { value: node, .. } => {
                self.node_work(key.file, node, context);
            }
            Kind::Path { .. } | Kind::Applied { .. } | Kind::Name(_) => {
                self.schedule_type_path(key, context)?;
            }
            Kind::Type { flavor, elements } => {
                let unit = self.report.package.unit(key.file);
                let elements = unit.configured.ast.list(elements);
                let count = elements
                    .len()
                    .saturating_sub(usize::from(flavor == TypeFlavor::Array));
                for child in elements[..count].iter().rev().copied() {
                    self.node_work(key.file, child, context);
                }
            }
            Kind::InferType => {
                return Err(self.error(
                    key,
                    "inference placeholders are not allowed in item signatures",
                ));
            }
            _ => return Err(self.error(key, "unsupported type annotation")),
        }
        Ok(())
    }
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
