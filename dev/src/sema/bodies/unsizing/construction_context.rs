//! A borrowed constructor needs a sized input hint before its outer unsizing.
use super::{Argument, Checker, Constructor, Error, NodeId, TermId};
use crate::sema::bodies::Expectation;
use crate::types::{ItemId, Primitive, TypeId};

impl Checker<'_> {
    pub(in crate::sema::bodies) fn constructor_hint(
        &mut self,
        node: NodeId,
        expectation: Expectation,
    ) -> Result<Option<TermId>, Error> {
        // A borrowed object is formed by unsizing the completed operand.
        // Its erased type supplies no field or element types to a constructor.
        if let Expectation::UnsizedRvalue(target) = expectation
            && matches!(
                self.contextual_head(node, target)?,
                Some(Constructor::Dynamic { .. })
            )
        {
            return Ok(None);
        }
        Ok(expectation.ty())
    }

    pub(in crate::sema::bodies) fn call_result_hint(
        &mut self,
        node: NodeId,
        result: TermId,
        expectation: Expectation,
        constructor: bool,
    ) -> Result<Option<TermId>, Error> {
        let Some(expected) = self.constructor_hint(node, expectation)? else {
            return Ok(None);
        };
        if constructor {
            return self.unsizing_constructor_context(node, result, Some(expected));
        }
        let (
            Some(Constructor::AdtArgs(owner, _)),
            Some(Constructor::Adt(target, _) | Constructor::AdtArgs(target, _)),
        ) = (
            self.contextual_head(node, result)?,
            self.contextual_head(node, expected)?,
        )
        else {
            return Ok(None);
        };
        if owner != target {
            return Ok(None);
        }
        self.unsizing_constructor_context(node, result, Some(expected))?
            .map(|hint| self.relax_nominal_result_hint(node, owner, result, hint))
            .transpose()
    }

    /// A checked wrapper conversion can change an open result input. Let the
    /// ordinary call obligations infer the source before its outer coercion.
    fn relax_nominal_result_hint(
        &mut self,
        node: NodeId,
        owner: ItemId,
        result: TermId,
        expected: TermId,
    ) -> Result<TermId, Error> {
        use crate::types::infer::Error as InferError;

        let Some(rules) = self.facts.unsizing_rules.as_ref() else {
            return Ok(expected);
        };
        let Some(slots) = rules
            .nominal_hint_slots(&self.facts.types, owner)
            .map_err(|error| self.error(node, error))?
        else {
            return Ok(expected);
        };
        self.state
            .infer
            .account_work(slots.len())
            .map_err(|error| self.error(node, error))?;
        let source = self
            .state
            .infer
            .item_arguments(&self.facts.types, result)
            .map_err(|error| self.error(node, error))?;
        let mut target = self
            .state
            .infer
            .item_arguments(&self.facts.types, expected)
            .map_err(|error| self.error(node, error))?;
        let mut changed = false;
        for ((&hint, from), target) in slots.iter().zip(source).zip(&mut target) {
            let (true, Argument::Type(from), Argument::Type(to)) = (hint, from, *target) else {
                continue;
            };
            if self
                .state
                .infer
                .unresolved_domain(from)
                .map_err(|error| self.error(node, error))?
                .is_none()
            {
                continue;
            }
            match self.state.infer.resolved(&mut self.facts.types, to) {
                Ok(ty)
                    if self.facts.types.contains_parameters(ty) == Some(false)
                        && self.facts.types.is_sized(ty) == Some(false) =>
                {
                    *target = Argument::Type(from);
                    changed = true;
                }
                Ok(_) | Err(InferError::Unresolved | InferError::Normalization) => {}
                Err(error) => return Err(self.error(node, error)),
            }
        }
        if changed {
            self.state
                .infer
                .nominal(owner, &target)
                .map_err(|error| self.error(node, error))
        } else {
            Ok(expected)
        }
    }

    /// Whether the expected result of a call gives a by-value formal input an
    /// unsized type, as `Box::new(array)` expected as `Box<[T]>` does. rustc
    /// then checks that argument against its formal type alone, so the result
    /// is inferred from the argument and coerced afterwards; equating the
    /// result with the expectation would wrongly make the input unsized.
    pub(in crate::sema::bodies) fn unsized_input_expectation(
        &mut self,
        node: NodeId,
        signature: TermId,
        parameters: u32,
        result: TermId,
        expected: TermId,
    ) -> Result<bool, Error> {
        if !matches!(
            self.contextual_head(node, expected)?,
            Some(Constructor::Adt(..) | Constructor::AdtArgs(..))
        ) {
            return Ok(false);
        }
        let formal = self
            .state
            .infer
            .item_arguments(&self.facts.types, result)
            .map_err(|e| self.error(node, e))?;
        let hinted = self
            .state
            .infer
            .item_arguments(&self.facts.types, expected)
            .map_err(|e| self.error(node, e))?;
        if formal.len() != hinted.len() {
            return Ok(false);
        }
        for index in 0..parameters as usize {
            let input = self.child(node, signature, index)?;
            if self.contextual_head(node, input)?.is_some() {
                continue;
            }
            let input = self
                .state
                .infer
                .root(input)
                .map_err(|e| self.error(node, e))?;
            for (slot, argument) in formal.iter().enumerate() {
                let (Argument::Type(formal), Some(Argument::Type(hint))) =
                    (*argument, hinted.get(slot).copied())
                else {
                    continue;
                };
                if self
                    .state
                    .infer
                    .root(formal)
                    .map_err(|e| self.error(node, e))?
                    == input
                    && matches!(
                        self.contextual_head(node, hint)?,
                        Some(
                            Constructor::Slice
                                | Constructor::Dynamic { .. }
                                | Constructor::Primitive(Primitive::Str)
                        )
                    )
                {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }
    pub(in crate::sema::bodies) fn unsizing_constructor_context(
        &mut self,
        node: NodeId,
        actual: TermId,
        expected: Option<TermId>,
    ) -> Result<Option<TermId>, Error> {
        let Some(expected) = expected else {
            return Ok(None);
        };
        let heads = (
            self.contextual_head(node, actual)?,
            self.contextual_head(node, expected)?,
        );
        if self.constructor_deref_destination(node, actual, expected)? {
            return Ok(None);
        }
        if matches!(heads,
            (Some(Constructor::Adt(a, _) | Constructor::AdtArgs(a, _)),
             Some(Constructor::Adt(b, _) | Constructor::AdtArgs(b, _))) if a != b)
            && self
                .report
                .package
                .requested_reborrow()
                .map_err(|e| self.error(node, e))?
        {
            // A different destination nominal can be the result of a genuine
            // CoerceShared operation. It cannot constrain the source constructor
            // backwards; the enclosing coercion must prove that operation.
            return Ok(None);
        }
        if self.fixed_constructor_inputs(node, actual, heads)? {
            return Ok(None);
        }
        let (Some(Constructor::AdtArgs(owner, _)), Some(Constructor::AdtArgs(target, _))) = heads
        else {
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
                    if self
                        .facts
                        .unsizing_rules
                        .as_ref()
                        .is_some_and(|rules| rules.sized_parameter(&self.facts.types, item))
                    {
                        return Ok(Some(expected));
                    }
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
                Some(Constructor::Slice | Constructor::Dynamic { .. }) if !parents.is_empty() => {
                    break;
                }
                _ => return Ok(Some(expected)),
            }
        }
        term = if matches!(
            self.contextual_head(node, term)?,
            Some(Constructor::Dynamic { .. })
        ) {
            // An object target supplies obligations after the sized constructor's
            // actual tail is inferred; it cannot become the stored field type.
            self.variable(node, crate::types::infer::Domain::Any)?
        } else {
            self.array_tail_context(node, term)?
        };
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
    fn fixed_constructor_inputs(
        &mut self,
        node: NodeId,
        actual: TermId,
        heads: (Option<Constructor>, Option<Constructor>),
    ) -> Result<bool, Error> {
        if !matches!(heads,
            (Some(Constructor::Adt(a, _) | Constructor::AdtArgs(a, _)),
            Some(Constructor::Adt(b, _) | Constructor::AdtArgs(b, _))) if a == b)
        {
            return Ok(false);
        }
        // Fixed inputs have nothing to infer backwards. Own fields and later
        // directed coercion still require their ordinary complete proofs.
        match self
            .state
            .infer
            .nominal_inputs_fixed(&self.facts.types, actual)
        {
            Ok(fixed) => Ok(fixed),
            Err(crate::types::infer::Error::Normalization) => Ok(true),
            Err(error) => Err(self.error(node, error)),
        }
    }
    fn array_tail_context(&mut self, node: NodeId, slice: TermId) -> Result<TermId, Error> {
        // Only the length is unknown. Retaining the element context is necessary
        // for nested function-item coercions and empty array inference.
        let element = self.child(node, slice, 0)?;
        let length = self
            .state
            .infer
            .const_variable(&self.facts.types, TypeId::primitive(Primitive::Usize))
            .map_err(|e| self.error(node, e))?;
        self.state
            .infer
            .construct(Constructor::ArrayInfer(length), &[element])
            .map_err(|e| self.error(node, e))
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
