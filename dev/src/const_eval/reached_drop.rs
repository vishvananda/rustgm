//! Each bool demand rechecks its exact current source intrinsic call and roles.
use super::{
    Frame, State,
    code::{Catalog, Code},
};
use crate::{
    sema::{compiler_items::NeedsDropSelection, model::Declarations},
    types::{Primitive, instance::Instance},
};
pub(crate) struct Call {
    instance: Instance,
    selection: NeedsDropSelection,
}
impl Call {
    pub(crate) fn selection(&self) -> NeedsDropSelection {
        self.selection
    }
    pub(super) fn instance(&self) -> Instance {
        self.instance
    }
}
impl State {
    pub(crate) fn source_needs_drop_call(
        &self,
        facts: &Declarations,
        catalog: &Catalog,
        instance: Instance,
    ) -> Result<Option<Call>, &'static str> {
        let frame = self
            .frames
            .last()
            .ok_or("source intrinsic preparation has no reached caller")?;
        catalog.validate_entry(frame.body)?;
        let body = catalog
            .code(frame.body)
            .ok_or("source intrinsic caller code is missing")?;
        validate(facts, body, frame, frame.next, instance)
    }
}
pub(super) fn validate(
    facts: &Declarations,
    body: &Code,
    frame: &Frame,
    position: u32,
    target: Instance,
) -> Result<Option<Call>, &'static str> {
    let Some(catalog) = facts.source_bindings.as_deref() else {
        return Ok(None);
    };
    let Some(selection) = catalog.needs_drop_selection(facts, target)? else {
        return Ok(None);
    };
    super::reached_layout::checked_call(
        facts,
        body,
        frame,
        position,
        target,
        (selection.definition, Primitive::Bool),
    )?;
    Ok(Some(Call {
        instance: target,
        selection,
    }))
}
