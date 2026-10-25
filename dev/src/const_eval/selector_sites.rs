//! Reached selector identity borrows checked code; evaluated F is already mapped.
use super::{
    Frame, Machine,
    code::{Catalog, Code},
};
use crate::{
    body::ir::{Action, Rvalue, ValueId},
    sema::{
        compiler_items::{Select, source_select},
        model::Declarations,
        values::{Reader, View},
    },
    types::{Kind, TypeId, callable::Convention, instance::Instance},
};

#[derive(Clone, Copy)]
pub(super) enum Phase {
    Suspended,
    Executing,
}
impl Phase {
    pub(super) fn position(self, frame: &Frame) -> Result<u32, &'static str> {
        match self {
            Self::Suspended => Ok(frame.next),
            Self::Executing => frame
                .next
                .checked_sub(1)
                .ok_or("source selector has no executing statement"),
        }
    }
}

/// This non-Copy witness cannot outlive or replace the actual caller snapshot.
pub(super) struct Call<'code> {
    code: &'code Code,
    frame: &'code Frame,
    value: ValueId,
    selection: Select,
    callback: Instance,
}
impl Call<'_> {
    pub(super) fn frame(&self) -> &Frame {
        self.frame
    }
    pub(super) fn selector(&self) -> Instance {
        self.selection.instance
    }
    pub(super) fn callback(&self) -> Instance {
        self.callback
    }
    pub(super) fn value(&self) -> ValueId {
        self.value
    }
    pub(super) fn operands(&self) -> Result<[ValueId; 3], &'static str> {
        let Rvalue::Call { arguments, .. } = self
            .code
            .values
            .get(self.value.0 as usize)
            .ok_or("source selector witness lost its actual Call")?
            .kind
        else {
            return Err("source selector witness is not a Call");
        };
        let Some(&[arg, constant, runtime]) = self
            .code
            .arguments
            .get(arguments.start as usize..arguments.end as usize)
        else {
            return Err("source selector witness lost its three operands");
        };
        Ok([arg, constant, runtime])
    }
    pub(super) fn tuple(&self) -> Result<ValueId, &'static str> {
        Ok(self.operands()?[0])
    }
    pub(super) fn width(&self, facts: &Declarations) -> Result<usize, &'static str> {
        let Some(Kind::Tuple(list)) = facts.types.kind(self.selection.types[0]) else {
            return Err("source selector witness lost its logical tuple");
        };
        Ok(facts
            .types
            .list(list)
            .ok_or("source selector witness lost its tuple fields")?
            .len())
    }
}

/// Only this actual checked Row's whole opening may admit this reached call.
/// The borrowed frame and call identity cannot authorize another invocation.
struct ReferenceOpening<'caller> {
    row: &'caller super::use_inputs::Row,
    frame: &'caller Frame,
    call: ValueId,
    signature: TypeId,
}
impl ReferenceOpening<'_> {
    fn matches(&self, row: &super::use_inputs::Row, frame: &Frame, call: ValueId) -> bool {
        std::ptr::eq(self.row, row) && std::ptr::eq(self.frame, frame) && self.call == call
    }
}

/// All graph properties and list equality below are constant-time typed facts.
/// Machine charges each attempted reached inspection; no per-call chain walk or
/// signature opening replaces the caller's completed source/ownership proof.
#[cfg(test)]
pub(super) fn validate<'code>(
    facts: &Declarations,
    catalog: &Catalog,
    body: &'code Code,
    frame: &'code Frame,
    phase: Phase,
) -> Result<Option<Call<'code>>, &'static str> {
    validate_source(facts, catalog, None, body, frame, phase)
}

