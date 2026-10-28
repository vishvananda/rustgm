//! Field operands stay with each reached call, while concrete layout is shared.
use super::{
    Constant, Failure, Frame, Machine, State,
    code::{Catalog, Code},
    executed_calls::Answer,
};
use crate::{
    sema::{
        compiler_items::OffsetSelection,
        model::{Declarations, Scalar},
    },
    types::{Primitive, TypeId, instance::Instance},
};

pub(crate) struct Call {
    selection: OffsetSelection,
    variant: u32,
    field: u32,
}
impl Call {
    pub(crate) fn definition(&self) -> crate::resolve::model::DefId {
        self.selection.definition
    }
    pub(crate) fn element(&self) -> TypeId {
        self.selection.element
    }
}

impl State {
    pub(crate) fn source_offset_call(
        &self,
        facts: &Declarations,
        catalog: &Catalog,
        target: Instance,
    ) -> Result<Option<Call>, &'static str> {
        if facts.source_bindings.is_none() {
            return Ok(None);
        }
        let frame = self
            .frames
            .last()
            .ok_or("source offset_of has no reached caller")?;
        catalog.validate_entry(frame.body)?;
        let body = catalog
            .code(frame.body)
            .ok_or("source offset_of caller code disappeared")?;
        validate(facts, body, frame, frame.next, target)
    }
}

fn validate(
    facts: &Declarations,
    body: &Code,
    frame: &Frame,
    position: u32,
    target: Instance,
) -> Result<Option<Call>, &'static str> {
    let Some(catalog) = facts.source_bindings.as_deref() else {
        return Ok(None);
    };
    let Some(selection) = catalog.offset_selection(facts, target)? else {
        return Ok(None);
    };
    let arguments = super::reached_layout::checked_call_arguments(
        facts,
        body,
        frame,
        position,
        target,
        (selection.definition, TypeId::primitive(Primitive::Usize)),
    )?;
    catalog.check_const_call(selection.definition)?;
    let [variant, field] = body
        .arguments
        .get(arguments.start as usize..arguments.end as usize)
        .ok_or("source offset_of lost its argument range")?
    else {
        return Err("source offset_of requires two actual evaluated u32 operands");
    };
    let operand = |id: super::ValueId| {
        if body
            .values
            .get(id.0 as usize)
            .is_none_or(|v| v.ty != TypeId::primitive(Primitive::U32))
        {
            return Err("source offset_of operand differs from its u32 signature");
        }
        let Scalar::Integer(bits) = Machine::value(frame, id)
            .and_then(Constant::as_scalar)
            .map_err(|_| "source offset_of operand is not evaluated")?
        else {
            return Err("source offset_of operand is not an integer");
        };
        u32::try_from(bits).map_err(|_| "source offset_of ordinal exceeds u32")
    };
    Ok(Some(Call {
        selection,
        variant: operand(*variant)?,
        field: operand(*field)?,
    }))
}

impl Machine<'_, '_> {
    pub(super) fn source_offset(
        &mut self,
        body: &Code,
        frame: &Frame,
        position: u32,
        target: Instance,
    ) -> Result<Option<Answer>, Failure> {
        let Some(call) = validate(self.evaluator.facts(), body, frame, position, target)? else {
            return Ok(None);
        };
        self.charge(64)?;
        self.bodies.validate_entry(frame.body)?;
        if self
            .bodies
            .code(frame.body)
            .is_none_or(|actual| !std::ptr::eq(actual, body))
        {
            return Err("source offset_of belongs to another reached caller".into());
        }
        self.evaluator
            .check_source_offset_report(call.definition())?;
        let facts = self.evaluator.facts();
        let Some(layout) = facts.layouts.get(call.element()) else {
            self.waiting = Some(super::Requirement::FunctionBody(target));
            return Ok(Some(Answer::Pending));
        };
        let variant = facts
            .layouts
            .variants(layout)
            .get(call.variant as usize)
            .ok_or("source offset_of variant ordinal is out of bounds")?;
        let offset = *facts
            .layouts
            .offsets(variant)
            .get(call.field as usize)
            .ok_or("source offset_of field ordinal is out of bounds")?;
        self.work.calls += 1;
        Ok(Some(Answer::Ready(Constant::scalar(Scalar::Integer(
            u128::from(offset),
        )))))
    }
}

#[cfg(test)]
#[path = "../../tests/invariants/queried_operands.rs"]
mod tests;
