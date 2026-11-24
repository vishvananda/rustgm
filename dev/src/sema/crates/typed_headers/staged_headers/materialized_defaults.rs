//! Reconstruct a checked default in the store that owns its rigid formal.
use super::{Error, Headers, error, s};
use crate::types::{
    Store, TypeId,
    arguments::{Argument, ConstId, Constant},
};

pub(super) fn materialize(
    rows: s::ValidatedView<'_>,
    slot: &s::ParameterSlot,
    argument: Argument,
    value: &s::ConstDefaultValue,
    store: &mut Store,
    budget: &mut Headers,
) -> Result<Argument, Error> {
    let Argument::Const(formal) = argument else {
        return Err(error("source const default lost its actual formal"));
    };
    let ty = store
        .const_type(formal)
        .ok_or_else(|| error("source const default lost its canonical type"))?;
    let s::ParameterKind::Const { ty: raw_ty } = slot.kind else {
        return Err(error("source const default names another kind of formal"));
    };
    match value {
        s::ConstDefaultValue::Scalar(bits) => scalar(store, ty, *bits, budget).map(Argument::Const),
        s::ConstDefaultValue::Record(values) => {
            let s::ConstFormalType::Nominal(owner) =
                s::const_formal_type(&rows.types, raw_ty).map_err(error)?
            else {
                return Err(error("source record const default lost its nominal type"));
            };
            let Some(s::HeaderRef::Nominal(index)) = rows.header(owner).map_err(error)? else {
                return Err(error("source const default lost its exact fields"));
            };
            let fields = s::const_defaults::fields(
                &rows.nominals[index as usize],
                &rows.declarations,
                &rows.fields,
            )
            .map_err(error)?;
            budget
                .pay(
                    fields.len(),
                    fields.len().saturating_mul(size_of::<ConstId>()),
                )
                .map_err(error)?;
            let mut children = Vec::with_capacity(fields.len());
            for (field, bits) in fields.iter().zip(values) {
                let ty = s::const_parameter_type(&rows.types, field.ty).map_err(error)?;
                let ty = crate::sema::crates::scalar(ty);
                children.push(scalar(store, ty, *bits, budget)?);
            }
            store
                .intern_aggregate_constant(ty, None, &children, |work, bytes| {
                    budget.pay(work, bytes)
                })
                .map(Argument::Const)
                .map_err(error)
        }
    }
}

fn scalar(
    store: &mut Store,
    ty: TypeId,
    bits: u128,
    budget: &mut Headers,
) -> Result<ConstId, Error> {
    store
        .intern_constant_charged(Constant { ty, bits }, |work, bytes| budget.pay(work, bytes))
        .map_err(error)
}
