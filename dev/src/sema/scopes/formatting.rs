//! Exact super-let endpoints reuse the existing expression/extension owner.
use super::{Context, Error, Id, Lifetime, NodeId, NodeKey, Syntax, Walker, Work};
use crate::support::library_types::LibraryValue;
impl Walker<'_> {
    fn format_default(&self, node: NodeId) -> Result<Id, Error> {
        let root = self.report.formatting_root(NodeKey {
            node,
            ..self.source
        })?;
        let Some(super::Node {
            temporary: Lifetime::Scope(scope),
            ..
        }) = self.facts.node(root.node)
        else {
            return Err(self.error(node, "formatting has no actual default temporary endpoint"));
        };
        Ok(scope)
    }
    pub(super) fn format_binding(
        &mut self,
        node: NodeId,
        value: NodeId,
        context: Context,
    ) -> Result<(), Error> {
        let scope = self.format_default(node)?;
        let Syntax::Let {
            pattern,
            value: Some(initializer),
            ty: None,
            otherwise: None,
        } = self.kind(value)
        else {
            return Err(self.error(node, "formatting changed its generated binding"));
        };
        let context = Context {
            temporary: Lifetime::Scope(scope),
            ..context
        };
        self.record(value, context)?;
        self.push(Work::Pattern(pattern, scope))?;
        // An unextended super-let does not extend a nested 2024 block tail.
        self.push(Work::Expression(initializer, context))
    }
    pub(super) fn extend_format_output(
        &mut self,
        node: NodeId,
        lifetime: Lifetime,
    ) -> Result<(), Error> {
        self.report.formatting_value(NodeKey {
            node,
            ..self.source
        })?;
        let Syntax::LibraryValue {
            kind: LibraryValue::FormatOutput { sites },
            ..
        } = self.kind(node)
        else {
            return Err(self.error(node, "formatting lost its output inventory"));
        };
        for index in (sites.start..sites.start + 2).rev() {
            self.push(Work::ExtendExpression(self.child(index), lifetime))?;
        }
        Ok(())
    }
    pub(super) fn finish_extensions(&mut self) -> Result<(), Error> {
        // Move the same map temporarily so endpoint updates need no second index.
        let extensions = std::mem::take(&mut self.facts.scratch.extensions);
        for (&node, &lifetime) in &extensions {
            let entry = self.facts.scratch.index.get(node)[0];
            let Some(info) = self.facts.scratch.nodes.get_mut(entry as usize) else {
                return Err(self.error(node, "extended expression lacks scope facts"));
            };
            info.temporary = lifetime;
            info.extended = true;
            if let Syntax::LibraryValue {
                kind: LibraryValue::FormatSuperLet { .. },
                value: Some(value),
            } = self.kind(node)
            {
                let default = self.format_default(node)?;
                let Lifetime::Scope(scope) = lifetime else {
                    return Err(self.error(
                        node,
                        "nonempty formatting is unavailable in static initialization",
                    ));
                };
                let Syntax::Let { pattern, .. } = self.kind(value) else {
                    return Err(self.error(node, "formatting lost its actual binding pattern"));
                };
                let definition = self
                    .types
                    .node(NodeKey {
                        node: pattern,
                        ..self.source
                    })
                    .and_then(|f| f.binding)
                    .ok_or_else(|| {
                        self.error(node, "formatting binding lacks ordinary resolution proof")
                    })?;
                if self.facts.scratch.bindings.get(&definition) != Some(&default) {
                    return Err(self.error(
                        node,
                        "formatting binding changed its actual default endpoint",
                    ));
                }
                self.facts.scratch.bindings.insert(definition, scope);
            }
        }
        self.facts.scratch.extensions = extensions;
        Ok(())
    }
}
