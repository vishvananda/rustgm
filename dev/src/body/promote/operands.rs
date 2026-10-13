//! Typed source provenance distinguishes an operand constant from a temporary use.
use super::{Engine, Error};
use crate::{
    body::ir::{Action, ValueId},
    resolve::unit::NodeKey,
    sema::model::Operation,
    syntax::ast::{Kind, Unary},
};
impl Engine<'_, '_> {
    pub(super) fn index_sources(&mut self) -> Result<(), Error> {
        self.charge(self.body.values.len())?;
        self.sources.resize(self.body.values.len(), None);
        for statement in &self.body.statements {
            self.charge(1)?;
            if let Action::Evaluate(id) = statement.action {
                let slot = self
                    .sources
                    .get_mut(id.0 as usize)
                    .ok_or("missing promotion evaluation value")?;
                if slot.replace(statement.source).is_some() {
                    return Err("repeated promotion value definition".into());
                }
            }
        }
        Ok(())
    }
    pub(super) fn immediate_operand(
        &mut self,
        value: ValueId,
        left_operand: bool,
    ) -> Result<bool, Error> {
        if self.sources.is_empty() {
            self.index_sources()?;
        }
        let node = self
            .sources
            .get(value.0 as usize)
            .copied()
            .flatten()
            .ok_or("missing promotion operand source")?;
        let ast = &self
            .report
            .package
            .unit(self.body.source.file)
            .configured
            .ast;
        let Kind::Binary { left, right, .. } = ast.node(node).kind else {
            return Err("promotion binary lacks typed operand syntax".into());
        };
        let mut node = if left_operand { left } else { right };
        loop {
            self.charge(1)?;
            match ast.node(node).kind {
                Kind::Group(child) | Kind::Attributed { value: child, .. } => node = child,
                Kind::Expansion {
                    children,
                    tail: Some(child),
                } if children.start == children.end => node = child,
                Kind::Literal(_) | Kind::Name(_) | Kind::Path { .. } => return Ok(true),
                Kind::Unary {
                    op: Unary::Negate, ..
                } => {
                    let key = NodeKey {
                        node,
                        ..self.body.source
                    };
                    return Ok(matches!(
                        self.evaluator.facts().node(key).and_then(|f| f.operation),
                        Some(Operation::Unary { literal: true, .. })
                    ));
                }
                _ => return Ok(false),
            }
        }
    }
}
