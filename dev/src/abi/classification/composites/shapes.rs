//! Validate every nested representation, including fields of objects passed in memory.
use super::{Class, Field, Kind, Node, Query, TypeId};
use crate::support::hash::HashSet;
use crate::{
    abi::layout::{Layout, Shape},
    types::{Kind as Type, aggregate::Representation},
};
enum Work {
    Enter(TypeId),
    Finish(TypeId, Node),
}
impl Query<'_> {
    pub(super) fn validate(&mut self, root: TypeId) -> Result<(), &'static str> {
        let mut work = vec![Work::Enter(root)];
        let mut active = HashSet::default();
        while let Some(next) = work.pop() {
            self.charge(1)?;
            match next {
                Work::Enter(ty) => {
                    if self.cache.nodes.contains_key(&ty) {
                        continue;
                    }
                    if !active.insert(ty) {
                        return Err("cycle in C aggregate field graph");
                    }
                    let node = self.node(ty)?;
                    self.retain(size_of::<Node>() + size_of::<TypeId>() + 48)?;
                    let children = match &node.kind {
                        Kind::Scalar(_) | Kind::Vector => Vec::new(),
                        Kind::Array { element, .. } => vec![*element],
                        Kind::Struct(fields) => fields.iter().map(|field| field.ty).collect(),
                    };
                    work.push(Work::Finish(ty, node));
                    work.extend(children.into_iter().rev().map(Work::Enter));
                }
                Work::Finish(ty, node) => {
                    active.remove(&ty);
                    self.cache.nodes.insert(ty, node);
                }
            }
        }
        Ok(())
    }
    fn node(&mut self, ty: TypeId) -> Result<Node, &'static str> {
        let layout = *self
            .layouts
            .get(ty)
            .ok_or("missing C aggregate field layout")?;
        if !layout.align.is_power_of_two() {
            return Err("invalid C aggregate alignment");
        }
        let kind = self
            .types
            .kind(ty)
            .ok_or("missing C aggregate field type")?;
        let kind = if super::super::scalar(kind)?
            && matches!(layout.shape, Shape::Scalar)
            && layout.size != 0
        {
            Kind::Scalar(if matches!(kind,Type::Primitive(p) if p.is_float()) {
                Class::Sse
            } else {
                Class::Integer
            })
        } else if self.carrier_word(ty, layout)? {
            Kind::Scalar(Class::Integer)
        } else if matches!(kind, Type::Array { .. } | Type::ArrayTerm { .. }) {
            self.array(layout)?
        } else if let Type::Adt(item, _) = kind {
            let definition = self
                .definitions
                .get(item)
                .ok_or("missing nested C aggregate declaration")?;
            if definition.repr.representation == Representation::Simd {
                if !matches!(
                    self.definitions.simd_shape(self.types, ty)?,
                    crate::types::aggregate::SimdShape::Ready { .. }
                ) {
                    return Err("C SIMD boundary requires completed vector fields");
                }
                if layout.size != 16 || layout.align != 16 {
                    return Err("C SIMD boundary supports only aligned 128-bit vectors");
                }
                Kind::Vector
            } else {
                if definition.kind != crate::types::aggregate::Kind::Struct
                    || definition.repr.representation != Representation::C
                    || definition.repr.packed.is_some()
                    || layout.size == 0
                {
                    return Err("C aggregate ABI profile requires nonempty aligned repr(C) fields");
                }
                self.fields(layout)?
            }
        } else {
            return Err("C aggregate ABI profile requires scalar, array or repr(C) fields");
        };
        Ok(Node {
            bytes: layout.size,
            kind,
        })
    }
    fn carrier_word(&mut self, ty: TypeId, layout: Layout) -> Result<bool, &'static str> {
        if let Some(niche) = self.layouts.niche(&layout)? {
            if niche.owner != ty {
                return Err("nullable C field belongs to another type");
            }
            return Ok(true);
        }
        let Some(Type::Adt(item, _)) = self.types.kind(ty) else {
            return Ok(false);
        };
        if self
            .definitions
            .get(item)
            .is_some_and(|definition| definition.repr.representation == Representation::Transparent)
        {
            return self
                .layouts
                .scalar_carrier(self.types, self.definitions, ty)
                .map(|carrier| carrier.is_some());
        }
        Ok(false)
    }
    fn array(&mut self, layout: Layout) -> Result<Kind, &'static str> {
        let Shape::Array {
            element,
            stride,
            length,
        } = layout.shape
        else {
            return Err("C array field has no checked array layout");
        };
        let element = self.types.storage_head(element)?;
        let child = *self
            .layouts
            .get(element)
            .ok_or("missing C array element layout")?;
        if stride != child.size
            || layout.align != child.align
            || stride.checked_mul(length) != Some(layout.size)
        {
            return Err("C array field span differs from element layout");
        }
        if length != 0 {
            self.field(layout, element, 0)?;
        }
        Ok(Kind::Array {
            element,
            stride,
            length,
        })
    }
    fn fields(&mut self, layout: Layout) -> Result<Kind, &'static str> {
        let variants = self.layouts.variants(&layout);
        let [variant] = variants else {
            return Err("C struct field has invalid variant layout");
        };
        let fields = self
            .types
            .list(variant.fields)
            .ok_or("missing C struct field types")?;
        let offsets = self.layouts.offsets(variant);
        if fields.len() != offsets.len() {
            return Err("C struct field offsets differ from types");
        }
        super::super::storage(
            self.retained,
            fields
                .len()
                .checked_mul(size_of::<Field>())
                .ok_or("C field storage overflow")?,
        )?;
        let mut children = Vec::with_capacity(fields.len());
        for (index, &ty) in fields.iter().enumerate() {
            super::super::account_work(self.steps, 1)?;
            let ty = self.types.storage_head(ty)?;
            let offset = offsets[index];
            self.field(layout, ty, offset)?;
            children.push(Field { ty, offset });
        }
        Ok(Kind::Struct(children.into_boxed_slice()))
    }
    fn field(&self, parent: Layout, ty: TypeId, offset: u64) -> Result<(), &'static str> {
        let child = self
            .layouts
            .get(ty)
            .ok_or("missing nested C field layout")?;
        if !child.align.is_power_of_two()
            || !parent.align.is_multiple_of(child.align)
            || !offset.is_multiple_of(u64::from(child.align))
        {
            return Err("C aggregate ABI field is not naturally aligned");
        }
        if offset
            .checked_add(child.size)
            .is_none_or(|end| end > parent.size)
        {
            return Err("C aggregate ABI field exceeds object span");
        }
        Ok(())
    }
}
