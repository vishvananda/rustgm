//! Scalar floating-point math intrinsics lower to C libm calls or bounded shims.
use super::{Lower, Selected, ir};
use crate::{
    lowir::{
        builder::Builder,
        program::{Function, Literal, Operand, SymbolId, Terminator, Value},
        vocabulary::{Binary, Predicate, Type},
    },
    types::{
        GenericArgs, Kind, Primitive, Store, TypeId, callable::Convention, instance::Instance,
    },
};

#[derive(Clone, Copy)]
struct LibmSpec {
    symbol: &'static str,
    scalar: Type,
    primitive: Primitive,
    arity: usize,
}

fn libm_spec(name: &str) -> Option<LibmSpec> {
    let (symbol, scalar, primitive, arity) = match name {
        "floorf32" => ("floorf", Type::F32, Primitive::F32, 1),
        "floorf64" => ("floor", Type::F64, Primitive::F64, 1),
        "ceilf32" => ("ceilf", Type::F32, Primitive::F32, 1),
        "ceilf64" => ("ceil", Type::F64, Primitive::F64, 1),
        "truncf32" => ("truncf", Type::F32, Primitive::F32, 1),
        "truncf64" => ("trunc", Type::F64, Primitive::F64, 1),
        "roundf32" => ("roundf", Type::F32, Primitive::F32, 1),
        "roundf64" => ("round", Type::F64, Primitive::F64, 1),
        "round_ties_even_f32" => ("roundevenf", Type::F32, Primitive::F32, 1),
        "round_ties_even_f64" => ("roundeven", Type::F64, Primitive::F64, 1),
        "sqrtf32" => ("sqrtf", Type::F32, Primitive::F32, 1),
        "sqrtf64" => ("sqrt", Type::F64, Primitive::F64, 1),
        "sinf32" => ("sinf", Type::F32, Primitive::F32, 1),
        "sinf64" => ("sin", Type::F64, Primitive::F64, 1),
        "cosf32" => ("cosf", Type::F32, Primitive::F32, 1),
        "cosf64" => ("cos", Type::F64, Primitive::F64, 1),
        "expf32" => ("expf", Type::F32, Primitive::F32, 1),
        "expf64" => ("exp", Type::F64, Primitive::F64, 1),
        "exp2f32" => ("exp2f", Type::F32, Primitive::F32, 1),
        "exp2f64" => ("exp2", Type::F64, Primitive::F64, 1),
        "logf32" => ("logf", Type::F32, Primitive::F32, 1),
        "logf64" => ("log", Type::F64, Primitive::F64, 1),
        "log2f32" => ("log2f", Type::F32, Primitive::F32, 1),
        "log2f64" => ("log2", Type::F64, Primitive::F64, 1),
        "log10f32" => ("log10f", Type::F32, Primitive::F32, 1),
        "log10f64" => ("log10", Type::F64, Primitive::F64, 1),
        "powf32" => ("powf", Type::F32, Primitive::F32, 2),
        "powf64" => ("pow", Type::F64, Primitive::F64, 2),
        "fmaf32" | "fmuladdf32" => ("fmaf", Type::F32, Primitive::F32, 3),
        "fmaf64" | "fmuladdf64" => ("fma", Type::F64, Primitive::F64, 3),
        _ => return None,
    };
    Some(LibmSpec {
        symbol,
        scalar,
        primitive,
        arity,
    })
}