pub(super) fn validate_source<'code>(
    facts: &Declarations,
    catalog: &Catalog,
    inputs: Option<&super::use_inputs::Inputs>,
    body: &'code Code,
    frame: &'code Frame,
    phase: Phase,
) -> Result<Option<Call<'code>>, &'static str> {
    let position = phase.position(frame)?;
    let Some(block) = body.blocks.get(frame.block.0 as usize) else {
        return Err("source selector has no current checked block");
    };
    if position < block.start || position >= block.end {
        return Err("source selector is outside its current checked block");
    }
    let Some(Action::Evaluate(id)) = body.statements.get(position as usize).map(|s| s.action)
    else {
        return Ok(None);
    };
    let value = body
        .values
        .get(id.0 as usize)
        .ok_or("source selector has no actual Call value")?;
    let Rvalue::Call {
        signature,
        callee,
        arguments,
    } = value.kind
    else {
        return Ok(None);
    };
    let ty = body
        .values
        .get(callee.0 as usize)
        .ok_or("source selector callee disappeared")?
        .ty;
    if !matches!(facts.types.kind(ty), Some(Kind::FunctionItem { .. })) {
        return Ok(None);
    }
    let raw = Instance::new(&facts.types, ty)?;
    let Some(selection) = source_select(facts, raw)? else {
        return Ok(None);
    };
    facts
        .source_bindings
        .as_deref()
        .ok_or("source selector catalog disappeared")?
        .check_const_call(selection.definition)?;
    catalog.validate_entry(frame.body)?;
    if !body.const_context
        || !std::ptr::eq(
            body,
            catalog
                .code(frame.body)
                .ok_or("source selector caller code disappeared")?,
        )
    {
        return Err("source selector belongs to another actual executable caller");
    }
    opened(facts, body, frame, callee, signature, value.ty, selection)?;
    let operands = operands(body, frame, arguments, selection)?;
    let callback = Machine::value(frame, operands[1])
        .map_err(|_| "source selector F was not evaluated")?
        .as_function()
        .ok_or("source selector F lost its actual function item")?;
    let reference = reference_opening(
        facts,
        catalog,
        inputs,
        body,
        (frame, id),
        selection,
        callback,
    )?;
    caller(
        facts,
        catalog,
        inputs,
        body,
        (frame, id),
        reference.as_ref(),
    )?;
    let width = callback_signature(
        facts,
        selection,
        callback,
        reference.as_ref().map(|r| r.signature),
    )?;
    let tuple =
        Machine::value(frame, operands[0]).map_err(|_| "source selector ARG was not evaluated")?;
    if !matches!(Reader::new(facts).view(tuple)?, View::Product { length, variant: None } if length == width as u64)
    {
        return Err("source selector ARG differs from its exact owned tuple");
    }
    let call = Call {
        code: body,
        frame,
        value: id,
        selection,
        callback,
    };
    #[cfg(test)]
    observation::record(facts, catalog, inputs, &call, reference.as_ref());
    Ok(Some(call))
}

fn reference_opening<'caller>(
    facts: &Declarations,
    catalog: &Catalog,
    inputs: Option<&'caller super::use_inputs::Inputs>,
    body: &Code,
    site: (&'caller Frame, ValueId),
    selection: Select,
    callback: Instance,
) -> Result<Option<ReferenceOpening<'caller>>, &'static str> {
    let Some(Kind::FunctionItem { signature, .. }) = facts.types.kind(callback.ty()) else {
        return Err("source selector callback lost its actual item");
    };
    if matches!(
        facts.types.kind(signature),
        Some(Kind::Function {
            bound_regions: 0,
            ..
        })
    ) {
        return Ok(None);
    }
    let inputs =
        inputs.ok_or("source const_eval_select callback binders require proved caller inputs")?;
    let row = inputs.checked(catalog, site.0.body, body)?;
    let signature = inputs
        .opening(catalog, site.0.body, body, site.1)?
        .validate(facts, selection, callback)?;
    Ok(Some(ReferenceOpening {
        row,
        frame: site.0,
        call: site.1,
        signature,
    }))
}

