//! Public named constants expose completed scalar bits without demanding runtime storage.
use super::{Declarations, DefId, Error, error, scalar};
use crate::{
    sema::values::{Reader, View},
    support::crates::{Constant, ExternalDef, Scalar, ScalarValue},
};

pub(super) fn export(
    facts: &Declarations,
    definition: DefId,
    external: ExternalDef,
) -> Result<Constant, Error> {
    if facts.generics.has_parameters(definition) || facts.associated.owner(definition).is_some() {
        return Err(error(
            "generic associated values are outside the scalar metadata profile",
        ));
    }
    let ty = facts.definitions[definition.0 as usize]
        .ok_or_else(|| error("missing constant export type"))?;
    let ty = scalar(facts, ty)?;
    let reader = Reader::new(facts);
    let value = reader
        .definition(definition)
        .ok_or_else(|| error("scalar constant export requires a completed value"))?;
    Constant::new(
        external,
        value_bits(ty, reader.view(value).map_err(error)?)?,
    )
    .map_err(error)
}

pub(super) fn value_bits(ty: Scalar, view: View) -> Result<ScalarValue, Error> {
    match (ty, view) {
        (
            Scalar::Unit,
            View::Product {
                length: 0,
                variant: None,
            },
        ) => ScalarValue::new(ty, 0).map_err(error),
        (_, View::Scalar(value)) => super::super::closed_values::encode(ty, value).map_err(error),
        _ => Err(error(
            "constant value disagrees with its scalar export type",
        )),
    }
}
