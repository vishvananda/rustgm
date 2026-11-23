//! Typed SIMD intrinsics use SSE2 operations or checked scalar lanes.
//! Shuffle uses its constant indices. Scalar bitmasks preserve source lane
//! order and zero the padding bits in their result.
use super::{Address, Lower, Selected, ir};
use crate::{
    lowir::{
        program::Value,
        vocabulary::{Binary, Convert, Predicate, Type, Vector},
    },
    sema::{
        compiler_items::{SimdOperation, SimdSelection},
        model::Scalar,
        values::View,
    },
    types::{Kind, Primitive, TypeId, aggregate::SimdShape, instance::Instance, length::Length},
};

/// The lanes of a closed `repr(simd)` type.
#[derive(Clone, Copy, PartialEq, Eq)]
struct Lanes {
    element: TypeId,
    count: u64,
}
/// The lowering selected for a validated instance.
enum Plan {
    Mask(Mask),
    /// One SSE2 sequence over the whole `v128`.
    Vector,
    /// Lanes of `bytes` each, copied by constant index.
    Copy {
        bytes: u64,
        inputs: u64,
        outputs: u64,
    },
}
#[derive(Clone, Copy)]
struct Mask {
    lanes: u64,
    stride: u64,
    array: bool,
}

const NOT_VECTOR: &str = "invalid monomorphization of a SIMD intrinsic: expected a SIMD vector type, found a non-SIMD type";
const LENGTH: &str =
    "invalid monomorphization of a SIMD intrinsic: expected a return type with the input's length";
const INTEGER_RESULT: &str =
    "invalid monomorphization of a SIMD intrinsic: expected a return type with integer elements";
const INTEGER_INPUT: &str = "invalid monomorphization of a SIMD intrinsic: unsupported operation on a vector of non-integer elements";
const BITMASK: &str = "invalid monomorphization of simd_bitmask: cannot return this type, expected the unsigned integer with one bit per lane or its byte array";
const SPLAT: &str =
    "invalid monomorphization of simd_splat: the value's type is not the vector's element type";
const SHUFFLE_INDEX: &str =
    "invalid monomorphization of simd_shuffle: the index must be a SIMD vector of `u32`";
const SHUFFLE_ELEMENT: &str = "invalid monomorphization of simd_shuffle: expected the input's element type in the return type";
const SHUFFLE_BOUND: &str =
    "invalid monomorphization of simd_shuffle: a shuffle index is out of bounds";
const SHUFFLE_CONSTANT: &str = "3rd argument of `simd_shuffle` is required to be a `const` item";
/// The named fence of each operation outside the 16-byte lowering.
const fn fence(operation: SimdOperation) -> &'static str {
    match operation {
        SimdOperation::Eq => "simd_eq lowering covers sixteen 8-bit integer lanes",
        SimdOperation::Gt => "simd_gt lowering covers sixteen signed 8-bit lanes",
        SimdOperation::Lt => "simd_lt lowering covers sixteen signed 8-bit lanes",
        SimdOperation::And => "simd_and lowering covers 16-byte integer vectors",
        SimdOperation::Or => "simd_or lowering covers 16-byte integer vectors",
        SimdOperation::Bitmask => "simd_bitmask scalar lowering covers up to 128 integer lanes",
        SimdOperation::ReduceAll => "simd_reduce_all lowering covers sixteen 8-bit lanes",
        SimdOperation::Shuffle => "simd_shuffle lowering covers integer lanes",
        SimdOperation::Splat => "simd_splat lowering covers sixteen 8-bit integer lanes",
    }
}
/// The rejected and fenced outcomes of one instance.
enum Refusal {
    Rejected(&'static str),
    Fenced,
}
impl From<&'static str> for Refusal {
    fn from(message: &'static str) -> Self {
        Self::Rejected(message)
    }
}

