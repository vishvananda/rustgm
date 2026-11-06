//! Whole-value transfers consume checked uses and prepared public-layout recipes.
use super::{Address, Lower, Selected};
use crate::{
    body::ir::ValueId,
    lowir::{program::MemorySpan, vocabulary::Type},
    types::{
        Kind, TypeId,
        unsizing::{CoercionPlan, CoercionSpan, CoercionStep},
    },
};

impl Lower<'_, '_, '_> {
    pub(super) fn nominal_coercion(
        &mut self,
        operation: ValueId,
        operand: ValueId,
        source: TypeId,
        target: TypeId,
    ) -> Result<Selected, &'static str> {
        let row = self.body.nominal_use(self.context.types, operation)?;
        let plan = self
            .context
            .constants
            .nominal_conversion(row.source, row.from, row.to)?;
        if (plan.source, plan.target) != (source, target) {
            return Err("nominal runtime conversion changed its actual application");
        }
        let input = self.value(operand)?;
        let from = if let Selected::Object(address) = input {
            address
        } else {
            let address = self
                .allocate(source)?
                .ok_or("nominal input has no actual storage")?;
            self.write(Some(address), input, source)?;
            address
        };
        let destination = self
            .allocate(target)?
            .ok_or("nominal result has no actual storage")?;
        let mut next = Some((plan, from, destination));
        let mut steps = 0_usize;
        while let Some((plan, from, to)) = next.take() {
            steps = steps.saturating_add(plan.fields.len()).saturating_add(1);
            if steps > 1_000_000 {
                return Err("nominal runtime conversion work budget exceeded");
            }
            for (index, &span) in plan.fields.iter().enumerate() {
                self.validate_transfer_span(plan, index, span)?;
                let a = self.offset(from, span.from)?;
                let b = self.offset(to, span.to)?;
                if index == plan.changed as usize {
                    match plan.step {
                        CoercionStep::Slice(array) => self.transfer_slice(span, array, a, b)?,
                        CoercionStep::Nested { source, target } => {
                            if (span.source, span.target) != (source, target) {
                                return Err("nested nominal transfer changed its actual field");
                            }
                            next =
                                Some((self.context.constants.nominal_plan(source, target)?, a, b));
                        }
                    }
                } else if span.bytes != 0 {
                    let source = self.pointer(a)?;
                    let target = self.pointer(b)?;
                    self.builder.copy_object(
                        MemorySpan {
                            bytes: span.bytes,
                            align: a.align.min(b.align),
                        },
                        source,
                        target,
                    )?;
                }
            }
        }
        Ok(Selected::Object(destination))
    }
    fn validate_transfer_span(
        &mut self,
        plan: &CoercionPlan,
        index: usize,
        span: CoercionSpan,
    ) -> Result<(), &'static str> {
        let a = self
            .context
            .aggregates
            .shape(self.context.types, plan.source)?;
        let b = self
            .context
            .aggregates
            .shape(self.context.types, plan.target)?;
        if a.definition.item != b.definition.item
            || a.definition.kind != crate::types::aggregate::Kind::Struct
            || a.variants().len() != 1
            || b.variants().len() != 1
        {
            return Err("nominal runtime plan changed its public struct owner");
        }
        let fields = |shape: crate::types::aggregate::Shape<'_>| {
            shape
                .variant(0)
                .and_then(|variant| self.context.types.list(variant.fields))
        };
        let a = fields(a).ok_or("nominal public source fields are missing")?;
        let b = fields(b).ok_or("nominal public target fields are missing")?;
        if a.len() != plan.fields.len()
            || b.len() != plan.fields.len()
            || a.get(index) != Some(&span.source)
            || b.get(index) != Some(&span.target)
        {
            return Err("nominal runtime span changed its public field identity");
        }
        let source = self.context.layout(plan.source)?;
        let target = self.context.layout(plan.target)?;
        let a = self.context.layout(span.source)?;
        let b = self.context.layout(span.target)?;
        if self.field_offset(&source, 0, index)? != span.from
            || self.field_offset(&target, 0, index)? != span.to
            || a.size != span.bytes
            || a.align.min(b.align) != span.align
            || span
                .from
                .checked_add(a.size)
                .is_none_or(|end| end > source.size)
            || span
                .to
                .checked_add(b.size)
                .is_none_or(|end| end > target.size)
            || (index != plan.changed as usize && a.size != b.size)
        {
            return Err("nominal runtime span changed its actual instantiated layout");
        }
        Ok(())
    }
    fn transfer_slice(
        &mut self,
        span: CoercionSpan,
        array: TypeId,
        source: Address,
        target: Address,
    ) -> Result<(), &'static str> {
        let pointee = |ty| match self.context.types.kind(ty) {
            Some(Kind::Reference { referent, .. }) => Some(referent),
            Some(Kind::Pointer { pointee, .. }) => Some(pointee),
            _ => None,
        };
        if pointee(span.source) != Some(array)
            || pointee(span.target)
                .is_none_or(|ty| !matches!(self.context.types.kind(ty), Some(Kind::Slice(_))))
            || self.context.layout(span.source)?.size != 8
            || self.context.layout(span.target)?.size != 16
        {
            return Err("nominal slice transfer changed its actual pointer terminal");
        }
        let Some(Kind::Array { length, .. }) = self.context.types.kind(array) else {
            return Err("nominal slice transfer lost its actual terminal array");
        };
        let data = self.builder.load(Type::Ptr, source.operand)?;
        let length = self.builder.integer(Type::I64, u128::from(length))?;
        self.builder.store(data, target.operand)?;
        let metadata = self.offset(target, 8)?;
        self.builder.store(length, metadata.operand)
    }
}