fn caller(
    facts: &Declarations,
    catalog: &Catalog,
    inputs: Option<&super::use_inputs::Inputs>,
    code: &Code,
    site: (&Frame, ValueId),
    reference: Option<&ReferenceOpening<'_>>,
) -> Result<(), &'static str> {
    let index = site.0.body;
    match code.owner {
        crate::body::ir::Owner::AnonymousConstant => {
            anonymous_caller(facts, catalog, inputs, code, site, reference)?;
        }
        crate::body::ir::Owner::Function { definition, .. } => {
            let function = facts
                .function_indices
                .get(&definition)
                .and_then(|&i| facts.functions.get(i))
                .ok_or("source selector caller has no actual function declaration")?;
            let (public, owned) = code
                .owner
                .signatures()
                .ok_or("source selector caller has no signature pair")?;
            if reference.is_none()
                && (!matches!(
                    facts.types.kind(public),
                    Some(Kind::Function {
                        bound_regions: 0,
                        ..
                    })
                ) || facts.types.needs_region_context(public) != Some(false))
            {
                return Err("source const_eval_select open caller signatures are not implemented");
            }
            if reference.is_none() && owned != public {
                return Err(
                    "source selector closed caller changed its public/internal signature pair",
                );
            }
            let Some(Kind::Function { result, .. }) = facts.types.kind(owned) else {
                return Err("source selector caller lost its opened execution signature");
            };
            if !facts.is_const_function(definition)
                || code.source.source() != Some(function.body.unwrap_or(function.node))
                || result != code.result
            {
                return Err("source selector caller changed its defining source/result");
            }
            if let Some(instance) = code.instance {
                if catalog.functions.get(&instance) != Some(&index)
                    || !matches!(facts.types.kind(instance.ty()), Some(Kind::FunctionItem { item, signature, .. }) if item.0 == definition.0 && signature == public)
                {
                    return Err(
                        "source selector caller changed its full instance/public signature/index",
                    );
                }
            } else if catalog.definitions.get(&definition) != Some(&index)
                || public != function.signature
                || facts.generics.has_parameters(definition)
            {
                return Err("source selector caller changed its original signature/index");
            }
        }
        crate::body::ir::Owner::Initializer { definition, kind } => {
            let initializer = facts
                .initializer_index(definition)
                .and_then(|i| facts.initializers.get(i))
                .ok_or("source selector caller has no actual initializer")?;
            let same_kind = match (kind, initializer.kind) {
                (
                    crate::sema::model::InitializerKind::Const,
                    crate::sema::model::InitializerKind::Const,
                ) => true,
                (
                    crate::sema::model::InitializerKind::Static { mutable: a },
                    crate::sema::model::InitializerKind::Static { mutable: b },
                ) => a == b,
                _ => false,
            };
            if catalog.definitions.get(&definition) != Some(&index)
                || code.source.source() != Some(initializer.semantic_source())
                || code.result != initializer.ty
                || code.instance.is_some()
                || !same_kind
            {
                return Err("source selector initializer changed its source/result/index");
            }
        }
        _ => {
            return Err(
                "source const_eval_select execution for this caller owner is not implemented",
            );
        }
    }
    Ok(())
}

fn anonymous_caller(
    facts: &Declarations,
    catalog: &Catalog,
    inputs: Option<&super::use_inputs::Inputs>,
    code: &Code,
    site: (&Frame, ValueId),
    reference: Option<&ReferenceOpening<'_>>,
) -> Result<(), &'static str> {
    let source = code
        .source
        .source()
        .ok_or("anonymous selector has no source root")?;
    catalog.expression(facts, site.0.body, source, code.result)?;
    let row = inputs
        .ok_or("anonymous selector requires its actual proved caller inputs")?
        .checked(catalog, site.0.body, code)?;
    if reference.is_some_and(|reference| !reference.matches(row, site.0, site.1)) {
        return Err("anonymous selector reference belongs to another proved row, frame or Call");
    }
    let contract = row.contract();
    if contract.universals != 0
        || !contract.parameters.is_empty()
        || contract.signature.is_some()
        || facts.types.needs_region_context(code.result) != Some(false)
    {
        return Err("anonymous selector open caller contracts are not implemented");
    }
    Ok(())
}

fn opened(
    facts: &Declarations,
    body: &Code,
    frame: &Frame,
    callee: ValueId,
    signature: TypeId,
    output: TypeId,
    selection: Select,
) -> Result<(), &'static str> {
    let Some(Kind::Function {
        bound_regions: 0,
        parameters,
        result,
        unsafe_: false,
        convention: Convention::RUST,
    }) = facts.types.kind(signature)
    else {
        return Err("source selector has no exact opened Rust Call signature");
    };
    if facts.types.list(parameters) != Some(&selection.types[..3])
        || result != selection.types[3]
        || output != result
        || Machine::value(frame, callee)
            .map_err(|_| "source selector callee was not evaluated")?
            .as_function()
            != Some(body.callable_target(selection.instance))
    {
        return Err("source selector differs from its exact evaluated Call/signature");
    }
    Ok(())
}

