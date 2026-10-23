//! Selected helper data is separate from checked body and caller authority.
use crate::{
    resolve::{model::DefId, unit::NodeKey},
    sema::model::Declarations,
    types::{Kind, TypeId, arguments::Argument, instance::Instance},
};
#[cfg(test)]
#[path = "../../../tests/invariants/glue_registration.rs"]
mod tests;

#[derive(Clone, Copy)]
pub(crate) struct DropGlueSelection {
    pub definition: DefId,
    pub instance: Instance,
    pub signature: TypeId,
    pub element: TypeId,
    pub source: NodeKey,
    pub empty_registry: bool,
}

/// A denial boundary, independent of cold/warm body cache state.
pub(crate) fn check_const_drop_glue(
    facts: &Declarations,
    instance: Instance,
) -> Result<(), &'static str> {
    if matches!(facts.types.kind(instance.ty()), Some(Kind::FunctionItem { item, .. })
        if super::drop_glue_member(facts, DefId(item.0))?)
    {
        return Err("source drop glue constant cleanup execution is not implemented");
    }
    Ok(())
}

pub(crate) fn source_drop_glue(
    facts: &Declarations,
    instance: Instance,
) -> Result<Option<DropGlueSelection>, &'static str> {
    let Some(Kind::FunctionItem {
        item,
        arguments,
        signature,
    }) = facts.types.kind(instance.ty())
    else {
        return Ok(None);
    };
    let definition = DefId(item.0);
    if !super::drop_glue_member(facts, definition)? {
        return Ok(None);
    }
    facts
        .source_bindings
        .as_deref()
        .ok_or("source drop glue catalog disappeared")?
        .require_needs_drop_roles()?;
    let mut arguments = facts
        .types
        .arguments(arguments)
        .ok_or("source drop glue arguments are malformed")?;
    let Some(Argument::Type(element)) = arguments.next() else {
        return Err("source drop glue requires its selected type argument");
    };
    if arguments.next().is_some()
        || facts.types.contains_parameters(element) != Some(false)
        || facts.types.contains_projections(element) != Some(false)
    {
        return Err("source drop glue requires a closed normalized type argument");
    }
    let function = facts
        .function_indices
        .get(&definition)
        .and_then(|&index| facts.functions.get(index))
        .ok_or("source drop glue declaration is missing")?;
    Ok(Some(DropGlueSelection {
        definition,
        instance,
        signature,
        element,
        source: function
            .body
            .ok_or("source drop glue requires its checked original body")?,
        empty_registry: facts.destructors.implementations().is_empty(),
    }))
}
