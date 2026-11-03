//! Numeric data words preserve bits without manufacturing allocation or write authority.
use super::{Backing, Constant, Failure, Machine, Reader, TypeId, continuations::Source};
use crate::{
    const_eval::memory::validity::ValueType,
    sema::{
        model::Scalar,
        values::{Metadata, MetadataShape, WordShape},
    },
    types::Primitive,
};
impl Machine<'_, '_> {
    pub(super) fn decode_numeric_word(
        &mut self,
        source: Source,
        offset: u64,
        validity: ValueType,
    ) -> Result<Constant, Failure> {
        let shape = Reader::new(self.evaluator.facts()).word_shape(validity.ty)?;
        let WordShape::Data {
            pointee,
            reference,
            metadata,
        } = shape
        else {
            return Err("numeric function words lack callable provenance".into());
        };
        let bits = self.numeric_word_bits(source, offset)?;
        let mut value = Constant::raw_address(bits);
        match metadata {
            MetadataShape::Thin => {}
            MetadataShape::Length if validity.may_dangle && reference.is_some() => {
                let next = offset
                    .checked_add(8)
                    .ok_or("numeric metadata offset overflow")?;
                value = value.unsize(self.numeric_word_bits(source, next)?)?;
            }
            _ => return Err("numeric wide-pointer words require supported metadata proof".into()),
        }
        if reference.is_some() {
            let layout = self.reference_bits(value, pointee)?;
            if !validity.may_dangle && layout.size != 0 {
                return Err("constant reference is dangling without allocation provenance".into());
            }
        }
        Ok(value)
    }
    fn numeric_word_bits(&mut self, source: Source, offset: u64) -> Result<u64, Failure> {
        let start = usize::try_from(offset).map_err(|_| "numeric word offset overflow")?;
        let end = start.checked_add(8).ok_or("numeric word extent overflow")?;
        let scalar =
            self.decode_byte_scalar(source, start, end, TypeId::primitive(Primitive::Usize))?;
        u64::try_from(crate::const_eval::values::integer(scalar)?)
            .map_err(|_| "numeric word exceeds target width".into())
    }
    pub(super) fn encode_numeric_word(
        &mut self,
        backing: &mut Backing,
        offset: u64,
        validity: ValueType,
        value: Constant,
        bits: u64,
    ) -> Result<(), Failure> {
        let shape = Reader::new(self.evaluator.facts()).word_shape(validity.ty)?;
        let WordShape::Data {
            pointee,
            reference,
            metadata,
        } = shape
        else {
            return Err("numeric function words lack callable provenance".into());
        };
        if reference.is_some() {
            let layout = self.reference_bits(value, pointee)?;
            if !validity.may_dangle && layout.size != 0 {
                return Err("constant reference is dangling without allocation provenance".into());
            }
        }
        let length = match (metadata, value.metadata()) {
            (MetadataShape::Thin, None) => None,
            (MetadataShape::Length, Some(Metadata::Length(length)))
                if validity.may_dangle && reference.is_some() =>
            {
                Some(length)
            }
            _ => return Err("numeric pointer metadata differs from its checked type".into()),
        };
        self.encode_bytes(
            backing,
            offset,
            TypeId::primitive(Primitive::Usize),
            Constant::scalar(Scalar::Integer(u128::from(bits))),
        )?;
        if let Some(length) = length {
            let next = offset
                .checked_add(8)
                .ok_or("numeric metadata offset overflow")?;
            self.encode_bytes(
                backing,
                next,
                TypeId::primitive(Primitive::Usize),
                Constant::scalar(Scalar::Integer(u128::from(length))),
            )?;
        }
        Ok(())
    }
}