impl Lower<'_, '_, '_> {
    pub(super) fn source_float_math(
        &mut self,
        name: &str,
        callee: TypeId,
        signature: TypeId,
        arguments: ir::ValueList,
        result: TypeId,
    ) -> Result<Option<Selected>, &'static str> {
        if let Some(spec) = libm_spec(name) {
            return self
                .source_libm_call(spec, callee, signature, arguments, result)
                .map(Some);
        }
        match name {
            "powif32" | "powif64" => self
                .source_powi(name == "powif32", callee, signature, arguments, result)
                .map(Some),
            "minimumf32" | "minimumf64" | "maximumf32" | "maximumf64" => self
                .source_ieee_minmax(name, callee, signature, arguments, result)
                .map(Some),
            _ => Ok(None),
        }
    }

    fn source_libm_call(
        &mut self,
        spec: LibmSpec,
        callee: TypeId,
        signature: TypeId,
        arguments: ir::ValueList,
        result: TypeId,
    ) -> Result<Selected, &'static str> {
        let expected = TypeId::primitive(spec.primitive);
        validate_uniform_signature(
            self.context.types,
            callee,
            signature,
            result,
            expected,
            spec.arity,
        )?;
        let inputs = self
            .body
            .arguments
            .get(arguments.start as usize..arguments.end as usize)
            .ok_or("source floating math operation has no arguments")?;
        if inputs.len() != spec.arity || inputs.iter().any(|&id| self.input_type(id) != Ok(result))
        {
            return Err("source floating math operation differs from its evaluated call");
        }
        self.context
            .require_reached_body(self.body_index, self.body)?;
        let mut operands = Vec::with_capacity(spec.arity);
        for &id in inputs {
            let value = self.value(id)?.runtime()?;
            if value.ty != spec.scalar {
                return Err("source floating math operand differs from its native representation");
            }
            operands.push(value);
        }
        let parameter_types = [spec.scalar; 3];
        let native =
            self.context
                .output
                .signature(&parameter_types[..spec.arity], spec.scalar, false)?;
        let symbol = self
            .context
            .output
            .declare(spec.symbol.into(), native, false)?;
        let value = self
            .builder
            .call(native, spec.scalar, Operand::Symbol(symbol), &operands)?
            .ok_or("source floating math call produced no value")?;
        Ok(Selected::Runtime(value))
    }

    fn source_powi(
        &mut self,
        single: bool,
        callee: TypeId,
        signature: TypeId,
        arguments: ir::ValueList,
        result: TypeId,
    ) -> Result<Selected, &'static str> {
        let (scalar, primitive) = if single {
            (Type::F32, Primitive::F32)
        } else {
            (Type::F64, Primitive::F64)
        };
        let expected = TypeId::primitive(primitive);
        let exponent = TypeId::primitive(Primitive::I32);
        let types = self.context.types;
        let Some(Kind::FunctionItem {
            signature: public,
            arguments: generic,
            ..
        }) = types.kind(callee)
        else {
            return Err("source powi operation lacks its complete function item");
        };
        if generic != GenericArgs::EMPTY
            || public != signature
            || result != expected
            || !matches!(types.kind(signature), Some(Kind::Function {
                bound_regions: 0, parameters, result: output, unsafe_: false, convention: Convention::RUST,
            }) if output == expected && types.list(parameters) == Some(&[expected, exponent]))
        {
            return Err("source powi operation differs from its checked declaration");
        }
        let inputs = self
            .body
            .arguments
            .get(arguments.start as usize..arguments.end as usize)
            .ok_or("source powi operation has no arguments")?;
        if inputs.len() != 2
            || self.input_type(inputs[0]) != Ok(expected)
            || self.input_type(inputs[1]) != Ok(exponent)
        {
            return Err("source powi operation differs from its evaluated call");
        }
        self.context
            .require_reached_body(self.body_index, self.body)?;
        let base = self.value(inputs[0])?.runtime()?;
        let power = self.value(inputs[1])?.runtime()?;
        if base.ty != scalar || power.ty != Type::I32 {
            return Err("source powi operands differ from their native representation");
        }
        let instance = self.body.callable_target(Instance::new(types, callee)?);
        let symbol = self.demand_powi_shim(instance, scalar)?;
        let native = self.context.output.symbols[symbol.0 as usize].signature;
        let value = self
            .builder
            .call(native, scalar, Operand::Symbol(symbol), &[base, power])?
            .ok_or("source powi shim produced no value")?;
        Ok(Selected::Runtime(value))
    }

    fn demand_powi_shim(
        &mut self,
        instance: Instance,
        scalar: Type,
    ) -> Result<SymbolId, &'static str> {
        if let Some(&symbol) = self.context.intrinsic_symbols.get(&instance) {
            return Ok(symbol);
        }
        self.context.reserve_shim()?;
        let native = self
            .context
            .output
            .signature(&[scalar, Type::I32], scalar, false)?;
        let body = build_powi(scalar)?;
        let symbol = self.context.output.declare(
            self.context
                .internal_name(format!("rg.source.powi.{}", scalar.name())),
            native,
            false,
        )?;
        self.context.output.define(symbol, body)?;
        self.context.intrinsic_symbols.insert(instance, symbol);
        Ok(symbol)
    }

    fn source_ieee_minmax(
        &mut self,
        name: &str,
        callee: TypeId,
        signature: TypeId,
        arguments: ir::ValueList,
        result: TypeId,
    ) -> Result<Selected, &'static str> {
        let (scalar, primitive) = if name.ends_with("f32") {
            (Type::F32, Primitive::F32)
        } else {
            (Type::F64, Primitive::F64)
        };
        let expected = TypeId::primitive(primitive);
        validate_uniform_signature(self.context.types, callee, signature, result, expected, 2)?;
        let inputs = self
            .body
            .arguments
            .get(arguments.start as usize..arguments.end as usize)
            .ok_or("source floating minmax operation has no arguments")?;
        if inputs.len() != 2
            || self.input_type(inputs[0]) != Ok(result)
            || self.input_type(inputs[1]) != Ok(result)
        {
            return Err("source floating minmax operation differs from its evaluated call");
        }
        self.context
            .require_reached_body(self.body_index, self.body)?;
        let left = self.value(inputs[0])?.runtime()?;
        let right = self.value(inputs[1])?.runtime()?;
        if left.ty != scalar || right.ty != scalar {
            return Err("source floating minmax operands differ from their native representation");
        }
        self.ieee_minmax_value(left, right, name.starts_with("minimum"))
            .map(Selected::Runtime)
    }

    fn ieee_minmax_value(
        &mut self,
        left: Value,
        right: Value,
        minimum: bool,
    ) -> Result<Value, &'static str> {
        let slot = Operand::Slot(self.builder.slot(left.ty)?);
        let left_nan = self.builder.compare(Predicate::Ne, left, left)?;
        let right_nan = self.builder.compare(Predicate::Ne, right, right)?;
        let check_right_nan = self.builder.block()?;
        let check_equal = self.builder.block()?;
        let merge_zeros = self.builder.block()?;
        let check_order = self.builder.block()?;
        let take_left = self.builder.block()?;
        let take_right = self.builder.block()?;
        let join = self.builder.block()?;
        self.builder.terminate(Terminator::Branch {
            condition: left_nan.operand,
            yes: take_left,
            no: check_right_nan,
        })?;
        self.builder.enter(check_right_nan)?;
        self.builder.terminate(Terminator::Branch {
            condition: right_nan.operand,
            yes: take_right,
            no: check_equal,
        })?;
        self.builder.enter(check_equal)?;
        let equal = self.builder.compare(Predicate::Eq, left, right)?;
        self.builder.terminate(Terminator::Branch {
            condition: equal.operand,
            yes: merge_zeros,
            no: check_order,
        })?;
        self.builder.enter(merge_zeros)?;
        let carrier = if left.ty == Type::F32 {
            Type::U32
        } else {
            Type::I64
        };
        let other = Operand::Slot(self.builder.slot(left.ty)?);
        self.builder.store(left, slot)?;
        self.builder.store(right, other)?;
        let left_bits = self.builder.load(carrier, slot)?;
        let right_bits = self.builder.load(carrier, other)?;
        let combined = self.builder.binary(
            if minimum { Binary::Or } else { Binary::And },
            left_bits,
            right_bits,
        )?;
        self.builder.store(combined, slot)?;
        self.builder.terminate(Terminator::Jump(join))?;
        self.builder.enter(check_order)?;
        let ordered = self.builder.compare(
            if minimum {
                Predicate::Lt
            } else {
                Predicate::Gt
            },
            left,
            right,
        )?;
        self.builder.terminate(Terminator::Branch {
            condition: ordered.operand,
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
}

fn validate_uniform_signature(
    types: &Store,
    callee: TypeId,
    signature: TypeId,
    result: TypeId,
    expected: TypeId,
    arity: usize,
) -> Result<(), &'static str> {
    let Some(Kind::FunctionItem {
        signature: public,
        arguments,
        ..
    }) = types.kind(callee)
    else {
        return Err("source floating math operation lacks its complete function item");
    };
    let inputs = [expected; 3];
    if arguments != GenericArgs::EMPTY
        || public != signature
        || result != expected
        || !matches!(types.kind(signature), Some(Kind::Function {
            bound_regions: 0, parameters, result: output, unsafe_: false, convention: Convention::RUST,
        }) if output == expected && types.list(parameters) == Some(&inputs[..arity]))
    {
        return Err("source floating math operation differs from its checked declaration");
    }
    Ok(())
}

