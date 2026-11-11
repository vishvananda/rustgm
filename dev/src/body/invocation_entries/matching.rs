//! Body entry wrappers use the shared typed closure signature matcher.
pub(super) use crate::sema::closures::matching::{
    Bindings, bind_instance, execution, execution_bindings, instance,
};
use crate::{sema::closures::Execution, types::Store};
pub(crate) fn source_bindings(
    types: &Store,
    formal: Execution,
    actual: crate::types::instance::Instance,
    original: u32,
) -> Result<super::SourceBindings, &'static str> {
    let bindings = bind_instance(types, formal.universals, formal.self_ty, actual.ty())?;
    if original > bindings.universals {
        return Err("closure source slots exceed its execution binder");
    }
    let mut slots = bindings.slots.into_iter().collect::<Vec<_>>();
    slots.sort_unstable_by_key(|&(slot, _)| slot);
    Ok(super::SourceBindings {
        original,
        slots,
        equalities: bindings.equalities,
        proof: None,
    })
}
