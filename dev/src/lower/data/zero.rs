//! Memoize zero-filled products without expanding repeats or following pointers.
use super::{Context, initializer::bits};
use crate::{
    abi::layout::Shape,
    sema::values::{Constant, View},
    types::TypeId,
};
impl Context<'_> {
    pub(super) fn zero_data(&mut self, ty: TypeId, value: Constant) -> Result<bool, &'static str> {
        let root = (ty, value);
        let mut work = vec![(root, false)];
        while let Some((key @ (ty, value), finish)) = work.pop() {
            self.data.charge(1)?;
            if self.data.zeroes.contains_key(&key) {
                continue;
            }
            let layout = self.layout(ty)?;
            if layout.size == 0 {
                self.data.zeroes.insert(key, true);
                continue;
            }
            let zero = match self.constants.view(value)? {
                View::Scalar(scalar) => bits(scalar) == 0,
                View::Reference(_)
                | View::Function(_)
                | View::Text(_)
                | View::Bytes(_)
                | View::BytePointer { .. } => false,
                View::Address { bits } => {
                    bits == 0 && value.length().is_none_or(|length| length == 0)
                }
                View::Repeat {
                    value: repeated,
                    length,
                } => {
                    let Shape::Array {
                        element,
                        length: actual,
                        ..
                    } = layout.shape
                    else {
                        return Err("constant repeat has no array layout");
                    };
                    if actual != length {
                        return Err("constant repeat length differs from layout");
                    }
                    if !finish {
                        work.push((key, true));
                        work.push(((element, repeated), false));
                        continue;
                    }
                    *self
                        .data
                        .zeroes
                        .get(&(element, repeated))
                        .ok_or("missing zero-repeat dependency")?
                }
                View::Product { length, .. } => {
                    if let Shape::Aggregate { tag: Some(_), .. } = layout.shape {
                        let variant = self
                            .layouts
                            .variants(&layout)
                            .get(self.constant_variant(value)?)
                            .ok_or("missing constant variant layout")?;
                        if variant.discriminant != 0 {
                            self.data.zeroes.insert(key, false);
                            continue;
                        }
                    }
                    self.data.charge(
                        usize::try_from(length).map_err(|_| "zero product exceeds host size")?,
                    )?;
                    if !finish {
                        work.push((key, true));
                    }
                    let mut zero = true;
                    for index in 0..length {
                        let (_, ty) = self.constant_field(&layout, Some(value), index)?;
                        let child = (ty, self.constants.field(value, index)?);
                        if finish {
                            zero &= *self
                                .data
                                .zeroes
                                .get(&child)
                                .ok_or("missing zero-product dependency")?;
                        } else {
                            work.push((child, false));
                        }
                    }
                    if !finish {
                        continue;
                    }
                    zero
                }
            };
            self.data.zeroes.insert(key, zero);
        }
        self.data
            .zeroes
            .get(&root)
            .copied()
            .ok_or("missing zero-data query result")
    }
}
