//! Sign-bit operations on checked 32- and 64-bit floating values.
use super::{Lower, Selected, ir};
use crate::{
    lowir::{
        program::{Operand, Value},
        vocabulary::{Binary, Type},
    },
    types::{
        GenericArgs, Kind, Primitive, Store, TypeId, arguments::Argument, callable::Convention,
    },
};

impl Lower<'_, '_, '_> {
    pub(super) fn source_float_sign(
        &mut self,
        name: &str,
        callee: TypeId,
        signature: TypeId,
        arguments: ir::ValueList,
        result: TypeId,
    ) -> Result<Selected, &'static str> {
        let types = self.context.types;
        let (scalar, arity) = shape(types, name, callee, signature, result)?;
        let inputs = self
            .body
            .arguments
            .get(arguments.start as usize..arguments.end as usize)
            .ok_or("source floating sign operation has no arguments")?;
        if inputs.len() != arity || inputs.iter().any(|&id| self.input_type(id) != Ok(result)) {
            return Err("source floating sign operation differs from its evaluated call");
        }
        self.context
            .require_reached_body(self.body_index, self.body)?;
        let magnitude = self.value(inputs[0])?.runtime()?;
        let sign = inputs
            .get(1)
            .map(|&id| self.value(id)?.runtime())
            .transpose()?;
        if magnitude.ty != scalar || sign.is_some_and(|value| value.ty != scalar) {
            return Err("source floating sign operands differ from their native representation");
        }
        if matches!(
            name,
            "minimum_number_nsz_f32"
                | "minimum_number_nsz_f64"
                | "maximum_number_nsz_f32"
                | "maximum_number_nsz_f64"
        ) {
            let right = sign.ok_or("source floating minmax requires two operands")?;
            let minimum = name.starts_with("minimum_");
            return self
                .float_minmax(magnitude, right, minimum)
                .map(Selected::Runtime);
        }
        self.float_sign_bits(magnitude, sign).map(Selected::Runtime)
    }

    fn float_minmax(
        &mut self,
        left: Value,
        right: Value,
        minimum: bool,
    ) -> Result<Value, &'static str> {
        use crate::lowir::{program::Terminator, vocabulary::Predicate};
        let slot = Operand::Slot(self.builder.slot(left.ty)?);
        let right_nan = self.builder.compare(Predicate::Ne, right, right)?;
        let ordered = self.builder.compare(
            if minimum {
                Predicate::Lt
            } else {
                Predicate::Gt
            },
            left,
            right,
        )?;
        let choose_left = self.builder.binary(Binary::Or, right_nan, ordered)?;
        let take_left = self.builder.block()?;
        let take_right = self.builder.block()?;
        let join = self.builder.block()?;
        self.builder.terminate(Terminator::Branch {
            condition: choose_left.operand,
            yes: take_left,
            no: take_right,
        })?;
        self.builder.enter(take_left)?;
        self.builder.store(left, slot)?;
        self.builder.terminate(Terminator::Jump(join))?;
        self.builder.enter(take_right)?;
        self.builder.store(right, slot)?;
        self.builder.terminate(Terminator::Jump(join))?;
        self.builder.enter(join)?;
        self.builder.load(left.ty, slot)
    }

    fn float_sign_bits(
        &mut self,
        magnitude: Value,
        sign: Option<Value>,
    ) -> Result<Value, &'static str> {
        let (carrier, top) = if magnitude.ty == Type::F32 {
            (Type::U32, 1_u128 << 31)
        } else {
            (Type::I64, 1_u128 << 63)
        };
        let builder = &mut self.builder;
        // A shared stack slot transfers the exact bit image across register classes.
        let slot = Operand::Slot(builder.slot(magnitude.ty)?);
        builder.store(magnitude, slot)?;
        let bits = builder.load(carrier, slot)?;
        let kept = builder.integer(carrier, top - 1)?;
        let mut bits = builder.binary(Binary::And, bits, kept)?;
        if let Some(sign) = sign {
            builder.store(sign, slot)?;
            let sign = builder.load(carrier, slot)?;
            let top = builder.integer(carrier, top)?;
            let sign = builder.binary(Binary::And, sign, top)?;
            bits = builder.binary(Binary::Or, bits, sign)?;
        }
        builder.store(bits, slot)?;
        builder.load(magnitude.ty, slot)
    }
}

fn shape(
    types: &Store,
    name: &str,
    callee: TypeId,
    signature: TypeId,
    result: TypeId,
) -> Result<(Type, usize), &'static str> {
    let (scalar, arity) = match (name, types.kind(result)) {
        ("fabs", Some(Kind::Primitive(Primitive::F32))) => (Type::F32, 1),
        ("fabs", Some(Kind::Primitive(Primitive::F64))) => (Type::F64, 1),
        (
            "copysignf32" | "minimum_number_nsz_f32" | "maximum_number_nsz_f32",
            Some(Kind::Primitive(Primitive::F32)),
        ) => (Type::F32, 2),
        (
            "copysignf64" | "minimum_number_nsz_f64" | "maximum_number_nsz_f64",
            Some(Kind::Primitive(Primitive::F64)),
        ) => (Type::F64, 2),
        _ => return Err("source floating sign operation requires f32 or f64"),
    };
    let Some(Kind::FunctionItem {
        signature: public,
        arguments,
        ..
    }) = types.kind(callee)
    else {
        return Err("source floating sign operation lacks its complete function item");
    };
    let arguments_match = if arity == 1 {
        types
            .arguments(arguments)
            .is_some_and(|arguments| arguments.eq([Argument::Type(result)]))
    } else {
        arguments == GenericArgs::EMPTY
    };
    if !arguments_match
        || public != signature
        || !matches!(types.kind(signature), Some(Kind::Function {
            bound_regions: 0, parameters, result: output, unsafe_: false, convention: Convention::RUST,
        }) if output == result && types.list(parameters) == Some(&[result, result][..arity]))
    {
        return Err("source floating sign operation differs from its checked declaration");
    }
    Ok((scalar, arity))
}
