//! A borrowed constructor needs a sized input hint before its outer unsizing.
use super::{Argument, Checker, Constructor, Error, NodeId, TermId};
use crate::types::{Primitive, TypeId};

impl Checker<'_> {
    pub(in crate::sema::bodies) fn unsizing_constructor_context(
        &mut self,
        node: NodeId,
        actual: TermId,
        expected: Option<TermId>,
    ) -> Result<Option<TermId>, Error> {
        let Some(expected) = expected else {
            return Ok(None);
        };
        let (Some(Constructor::AdtArgs(owner, _)), Some(Constructor::AdtArgs(target, _))) = (
            self.contextual_head(node, actual)?,
            self.contextual_head(node, expected)?,
        ) else {
            return Ok(Some(expected));
        };
        if owner != target {
            return Ok(Some(expected));
        }
        let mut term = expected;
        let mut parents = Vec::new();
        loop {
            self.state.reference_steps += 1;
            if self.state.reference_steps > 1_000_000 {
                return Err(self.error(node, "unsizing constructor context work budget exceeded"));
            }
            match self.contextual_head(node, term)? {
                Some(Constructor::AdtArgs(item, _)) => {
                    // Early constant bodies can precede a declaration's fields.
                    // A missing template supplies no hint and is never cached.
                    if self.facts.aggregates.get(item).is_none() {
                        return Ok(Some(expected));
                    }
                    let slot = self
                        .facts
                        .unsizing_rules
                        .get_or_insert_with(Default::default)
                        .parameter(&self.facts.types, &self.facts.aggregates, item)
                        .map_err(|e| self.error(node, e))?;
                    let Some(slot) = slot else {
                        // Projection inputs cannot be inferred backwards from an
                        // associated output. Keep explicitly written source markers;
                        // the enclosing coercion will prove their actual field relation.
                        return self.projected_constructor_context(node, actual, expected, item);
                    };
                    let arguments = self
                        .state
                        .infer
                        .item_arguments(&self.facts.types, term)
                        .map_err(|e| self.error(node, e))?;
                    let Some(Argument::Type(child)) = arguments.get(slot as usize) else {
                        return Err(self.error(node, "invalid constructor tail argument"));
                    };
                    term = *child;
                    parents.push((item, arguments, slot as usize));
                }
                Some(Constructor::Slice) if !parents.is_empty() => break,
                _ => return Ok(Some(expected)),
            }
        }
        // Only the length is unknown. Retaining the element context is necessary
        // for nested function-item coercions and empty array inference.
        let element = self.child(node, term, 0)?;
        let length = self
            .state
            .infer
            .const_variable(&self.facts.types, TypeId::primitive(Primitive::Usize))
            .map_err(|e| self.error(node, e))?;
        term = self
            .state
            .infer
            .construct(Constructor::ArrayInfer(length), &[element])
            .map_err(|e| self.error(node, e))?;
        for (item, mut arguments, slot) in parents.into_iter().rev() {
            arguments[slot] = Argument::Type(term);
            term = self
                .state
                .infer
                .nominal(item, &arguments)
                .map_err(|e| self.error(node, e))?;
        }
        Ok(Some(term))
    }
    fn projected_constructor_context(
        &mut self,
        node: NodeId,
        actual: TermId,
        expected: TermId,
        item: crate::types::ItemId,
    ) -> Result<Option<TermId>, Error> {
        let projected = self
            .facts
            .unsizing_rules
            .get_or_insert_with(Default::default)
            .projected_parameters(&self.facts.types, &self.facts.aggregates, item)
            .map(|slots| !slots.is_empty())
            .map_err(|e| self.error(node, e))?;
        if projected
            && self
                .state
                .infer
                .incompatible(&self.facts.types, actual, expected)
                .map_err(|e| self.error(node, e))?
        {
            Ok(None)
        } else {
            Ok(Some(expected))
        }
    }
}
