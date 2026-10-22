//! Memoize zero-filled products without expanding repeats or following pointers.
use super::{Context, initializer::bits};
use crate::{
    abi::layout::Shape,
    sema::values::{Constant, View},
    types::TypeId,
};
impl Context<'_> {
    fn zero_image(
        &mut self,
        ty: TypeId,
        value: Constant,
        size: u64,
        layout_size: u64,
    ) -> Result<bool, &'static str> {
        self.constants.require_publishable_array(value)?;
        let image = self.constants.image(value)?;
        if image.ty != ty || size != layout_size || image.bytes.len() as u64 != size {
            return Err("constant union image differs from zero storage type");
        }
        if !image.fragments.is_empty() {
            return Ok(false);
        }
        self.data.charge(image.bytes.len())?;
        Ok(image
            .bytes
            .iter()
            .enumerate()
            .all(|(i, byte)| !image.initialized(i) || *byte == 0))
    }
    fn zero_bytes(&mut self, ty: TypeId, value: Constant) -> Result<bool, &'static str> {
        let bytes = self.constants.stored_bytes(ty, value)?;
        self.data.charge(bytes.len())?;
        Ok(bytes.iter().all(|&byte| byte == 0))
    }
    fn nonzero_enum_tag(
        &self,
        layout: &crate::abi::layout::Layout,
        value: Constant,
    ) -> Result<bool, &'static str> {
        if !matches!(layout.shape, Shape::Aggregate { tag: Some(_), .. }) {
            return Ok(false);
        }
        self.layouts
            .variants(layout)
            .get(self.constant_variant(value)?)
            .map(|variant| variant.discriminant != 0)
            .ok_or("missing constant variant layout")
    }
    fn nullable_zero(
        &self,
        layout: &crate::abi::layout::Layout,
        value: Constant,
        length: u64,
    ) -> Result<bool, &'static str> {
        let Some(niche) = self.layouts.niche(layout)? else {
            return Ok(false);
        };
        let ordinal = self.constant_variant(value)?;
        if ordinal == niche.empty as usize && length == 0 {
            return Ok(true);
        }
        if ordinal != niche.payload as usize || length != 1 {
            return Err("nullable zero test differs from its canonical variant");
        }
        Ok(false)
    }
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
                View::Image { size } => self.zero_image(ty, value, size, layout.size)?,
                View::Reference(_)
                | View::Function(_)
                | View::Text(_)
                | View::Bytes(_)
                | View::BytePointer { .. } => false,
                View::ByteRange { .. } => self.zero_bytes(ty, value)?,
                View::Address { bits } => {
                    bits == 0
                        && matches!(
                            value.metadata(),
                            None | Some(crate::sema::values::Metadata::Length(0))
                        )
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
                    if self.nonzero_enum_tag(&layout, value)? {
                        self.data.zeroes.insert(key, false);
                        continue;
                    }
                    if self.nullable_zero(&layout, value, length)? {
                        self.data.zeroes.insert(key, true);
                        continue;
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
