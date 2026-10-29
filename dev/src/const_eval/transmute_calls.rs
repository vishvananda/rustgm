//! Source transmute consumes one actual checked unsafe const call and its value.
use super::{
    Failure, Frame, Machine, State,
    code::{Catalog, Code},
    executed_calls::Answer,
};
use crate::{
    body::ir::ValueId,
    resolve::expansion::Report,
    sema::{compiler_items::TransmuteSelection, model::Declarations},
    types::{Kind, instance::Instance},
};

pub(crate) struct Call {
    selection: TransmuteSelection,
    input: ValueId,
}
impl Call {
    pub(crate) fn selection(&self) -> &TransmuteSelection {
        &self.selection
    }
}
fn validate(
    facts: &Declarations,
    body: &Code,
    frame: &Frame,
    position: u32,
    target: Instance,
    selected: TransmuteSelection,
) -> Result<Call, &'static str> {
    let arguments = super::reached_layout::checked_call_arguments_with_signature(
        facts,
        body,
        frame,
        position,
        target,
        (selected.definition, selected.destination),
        |actual, public| {
            public == selected.signature && selected.opened_signature_matches(&facts.types, actual)
        },
    )?;
    facts
        .source_bindings
        .as_deref()
        .ok_or("source transmute catalog disappeared")?
        .check_const_call(selected.definition)?;
    let [input] = body
        .arguments
        .get(arguments.start as usize..arguments.end as usize)
        .ok_or("source transmute lost its checked input")?
    else {
        return Err("source transmute requires exactly one actual value input");
    };
    if body
        .values
        .get(input.0 as usize)
        .is_none_or(|value| value.ty != selected.source)
        || !matches!(facts.types.kind(target.ty()),Some(Kind::FunctionItem{signature,..}) if signature==selected.signature)
    {
        return Err("source transmute changed its checked input or signature");
    }
    Ok(Call {
        selection: selected,
        input: *input,
    })
}
impl State {
    pub(crate) fn source_transmute_call(
        &self,
        report: &Report,
        facts: &Declarations,
        catalog: &Catalog,
        target: Instance,
    ) -> Result<Option<Call>, &'static str> {
        let frame = self
            .frames
            .last()
            .ok_or("source transmute requires its reached caller")?;
        catalog.validate_entry(frame.body)?;
        let body = catalog
            .code(frame.body)
            .ok_or("source transmute caller code disappeared")?;
        let Some(source) = facts.source_bindings.as_deref() else {
            return Ok(None);
        };
        let Some(selected) = source.reinterpretation_selection(facts, target, body.source)? else {
            return Ok(None);
        };
        source.require_transmute_report(report, facts, &selected)?;
        validate(facts, body, frame, frame.next, target, selected).map(Some)
    }
}
impl Machine<'_, '_> {
    pub(super) fn source_transmute(
        &mut self,
        body: &Code,
        frame: &Frame,
        position: u32,
        target: Instance,
    ) -> Result<Option<Answer>, Failure> {
        let Some(selected) = self
            .evaluator
            .checked_source_transmute(target, body.source)?
        else {
            return Ok(None);
        };
        self.charge(32)?;
        self.bodies.validate_entry(frame.body)?;
        if self
            .bodies
            .code(frame.body)
            .is_none_or(|current| !std::ptr::eq(current, body))
        {
            return Err("source transmute belongs to another actual caller Code".into());
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
            .get(call.selection.source)
            .is_none()
            || self
                .evaluator
                .facts()
                .layouts
                .get(call.selection.destination)
                .is_none()
        {
            self.waiting = Some(super::Requirement::FunctionBody(target));
            return Ok(Some(Answer::Pending));
        }
        let input = Self::value(frame, call.input)?;
        let value =
            self.transmute_value(call.selection.source, call.selection.destination, input)?;
        match value {
            super::memory::Read::Value(value) => {
                self.work.calls += 1;
                Ok(Some(Answer::Ready(value)))
            }
            super::memory::Read::Pending => Ok(Some(Answer::Pending)),
        }
    }
}
