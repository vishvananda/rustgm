//! Symbolic words never expose placeholder bytes as numeric values.
use super::continuations::Source;
use super::relocations::{Fragment, WordId};
use super::{Backing, Constant, Failure, Machine, Read, Reader, TypeId};
use crate::const_eval::memory::{RawAccess, RawAlignment, Resolved};
use crate::sema::values::{Metadata, MetadataShape, WordShape, WordTarget};
impl Machine<'_, '_> {
    fn encode_symbolic_word(
        &mut self,
        backing: &mut Backing,
        offset: u64,
        target: WordTarget,
    ) -> Result<(), Failure> {
        let start =
            usize::try_from(offset).map_err(|_| "symbolic word offset exceeds host size")?;
        let end = start
            .checked_add(8)
            .filter(|&end| end <= backing.bytes.len())
            .ok_or("symbolic word exceeds storage bounds")?;
        let word = self.intern_word(target)?;
        self.buffer_fragments(
            backing,
            offset,
            8,
            &[Fragment {
                offset: 0,
                source_offset: 0,
                length: 8,
                word,
            }],
        )?;
        self.charge(8)?;
        backing.bytes[start..end].fill(0);
        for index in start..end {
            backing.set_initialized(index, true);
        }
        Ok(())
    }
    pub(super) fn encode_byte_word(
        &mut self,
        backing: &mut Backing,
        offset: u64,
        ty: TypeId,
        value: Constant,
    ) -> Result<(), Failure> {
        let shape = Reader::new(self.evaluator.facts()).word_shape(ty)?;
        let value = if matches!(
            shape,
            WordShape::Data {
                reference: Some(false),
                ..
            }
        ) {
            value.restrict_shared()
        } else {
            value
        };
        if let crate::sema::values::View::Address { bits } =
            Reader::new(self.evaluator.facts()).view(value)?
        {
            if !matches!(
                shape,
                WordShape::Data {
                    reference: None,
                    metadata: MetadataShape::Thin,
                    ..
                }
            ) || value.metadata().is_some()
            {
                return Err(
                    "numeric reference or wide-pointer words require a supported validity profile"
                        .into(),
                );
            }
            return self.encode_bytes(
                backing,
                offset,
                TypeId::primitive(crate::types::Primitive::Usize),
                Constant::scalar(crate::sema::model::Scalar::Integer(u128::from(bits))),
            );
        }
        let (target, metadata) = self.evaluator.pointer_words(value)?;
        match shape {
            WordShape::Function => {
                Reader::new(self.evaluator.facts()).function_word(ty, target)?;
                if metadata.is_some() {
                    return Err("function word has unexpected metadata".into());
                }
            }
            WordShape::Data {
                reference,
                metadata: expected,
                ..
            } => {
                let function = matches!(target,WordTarget::Value(value) if value.as_function().is_some())
                    && reference.is_none()
                    && matches!(expected, MetadataShape::Thin);
                if !function
                    && !matches!(target, WordTarget::Value(value) if value.location().is_some())
                {
                    return Err("data pointer word has no allocation provenance".into());
                }
                match (expected, metadata) {
                    (MetadataShape::Thin, None)
                    | (MetadataShape::Length, Some(Metadata::Length(_)))
                    | (MetadataShape::Object, Some(Metadata::Object { .. })) => {}
                    _ => return Err("pointer word metadata differs from its checked type".into()),
                }
            }
        }
        self.encode_symbolic_word(backing, offset, target)?;
        if let Some(metadata) = metadata {
            let offset = offset
                .checked_add(8)
                .ok_or("symbolic metadata offset overflow")?;
            match metadata {
                Metadata::Length(length) => self.encode_bytes(
                    backing,
                    offset,
                    TypeId::primitive(crate::types::Primitive::Usize),
                    Constant::scalar(crate::sema::model::Scalar::Integer(u128::from(length))),
                )?,
                Metadata::Object { concrete, target } => self.encode_symbolic_word(
                    backing,
                    offset,
                    WordTarget::Vtable { concrete, target },
                )?,
            }
        }
        Ok(())
    }
    fn read_symbolic_word(&mut self, source: Source, offset: u64) -> Result<WordId, Failure> {
        self.charge(8)?;
        let start =
            usize::try_from(offset).map_err(|_| "symbolic read offset exceeds host size")?;
        let backing = self.byte_data(source)?;
        let end = start
            .checked_add(8)
            .filter(|&end| end <= backing.bytes.len())
            .ok_or("symbolic read exceeds backing bounds")?;
        if (start..end).any(|index| !backing.initialized(index)) {
            return Err("symbolic pointer read uses uninitialized bytes".into());
        }
        let fragments = self.source_fragments(source, offset, 8)?;
        let [fragment] = fragments.as_slice() else {
            return Err("symbolic pointer read requires one complete provenance word".into());
        };
        if fragment.offset != 0 || fragment.source_offset != 0 || fragment.length != 8 {
            return Err("symbolic pointer read has incomplete or misordered provenance".into());
        }
        Ok(fragment.word)
    }
    pub(super) fn decode_byte_word(
        &mut self,
        source: Source,
        offset: u64,
        ty: TypeId,
    ) -> Result<(Constant, Option<super::continuations::Duty>), Failure> {
        let shape = Reader::new(self.evaluator.facts()).word_shape(ty)?;
        if !self.source_has_fragments(source, offset, 8)? {
            if !matches!(
                shape,
                WordShape::Data {
                    reference: None,
                    metadata: MetadataShape::Thin,
                    ..
                }
            ) {
                return Err("numeric references, function or wide-pointer words require supported validity proof".into());
            }
            let start =
                usize::try_from(offset).map_err(|_| "numeric pointer word offset overflow")?;
            let end = start
                .checked_add(8)
                .ok_or("numeric pointer word extent overflow")?;
            let scalar = self.decode_byte_scalar(
                source,
                start,
                end,
                TypeId::primitive(crate::types::Primitive::Usize),
            )?;
            let bits = u64::try_from(crate::const_eval::values::integer(scalar)?)
                .map_err(|_| "numeric pointer exceeds target width")?;
            return Ok((Constant::raw_address(bits), None));
        }
        let word = self.read_symbolic_word(source, offset)?;
        match shape {
            WordShape::Function => Ok((
                Reader::new(self.evaluator.facts()).function_word(ty, self.word_target(word)?)?,
                None,
            )),
            WordShape::Data {
                pointee,
                reference,
                metadata,
            } => {
                let target = self.word_target(word)?;
                if let WordTarget::Value(value) = target
                    && value.as_function().is_some()
                {
                    if reference.is_some() || !matches!(metadata, MetadataShape::Thin) {
                        return Err(
                            "function words as data references require supported provenance".into(),
                        );
                    }
                    return Ok((value, None));
                }
                let address = self.word_address(word)?;
                let (value, physical) = self.decode_data_address(
                    source,
                    offset,
                    address,
                    pointee,
                    reference.is_some(),
                    metadata,
                )?;
                if let Some(mutable) = reference {
                    if mutable
                        && value.write_permission()
                            != Some(crate::sema::values::WritePermission::Writable)
                    {
                        return Err(
                            "mutable symbolic reference has shared-derived pointer authority"
                                .into(),
                        );
                    }
                    let value = if mutable {
                        value
                    } else {
                        value.restrict_shared()
                    };
                    Ok((
                        value,
                        Some(super::continuations::Duty {
                            address: value,
                            ty: physical,
                            mutable,
                        }),
                    ))
                } else {
                    Ok((value, None))
                }
            }
        }
    }
    fn decode_data_address(
        &mut self,
        source: Source,
        offset: u64,
        address: crate::sema::values::WordAddress,
        pointee: TypeId,
        reference: bool,
        metadata: MetadataShape,
    ) -> Result<(Constant, TypeId), Failure> {
        Ok(match metadata {
            MetadataShape::Thin => (self.reconstruct_word(address, pointee)?, pointee),
            MetadataShape::Length => {
                let next = offset
                    .checked_add(8)
                    .ok_or("slice metadata offset overflow")?;
                let start =
                    usize::try_from(next).map_err(|_| "slice metadata offset exceeds host size")?;
                let end = start
                    .checked_add(8)
                    .ok_or("slice metadata extent overflow")?;
                let length = self.decode_byte_scalar(
                    source,
                    start,
                    end,
                    TypeId::primitive(crate::types::Primitive::Usize),
                )?;
                let length = u64::try_from(crate::const_eval::values::integer(length)?)
                    .map_err(|_| "slice metadata exceeds usize")?;
                (
                    self.reconstruct_sequence_word(address, pointee, length, reference)?,
                    pointee,
                )
            }
            MetadataShape::Object => {
                let next = offset
                    .checked_add(8)
                    .ok_or("vtable metadata offset overflow")?;
                let vtable = self.read_symbolic_word(source, next)?;
                let WordTarget::Vtable { concrete, target } = self.word_target(vtable)? else {
                    return Err("object metadata is not a symbolic vtable word".into());
                };
                if self.erased_word_type(target)? != self.erased_word_type(pointee)? {
                    return Err("object vtable differs from the demanded dynamic type".into());
                }
                let (value, concrete) = self.reconstruct_object_word(address, concrete)?;
                let value = value.with_metadata(Metadata::Object {
                    concrete,
                    target: pointee,
                })?;
                (value, concrete)
            }
        })
    }
    pub(super) fn reconstruct_sequence_word(
        &mut self,
        address: crate::sema::values::WordAddress,
        pointee: TypeId,
        length: u64,
        reference: bool,
    ) -> Result<Constant, Failure> {
        let element = Reader::new(self.evaluator.facts()).sequence_element(pointee)?;
        let root = Reader::new(self.evaluator.facts())
            .location(address.root())?
            .ty;
        if let Some(crate::types::Kind::Array {
            element: actual, ..
        }) = self.evaluator.facts().types.kind(root)
            && Reader::new(self.evaluator.facts()).same_storage_type(actual, element)
        {
            if reference {
                self.footprint(root)?;
                return self.evaluator.word_sequence_range(address, length, pointee);
            }
            if address.offset() == 0 {
                return self.reconstruct_word(address, root)?.unsize(length);
            }
        }
        let selected = self.reconstruct_word(address, element)?;
        if !reference {
            return selected.unsize(length);
        }
        let (parent, from) = selected.projection_parent(Reader::new(self.evaluator.facts()))?;
        self.evaluator.subsequence(parent, from, length, pointee)
    }
    pub(super) fn validate_word_reference(
        &mut self,
        address: Constant,
        ty: TypeId,
        mutable: bool,
    ) -> Result<Read<Constant>, Failure> {
        if matches!(
            self.evaluator.facts().types.kind(ty),
            Some(
                crate::types::Kind::Slice(_)
                    | crate::types::Kind::Primitive(crate::types::Primitive::Str)
            )
        ) {
            return self.validate_sequence_word_reference(address, ty, mutable);
        }
        let place = Resolved {
            address,
            ty,
            raw: Some(RawAccess {
                alignment: RawAlignment::Required,
                location: address.location(),
                ty,
            }),
            union: None,
        };
        let result = if mutable {
            self.mutable_raw_reference(place)?
        } else {
            self.shared_raw_reference(place)?
        };
        Ok(result.map_or(Read::Pending, Read::Value))
    }
}
