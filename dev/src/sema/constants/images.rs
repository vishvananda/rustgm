//! Immutable scalar union storage has initialized bytes and no active member.
pub(super) mod scalars;
use super::{
    Value,
    values::{ImageFragment, WordTarget},
};
use crate::types::TypeId;
use std::collections::HashMap;

pub(super) const MAX_IMAGE: usize = 65_536;
const MAX_RETAINED: usize = 64 * 1024 * 1024;
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(in crate::sema) struct Id(u32);
pub(super) struct Image {
    pub ty: TypeId,
    pub bytes: Box<[u8]>,
    pub initialized: Box<[u64]>,
    pub fragments: Box<[ImageFragment]>,
}
pub(super) struct Family {
    pub start: u32,
    pub size: usize,
    pub scalar: bool,
    pub fields: Box<[TypeId]>,
}
#[derive(Default)]
pub(in crate::sema) struct Images {
    nodes: Vec<Image>,
    pub(super) families: HashMap<TypeId, Family>,
    retained: usize,
}
impl Images {
    #[cfg(test)]
    pub(in crate::sema::constants) fn retained_for_test(&self) -> usize {
        self.retained
    }

    fn reserve(&mut self, bytes: usize) -> Result<(), &'static str> {
        self.retained = self
            .retained
            .checked_add(bytes)
            .filter(|n| *n <= MAX_RETAINED)
            .ok_or("constant union image storage exceeds 64 MiB budget")?;
        Ok(())
    }
    pub(super) fn reserve_family(&mut self, fields: usize) -> Result<(), &'static str> {
        // Cover temporary Vec plus final Box and conservative hash capacity before
        // constructing the field buffer. Each family charges spare index/header space.
        let bytes = fields
            .checked_mul(2 * size_of::<TypeId>())
            .and_then(|n| n.checked_add(256 + 4 * size_of::<Family>()))
            .ok_or("constant union family storage overflow")?;
        self.reserve(bytes)?;
        self.families
            .try_reserve(1)
            .map_err(|_| "constant union family allocation failed")
    }
    #[cfg(test)]
    pub(super) fn reserve_image(&mut self, size: usize) -> Result<(), &'static str> {
        self.reserve_image_fragments(size, 0)
    }
    pub(super) fn reserve_image_fragments(
        &mut self,
        size: usize,
        fragments: usize,
    ) -> Result<(), &'static str> {
        self.snapshot_capacity::<true>(size, fragments)
    }
    pub(super) fn reserve_array_fragments(
        &mut self,
        size: usize,
        fragments: usize,
    ) -> Result<(), &'static str> {
        self.snapshot_capacity::<false>(size, fragments)
    }
    fn snapshot_capacity<const UNION: bool>(
        &mut self,
        size: usize,
        fragments: usize,
    ) -> Result<(), &'static str> {
        if UNION && size > MAX_IMAGE {
            return Err("constant union image exceeds 64 KiB limit");
        }
        u32::try_from(self.nodes.len()).map_err(|_| "constant union image identity exhausted")?;
        if fragments > size {
            return Err("constant union has more fragments than bytes");
        }
        let words = size.div_ceil(64);
        // Both cloning buffers and their immutable publication are covered, as
        // are the arena's spare capacity and the first small Vec allocation.
        let bytes = size
            .checked_mul(2)
            .and_then(|n| n.checked_add(words * 2 * size_of::<u64>()))
            .and_then(|n| n.checked_add(4 * size_of::<Image>()))
            .and_then(|n| {
                fragments
                    .checked_mul(2 * size_of::<ImageFragment>())
                    .and_then(|extra| n.checked_add(extra))
            })
            .ok_or("constant union image storage overflow")?;
        self.reserve(bytes)?;
        self.nodes
            .try_reserve(1)
            .map_err(|_| "constant union image allocation failed")
    }
    pub(super) fn insert(&mut self, image: Image) -> Result<Value, &'static str> {
        self.insert_snapshot::<true>(image)
    }
    pub(super) fn insert_array(&mut self, image: Image) -> Result<Value, &'static str> {
        self.insert_snapshot::<false>(image)
    }
    fn insert_snapshot<const UNION: bool>(&mut self, image: Image) -> Result<Value, &'static str> {
        if (UNION && image.bytes.len() > MAX_IMAGE)
            || image.initialized.len() != image.bytes.len().div_ceil(64)
        {
            return Err("malformed constant union image");
        }
        Self::validate_fragments(&image)?;
        let id = Id(u32::try_from(self.nodes.len())
            .map_err(|_| "constant union image identity exhausted")?);
        self.nodes.push(image);
        Ok(Value::Image(id))
    }
    fn validate_fragments(image: &Image) -> Result<(), &'static str> {
        let mut previous_end = 0;
        for fragment in &image.fragments {
            let end = fragment
                .offset
                .checked_add(u64::from(fragment.length))
                .filter(|&end| end <= image.bytes.len() as u64)
                .ok_or("constant image fragment exceeds its storage")?;
            if fragment.length == 0
                || fragment.offset < previous_end
                || u16::from(fragment.source_offset) + u16::from(fragment.length) > 8
            {
                return Err("constant image fragment has invalid word coverage");
            }
            for index in fragment.offset..end {
                let index =
                    usize::try_from(index).map_err(|_| "image fragment exceeds host size")?;
                if image.initialized[index / 64] & (1u64 << (index % 64)) == 0 {
                    return Err("constant symbolic fragment contains uninitialized bytes");
                }
            }
            if let WordTarget::Value(value) = fragment.target {
                match value.0 {
                    Value::Reference(pointer) if pointer.metadata.is_none() => {}
                    Value::Function(_) => {}
                    _ => return Err("constant image fragment has invalid symbolic word"),
                }
            }
            previous_end = end;
        }
        Ok(())
    }
    pub(super) fn get(&self, id: Id) -> Result<&Image, &'static str> {
        self.nodes
            .get(id.0 as usize)
            .ok_or("missing constant union image")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Primitive;
    #[test]
    fn storage_limits_precede_capacity_and_payload_allocation() -> Result<(), &'static str> {
        let mut arena = Images::default();
        assert_eq!(
            arena.reserve_image(MAX_IMAGE + 1),
            Err("constant union image exceeds 64 KiB limit")
        );
        assert_eq!(arena.nodes.capacity(), 0);
        arena.retained = MAX_RETAINED;
        assert_eq!(
            arena.reserve_image(16),
            Err("constant union image storage exceeds 64 MiB budget")
        );
        assert_eq!(arena.nodes.capacity(), 0);
        assert_eq!(
            arena.reserve_family(1),
            Err("constant union image storage exceeds 64 MiB budget")
        );
        assert_eq!(arena.families.capacity(), 0);
        assert!(arena.reserve_family(usize::MAX).is_err());
        let mut normal = Images::default();
        normal.reserve_image(MAX_IMAGE)?;
        assert!(normal.retained >= 2 * MAX_IMAGE + 2 * MAX_IMAGE.div_ceil(64) * size_of::<u64>());
        Ok(())
    }
    #[test]
    fn initialization_and_scalar_validity_are_separate() -> Result<(), &'static str> {
        use crate::sema::model::Scalar;
        assert_eq!(
            scalars::decode(Primitive::U32, &[1, 0, 0, 0], &[1]),
            Err("constant union scalar read uses uninitialized bytes")
        );
        assert_eq!(
            scalars::decode(Primitive::Bool, &[2], &[1]),
            Err("constant union scalar read has invalid bool representation")
        );
        assert_eq!(
            scalars::decode(Primitive::U8, &[2], &[1])?,
            Scalar::Integer(2)
        );
        assert_eq!(
            scalars::decode(Primitive::Char, &[0, 216, 0, 0], &[15]),
            Err("constant union scalar read has invalid char representation")
        );
        let mut bytes = [0; 16];
        bytes[15] = 128;
        assert_eq!(
            scalars::decode(Primitive::U128, &bytes, &[65535])?,
            Scalar::Integer(1u128 << 127)
        );
        Ok(())
    }
}
