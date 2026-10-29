//! Imported closed scalar values have typed facts but no source initializer or storage.
use super::{Declarations, Error, Report, error, scalar};
use crate::{
    sema::values::{Constant, Evaluator},
    support::crates::Scalar,
};

pub(super) fn seed(report: &Report, facts: &mut Declarations) -> Result<(), Error> {
    let Some(imported) = &report.resolution.crates else {
        return Ok(());
    };
    if imported.constants().next().is_none() {
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
    let mut evaluator = Evaluator::new(report, facts);
    for (definition, value) in imported.constants() {
        let value = if value.value().ty() == Scalar::Unit {
            evaluator
                .fields(&[], None)
                .map_err(|e| error(e.message()))?
        } else {
            Constant::scalar(convert(value.value()).map_err(error)?)
        };
        evaluator.complete_definition(definition, value);
    }
    Ok(())
}
pub(super) use super::closed_values::decode as convert;
