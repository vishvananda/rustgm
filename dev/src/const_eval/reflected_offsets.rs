//! Actual checked Field calls borrow physical coordinates from the shared layout cache.
use super::{
    Failure, Frame, Machine, State,
    code::{Catalog, Code},
    executed_calls::Answer,
    use_inputs::Inputs,
};
use crate::{
    sema::{
        compiler_items::FieldOffsetSelection,
        model::{Declarations, Scalar},
    },
    types::{Primitive, TypeId, instance::Instance},
};

impl State {
    pub(crate) fn source_field_offset_call(
        &self,
        facts: &Declarations,
        catalog: &Catalog,
        inputs: Option<&Inputs>,
        target: Instance,
    ) -> Result<Option<FieldOffsetSelection>, &'static str> {
        let frame = self
            .frames
            .last()
            .ok_or("source field_offset has no reached caller")?;
        let body = catalog
            .code(frame.body)
            .ok_or("source field_offset caller disappeared")?;
        validate(facts, catalog, inputs, body, frame, frame.next, target)
    }
}

fn validate(
    facts: &Declarations,
    catalog: &Catalog,
    inputs: Option<&Inputs>,
    body: &Code,
    frame: &Frame,
    position: u32,
    target: Instance,
) -> Result<Option<FieldOffsetSelection>, &'static str> {
    let Some(source) = facts.source_bindings.as_deref() else {
        return Ok(None);
    };
    let Some(selected) = source.field_offset_selection(facts, target, body.source)? else {
        return Ok(None);
    };
    // A manually snapshotted CFG, a different Code or a different caller's
    // retained contract cannot establish this call's ordinary F:Field proof.
    inputs
        .ok_or("source field_offset requires its proved caller inputs")?
        .checked(catalog, frame.body, body)?;
    super::reached_layout::checked_call(
        facts,
        body,
        frame,
        position,
        target,
        (selected.definition, TypeId::primitive(Primitive::Usize)),
    )?;
    Ok(Some(selected))
}

impl Machine<'_, '_> {
    pub(super) fn source_field_offset(
        &mut self,
        body: &Code,
        frame: &Frame,
        position: u32,
        target: Instance,
    ) -> Result<Option<Answer>, Failure> {
        let Some(selected) = validate(
            self.evaluator.facts(),
            &self.bodies,
            self.bodies.inputs(),
            body,
            frame,
            position,
            target,
        )?
        else {
            return Ok(None);
        };
        self.charge(64)?;
        self.evaluator.check_source_field_offset_report(&selected)?;
        let facts = self.evaluator.facts();
        let coordinates = selected.coordinates;
        let Some(layout) = facts.layouts.get(coordinates.base) else {
            self.waiting = Some(super::Requirement::FunctionBody(target));
            return Ok(Some(Answer::Pending));
        };
        let variant = facts
            .layouts
            .variants(layout)
            .get(coordinates.variant as usize)
            .ok_or("source field_offset variant ordinal is out of bounds")?;
        let offset = *facts
            .layouts
            .offsets(variant)
            .get(coordinates.field as usize)
            .ok_or("source field_offset field ordinal is out of bounds")?;
        self.work.calls += 1;
        Ok(Some(Answer::Ready(super::Constant::scalar(
            Scalar::Integer(u128::from(offset)),
        ))))
    }
}

#[cfg(test)]
#[path = "../../tests/invariants/reflection_offset_calls.rs"]
mod tests;
