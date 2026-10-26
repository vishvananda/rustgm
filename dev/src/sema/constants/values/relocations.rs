//! Symbolic words retain provenance independently of their storage encoding.
mod addresses;
use super::Constant;
use crate::types::TypeId;
pub(crate) use addresses::WordAddress;
/// A checked pointer/function word or an existing canonical object vtable.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum WordTarget {
    Value(Constant),
    Vtable { concrete: TypeId, target: TypeId },
}
/// A byte interval copied from one symbolic eight-byte target word.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ImageFragment {
    pub offset: u64,
    pub source_offset: u8,
    pub length: u8,
    pub target: WordTarget,
}
impl ImageFragment {
    /// Clip storage coverage without changing symbolic identity or word coordinates.
    pub(crate) fn clipped(self, start: u64, end: u64) -> Result<Self, &'static str> {
        let from = start.max(self.offset);
        let to = end.min(
            self.offset
                .checked_add(u64::from(self.length))
                .ok_or("symbolic fragment extent overflow")?,
        );
        if from >= to {
            return Err("symbolic fragment has no selected coverage");
        }
        let skipped =
            u8::try_from(from - self.offset).map_err(|_| "symbolic fragment source overflow")?;
        Ok(Self {
            offset: from
                .checked_sub(start)
                .ok_or("symbolic fragment precedes selection")?,
            source_offset: self
                .source_offset
                .checked_add(skipped)
                .ok_or("symbolic fragment source overflow")?,
            length: u8::try_from(to - from).map_err(|_| "symbolic fragment length overflow")?,
            target: self.target,
        })
    }
}

#[derive(Clone, Copy)]
pub(crate) enum MetadataShape {
    Thin,
    Length,
    Object,
}
#[derive(Clone, Copy)]
pub(crate) enum WordShape {
    Function,
    Data {
        pointee: TypeId,
        reference: Option<bool>,
        metadata: MetadataShape,
    },
}
impl super::Reader<'_> {
    /// The byte storage of `str` has the same element validity as `[u8]`.
    pub(crate) fn sequence_element(self, ty: TypeId) -> Result<TypeId, &'static str> {
        use crate::types::{Kind, Primitive};
        match self.facts.types.kind(ty) {
            Some(Kind::Slice(element)) => Ok(element),
            Some(Kind::Primitive(Primitive::Str)) => Ok(TypeId::primitive(Primitive::U8)),
            _ => Err("symbolic sequence requires a slice or str type"),
        }
    }
    pub(crate) fn word_shape(self, ty: TypeId) -> Result<WordShape, &'static str> {
        use crate::{abi::layout::Shape, types::Kind};
        let shape = match self.facts.types.kind(ty) {
            Some(Kind::Function { .. }) => WordShape::Function,
            Some(Kind::Pointer { pointee, .. }) => WordShape::Data {
                pointee,
                reference: None,
                metadata: self.word_metadata(pointee),
            },
            Some(Kind::Reference {
                referent, mutable, ..
            }) => WordShape::Data {
                pointee: referent,
                reference: Some(mutable),
                metadata: self.word_metadata(referent),
            },
            _ => return Err("constant symbolic word requires a pointer or function type"),
        };
        let layout = self
            .facts
            .layouts
            .get(ty)
            .ok_or("missing prepared pointer word layout")?;
        let wide = matches!(
            shape,
            WordShape::Data {
                metadata: MetadataShape::Length | MetadataShape::Object,
                ..
            }
        );
        if (wide && (!matches!(layout.shape, Shape::WidePointer) || layout.size != 16))
            || (!wide && (!matches!(layout.shape, Shape::Scalar) || layout.size != 8))
        {
            return Err("symbolic word type differs from its retained pointer layout");
        }
        Ok(shape)
    }
    fn word_metadata(self, ty: TypeId) -> MetadataShape {
        match self.facts.types.kind(ty) {
            Some(
                crate::types::Kind::Slice(_)
                | crate::types::Kind::Primitive(crate::types::Primitive::Str),
            ) => MetadataShape::Length,
            Some(crate::types::Kind::Dynamic { .. }) => MetadataShape::Object,
            _ => MetadataShape::Thin,
        }
    }
}

use super::{Evaluator, Failure, Metadata, Reader, Value};
impl Evaluator<'_> {
    pub(crate) fn word_shape_key(
        &mut self,
        fold: &mut crate::types::substitute::Bound,
        ty: TypeId,
        work: usize,
        bytes: usize,
    ) -> Result<TypeId, Failure> {
        fold.apply_bounded(&mut self.queries.typing.facts.types, ty, work, bytes)
            .map_err(Failure::Invalid)
    }
    /// Split physical pointer words without changing their original write authority.
    pub(crate) fn pointer_words(
        &mut self,
        value: Constant,
    ) -> Result<(WordTarget, Option<Metadata>), Failure> {
        let value = if matches!(value.0, Value::BytePointer { .. } | Value::Text(_)) {
            self.literal_address(value)?
        } else {
            value
        };
        match value.0 {
            Value::Reference(mut pointer) => {
                Reader::new(self.facts())
                    .location(value.location().ok_or("missing pointer word origin")?)?;
                let metadata = pointer.metadata.take();
                Ok((
                    WordTarget::Value(Constant(Value::Reference(pointer))),
                    metadata,
                ))
            }
            Value::Function(_) => Ok((WordTarget::Value(value), None)),
            _ => Err("constant numeric or non-pointer symbolic word is not implemented".into()),
        }
    }
}
impl Reader<'_> {
    pub(crate) fn function_word(
        self,
        expected: TypeId,
        target: WordTarget,
    ) -> Result<Constant, &'static str> {
        let WordTarget::Value(value) = target else {
            return Err("function pointer word is not a callable");
        };
        let super::View::Function(instance) = self.view(value)? else {
            return Err("function pointer word lacks canonical callable identity");
        };
        let types = &self.facts.types;
        let (canonical, signature) = match types.kind(instance.ty()) {
            Some(crate::types::Kind::FunctionItem { signature, .. }) => (
                crate::types::instance::Instance::new(types, instance.ty())?,
                signature,
            ),
            Some(crate::types::Kind::Closure { signature, .. }) => (
                crate::types::instance::Instance::closure(types, instance.ty())?,
                signature,
            ),
            _ => return Err("symbolic function word has an invalid instance"),
        };
        if canonical != instance
            || !matches!(
                types.kind(signature),
                Some(crate::types::Kind::Function { .. })
            )
        {
            return Err("symbolic function word lacks its canonical ordinary signature");
        }
        if !matches!(
            types.kind(expected),
            Some(crate::types::Kind::Function { .. })
        ) {
            return Err("symbolic function demand lacks a function type");
        }
        // A function word retains its genuine original target. Its typed slot
        // supplies the destination signature; forming it does not call it or
        // establish compatibility/const/unsafe permission for an eventual call.
        Ok(Constant::function(instance))
    }
}
