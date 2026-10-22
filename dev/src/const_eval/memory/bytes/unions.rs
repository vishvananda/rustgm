//! Union construction and selected typed reads reuse the physical byte codec.
use super::{Constant, Failure, Machine, Read, Reader, TypeId, continuations::Source};
use crate::{
    const_eval::{
        memory::{Range, UnionField},
        static_reads::ReadPurpose,
    },
    sema::members::Selection,
};

impl Machine<'_, '_> {
    pub(in crate::const_eval) fn make_union(
        &mut self,
        ty: TypeId,
        field: Selection,
        value: Constant,
    ) -> Result<Constant, Failure> {
        let (selected, _, work) = self.evaluator.union_field_type(
            ty,
            field,
            self.limits.steps.saturating_sub(self.work.steps),
        )?;
        self.charge(work)?;
        if Reader::new(self.evaluator.facts()).scalar_union(ty)? {
            let (value, work) = self.evaluator.union_value(
                ty,
                field,
                value,
                self.limits.steps.saturating_sub(self.work.steps),
            )?;
            self.charge(work)?;
            return Ok(value);
        }
        if !self.byte_family(ty)? {
            return Err(
                "constant union construction requires a supported closed byte family".into(),
            );
        }
        let mut buffer = self.empty_backing(ty)?;
        let previous = self.slots;
        self.slots = previous
            .checked_add(buffer.retained_slots())
            .ok_or("union byte construction retention overflow")?;
        let result = (|| {
            self.encode_bytes(&mut buffer, 0, selected, value)?;
            let fragments = self.buffer_word_targets(&buffer)?;
            self.slots = self
                .slots
                .checked_add(
                    fragments
                        .capacity()
                        .saturating_mul(size_of::<crate::sema::values::ImageFragment>())
                        .div_ceil(size_of::<Constant>()),
                )
                .ok_or("union word publication retention overflow")?;
            let (value, work) = self.evaluator.union_snapshot(
                ty,
                &buffer.bytes,
                &buffer.initialized,
                &fragments,
                self.limits.steps.saturating_sub(self.work.steps),
            )?;
            self.charge(work)?;
            Ok(value)
        })();
        self.slots = previous;
        result
    }
    pub(in crate::const_eval::memory) fn read_image_path(
        &mut self,
        value: Constant,
        ty: TypeId,
        path: &[u64],
        range: Option<Range>,
        purpose: ReadPurpose,
    ) -> Result<Read<Constant>, Failure> {
        if Reader::new(self.evaluator.facts()).image(value)?.ty != ty {
            return Err("constant image differs from its stored owner type".into());
        }
        let (selected, offset) = self.storage_offset(ty, path)?;
        if let Some(range) = range {
            self.decode_source_range(Source::Image(value), offset, selected, range, purpose)
        } else {
            self.decode_source(Source::Image(value), offset, selected, purpose)
        }
    }
    pub(in crate::const_eval::memory) fn read_image_field(
        &mut self,
        value: Constant,
        union: UnionField,
        leaf: TypeId,
        purpose: ReadPurpose,
    ) -> Result<Read<Constant>, Failure> {
        let (ty, _, work) = self.evaluator.union_field_type(
            union.owner,
            union.field,
            self.limits.steps.saturating_sub(self.work.steps),
        )?;
        self.charge(work)?;
        if !self.image_field_compatible(ty, leaf) {
            return Err("constant image field differs from its checked type".into());
        }
        let Selection::Nominal(field) = union.field else {
            return Err("constant image field lacks nominal identity".into());
        };
        let ordinal = self
            .evaluator
            .facts()
            .members
            .ordinal(field)
            .ok_or("missing constant image field ordinal")?;
        self.read_image_path(value, union.owner, &[u64::from(ordinal)], None, purpose)
    }
    /// The physical object word has no lifetime bits. Its demanded lifetime is
    /// retained by the decoder, never proved by this storage comparison.
    pub(super) fn image_field_compatible(&self, actual: TypeId, demanded: TypeId) -> bool {
        let reader = Reader::new(self.evaluator.facts());
        if reader.same_storage_type(actual, demanded) {
            return true;
        }
        let types = &self.evaluator.facts().types;
        let (
            Some(crate::types::Kind::Pointer {
                mutable: a,
                pointee: x,
            }),
            Some(crate::types::Kind::Pointer {
                mutable: b,
                pointee: y,
            }),
        ) = (types.kind(actual), types.kind(demanded))
        else {
            return false;
        };
        let (
            Some(crate::types::Kind::Dynamic {
                shape: a_shape,
                arguments: a_args,
                bound_regions: a_bound,
                ..
            }),
            Some(crate::types::Kind::Dynamic {
                shape: b_shape,
                arguments: b_args,
                bound_regions: b_bound,
                ..
            }),
        ) = (types.kind(x), types.kind(y))
        else {
            return false;
        };
        a == b && a_shape == b_shape && a_args == b_args && a_bound == b_bound
    }
    fn buffer_word_targets(
        &mut self,
        buffer: &super::Backing,
    ) -> Result<Vec<crate::sema::values::ImageFragment>, Failure> {
        let Some(tree) = &buffer.fragments else {
            return Ok(Vec::new());
        };
        self.charge(tree.len())?;
        self.reserve_cells(
            tree.len()
                .saturating_mul(2 * size_of::<crate::sema::values::ImageFragment>())
                .saturating_add(128)
                .div_ceil(size_of::<Constant>()),
        )?;
        let mut result = Vec::new();
        result
            .try_reserve_exact(tree.len())
            .map_err(|_| "union symbolic word publication allocation failed")?;
        for fragment in tree.values() {
            result.push(crate::sema::values::ImageFragment {
                offset: fragment.offset,
                source_offset: fragment.source_offset,
                length: fragment.length,
                target: self.word_target(fragment.word)?,
            });
        }
        Ok(result)
    }
}
