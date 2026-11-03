//! Physical offsets, tags and union alternatives remain correlated in the value language.
use super::*;
use crate::{abi::layout::Variant, types::aggregate};

#[derive(Clone, Copy)]
struct Field {
    offset: u64,
    size: u64,
    value: LayoutId,
}
impl Describer<'_> {
    pub(super) fn aggregate(
        &mut self,
        ty: TypeId,
        layout: Layout,
        budget: &mut Budget<'_>,
    ) -> Result<LayoutId, &'static str> {
        if !layout.inhabited {
            return self.language.empty(layout.size, budget);
        }
        let union = if let Some(Kind::Adt(item, _)) = self.types.kind(ty) {
            self.definitions
                .get(item)
                .ok_or("missing physical nominal definition")?
                .kind
                == aggregate::Kind::Union
        } else {
            false
        };
        let mut alternatives = Vec::new();
        for (ordinal, variant) in self.layouts.variants(&layout).iter().enumerate() {
            let mut fields = self.fields(variant, budget)?;
            if union {
                for field in fields {
                    if field.offset != 0 {
                        return Err("union field has a nonzero physical offset");
                    }
                    let value = self.product(&[field], layout.size, budget)?;
                    push(&mut alternatives, value, budget)?;
                }
                continue;
            }
            match layout.shape {
                Shape::Aggregate { tag: Some(tag), .. } => {
                    let (_, bits) = tag.ty.integer().ok_or("noninteger physical enum tag")?;
                    let value = self.tag(variant.discriminant, u64::from(bits / 8), budget)?;
                    push(
                        &mut fields,
                        Field {
                            offset: tag.offset,
                            size: u64::from(bits / 8),
                            value,
                        },
                        budget,
                    )?;
                }
                Shape::Niche { .. } => {
                    let niche = self
                        .layouts
                        .niche(&layout)?
                        .ok_or("missing physical niche")?;
                    if ordinal == niche.empty as usize {
                        let size = niche.carrier.bytes();
                        let value = self.tag(0, size, budget)?;
                        push(
                            &mut fields,
                            Field {
                                offset: niche.offset,
                                size,
                                value,
                            },
                            budget,
                        )?;
                    } else if ordinal != niche.payload as usize {
                        return Err("unknown physical niche alternative");
                    }
                }
                Shape::Aggregate { .. } => {}
                _ => return Err("aggregate language requires its actual physical field layout"),
            }
            let value = self.product(&fields, layout.size, budget)?;
            push(&mut alternatives, value, budget)?;
        }
        if alternatives.is_empty() {
            self.language.empty(layout.size, budget)
        } else {
            self.language.alternatives(alternatives, budget)
        }
    }
    fn fields(
        &self,
        variant: &Variant,
        budget: &mut Budget<'_>,
    ) -> Result<Vec<Field>, &'static str> {
        let types = self
            .types
            .list(variant.fields)
            .ok_or("missing physical field types")?;
        let offsets = self.layouts.offsets(variant);
        if types.len() != offsets.len() {
            return Err("physical fields and offsets disagree");
        }
        let mut fields = Vec::new();
        for (&ty, &offset) in types.iter().zip(offsets) {
            push(
                &mut fields,
                Field {
                    offset,
                    size: self.layout(ty)?.size,
                    value: self.child(ty)?,
                },
                budget,
            )?;
        }
        Ok(fields)
    }
    fn product(
        &mut self,
        fields: &[Field],
        size: u64,
        budget: &mut Budget<'_>,
    ) -> Result<LayoutId, &'static str> {
        let mut ordered = Vec::new();
        for &field in fields {
            push(&mut ordered, field, budget)?;
        }
        budget.charge(ordered.len().saturating_mul(usize::BITS as usize + 1), 0)?;
        // Empty fields may share an address with a nonempty field, but keep their guards.
        ordered.sort_unstable_by_key(|field| (field.offset, field.size != 0));
        let mut cursor = 0;
        let mut result = self.language.epsilon(budget)?;
        for field in ordered {
            let padding = field
                .offset
                .checked_sub(cursor)
                .ok_or("overlapping physical value fields")?;
            let gap = self.bytes(ByteRange::UNINITIALIZED, padding, budget)?;
            result = self.language.sequence(result, gap, budget)?;
            result = self.language.sequence(result, field.value, budget)?;
            cursor = field
                .offset
                .checked_add(field.size)
                .ok_or("physical field endpoint overflow")?;
        }
        let padding = size
            .checked_sub(cursor)
            .ok_or("field extends beyond its physical aggregate")?;
        let tail = self.bytes(ByteRange::UNINITIALIZED, padding, budget)?;
        self.language.sequence(result, tail, budget)
    }
    fn tag(
        &mut self,
        bits: u128,
        size: u64,
        budget: &mut Budget<'_>,
    ) -> Result<LayoutId, &'static str> {
        let width = usize::try_from(size).map_err(|_| "physical tag width overflow")?;
        let bytes = bits.to_le_bytes();
        let bytes = bytes
            .get(..width)
            .ok_or("physical tag exceeds target scalar width")?;
        let mut result = self.language.epsilon(budget)?;
        for &value in bytes {
            let start = u16::from(value);
            let byte = self
                .language
                .byte(ByteRange::new(start, start + 1)?, budget)?;
            result = self.language.sequence(result, byte, budget)?;
        }
        Ok(result)
    }
}
