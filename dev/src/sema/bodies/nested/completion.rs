//! Common interface equations and parked-state restoration are independent of body kind.
use super::{Checker, Direction, Error, InputRefresh, Work};
use crate::{sema::bodies::Dependency, types::infer::arguments::Argument};
impl Checker<'_> {
    pub(in crate::sema::bodies) fn resume_body_inputs(
        &mut self,
    ) -> Result<Option<Dependency>, Error> {
        while matches!(self.state.work.last(), Some(Work::BodyInputs(_))) {
            let Some(Work::BodyInputs(inputs)) = self.state.work.pop() else {
                return Err(self.error(self.state.body.node, "missing closure interface work"));
            };
            if let Some(dependency) = self.finish_body_inputs(*inputs)? {
                return Ok(Some(dependency));
            }
        }
        Ok(None)
    }
    pub(in crate::sema::bodies) fn finish_body_inputs(
        &mut self,
        mut inputs: InputRefresh,
    ) -> Result<Option<Dependency>, Error> {
        // Only inward roots belong to the child's complete lifetime namespace.
        // Outward roots have already closed through their exact Interface;
        // standalone public input roots must keep that callable's own binder.
        if matches!(inputs.direction, Direction::Inward) && !inputs.prepared {
            let mut mapping = crate::types::infer::RegionMapping::default();
            for (left, right) in &mut inputs.pairs {
                for argument in [left, right] {
                    if let Argument::Type(term) = argument {
                        *term = self.alias_inferred_mapped(inputs.source, *term, &mut mapping)?;
                    }
                }
            }
            inputs.prepared = true;
        }
        for &(left, right) in &inputs.pairs {
            for argument in [left, right] {
                if let Argument::Type(term) = argument {
                    self.defer_normalization(
                        inputs.source,
                        crate::types::lifetime_proof::ScopeId::ROOT,
                        term,
                    )?;
                }
            }
        }
        if let super::super::projections::Advance::Definition(dependency) =
            self.poll_projections(inputs.source)?
        {
            self.state.work.push(Work::BodyInputs(Box::new(inputs)));
            return Ok(Some(dependency));
        }
        self.state
            .infer
            .equate_input_pairs(&self.facts.types, &inputs.pairs)
            .map_err(|e| self.error(inputs.source, e))?;
        Ok(None)
    }
    pub(in crate::sema::bodies) fn continue_nested_bodies(&mut self) -> Result<bool, Error> {
        let Some(mut scheduler) = self.state.closures.take() else {
            return Ok(false);
        };
        if self.pending_body() {
            self.state.closures = Some(scheduler);
            return Ok(true);
        }
        let Some(index) = scheduler.active else {
            self.state.closure_children.clear();
            self.state.closure_cursor = 0;
            return Ok(false);
        };
        let source = scheduler.entries[index].source;
        let (active, parent) = scheduler
            .stack
            .pop()
            .ok_or_else(|| self.error(self.state.body.node, "missing closure completion parent"))?;
        self.state = *parent;
        scheduler.active = active;
        self.state.closures = Some(scheduler);
        self.defer_normalization(
            source,
            crate::types::lifetime_proof::ScopeId::ROOT,
            self.term(source)?,
        )?;
        Ok(true)
    }
}
