//! Feed inherited callable inputs into ordinary closure inference before fallback.
use super::{Checker, Constructor, Error, NodeId, TermId};
use crate::types::{
    infer::{Instantiation, arguments::Argument},
    substitute,
};

impl Checker<'_> {
    /// Alias caller slots before introducing query-local quantifiers. Ordinary
    /// nonclosure conversions keep their original lifetime obligations and owner.
    pub(super) fn scoped_object_inputs(
        &mut self,
        node: NodeId,
        mut inputs: [TermId; 2],
    ) -> Result<[TermId; 2], Error> {
        if self.state.body_binders == 0 {
            return Ok(inputs);
        }
        let target = self
            .head(node, inputs[1])?
            .ok_or_else(|| self.error(node, "object conversion has no target constructor"))?;
        let count = self
            .state
            .infer
            .binder_regions(target)
            .map_err(|e| self.error(node, e))?
            .ok_or_else(|| self.error(node, "object conversion has no target binder"))?;
        // Inherited callable demands retain even an empty declared quantifier.
        if count == 0 && !matches!(self.head(node, inputs[0])?, Some(Constructor::Closure(..))) {
            return Ok(inputs);
        }
        let mut mapping = crate::types::infer::RegionMapping::default();
        for input in &mut inputs {
            if self
                .state
                .infer
                .contains_escaping_regions(&self.facts.types, *input)
                .map_err(|e| self.error(node, e))?
            {
                *input = self.alias_inferred_mapped(node, *input, &mut mapping)?;
            }
        }
        Ok(inputs)
    }

    pub(super) fn object_callable_demands(
        &mut self,
        node: NodeId,
        concrete: TermId,
        target: TermId,
    ) -> Result<(), Error> {
        if !matches!(self.head(node, concrete)?, Some(Constructor::Closure(..))) {
            return Ok(());
        }
        let target = match self.state.infer.resolved(&mut self.facts.types, target) {
            Ok(ty) => ty,
            Err(
                crate::types::infer::Error::Unresolved | crate::types::infer::Error::Normalization,
            ) => return self.pending_object_demands(node, concrete, target),
            Err(e) => return Err(self.error(node, e)),
        };
        let (template, steps) =
            crate::sema::traits::object_bounds::demands::get(self.facts, target)
                .map_err(|e| self.error(node, e))?;
        self.state
            .infer
            .account_work(steps)
            .map_err(|e| self.error(node, e))?;
        let Some(template) = template else {
            return Ok(());
        };
        self.apply_object_demands(
            node,
            &template,
            &[(template.parameter, Argument::Type(concrete))],
        )
    }

    fn pending_object_demands(
        &mut self,
        node: NodeId,
        concrete: TermId,
        target: TermId,
    ) -> Result<(), Error> {
        let Some(principal) = self
            .state
            .infer
            .object_principal(&self.facts.types, target)
            .map_err(|e| self.error(node, e))?
        else {
            return Ok(());
        };
        let mut prepared =
            crate::sema::traits::object_bounds::demands::pending::get(self.facts, &principal)
                .map_err(|e| self.error(node, e))?;
        self.state
            .infer
            .account_work(prepared.steps)
            .map_err(|e| self.error(node, e))?;
        prepared
            .bindings
            .push((prepared.template.parameter, Argument::Type(concrete)));
        self.apply_object_demands(node, &prepared.template, &prepared.bindings)
    }

    fn apply_object_demands(
        &mut self,
        node: NodeId,
        template: &crate::sema::traits::object_bounds::demands::Template,
        bindings: &[(crate::types::parameters::ParameterId, Argument)],
    ) -> Result<(), Error> {
        let mut inference = Instantiation::new(&mut self.state.infer, &self.facts.types, bindings)
            .map_err(|e| self.error(node, e))?;
        let mut regions = substitute::Bound::new(Vec::new());
        for &clause in template.clauses.iter() {
            self.instantiate_bound_obligation(node, clause, None, &mut regions, &mut inference)?;
        }
        self.state
            .infer
            .account_work(regions.steps)
            .map_err(|e| self.error(node, e))?;
        Ok(())
    }
}
