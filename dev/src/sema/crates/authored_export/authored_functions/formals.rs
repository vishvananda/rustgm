//! Only real simple input declarations are associated with formal value slots.
use super::{Context, Declarations, Kind, Report, Stage, g, s};
use crate::{resolve::unit::NodeKey, syntax::ast::Kind as Syntax};

impl Stage {
    pub(in super::super) fn formals(
        &mut self,
        report: &Report,
        facts: &Declarations,
        function: &crate::sema::model::Function,
        context: Context,
    ) -> Result<(g::Range, Option<s::Receiver>), &'static str> {
        let unit = report.package.unit(function.node.file);
        let ast = &unit.configured.ast;
        let start =
            u32::try_from(self.rows.formals.len()).map_err(|_| "source formal table overflow")?;
        let mut receiver = None;
        for &parameter in ast.list(function.parameters) {
            self.charge(1)?;
            if !unit.configured.active[parameter.get() as usize] {
                continue;
            }
            let mut node = parameter;
            while let Syntax::Attributed { value, .. } = ast.node(node).kind {
                self.charge(1)?;
                node = value;
            }
            let source = NodeKey {
                node,
                ..function.node
            };
            let ordinal = u32::try_from(self.rows.formals.len())
                .map_err(|_| "source formal table overflow")?
                - start;
            let binding = if let Some(actual) = facts.receivers.get(&source) {
                if ordinal != 0 || receiver.is_some() {
                    return Err("source receiver is not the exact first formal");
                }
                receiver = Some(s::Receiver {
                    formal: ordinal,
                    base: self.ty(facts, actual.base, context)?,
                    references: actual.references,
                    mutable_binding: actual.mutable,
                });
                Some(actual.definition)
            } else {
                let Syntax::Parameter { pattern, .. } = ast.node(node).kind else {
                    return Err("source function parameter has no actual formal syntax");
                };
                match ast.node(pattern).kind {
                    Syntax::Wildcard => None,
                    Syntax::Binding {
                        by_ref: false,
                        subpattern: None,
                        ..
                    } => Some(
                        facts
                            .node(NodeKey {
                                node: pattern,
                                ..source
                            })
                            .and_then(|facts| facts.binding)
                            .ok_or("source formal has no actual resolver binding")?,
                    ),
                    _ => return Err("source destructured parameter transport is not implemented"),
                }
            };
            let declaration = binding
                .map(|binding| self.declaration(binding))
                .transpose()?;
            if declaration.is_some_and(|id| {
                self.rows.declarations[id.0 as usize].definition.kind
                    != s::DefinitionKind::ValueParameter
            }) {
                return Err("source formal binding is not an actual value parameter declaration");
            }
            self.reserve(4 * size_of::<s::Formal>())?;
            self.rows.formals.push(s::Formal {
                declaration,
                ordinal,
            });
        }
        let end =
            u32::try_from(self.rows.formals.len()).map_err(|_| "source formal table overflow")?;
        let Some(Kind::Function { parameters, .. }) = facts.types.kind(function.signature) else {
            return Err("source formal signature is unknown");
        };
        if facts
            .types
            .list(parameters)
            .is_none_or(|parameters| parameters.len() != (end - start) as usize)
        {
            return Err("source formal inventory disagrees with the actual function signature");
        }
        Ok((g::Range { start, end }, receiver))
    }
}
