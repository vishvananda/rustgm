//! A reached signed wrapping call changes pointer coordinates, never access authority.
use super::{
    Failure, Frame, Machine, State,
    code::{Catalog, Code},
    executed_calls::Answer,
};
use crate::{
    body::ir::ValueId,
    resolve::expansion::Report,
    sema::{compiler_items::WrappingOffsetSelection, model::Declarations},
    types::instance::Instance,
};
struct Call {
    selection: WrappingOffsetSelection,
    inputs: [ValueId; 2],
}
fn validate(
    facts: &Declarations,
    body: &Code,
    frame: &Frame,
    position: u32,
    target: Instance,
    selection: WrappingOffsetSelection,
) -> Result<Call, &'static str> {
    let arguments = super::reached_layout::checked_call_arguments_with_signature(
        facts,
        body,
        frame,
        position,
        target,
        (selection.definition(), selection.pointer()),
        |actual, public| {
            public == selection.signature()
                && selection.opened_signature_matches(&facts.types, actual)
        },
    )?;
    facts
        .source_bindings
        .as_deref()
        .ok_or("source arith_offset catalog disappeared")?
        .check_const_call(selection.definition())?;
    let Some(&[pointer, count]) = body
        .arguments
        .get(arguments.start as usize..arguments.end as usize)
    else {
        return Err("source arith_offset requires two actual evaluated inputs");
    };
    if [pointer, count]
        .into_iter()
        .zip(selection.inputs())
        .any(|(id, ty)| body.values.get(id.0 as usize).is_none_or(|v| v.ty != ty))
    {
        return Err("source arith_offset changed its checked pointer or signed count type");
    }
    Ok(Call {
        selection,
        inputs: [pointer, count],
    })
}
impl State {
    pub(crate) fn source_wrapping_offset_element(
        &self,
        report: &Report,
        facts: &Declarations,
        catalog: &Catalog,
        target: Instance,
    ) -> Result<Option<crate::types::TypeId>, &'static str> {
        let frame = self
            .frames
            .last()
            .ok_or("source arith_offset requires its reached caller")?;
        catalog.validate_entry(frame.body)?;
        let body = catalog
            .code(frame.body)
            .ok_or("source arith_offset caller code disappeared")?;
        let Some(source) = facts.source_bindings.as_deref() else {
            return Ok(None);
        };
        let Some(selected) = source.wrapping_offset_selection(facts, target, body.source)? else {
            return Ok(None);
        };
        source.require_wrapping_offset_report(report, facts, &selected)?;
        let call = validate(facts, body, frame, frame.next, target, selected)?;
        Ok(Some(call.selection.element()))
    }
}
impl Machine<'_, '_> {
    pub(super) fn source_wrapping_offset(
        &mut self,
        body: &Code,
        frame: &Frame,
        position: u32,
        target: Instance,
    ) -> Result<Option<Answer>, Failure> {
        let Some(selected) = self
            .evaluator
            .checked_source_wrapping_offset(target, body.source)?
        else {
            return Ok(None);
        };
        self.charge(64)?;
        self.bodies.validate_entry(frame.body)?;
        if self
            .bodies
            .code(frame.body)
            .is_none_or(|actual| !std::ptr::eq(actual, body))
        {
            return Err("source arith_offset belongs to another actual caller Code".into());
        }
        let call = validate(
            self.evaluator.facts(),
            body,
            frame,
            position,
            target,
            selected,
        )?;
        if self
            .evaluator
            .facts()
            .layouts
            .get(call.selection.element())
            .is_none()
        {
            self.waiting = Some(super::Requirement::FunctionBody(target));
            return Ok(Some(Answer::Pending));
        }
        // Both arguments are actual evaluated SSA values, including a ZST displacement.
        let pointer = Self::value(frame, call.inputs[0])?;
        let count = u64::try_from(super::values::integer(Self::value(frame, call.inputs[1])?)?)
            .map_err(|_| "source arith_offset signed word exceeds its actual isize width")?;
        let (layout, work) = self.evaluator.address_layout(
            call.selection.element(),
            self.limits.steps.saturating_sub(self.work.steps),
        )?;
        self.charge(work)?;
        let displacement = count.wrapping_mul(layout.size);
        self.reserve_cells(4)?;
        let mut work = 0;
        let value = self.evaluator.wrapping_byte_projection(
            pointer,
            displacement,
            call.selection.element(),
            &mut work,
            self.limits.steps.saturating_sub(self.work.steps),
        );
        self.charge(work)?;
        let value = value?;
        self.work.calls += 1;
        Ok(Some(Answer::Ready(value)))
    }
}
