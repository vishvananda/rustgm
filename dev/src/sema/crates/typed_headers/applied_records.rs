//! Applied callable data comes from its real completed ordinary registration.
use super::{Domain, Error, Port, Recorded, ReservationWork, Store, s};
use crate::{
    sema::crates::typed_headers::{error, predicate_remap::translation::feature_names::paid},
    support::crates::generics::TypeRef,
    types::{ItemId, Kind, TypeId, arguments::Argument, substitute::Bound},
};

pub(super) fn map(
    port: &mut Recorded<'_, '_, '_>,
    store: &mut Store,
    row: s::Type,
    mapped: &dyn Fn(TypeRef) -> Result<TypeId, Error>,
    work: &mut ReservationWork,
) -> Result<TypeId, Error> {
    if !matches!(port.body, Domain::Ordinary(_)) {
        return Err(error(
            "selected applications require their actual selected producer",
        ));
    }
    let (s::Type::FunctionItem(reference) | s::Type::ApplicationSignature(reference)) = row else {
        return Err(error(
            "ordinary application requires its actual application recipe",
        ));
    };
    let rows = port.rows();
    let raw = rows
        .function_applications
        .get(reference.0 as usize)
        .ok_or_else(|| error("ordinary application lost its actual row"))?;
    if raw.declaration.artifact != rows.policy.provider {
        return Err(error(
            "ordinary application belongs to another actual provider",
        ));
    }
    let item = ItemId(port.definition(raw.declaration.declaration, work)?.0);
    let callables = port.callables;
    let (template, parameters) = callables
        .registration(item)
        .ok_or_else(|| error("ordinary application requires its actual completed callable"))?;
    let actuals = raw.arguments.slice(&rows.arguments).map_err(error)?;
    let early = template.parameters as usize;
    if actuals.len() != early.saturating_add(parameters.len()) {
        return Err(error(
            "ordinary application changed its complete formal arity",
        ));
    }
    work.charge(actuals.len().saturating_add(1))?;
    work.reserve(
        actuals
            .len()
            .saturating_mul(4 * size_of::<Argument>())
            .saturating_add(
                parameters
                    .len()
                    .saturating_mul(size_of::<(crate::types::parameters::ParameterId, Argument)>()),
            ),
    )?;
    let mut arguments = Vec::with_capacity(actuals.len());
    for actual in actuals {
        arguments.push(match *actual {
            s::Argument::Type(root) => Argument::Type(mapped(root)?),
            s::Argument::Region(reference) => match port.region(reference, work)? {
                super::super::Demand::Ready(region) => Argument::Lifetime(region),
                super::super::Demand::Blocked(_) => {
                    return Err(error(
                        "ordinary application requires its actual regional domain",
                    ));
                }
            },
        });
    }
    let regions = arguments[..early]
        .iter()
        .map(|argument| match argument {
            Argument::Lifetime(region) => Ok(*region),
            _ => Err(error("ordinary callable early argument is not a lifetime")),
        })
        .collect::<Result<Vec<_>, _>>()?;
    let bindings = parameters
        .iter()
        .copied()
        .zip(arguments[early..].iter().copied())
        .collect::<Vec<_>>();
    paid(work, |pay| {
        let mut fold = Bound::parameters_captured_charged(store, &bindings, regions, &pay)?;
        let signature = fold.signature_charged(store, template.signature, 0, &pay)?;
        if matches!(row, s::Type::ApplicationSignature(_)) {
            return Ok(signature);
        }
        let arguments = store.intern_arguments_observed::<true>(&arguments, &pay)?;
        store.intern_observed::<true>(
            Kind::FunctionItem {
                item,
                arguments,
                signature,
            },
            &pay,
        )
    })
}
