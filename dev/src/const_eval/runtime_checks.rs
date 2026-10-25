//! CTFE consumes a fresh checked call; the runtime setting never affects its value.
use super::{
    Frame, Machine, Requirement, State,
    code::{Catalog, Code},
    executed_calls::Answer,
};
use crate::{
    sema::{
        compiler_items::RuntimeCheckSelection,
        model::{Declarations, Scalar},
    },
    types::{Primitive, instance::Instance},
};

fn validate(
    facts: &Declarations,
    body: &Code,
    frame: &Frame,
    position: u32,
    target: Instance,
) -> Result<Option<RuntimeCheckSelection>, &'static str> {
    let Some(catalog) = facts.source_bindings.as_deref() else {
        return Ok(None);
    };
    let Some(selection) = catalog.runtime_check_selection(facts, target)? else {
        return Ok(None);
    };
    catalog.runtime_check_binding(body.source)?;
    super::reached_layout::checked_call(
        facts,
        body,
        frame,
        position,
        target,
        (selection.definition, Primitive::Bool),
    )?;
    Ok(Some(selection))
}

impl State {
    pub(crate) fn source_runtime_check_call(
        &self,
        facts: &Declarations,
        catalog: &Catalog,
        target: Instance,
    ) -> Result<Option<RuntimeCheckSelection>, &'static str> {
        let frame = self
            .frames
            .last()
            .ok_or("source ub_checks requires a reached caller")?;
        catalog.validate_entry(frame.body)?;
        let body = catalog
            .code(frame.body)
            .ok_or("source ub_checks caller code disappeared")?;
        validate(facts, body, frame, frame.next, target)
    }
}

impl Machine<'_, '_> {
    pub(super) fn source_runtime_check(
        &mut self,
        body: &Code,
        frame: &Frame,
        position: u32,
        target: Instance,
    ) -> Result<Option<Answer>, super::Failure> {
        let Some(catalog) = self.evaluator.facts().source_bindings.as_deref() else {
            return Ok(None);
        };
        let Some(crate::types::Kind::FunctionItem { item, .. }) =
            self.evaluator.facts().types.kind(target.ty())
        else {
            return Ok(None);
        };
        if catalog.intrinsic_kind(crate::resolve::model::DefId(item.0))?
            != Some(crate::support::bindings::SourceIntrinsic::UbChecks)
        {
            return Ok(None);
        }
        self.charge(32)?;
        self.bodies.validate_entry(frame.body)?;
        if self
            .bodies
            .code(frame.body)
            .is_none_or(|actual| !std::ptr::eq(actual, body))
        {
            return Err("source ub_checks belongs to another actual caller code".into());
        }
        let selection = validate(self.evaluator.facts(), body, frame, position, target)?
            .ok_or("source ub_checks lost its actual checked call")?;
        if selection.fallback {
            let Some(index) = self
                .bodies
                .functions
                .get(&target)
                .copied()
                .or_else(|| self.bodies.definitions.get(&selection.definition).copied())
            else {
                self.waiting = Some(Requirement::FunctionBody(target));
                return Ok(Some(Answer::Pending));
            };
            self.validate_invoked_entry(index, target)?;
            let facts = self.evaluator.facts();
            let function = facts
                .function_indices
                .get(&selection.definition)
                .and_then(|&i| facts.functions.get(i))
                .ok_or("source ub_checks fallback declaration disappeared")?;
            if self
                .bodies
                .code(index)
                .is_none_or(|code| code.source.source() != function.body)
            {
                return Err("source ub_checks fallback changed its actual body origin".into());
            }
        }
        self.work.calls += 1;
        Ok(Some(Answer::Ready(super::Constant::scalar(
            Scalar::Boolean(true),
        ))))
    }
}

#[cfg(test)]
#[path = "../../tests/invariants/ub_calls.rs"]
mod tests;
