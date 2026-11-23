//! Emit fixed-width integer lanes with the established scalar `LowIR` operations.
use super::{Address, Lower, Plan, Refusal, Selected, SimdOperation, Type, TypeId, ir};
use crate::lowir::vocabulary::{Binary, Convert, Predicate, Unary};

pub(super) struct ScalarLanes {
    count: u64,
    ty: Type,
    result: Type,
}

impl ScalarLanes {
    pub(super) fn plan(count: u64, bits: u16, result_bits: u16) -> Result<Plan, Refusal> {
        if count > 128 {
            return Err(Refusal::Fenced);
        }
        Ok(Plan::Scalar(Self {
            count,
            ty: integer_lane(bits)?,
            result: integer_lane(result_bits)?,
        }))
    }

    pub(super) fn with_byte_vector(
        count: u64,
        bits: u16,
        result_bits: u16,
    ) -> Result<Plan, Refusal> {
        if count == 16 && bits == 8 && result_bits == 8 {
            return Ok(Plan::Vector);
        }
        Self::plan(count, bits, result_bits)
    }

    fn stride(&self) -> u64 {
        u64::from(self.ty.bits() / 8)
    }

    fn result_stride(&self) -> u64 {
        u64::from(self.result.bits() / 8)
    }
}

fn integer_lane(bits: u16) -> Result<Type, Refusal> {
    Ok(match bits {
        8 => Type::I8,
        16 => Type::I16,
        32 => Type::I32,
        64 => Type::I64,
        128 => Type::I128,
        _ => return Err(Refusal::Fenced),
    })
}

impl Lower<'_, '_, '_> {
    pub(super) fn scalar_lanes(
        &mut self,
        operation: SimdOperation,
        output: TypeId,
        lanes: &ScalarLanes,
        first: ir::ValueId,
        second: ir::ValueId,
    ) -> Result<Selected, &'static str> {
        match operation {
            SimdOperation::Splat => self.scalar_splat(output, lanes, first),
            SimdOperation::ReduceAll => self.scalar_reduce(output, lanes, first),
            SimdOperation::Eq | SimdOperation::And | SimdOperation::Or => {
                self.scalar_lane_pairs(operation, output, lanes, [first, second])
            }
            _ => Err("source SIMD operation has no scalar lane lowering"),
        }
    }

    fn scalar_vector(
        &mut self,
        input: ir::ValueId,
        lanes: &ScalarLanes,
    ) -> Result<Address, &'static str> {
        let ty = self.input_type(input)?;
        self.require_lane_storage(ty, lanes.count * lanes.stride())?;
        self.vector_address(input)
    }

    fn scalar_result(&mut self, ty: TypeId, lanes: &ScalarLanes) -> Result<Address, &'static str> {
        self.require_lane_storage(ty, lanes.count * lanes.result_stride())?;
        self.allocate(ty)?
            .ok_or("source SIMD result has no scalar lane storage")
    }

    fn require_lane_storage(&mut self, ty: TypeId, payload: u64) -> Result<(), &'static str> {
        if self.context.layout(ty)?.size < payload {
            return Err("source SIMD lanes exceed their native storage");
        }
        Ok(())
    }

    fn scalar_splat(
        &mut self,
        output: TypeId,
        lanes: &ScalarLanes,
        input: ir::ValueId,
    ) -> Result<Selected, &'static str> {
        let value = self.value(input)?.runtime()?;
        if value.ty.bits() != lanes.ty.bits() || !value.ty.is_integer() {
            return Err("source SIMD splat input differs from its integer lane");
        }
        let destination = self.scalar_result(output, lanes)?;
        for lane in 0..lanes.count {
            let address = self.offset(destination, lane * lanes.result_stride())?;
            self.builder.store(value, address.operand)?;
        }
        Ok(Selected::Object(destination))
    }

    fn scalar_reduce(
        &mut self,
        output: TypeId,
        lanes: &ScalarLanes,
        input: ir::ValueId,
    ) -> Result<Selected, &'static str> {
        let source = self.scalar_vector(input, lanes)?;
        let zero = self.builder.integer(lanes.ty, 0)?;
        let mut all = self.builder.integer(Type::I64, 1)?;
        for lane in 0..lanes.count {
            let from = self.offset(source, lane * lanes.stride())?;
            let value = self.builder.load(lanes.ty, from.operand)?;
            // As with pmovmskb, the sign bit expresses the mask truth value.
            let sign = self.builder.compare(Predicate::Lt, value, zero)?;
            all = self.builder.binary(Binary::And, all, sign)?;
        }
        let ty = self.context.representation(output)?.result();
        self.builder
            .convert(Convert::Trunc, ty, all)
            .map(Selected::Runtime)
    }

    fn scalar_lane_pairs(
        &mut self,
        operation: SimdOperation,
        output: TypeId,
        lanes: &ScalarLanes,
        [first, second]: [ir::ValueId; 2],
    ) -> Result<Selected, &'static str> {
        let first = self.scalar_vector(first, lanes)?;
        let second = self.scalar_vector(second, lanes)?;
        let destination = self.scalar_result(output, lanes)?;
        for lane in 0..lanes.count {
            let offset = lane * lanes.stride();
            let left = self.offset(first, offset)?;
            let right = self.offset(second, offset)?;
            let left = self.builder.load(lanes.ty, left.operand)?;
            let right = self.builder.load(lanes.ty, right.operand)?;
            let value = match operation {
                SimdOperation::Eq => {
                    let equal = self.builder.compare(Predicate::Eq, left, right)?;
                    let bit = crate::lower::casts::integer(
                        &mut self.builder,
                        equal,
                        lanes.result,
                        false,
                    )?;
                    self.builder.unary(Unary::Neg, bit)?
                }
                SimdOperation::And => self.builder.binary(Binary::And, left, right)?,
                SimdOperation::Or => self.builder.binary(Binary::Or, left, right)?,
                _ => return Err("source SIMD lane pair has no scalar operation"),
            };
            let location = self.offset(destination, lane * lanes.result_stride())?;
            self.builder.store(value, location.operand)?;
        }
        Ok(Selected::Object(destination))
    }
}
