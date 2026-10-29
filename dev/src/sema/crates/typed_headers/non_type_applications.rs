//! The v4 ordinary bridge admits only existing, closed primitive call recipes.
use super::super::predicate_remap::translation::feature_names::paid;
use super::{Argument, Error, Headers, Kind, Ordinary, Port, Store, TypeId, Worklist, error, g, s};
use crate::types::ItemId;

pub(super) fn prepare(
    port: &mut Ordinary<'_, '_>,
    store: &mut Store,
    cursor: &Worklist<Vec<Option<TypeId>>>,
    root: g::TypeRef,
    budget: &mut Headers,
) -> Result<Option<TypeId>, Error> {
    let rows = port.rows();
    let row = rows.types[root.0 as usize];
    let (s::Type::FunctionItem(reference) | s::Type::ApplicationSignature(reference)) = row else {
        return Ok(None);
    };
    if rows.descriptor().profile() != s::Profile::Source2024ModuleFeaturePredicates {
        return Ok(None);
    }
    budget.charge(8)?;
    let application = &rows.function_applications[reference.0 as usize];
    if application.declaration.artifact != s::ArtifactRef(0) {
        return Err(error(
            "primitive source application requires its actual defining artifact",
        ));
    }
    let owner = application.declaration.declaration;
    let Some(s::HeaderRef::Function(index)) = rows.header(owner).map_err(error)? else {
        return Err(error(
            "primitive application lost its actual function header",
        ));
    };
    let function = &rows.functions[index as usize];
    let actuals = application
        .arguments
        .slice(&rows.arguments)
        .map_err(error)?;
    let expected = if let Some(parent) = function.parent {
        match rows.header(parent).map_err(error)? {
            Some(s::HeaderRef::Trait(index)) => {
                let header = &rows.traits[index as usize];
                if header.written_parameters != 0 {
                    return Err(error(
                        "generic trait application requires its wider source profile",
                    ));
                }
                1
            }
            Some(s::HeaderRef::Implementation(_)) => 0,
            _ => {
                return Err(error(
                    "primitive application has a foreign associated owner",
                ));
            }
        }
    } else {
        0
    };
    if actuals.len() != expected {
        return Err(error(
            "primitive application omitted its exact closed Self arguments",
        ));
    }
    budget.reserve(4 * actuals.len() * size_of::<Argument>())?;
    let arguments = actuals
        .iter()
        .map(|argument| {
            let s::Argument::Type(root) = argument else {
                return Err(error(
                    "regional application requires its wider source profile",
                ));
            };
            if !matches!(rows.types.get(root.0 as usize), Some(s::Type::Scalar(_))) {
                return Err(error(
                    "nonprimitive application requires its wider source profile",
                ));
            }
            cursor.child(*root).map(Argument::Type)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let signature = signature(port, store, cursor, function.callable, budget)?;
    if matches!(row, s::Type::ApplicationSignature(_)) {
        return Ok(Some(signature));
    }
    let definition = port.definition(owner, budget)?;
    paid(budget, |pay| {
        let arguments = store.intern_arguments_observed::<true>(&arguments, &pay)?;
        store.intern_observed::<true>(
            Kind::FunctionItem {
                item: ItemId(definition.0),
                arguments,
                signature,
            },
            &pay,
        )
    })
    .map(Some)
}
fn signature(
    port: &mut Ordinary<'_, '_>,
    store: &mut Store,
    cursor: &Worklist<Vec<Option<TypeId>>>,
    signature: g::Signature,
    budget: &mut Headers,
) -> Result<TypeId, Error> {
    let rows = port.rows();
    let parameters = signature.parameters.slice(&rows.roots).map_err(error)?;
    budget.charge(parameters.len().saturating_add(1))?;
    if rows.binders[signature.binder.0 as usize].count != 0
        || parameters
            .iter()
            .chain(std::iter::once(&signature.result))
            .any(|root| !matches!(rows.types.get(root.0 as usize), Some(s::Type::Scalar(_))))
    {
        return Err(error(
            "source application requires its closed primitive signature",
        ));
    }
    budget.reserve(parameters.len().saturating_mul(4 * size_of::<TypeId>()))?;
    let parameters = parameters
        .iter()
        .map(|&root| cursor.child(root))
        .collect::<Result<Vec<_>, _>>()?;
    let result = cursor.child(signature.result)?;
    paid(budget, |pay| {
        let parameters = store.intern_list_observed::<true>(&parameters, &pay)?;
        store.intern_observed::<true>(
            Kind::Function {
                bound_regions: 0,
                parameters,
                result,
                unsafe_: signature.unsafe_,
                convention: crate::types::callable::Convention::RUST,
            },
            &pay,
        )
    })
}