/// Binary exponentiation matching LLVM compiler-rt `__powisf2` / `__powidf2`.
fn build_powi(scalar: Type) -> Result<Function, &'static str> {
    let one_literal = if scalar == Type::F32 {
        Literal::F32(1.0_f32.to_bits())
    } else {
        Literal::F64(1.0_f64.to_bits())
    };
    let mut b = Builder::default();
    let (entry, negate, positive, header, finish, invert, done) = (
        b.block()?,
        b.block()?,
        b.block()?,
        b.block()?,
        b.block()?,
        b.block()?,
        b.block()?,
    );
    let base_slot = Operand::Slot(b.slot(scalar)?);
    let result_slot = Operand::Slot(b.slot(scalar)?);
    let exp_slot = Operand::Slot(b.slot(Type::I32)?);

    b.enter(entry)?;
    let x = Value {
        ty: scalar,
        operand: Operand::Parameter(0),
    };
    let n = Value {
        ty: Type::I32,
        operand: Operand::Parameter(1),
    };
    let one_init = b.literal(scalar, one_literal)?;
    b.store(x, base_slot)?;
    b.store(one_init, result_slot)?;
    let signed_zero = b.integer(Type::I32, 0)?;
    let is_neg = b.compare(Predicate::Lt, n, signed_zero)?;
    b.terminate(Terminator::Branch {
        condition: is_neg.operand,
        yes: negate,
        no: positive,
    })?;

    b.enter(negate)?;
    let neg_zero = b.integer(Type::I32, 0)?;
    let abs_neg = b.binary(Binary::Sub, neg_zero, n)?;
    b.store(abs_neg, exp_slot)?;
    b.terminate(Terminator::Jump(header))?;

    b.enter(positive)?;
    b.store(n, exp_slot)?;
    b.terminate(Terminator::Jump(header))?;

    build_powi_loop(
        &mut b,
        scalar,
        (header, finish),
        (base_slot, result_slot, exp_slot),
    )?;

    b.enter(finish)?;
    b.terminate(Terminator::Branch {
        condition: is_neg.operand,
        yes: invert,
        no: done,
    })?;

    b.enter(invert)?;
    let one_recip = b.literal(scalar, one_literal)?;
    let acc_inv = b.load(scalar, result_slot)?;
    let recip = b.binary(Binary::Div, one_recip, acc_inv)?;
    b.terminate(Terminator::Return(Some(recip)))?;

    b.enter(done)?;
    let acc_done = b.load(scalar, result_slot)?;
    b.terminate(Terminator::Return(Some(acc_done)))?;

    b.finish()
}

