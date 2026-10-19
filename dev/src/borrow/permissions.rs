//! Deferred builtin uses are verified only against a completed lifetime proof.
use super::{Environment, Error, regions};
use crate::types::{
    copy::Copies,
    parameters::{environment::Environment as Parameters, predicates::Bound, shape::Shapes},
    relations::RegionEqualities,
};
use std::{hash::BuildHasher, sync::Arc};

pub(super) fn check<S: BuildHasher>(
    context: &mut Environment<'_, S>,
    proof: &regions::Body,
) -> Result<Option<Arc<Parameters>>, Error> {
    let Some(requirements) = &proof.builtins else {
        return Ok(None);
    };
    let error = |message: &str| Error {
        node: Some(proof.source),
        message: message.into(),
    };
    let parameters = requirements
        .environment
        .as_ref()
        .map(|parameters| {
            let equalities = RegionEqualities::proven(&proof.solution, &proof.identities)
                .map_err(|e| error(&e.to_string()))?;
            Ok::<_, Error>(Arc::new(
                parameters
                    .as_ref()
                    .clone()
                    .with_region_equalities(equalities),
            ))
        })
        .transpose()?;
    let mut copies = parameters
        .as_ref()
        .map_or_else(Copies::default, |e| Copies::in_environment(Arc::clone(e)));
    let mut shapes = parameters.as_ref().map(|e| Shapes::new(Arc::clone(e)));
    for &(source, predicate) in &requirements.predicates {
        let error = |message: &str| Error {
            node: Some(source),
            message: message.into(),
        };
        let (valid, message) = match predicate.bound {
            Bound::Copy | Bound::Clone => {
                let properties = copies
                    .demand(
                        context.types,
                        context.definitions,
                        context.derives,
                        predicate.ty,
                    )
                    .map_err(error)?;
                if predicate.bound == Bound::Copy {
                    (properties.copy, "generic argument must implement Copy")
                } else {
                    (properties.clone_, "generic argument must implement Clone")
                }
            }
            Bound::Sized => {
                let sized = if let Some(shapes) = &mut shapes {
                    shapes
                        .demand(context.types, predicate.ty)
                        .map_err(error)?
                        .sized
                } else {
                    context
                        .types
                        .is_sized(predicate.ty)
                        .ok_or_else(|| error("invalid builtin requirement subject"))?
                };
                (sized, "generic argument must be sized")
            }
            _ => return Err(error("deferred permission is not a builtin requirement")),
        };
        if !valid {
            return Err(error(message));
        }
    }
    Ok(parameters)
}
