//! Call-local opened input/output data is retained only under its exact proved Row.
use super::{Declarations, HashMap, Instance, Kind, TypeId};
use crate::{
    body::ir::ValueId,
    types::{GenericArgs, callable::Convention},
};
#[cfg(test)]
#[path = "../../tests/invariants/opened_callbacks.rs"]
mod tests;
#[cfg(test)]
#[path = "../../tests/invariants/invocation_scaling.rs"]
mod width_tests;
#[derive(Clone, Copy)]
pub(in crate::const_eval) struct Opening {
    selector: Instance,
    constant: Instance,
    signature: TypeId,
}
pub(super) struct Rows {
    calls: HashMap<ValueId, Opening>,
}
impl Rows {
    pub(super) fn bytes(&self) -> usize {
        size_of::<Self>().saturating_add(crate::support::retained::map(&self.calls))
    }
    pub(super) fn get(&self, call: ValueId) -> Option<Opening> {
        self.calls.get(&call).copied()
    }
}
/// The caller has already completed the actual whole `FnOnce` input/output proof.
/// This constructor creates no fresh regions or generic/trait proof witness.
pub(super) fn push(
    facts: &mut Declarations,
    rows: &mut Option<Box<Rows>>,
    call: ValueId,
    selection: crate::sema::compiler_items::Select,
    bytes: usize,
    work: &mut usize,
) -> Result<(), &'static str> {
    *work = work.saturating_add(1);
    let Some(Kind::FunctionItem {
        signature,
        arguments: GenericArgs::EMPTY,
        ..
    }) = facts.types.kind(selection.types[1])
    else {
        return Ok(());
    };
    *work = work.saturating_add(1);
    let Some(Kind::Function {
        bound_regions,
        unsafe_: false,
        convention: Convention::RUST,
        ..
    }) = facts.types.kind(signature)
    else {
        return Ok(());
    };
    if bound_regions == 0 {
        return Ok(());
    }
    let Some(Kind::FunctionItem { item, .. }) = facts.types.kind(selection.types[1]) else {
        return Ok(());
    };
    let definition = crate::resolve::model::DefId(item.0);
    *work = work.saturating_add(3);
    let catalog = facts
        .source_bindings
        .as_deref()
        .ok_or("reference selector source catalog disappeared")?;
    if facts.associated.owner(definition).is_some()
        || catalog.intrinsic_kind(definition)?.is_some()
        || catalog.requires_const_payment(definition)?
    {
        return Ok(());
    }
    *work = work.saturating_add(1);
    if facts
        .function_header(definition)?
        .is_none_or(|header| header.declaration.source().is_none())
    {
        return Ok(());
    }
    let Some(Kind::Tuple(parameters)) = facts.types.kind(selection.types[0]) else {
        return Err("reference selector lost its exact ARG tuple");
    };
    *work = work.saturating_add(
        facts
            .types
            .list(parameters)
            .ok_or("reference selector lost its tuple fields")?
            .len()
            .saturating_add(1),
    );
    let signature = facts.types.intern(Kind::Function {
        bound_regions: 0,
        unsafe_: false,
        convention: Convention::RUST,
        parameters,
        result: selection.types[3],
    })?;
    let count = rows.as_deref().map_or(0, |rows| rows.calls.len());
    if size_of::<Rows>().saturating_add(crate::support::retained::new_map::<ValueId, Opening>(
        count + 1,
    )) > bytes
    {
        return Err("reference selector descriptor storage exceeds input budget");
    }
    let rows = rows.get_or_insert_with(|| {
        Box::new(Rows {
            calls: HashMap::default(),
        })
    });
    rows.calls
        .try_reserve(1)
        .map_err(|_| "reference selector descriptor allocation failed")?;
    if rows.bytes() > bytes {
        return Err("reference selector descriptor payload exceeds input budget");
    }
    rows.calls.insert(
        call,
        Opening {
            selector: selection.instance,
            constant: Instance::new(&facts.types, selection.types[1])?,
            signature,
        },
    );
    Ok(())
}
impl Opening {
    pub(in crate::const_eval) fn validate(
        self,
        facts: &Declarations,
        selection: crate::sema::compiler_items::Select,
        actual: Instance,
    ) -> Result<TypeId, &'static str> {
        if self.selector != selection.instance
            || self.constant.ty() != selection.types[1]
            || self.constant != actual
        {
            return Err("reference selector descriptor changed its full selector or F instance");
        }
        let Some(Kind::FunctionItem {
            arguments: GenericArgs::EMPTY,
            signature: public,
            ..
        }) = facts.types.kind(actual.ty())
        else {
            return Err("reference selector requires its complete nongeneric function item");
        };
        if !matches!(facts.types.kind(public), Some(Kind::Function { bound_regions, unsafe_: false, convention: Convention::RUST, .. }) if bound_regions != 0)
            || facts.types.needs_region_context(actual.ty()) != Some(false)
        {
            return Err("reference selector changed its actual internally bound signature");
        }
        let Some(Kind::FunctionItem { item, .. }) = facts.types.kind(actual.ty()) else {
            return Err("reference selector lost F item");
        };
        let definition = crate::resolve::model::DefId(item.0);
        let catalog = facts
            .source_bindings
            .as_deref()
            .ok_or("reference selector catalog disappeared")?;
        if !facts.is_const_function(definition)
            || facts.associated.owner(definition).is_some()
            || catalog.intrinsic_kind(definition)?.is_some()
            || catalog.requires_const_payment(definition)?
            || facts
                .function_header(definition)?
                .is_none_or(|header| header.declaration.source().is_none())
        {
            return Err(
                "reference selector requires its ordinary source const function declaration",
            );
        }
        let Some(Kind::Tuple(fields)) = facts.types.kind(selection.types[0]) else {
            return Err("reference selector ARG is not a tuple");
        };
        if !matches!(facts.types.kind(self.signature), Some(Kind::Function { bound_regions: 0, parameters, result, unsafe_: false, convention: Convention::RUST }) if parameters == fields && result == selection.types[3])
        {
            return Err("reference selector changed its whole opened inputs and output");
        }
        Ok(self.signature)
    }
}
