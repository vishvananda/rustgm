//! Imported closed scalar values have typed facts but no source initializer or storage.
use super::{Declarations, Error, Report, error, scalar};
use crate::{
    literal::value::FloatBits,
    sema::{
        model::Scalar as Value,
        values::{Constant, Evaluator},
    },
    support::crates::{Scalar, ScalarValue},
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
fn convert(value: ScalarValue) -> Result<Value, &'static str> {
    let bits = value.bits();
    Ok(match value.ty() {
        Scalar::Unit => return Err("unit constant requires its typed empty value"),
        Scalar::Bool => Value::Boolean(bits != 0),
        Scalar::Char => Value::Character(
            u32::try_from(bits)
                .ok()
                .and_then(char::from_u32)
                .ok_or("invalid imported constant character")?,
        ),
        Scalar::F32 => Value::Float(FloatBits::F32(
            u32::try_from(bits).map_err(|_| "invalid imported f32 bits")?,
        )),
        Scalar::F64 => Value::Float(FloatBits::F64(
            u64::try_from(bits).map_err(|_| "invalid imported f64 bits")?,
        )),
        Scalar::U8
        | Scalar::U16
        | Scalar::U32
        | Scalar::U64
        | Scalar::U128
        | Scalar::Usize
        | Scalar::I8
        | Scalar::I16
        | Scalar::I32
        | Scalar::I64
        | Scalar::I128
        | Scalar::Isize => Value::Integer(bits),
    })
}
