//! Feed inherited callable inputs into ordinary closure inference before fallback.
use super::{Checker, Constructor, Error, NodeId, TermId};
use crate::types::{
    infer::{Instantiation, arguments::Argument},
    substitute,
};

impl Checker<'_> {
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
            ) => return Ok(()),
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
        let mut inference = Instantiation::new(
            &mut self.state.infer,
            &self.facts.types,
            &[(template.parameter, Argument::Type(concrete))],
        )
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
