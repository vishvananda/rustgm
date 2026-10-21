//! Syntactic lifetime extension consumes resolution, never callable type guesses.
use super::{Error, Lifetime, NodeId, Syntax, Unary, Walker, Work};
use crate::{resolve::model::Target, syntax::ast::BlockFlavor};

impl Walker<'_> {
    pub(super) fn extending_pattern(&mut self, root: NodeId) -> Result<bool, Error> {
        self.facts.scratch.pattern_work.push(root);
        while let Some(node) = self.facts.scratch.pattern_work.pop() {
            if !self.active(node) {
                continue;
            }
            // Account for this separate, nonrecursive qualification walk.
            if self.facts.work + self.facts.scratch.pattern_work.len() >= 1_000_000 {
                return Err(self.error(root, "temporary scope work limit exceeded"));
            }
            self.facts.work += 1;
            match self.kind(node) {
                Syntax::Binding { by_ref: true, .. } => {
                    self.facts.scratch.pattern_work.clear();
                    return Ok(true);
                }
                Syntax::Pattern { elements, .. }
                | Syntax::StructPattern {
                    fields: elements, ..
                } => {
                    let count = (elements.end - elements.start) as usize;
                    if count + self.facts.scratch.pattern_work.len() + self.facts.work > 1_000_000 {
                        return Err(self.error(root, "temporary scope work limit exceeded"));
                    }
                    for index in elements.start..elements.end {
                        self.facts.scratch.pattern_work.push(self.child(index));
                    }
                }
                Syntax::PatternField { pattern, .. }
                | Syntax::Group(pattern)
                | Syntax::Attributed { value: pattern, .. } => {
                    self.facts.scratch.pattern_work.push(pattern);
                }
                // By-value @ bindings and reference patterns are not extending patterns.
                _ => {}
            }
        }
        Ok(false)
    }
    pub(super) fn extend_place(&mut self, node: NodeId, lifetime: Lifetime) -> Result<(), Error> {
        if self.facts.scratch.extensions.get(&node) == Some(&lifetime) {
            return Ok(());
        }
        if self
            .facts
            .scratch
            .extensions
            .insert(node, lifetime)
            .is_some()
        {
            return Err(self.error(node, "temporary has conflicting lifetime extensions"));
        }
        match self.kind(node) {
            Syntax::Expansion {
                children,
                tail: Some(value),
            } if children.start == children.end => {
                self.push(Work::ExtendPlace(value, lifetime))?;
            }
            Syntax::Unary {
                op:
                    Unary::Borrow
                    | Unary::BorrowMut
                    | Unary::Dereference
                    | Unary::RawConst
                    | Unary::RawMut,
                operand: value,
            }
            | Syntax::Field { value, .. }
            | Syntax::Index { value, .. }
            | Syntax::Group(value)
            | Syntax::Attributed { value, .. } => {
                self.push(Work::ExtendPlace(value, lifetime))?;
            }
            _ => {}
        }
        Ok(())
    }
    pub(super) fn extend_expression(
        &mut self,
        node: NodeId,
        lifetime: Lifetime,
    ) -> Result<(), Error> {
        if node != self.source.node
            && matches!(
                self.kind(node),
                Syntax::Block {
                    flavor: BlockFlavor::Const,
                    ..
                }
            )
        {
            return Ok(());
        }
        match self.kind(node) {
            Syntax::Unary {
                op: Unary::Borrow | Unary::BorrowMut | Unary::RawConst | Unary::RawMut,
                operand,
            } => {
                self.push(Work::ExtendExpression(operand, lifetime))?;
                self.push(Work::ExtendPlace(operand, lifetime))?;
            }
            Syntax::Array(elements) | Syntax::Tuple(elements) => {
                self.extend_list(elements, lifetime)?;
            }
            Syntax::LibraryValue {
                value: Some(value), ..
            }
            | Syntax::Group(value)
            | Syntax::Attributed { value, .. }
            | Syntax::Cast { value, .. }
            | Syntax::FieldInit { value, .. }
            | Syntax::Arm { value, .. } => {
                self.push(Work::ExtendExpression(value, lifetime))?;
            }
            Syntax::Block {
                flavor: BlockFlavor::Plain | BlockFlavor::Unsafe | BlockFlavor::Const,
                tail,
                ..
            }
            | Syntax::Expansion { tail, .. } => {
                if let Some(tail) = tail {
                    self.push(Work::ExtendExpression(tail, lifetime))?;
                }
            }
            Syntax::If {
                then, otherwise, ..
            } => {
                self.push(Work::ExtendExpression(then, lifetime))?;
                if let Some(otherwise) = otherwise {
                    self.push(Work::ExtendExpression(otherwise, lifetime))?;
                }
            }
            Syntax::Match { arms, .. } => self.extend_list(arms, lifetime)?,
            Syntax::Struct { fields, .. } => {
                self.extend_list(fields, lifetime)?;
            }
            Syntax::Call { callee, arguments } if self.direct_constructor(callee)? => {
                self.extend_list(arguments, lifetime)?;
            }
            _ => {}
        }
        Ok(())
    }
    fn extend_list(&mut self, list: super::List, lifetime: Lifetime) -> Result<(), Error> {
        for index in (list.start..list.end).rev() {
            self.push(Work::ExtendExpression(self.child(index), lifetime))?;
        }
        Ok(())
    }
    fn direct_constructor(&mut self, mut node: NodeId) -> Result<bool, Error> {
        loop {
            if self.facts.work >= 1_000_000 {
                return Err(self.error(node, "temporary scope work limit exceeded"));
            }
            self.facts.work += 1;
            match self.kind(node) {
                Syntax::Group(value) | Syntax::Attributed { value, .. } => node = value,
                Syntax::Expansion {
                    children,
                    tail: Some(value),
                } if children.start == children.end => node = value,
                Syntax::Name(_) | Syntax::Path { .. } => {
                    let key = super::NodeKey {
                        node,
                        ..self.source
                    };
                    return Ok(matches!(self.types.node(key).and_then(|f| f.reference),
                        Some(Target::Definition(id)) if self.types.members.constructors.contains_key(&id)));
                }
                _ => return Ok(false),
            }
        }
    }
}
