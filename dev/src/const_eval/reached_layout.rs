//! Each reached source intrinsic call retains its own checked caller authority.
use super::{
    Frame, Machine, State,
    code::{Catalog, Code},
};
use crate::{
    body::ir::{Action, Rvalue, ValueId},
    sema::{compiler_items::Selection, model::Declarations},
    types::{Kind, Primitive, instance::Instance},
};

/// A per-call witness. No caller permission is retained in the layout result cache.
pub(crate) struct Call {
    instance: Instance,
    selection: Selection,
}
impl Call {
    pub(crate) fn selection(&self) -> Selection {
        self.selection
    }
    pub(super) fn instance(&self) -> Instance {
        self.instance
    }
}

impl State {
    pub(crate) fn source_layout_call(
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
            .ok_or("source layout caller code is missing")?;
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
    let Some(selection) = catalog.layout_selection(facts, target)? else {
        return Ok(None);
    };
    checked_call(
        facts,
        body,
        frame,
        position,
        target,
        (selection.definition, Primitive::Usize),
    )?;
    Ok(Some(Call {
        instance: target,
        selection,
    }))
}

/// Shared structural checks do not retain a caller token or infer const permission.
pub(super) fn checked_call(
    facts: &Declarations,
    body: &Code,
    frame: &Frame,
    position: u32,
    target: Instance,
    contract: (crate::resolve::model::DefId, Primitive),
) -> Result<(), &'static str> {
    let block = body
        .blocks
        .get(frame.block.0 as usize)
        .ok_or("source intrinsic call has no checked block")?;
    if position < block.start || position >= block.end {
        return Err("source intrinsic call is outside the current checked block");
    }
    let Some(Action::Evaluate(destination)) =
        body.statements.get(position as usize).map(|s| s.action)
    else {
        return Err("source intrinsic preparation is not a direct checked call");
    };
    let value = body
        .values
        .get(destination.0 as usize)
        .ok_or("source intrinsic call has no destination")?;
    let Rvalue::Call {
        signature,
        callee,
        arguments,
    } = value.kind
    else {
        return Err("source intrinsic preparation is not a function call");
    };
    let Some(Kind::FunctionItem {
        signature: selected_signature,
        ..
    }) = facts.types.kind(target.ty())
    else {
        return Err("source intrinsic target lost its item identity");
    };
    if !body.const_context
        || !facts.is_const_function(contract.0)
        || signature != selected_signature
        || value.ty != crate::types::TypeId::primitive(contract.1)
        || body
            .values
            .get(callee.0 as usize)
            .is_none_or(|v| v.ty != target.ty())
        || !body
            .arguments
            .get(arguments.start as usize..arguments.end as usize)
            .is_some_and(<[ValueId]>::is_empty)
        || Machine::value(frame, callee)
            .map_err(|_| "source intrinsic call has no evaluated callee")?
            .as_function()
            != Some(target)
    {
        return Err("source intrinsic call differs from its checked const contract");
    }
    facts
        .source_bindings
        .as_deref()
        .ok_or("source intrinsic catalog disappeared")?
        .check_const_call(contract.0)?;
    Ok(())
}

#[cfg(test)]
#[path = "../../tests/invariants/provider_calls.rs"]
mod tests;
