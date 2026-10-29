//! Regionless v7 free applications reuse actual slots and paid binder substitution.
use super::super::Demand;
use super::{
    Argument, Error, Headers, ItemId, Kind, Ordinary, Port, Store, TypeId, Worklist, error, g,
    paid, s,
};
use crate::types::substitute::Bound;

pub(super) fn prepare(
    port: &mut Ordinary<'_, '_>,
    store: &mut Store,
    cursor: &mut Worklist<Vec<Option<TypeId>>>,
    row: s::Type,
    function: s::FunctionHeader,
    actuals: &[s::Argument],
    budget: &mut Headers,
) -> Result<TypeId, Error> {
    // The immutable descriptor already proved complete actual/formal arity and
    // scope ownership with its single HeaderIndex/DomainIndex construction.
    budget.charge(3)?;
    if function.parent.is_some()
        || function.scope != function.declaration
        || function.regions.total != 0
    {
        return Err(error(
            "source generic application requires its exact free regionless declaration",
        ));
    }
    budget.reserve(actuals.len().saturating_mul(4 * size_of::<Argument>()))?;
    let mut arguments = Vec::with_capacity(actuals.len());
    for actual in actuals {
        budget.charge(1)?;
        let s::Argument::Type(root) = actual else {
            return Err(error(
                "source generic region arguments require their wider profile",
            ));
        };
        arguments.push(Argument::Type(leaf(port, store, cursor, *root, budget)?));
    }
    let signature = signature(port, store, cursor, function.callable, budget)?;
    let owner = ItemId(port.definition(function.declaration, budget)?.0);
    paid(budget, |pay| {
        let arguments = store.intern_arguments_observed::<true>(&arguments, &pay)?;
        let mut fold = Bound::joint_charged(store, owner, arguments, &pay)?;
        let signature = fold.signature_charged(store, signature, 0, &pay)?;
        if matches!(row, s::Type::ApplicationSignature(_)) {
            return Ok(signature);
        }
        store.intern_observed::<true>(
            Kind::FunctionItem {
                item: owner,
                arguments,
                signature,
            },
            &pay,
        )
    })
}
fn leaf(
    port: &mut Ordinary<'_, '_>,
    store: &mut Store,
    cursor: &mut Worklist<Vec<Option<TypeId>>>,
    root: g::TypeRef,
    budget: &mut Headers,
) -> Result<TypeId, Error> {
    budget.charge(1)?;
    if !matches!(
        port.rows().types.get(root.0 as usize),
        Some(s::Type::Scalar(_) | s::Type::Parameter(_))
    ) {
        return Err(error(
            "source generic application requires its scalar or actual parameter roots",
        ));
    }
    match cursor.demand(port, store, root, budget)? {
        Demand::Ready(ty) => Ok(ty),
        Demand::Blocked(_) => Err(error(
            "source generic leaf lost its actual canonical mapping",
        )),
    }
}
fn signature(
    port: &mut Ordinary<'_, '_>,
    store: &mut Store,
    cursor: &mut Worklist<Vec<Option<TypeId>>>,
    raw: g::Signature,
    budget: &mut Headers,
) -> Result<TypeId, Error> {
    let rows = port.rows();
    let frame = rows
        .binders
        .get(raw.binder.0 as usize)
        .ok_or_else(|| error("source generic callable frame is absent"))?;
    if frame.count != 0 {
        return Err(error(
            "source generic callable regions require their wider profile",
        ));
    }
    let roots = raw.parameters.slice(&rows.roots).map_err(error)?;
    budget.reserve(roots.len().saturating_mul(4 * size_of::<TypeId>()))?;
    let mut parameters = Vec::with_capacity(roots.len());
    for root in roots {
        parameters.push(leaf(port, store, cursor, *root, budget)?);
    }
    let result = leaf(port, store, cursor, raw.result, budget)?;
    let convention = super::super::signature_shapes::convention(raw).map_err(error)?;
    paid(budget, |pay| {
        let parameters = store.intern_list_observed::<true>(&parameters, &pay)?;
        store.intern_observed::<true>(
            Kind::Function {
                bound_regions: 0,
                parameters,
                result,
                unsafe_: raw.unsafe_,
                convention,
            },
            &pay,
        )
    })
}
