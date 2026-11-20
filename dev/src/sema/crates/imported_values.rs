//! Imported closed scalar values have typed facts but no source initializer or storage.
use super::{Declarations, Error, Report, error, scalar};
use crate::{
    resolve::model::DefId,
    sema::{
        associated::OwnerKind,
        values::{Constant, Evaluator, Instance},
    },
    support::crates::{Scalar, ScalarValue, generics::source as s},
    types::GenericArgs,
};

pub(super) fn seed(report: &Report, facts: &mut Declarations) -> Result<(), Error> {
    let Some(imported) = &report.resolution.crates else {
        return Ok(());
    };
    let source = source_constants(imported)?;
    if imported.constants().next().is_none() && source.is_empty() {
        return Ok(());
    }
    for (definition, value) in imported.constants() {
        facts.definitions[definition.0 as usize] = Some(scalar(value.value().ty()));
        facts
            .crates
            .get_or_insert_default()
            .imported_constants
            .insert(definition);
    }
    // A source constant's cell is already its installed header type.
    for &(definition, _) in &source {
        if facts.associated.owner(definition).is_none() {
            facts
                .crates
                .get_or_insert_default()
                .imported_constants
                .insert(definition);
        }
    }
    let mut evaluator = Evaluator::new(report, facts);
    for (definition, value) in imported.constants() {
        complete(&mut evaluator, definition, value.value())?;
    }
    for (definition, value) in source {
        complete(&mut evaluator, definition, value)?;
    }
    Ok(())
}
/// Version thirteen: each source artifact's constant rows, already validated
/// as closed canonical values of their declared primitive types.
fn source_constants(
    imported: &crate::resolve::crates::Imported,
) -> Result<Vec<(DefId, ScalarValue)>, Error> {
    let mut constants = Vec::new();
    for artifact in imported.dependencies().artifacts() {
        let Some(descriptor) = artifact.source_descriptor() else {
            continue;
        };
        let rows = descriptor.rows();
        for (declaration, row) in rows.constant_rows() {
            // The v14 validator admits absent values only for the exact
            // required range bounds. Their type cells still install normally.
            if row.value.is_none() && descriptor.profile().declaration_policies() {
                continue;
            }
            let definition = imported
                .generic_definition(descriptor.identity(), declaration)
                .map_err(error)?;
            let Some(s::ConstTerm::Value(value)) = row.value else {
                return Err(error("source constant seed lost its validated value"));
            };
            let Some(&s::Type::Scalar(ty)) = rows.types.get(value.ty.0 as usize) else {
                return Err(error("source constant seed lost its validated value"));
            };
            let value = ScalarValue::new(ty, value.bits).map_err(error)?;
            constants.push((definition, value));
        }
    }
    Ok(constants)
}
/// A unit value is its typed empty aggregate; every other value is its scalar.
fn complete(
    evaluator: &mut Evaluator<'_>,
    definition: DefId,
    value: ScalarValue,
) -> Result<(), Error> {
    let value = if value.ty() == Scalar::Unit {
        evaluator
            .fields(&[], None)
            .map_err(|e| error(e.message()))?
    } else {
        Constant::scalar(convert(value).map_err(error)?)
    };
    if let Some(id) = evaluator.facts().associated.owner(definition) {
        let facts = evaluator.facts();
        if !facts
            .associated
            .get(id)
            .is_some_and(|owner| matches!(owner.kind, OwnerKind::ImportedInherent(_)))
        {
            return Err(error(
                "source constant value requires its installed inherent owner",
            ));
        }
        let ty = facts
            .definitions
            .get(definition.0 as usize)
            .copied()
            .flatten()
            .ok_or_else(|| error("source inherent constant lost its declared type"))?;
        evaluator
            .complete_associated(
                Instance {
                    definition,
                    arguments: GenericArgs::EMPTY,
                },
                ty,
                value,
            )
            .map_err(|failure| error(failure.message()))?;
    } else {
        evaluator.complete_definition(definition, value);
    }
    Ok(())
}
pub(super) use super::closed_values::decode as convert;