fn build_powi_loop(
    b: &mut Builder,
    scalar: Type,
    (header, finish): (
        crate::lowir::program::BlockId,
        crate::lowir::program::BlockId,
    ),
    (base_slot, result_slot, exp_slot): (Operand, Operand, Operand),
) -> Result<(), &'static str> {
    let (body, multiply, shift, square) = (b.block()?, b.block()?, b.block()?, b.block()?);

    b.enter(header)?;
    let cur = b.load(Type::I32, exp_slot)?;
    let header_zero = b.integer(Type::I32, 0)?;
    let more = b.compare(Predicate::Ne, cur, header_zero)?;
    b.terminate(Terminator::Branch {
        condition: more.operand,
        yes: body,
        no: finish,
    })?;

    b.enter(body)?;
    let bit_mask = b.integer(Type::I32, 1)?;
    let bit = b.binary(Binary::And, cur, bit_mask)?;
    let body_zero = b.integer(Type::I32, 0)?;
    let odd = b.compare(Predicate::Ne, bit, body_zero)?;
    b.terminate(Terminator::Branch {
        condition: odd.operand,
        yes: multiply,
        no: shift,
    })?;

    b.enter(multiply)?;
    let acc = b.load(scalar, result_slot)?;
    let cur_base = b.load(scalar, base_slot)?;
    let prod = b.binary(Binary::Mul, acc, cur_base)?;
    b.store(prod, result_slot)?;
    b.terminate(Terminator::Jump(shift))?;

    b.enter(shift)?;
    let shift_one = b.integer(Type::I32, 1)?;
    let next = b.binary(Binary::Ushr, cur, shift_one)?;
    b.store(next, exp_slot)?;
    let shift_zero = b.integer(Type::I32, 0)?;
    let keep = b.compare(Predicate::Ne, next, shift_zero)?;
    b.terminate(Terminator::Branch {
        condition: keep.operand,
        yes: square,
        no: finish,
    })?;

    b.enter(square)?;
    let cur_base_sq = b.load(scalar, base_slot)?;
    let sq = b.binary(Binary::Mul, cur_base_sq, cur_base_sq)?;
    b.store(sq, base_slot)?;
    b.terminate(Terminator::Jump(header))
}