fn operands(
    body: &Code,
    frame: &Frame,
    arguments: crate::body::ir::ValueList,
    selection: Select,
) -> Result<[ValueId; 3], &'static str> {
    let &[arg, constant, runtime] = body
        .arguments
        .get(arguments.start as usize..arguments.end as usize)
        .ok_or("source selector has an invalid actual argument range")?
    else {
        return Err("source selector requires three actual evaluated operands");
    };
    let operands = [arg, constant, runtime];
    for (value, expected) in operands.into_iter().zip(selection.types) {
        if body
            .values
            .get(value.0 as usize)
            .is_none_or(|v| v.ty != expected)
        {
            return Err("source selector operand changed its exact logical root");
        }
        Machine::value(frame, value).map_err(|_| "source selector operand was not evaluated")?;
    }
    let runtime =
        Machine::value(frame, runtime).map_err(|_| "source selector G was not evaluated")?;
    if runtime.as_function().is_none() {
        return Err("source selector G lost its evaluated callable value");
    }
    // source_select already requires the exact completed Tuple/FnOnce roles;
    // reading those operands creates no demand for the unused G body.
    Ok(operands)
}

fn callback_signature(
    facts: &Declarations,
    selection: Select,
    actual: Instance,
    reference: Option<TypeId>,
) -> Result<usize, &'static str> {
    let Some(Kind::FunctionItem { item: logical, .. }) = facts.types.kind(selection.types[1])
    else {
        return Err("source selector logical F is not an item");
    };
    let Some(Kind::FunctionItem {
        item, signature, ..
    }) = facts.types.kind(actual.ty())
    else {
        return Err("source selector actual F is not an item");
    };
    if item != logical || !facts.is_const_function(crate::resolve::model::DefId(item.0)) {
        return Err("source selector F differs from its actual const declaration");
    }
    let Some(Kind::Function {
        parameters,
        result,
        bound_regions: 0,
        unsafe_: false,
        convention: Convention::RUST,
    }) = facts.types.kind(reference.unwrap_or(signature))
    else {
        return Err(if reference.is_some() {
            "reference selector lost its proved opened signature"
        } else {
            "source const_eval_select callback binders are not implemented"
        });
    };
    if facts.types.needs_region_context(actual.ty()) != Some(false)
        || (reference.is_none()
            && (facts.types.needs_region_context(selection.types[0]) != Some(false)
                || facts.types.needs_region_context(selection.types[3]) != Some(false)))
    {
        return Err("source const_eval_select lifetime-bearing callback entry is not implemented");
    }
    let header = facts
        .function_header(crate::resolve::model::DefId(item.0))?
        .ok_or("source selector F declaration is pending")?;
    if header.declaration.source().is_none() {
        return Err("source const_eval_select imported callback eligibility is not implemented");
    }
    if reference.is_none() {
        let shifted = facts
            .types
            .prepared_shift(selection.types[0], 1)?
            .ok_or("source selector callback tuple relocation was not prepared")?;
        let Some(Kind::Tuple(expected)) = facts.types.kind(shifted) else {
            return Err("source selector callback tuple relocation changed its shape");
        };
        if parameters != expected
            || facts.types.prepared_shift(selection.types[3], 1)? != Some(result)
        {
            return Err("source selector actual F signature differs from its exact tuple/result");
        }
    } else if result != selection.types[3] {
        return Err("reference selector changed its caller-opened result");
    }
    let width = facts
        .types
        .list(parameters)
        .ok_or("source selector callback parameters disappeared")?
        .len();
    if width != 0
        && facts
            .source_bindings
            .as_deref()
            .ok_or("source selector catalog disappeared")?
            .requires_const_payment(crate::resolve::model::DefId(item.0))?
    {
        return Err(
            "source const_eval_select nonempty conditional callback payment is not implemented",
        );
    }
    Ok(width)
}

#[cfg(test)]
#[path = "../../tests/invariants/selector_guards.rs"]
mod tests;

#[cfg(test)]
#[path = "../../tests/invariants/executed_reference_calls.rs"]
mod observation;