impl Lower<'_, '_, '_> {
    /// Lowers an authenticated direct call of a SIMD declaration, or `None`
    /// for any other call and, in the bring-up profile, for a fenced shape.
    pub(super) fn source_lanes(
        &mut self,
        id: ir::ValueId,
    ) -> Result<Option<Selected>, &'static str> {
        let value = *self
            .body
            .values
            .get(id.0 as usize)
            .ok_or("source SIMD call disappeared")?;
        let ir::Rvalue::Call {
            signature,
            callee,
            arguments,
        } = value.kind
        else {
            return Ok(None);
        };
        let ty = self
            .body
            .values
            .get(callee.0 as usize)
            .ok_or("source SIMD callee disappeared")?
            .ty;
        if !matches!(self.context.types.kind(ty), Some(Kind::FunctionItem { .. })) {
            return Ok(None);
        }
        let instance = Instance::new(self.context.types, ty)?;
        let Some(selected) = self
            .context
            .constants
            .source_simd(instance, self.body.source)?
        else {
            return Ok(None);
        };
        self.context
            .require_reached_body(self.body_index, self.body)?;
        let inputs = self
            .body
            .arguments
            .get(arguments.start as usize..arguments.end as usize)
            .ok_or("source SIMD arguments disappeared")?;
        if inputs.len() != selected.inputs().len()
            || !selected.opened_signature_matches(self.context.types, signature)
            || value.ty != selected.output
            || inputs.iter().zip(selected.inputs()).any(|(id, &ty)| {
                self.body
                    .values
                    .get(id.0 as usize)
                    .is_none_or(|v| v.ty != ty)
            })
            || !matches!(self.value(callee)?, Selected::Item(actual) if actual==self.body.callable_target(instance))
        {
            return Err("source SIMD intrinsic differs from its actual checked direct call");
        }
        let mut operands = [ir::ValueId(0); 3];
        operands[..inputs.len()].copy_from_slice(inputs);
        let plan = match self.lane_plan(&selected) {
            Ok(plan) => plan,
            Err(Refusal::Rejected(message)) => return Err(message),
            Err(Refusal::Fenced) if crate::support::bringup::selected().is_some() => {
                return Ok(None);
            }
            Err(Refusal::Fenced) => return Err(fence(selected.operation)),
        };
        self.lanes_operation(&selected, plan, operands).map(Some)
    }
    /// The closed lanes of a `repr(simd)` type, or `None` for another type.
    fn simd_lanes(&self, ty: TypeId) -> Result<Option<Lanes>, &'static str> {
        match self.context.aggregates.simd_shape(self.context.types, ty)? {
            SimdShape::NotSimd => Ok(None),
            SimdShape::Pending => Err("source SIMD intrinsic requires a closed vector type"),
            SimdShape::Ready { lane, count } => Ok(Some(Lanes {
                element: lane,
                count,
            })),
        }
    }
    fn integer_lanes(&self, ty: TypeId) -> Result<Option<(Lanes, bool, u16)>, &'static str> {
        let Some(lanes) = self.simd_lanes(ty)? else {
            return Ok(None);
        };
        Ok(match self.context.types.kind(lanes.element) {
            Some(Kind::Primitive(primitive)) => primitive
                .integer()
                .map(|(signed, bits)| (lanes, signed, bits)),
            _ => None,
        })
    }
    /// rustc's monomorphization checks, then the shape this lowering covers.
    fn lane_plan(&self, selected: &SimdSelection) -> Result<Plan, Refusal> {
        let [input, ..] = *selected.inputs() else {
            return Err("source SIMD intrinsic has no input".into());
        };
        let output = selected.output;
        let bytes = |lanes: Option<(Lanes, bool, u16)>| {
            lanes.filter(|(lanes, _, bits)| lanes.count == 16 && *bits == 8)
        };
        match selected.operation {
            SimdOperation::Eq | SimdOperation::Gt | SimdOperation::Lt => {
                let source = self.simd_lanes(input)?.ok_or(NOT_VECTOR)?;
                let result = self.simd_lanes(output)?.ok_or(NOT_VECTOR)?;
                if result.count != source.count {
                    return Err(LENGTH.into());
                }
                let Some((_, _, result_bits)) = self.integer_lanes(output)? else {
                    return Err(INTEGER_RESULT.into());
                };
                let Some((_, signed, _)) = bytes(self.integer_lanes(input)?) else {
                    return Err(Refusal::Fenced);
                };
                let ordered = selected.operation != SimdOperation::Eq;
                if result_bits != 8 || (ordered && !signed) {
                    return Err(Refusal::Fenced);
                }
                Ok(Plan::Vector)
            }
            SimdOperation::And | SimdOperation::Or => {
                self.simd_lanes(input)?.ok_or(NOT_VECTOR)?;
                let (lanes, _, bits) = self.integer_lanes(input)?.ok_or(INTEGER_INPUT)?;
                if lanes.count * u64::from(bits) != 128 {
                    return Err(Refusal::Fenced);
                }
                Ok(Plan::Vector)
            }
            SimdOperation::Bitmask => self.bitmask_plan(input, output),
            SimdOperation::ReduceAll => {
                self.simd_lanes(input)?.ok_or(NOT_VECTOR)?;
                let lanes = self.integer_lanes(input)?.ok_or(INTEGER_INPUT)?;
                bytes(Some(lanes)).ok_or(Refusal::Fenced)?;
                Ok(Plan::Vector)
            }
            SimdOperation::Shuffle => self.shuffle_plan(selected),
            SimdOperation::Splat => {
                let lanes = self.simd_lanes(output)?.ok_or(NOT_VECTOR)?;
                if lanes.element != input {
                    return Err(SPLAT.into());
                }
                bytes(self.integer_lanes(output)?).ok_or(Refusal::Fenced)?;
                Ok(Plan::Vector)
            }
        }
    }
    fn bitmask_plan(&self, input: TypeId, output: TypeId) -> Result<Plan, Refusal> {
        let lanes = self.simd_lanes(input)?.ok_or(NOT_VECTOR)?;
        let (_, _, bits) = self.integer_lanes(input)?.ok_or(INTEGER_INPUT)?;
        let width = lanes
            .count
            .max(8)
            .checked_next_power_of_two()
            .ok_or(BITMASK)?;
        let array = match self.context.types.kind(output) {
            Some(Kind::Primitive(primitive))
                if primitive
                    .integer()
                    .is_some_and(|(signed, bits)| !signed && u64::from(bits) == width) =>
            {
                false
            }
            Some(Kind::Array { element, .. } | Kind::ArrayTerm { element, .. })
                if element == TypeId::primitive(Primitive::U8)
                    && Length::of(self.context.types, output)?.known()
                        == Some(lanes.count.div_ceil(8)) =>
            {
                true
            }
            _ => return Err(BITMASK.into()),
        };
        if lanes.count == 16 && bits == 8 && output == TypeId::primitive(Primitive::U16) {
            return Ok(Plan::Vector);
        }
        // Bounded straight-line extraction covers the integer carriers and
        // the portable SIMD mask sizes. Larger byte arrays need a loop.
        if lanes.count > 128 {
            return Err(Refusal::Fenced);
        }
        Ok(Plan::Mask(Mask {
            lanes: lanes.count,
            stride: u64::from(bits / 8),
            array,
        }))
    }
    fn shuffle_plan(&self, selected: &SimdSelection) -> Result<Plan, Refusal> {
        let [input, _, index] = *selected.inputs() else {
            return Err("source simd_shuffle lost its index".into());
        };
        let source = self.simd_lanes(input)?.ok_or(NOT_VECTOR)?;
        let indices = self
            .simd_lanes(index)?
            .filter(|lanes| lanes.element == TypeId::primitive(Primitive::U32))
            .ok_or(SHUFFLE_INDEX)?;
        let result = self.simd_lanes(selected.output)?.ok_or(NOT_VECTOR)?;
        if result.count != indices.count {
            return Err(LENGTH.into());
        }
        if result.element != source.element {
            return Err(SHUFFLE_ELEMENT.into());
        }
        let Some((_, _, bits)) = self.integer_lanes(input)? else {
            return Err(Refusal::Fenced);
        };
        Ok(Plan::Copy {
            bytes: u64::from(bits / 8),
            inputs: source.count,
            outputs: result.count,
        })
    }
    fn lanes_operation(
        &mut self,
        selected: &SimdSelection,
        plan: Plan,
        [first, second, third]: [ir::ValueId; 3],
    ) -> Result<Selected, &'static str> {
        let output = selected.output;
        match (selected.operation, plan) {
            (SimdOperation::Splat, _) => {
                let lane = self.value(first)?.runtime()?;
                let wide = self.builder.convert(Convert::Zext, Type::I32, lane)?;
                let copies = self.builder.integer(Type::I32, 0x0101_0101)?;
                let word = self.builder.binary(Binary::Mul, wide, copies)?;
                let low = self.builder.vector(Vector::FromI32, word, None, 0)?;
                let all = self.builder.vector(Vector::Pshufd, low, None, 0)?;
                self.vector_result(all, output)
            }
            (
                SimdOperation::Shuffle,
                Plan::Copy {
                    bytes,
                    inputs,
                    outputs,
                },
            ) => self.shuffle(first, second, third, (bytes, inputs, outputs), output),
            (SimdOperation::Bitmask, Plan::Mask(mask)) => self.bitmask_scalar(first, output, mask),
            (SimdOperation::Bitmask, Plan::Vector) => {
                let mask = self.vector_input(first, Vector::Pmovmskb, None)?;
                let ty = self.context.representation(output)?.result();
                self.builder
                    .convert(Convert::Trunc, ty, mask)
                    .map(Selected::Runtime)
            }
            (SimdOperation::ReduceAll, _) => {
                // rustc 1.98.0 reads each lane's sign bit.
                let mask = self.vector_input(first, Vector::Pmovmskb, None)?;
                let all = self.builder.integer(Type::I32, 0xFFFF)?;
                let equal = self.builder.compare(Predicate::Eq, mask, all)?;
                let ty = self.context.representation(output)?.result();
                self.builder
                    .convert(Convert::Trunc, ty, equal)
                    .map(Selected::Runtime)
            }
            (operation, _) => {
                let (op, swapped) = match operation {
                    SimdOperation::Eq => (Vector::Pcmpeqb, false),
                    SimdOperation::Gt => (Vector::Pcmpgtb, false),
                    SimdOperation::Lt => (Vector::Pcmpgtb, true),
                    SimdOperation::And => (Vector::Pand, false),
                    SimdOperation::Or => (Vector::Por, false),
                    _ => return Err("source SIMD intrinsic lost its planned operation"),
                };
                let (left, right) = if swapped {
                    (second, first)
                } else {
                    (first, second)
                };
                let right = self.vector_operand(right)?;
                let result = self.vector_input(left, op, Some(right))?;
                self.vector_result(result, output)
            }
        }
    }
    fn bitmask_scalar(
        &mut self,
        id: ir::ValueId,
        output: TypeId,
        mask: Mask,
    ) -> Result<Selected, &'static str> {
        let input = self.input_type(id)?;
        if self.context.layout(input)?.size < mask.lanes * mask.stride {
            return Err("source SIMD mask lanes exceed their native storage");
        }
        let source = self.vector_address(id)?;
        if !mask.array {
            let carrier = self.context.representation(output)?.result();
            return self
                .mask_bits(source, mask.stride, 0..mask.lanes, carrier)
                .map(Selected::Runtime);
        }
        let destination = self
            .allocate(output)?
            .ok_or("source SIMD bitmask has no byte array result storage")?;
        for byte in 0..mask.lanes.div_ceil(8) {
            let first = byte * 8;
            let end = (first + 8).min(mask.lanes);
            let bits = self.mask_bits(source, mask.stride, first..end, Type::U8)?;
            let target = self.offset(destination, byte)?;
            self.builder.store(bits, target.operand)?;
        }
        Ok(Selected::Object(destination))
    }

    fn mask_bits(
        &mut self,
        source: Address,
        stride: u64,
        lanes: std::ops::Range<u64>,
        carrier: Type,
    ) -> Result<Value, &'static str> {
        let mut result = self.builder.integer(carrier, 0)?;
        let one = self.builder.integer(Type::U8, 1)?;
        for (index, lane) in (0_u64..).zip(lanes) {
            // Valid mask lanes are either all zeroes or all ones. On this
            // little-endian target their first byte supplies the low bit.
            let address = self.offset(source, lane * stride)?;
            let loaded = self.builder.load(Type::U8, address.operand)?;
            let bit = self.builder.binary(Binary::And, loaded, one)?;
            let bit = if carrier == Type::U8 {
                bit
            } else {
                self.builder.convert(Convert::Zext, carrier, bit)?
            };
            let distance = self.builder.integer(carrier, u128::from(index))?;
            let positioned = self.builder.binary(Binary::Shl, bit, distance)?;
            result = self.builder.binary(Binary::Or, result, positioned)?;
        }
        Ok(result)
    }

    fn vector_address(&mut self, id: ir::ValueId) -> Result<Address, &'static str> {
        match self.value(id)? {
            Selected::Object(address) => Ok(address),
            _ => Err("source SIMD operand has no vector object"),
        }
    }
    fn vector_operand(&mut self, id: ir::ValueId) -> Result<Value, &'static str> {
        let address = self.vector_address(id)?;
        self.builder.load(Type::V128, address.operand)
    }
    fn vector_input(
        &mut self,
        id: ir::ValueId,
        op: Vector,
        right: Option<Value>,
    ) -> Result<Value, &'static str> {
        let left = self.vector_operand(id)?;
        self.builder.vector(op, left, right, 0)
    }
    fn vector_result(&mut self, value: Value, ty: TypeId) -> Result<Selected, &'static str> {
        let destination = self
            .allocate(ty)?
            .ok_or("source SIMD result has no vector object")?;
        self.builder.store(value, destination.operand)?;
        Ok(Selected::Object(destination))
    }
    /// Output lane `i` is lane `index[i]` of the two inputs placed end to
    /// end; the indices are the constant rustc requires.
    fn shuffle(
        &mut self,
        first: ir::ValueId,
        second: ir::ValueId,
        index: ir::ValueId,
        (bytes, inputs, outputs): (u64, u64, u64),
        ty: TypeId,
    ) -> Result<Selected, &'static str> {
        let indices = self.shuffle_indices(index, outputs)?;
        if indices.iter().any(|&lane| u64::from(lane) >= 2 * inputs) {
            return Err(SHUFFLE_BOUND);
        }
        let sources = [self.vector_address(first)?, self.vector_address(second)?];
        let destination = self
            .allocate(ty)?
            .ok_or("source simd_shuffle result has no vector object")?;
        let lane_type = match bytes {
            1 => Type::U8,
            2 => Type::U16,
            4 => Type::U32,
            8 => Type::I64,
            16 => Type::I128,
            _ => return Err("source simd_shuffle lane width is not an integer width"),
        };
        for (position, lane) in (0_u64..).zip(indices) {
            let lane = u64::from(lane);
            let (source, lane) = if lane < inputs {
                (sources[0], lane)
            } else {
                (sources[1], lane - inputs)
            };
            let from = self.offset(source, lane * bytes)?;
            let value = self.builder.load(lane_type, from.operand)?;
            let to = self.offset(destination, position * bytes)?;
            self.builder.store(value, to.operand)?;
        }
        Ok(Selected::Object(destination))
    }
    /// The evaluated constant index vector of a shuffle.
    fn shuffle_indices(&self, index: ir::ValueId, count: u64) -> Result<Vec<u32>, &'static str> {
        let value = self
            .body
            .values
            .get(index.0 as usize)
            .ok_or("source simd_shuffle index disappeared")?;
        let constants = self.context.constants;
        let constant = match value.kind {
            ir::Rvalue::InlineConstant(source) => constants.inline(source),
            ir::Rvalue::InlineInstance(instance) => constants.inline_instance(instance, value.ty),
            ir::Rvalue::Constant(definition) => constants.definition(definition),
            ir::Rvalue::AssociatedConstant(instance) => constants.associated(instance),
            _ => return Err(SHUFFLE_CONSTANT),
        }
        .ok_or("source simd_shuffle index constant was not evaluated")?;
        let array = constants.field(constant, 0)?;
        let mut indices = Vec::new();
        indices
            .try_reserve_exact(usize::try_from(count).map_err(|_| "shuffle index count overflow")?)
            .map_err(|_| "shuffle index allocation failed")?;
        for lane in 0..count {
            let View::Scalar(Scalar::Integer(bits)) =
                constants.view(constants.field(array, lane)?)?
            else {
                return Err("source simd_shuffle index lane is not an integer");
            };
            indices.push(u32::try_from(bits).map_err(|_| SHUFFLE_BOUND)?);
        }
        Ok(indices)
    }
}
