//! Active variants are read independently of initialized payload fields.
use super::{Cell, Constant, Failure, Machine, Origin, Read, Reader, Resolved, View};
use crate::{const_eval::static_reads::ReadPurpose, sema::members::VariantId, types::TypeId};

impl Machine<'_, '_> {
    pub(in crate::const_eval) fn read_active_variant(&mut self, place: Resolved)
        -> Result<Read<VariantId>, Failure> {
        self.charge(1)?;
        if let Some(selected) = self.byte_place(place)? {
            self.check_byte_access(place, selected, false)?;
            return self.read_byte_variant_access(selected.root, selected.offset, selected.ty);
        }
        let selected = self.path(place.address)?;
        let (root, path) = selected.fields_only()?;
        let location = Reader::new(self.evaluator.facts()).location(root)?;
        if let Origin::Static(id) = location.origin {
            self.check_static_read(id)?;
            self.check_static_value_read(location.ty, place.ty, ReadPurpose::Value)?;
        }
        if let Some(access) = place.raw { self.check_raw_read(place, access, root, &path)?; }
        if self.byte_root(root).is_some() {
            let (ty, offset) = self.storage_offset(location.ty, &path)?;
            if !Reader::new(self.evaluator.facts()).same_storage_type(ty, place.ty) {
                return Err("constant discriminant changed its actual stored type".into());
            }
            return self.read_byte_variant_access(root, offset, ty);
        }
        let variant = if self.memory.roots.contains_key(&root) {
            let (cell, ty, rest) = self.read_cell_selection(root, &path)?;
            match self.memory.cells[cell] {
                Cell::Fields { variant: Some(variant), .. } => variant,
                Cell::Frozen(value) => self.frozen_active_variant(value, ty, &path[rest..])?,
                Cell::Empty => return Err(Failure::Fault("constant reads an uninitialized enum tag")),
                _ => return Err("constant discriminant requires its actual enum storage".into()),
            }
        } else {
            let value = match location.origin {
                Origin::Static(id) => {
                    let Some(value) = Reader::new(self.evaluator.facts()).definition(id) else {
                        self.waiting = Some(crate::const_eval::Requirement::InitializerBody(id));
                        return Ok(Read::Pending);
                    };
                    value
                }
                _ => location.value.ok_or("constant enum allocation has no contents")?,
            };
            self.frozen_active_variant(value, location.ty, &path)?
        };
        let Some(crate::types::Kind::Adt(owner, _)) = self.evaluator.facts().types.kind(place.ty) else {
            return Err("constant discriminant requires an actual enum type".into());
        };
        if self.evaluator.facts().members.variants.get(variant.0 as usize)
            .is_none_or(|member| member.owner != owner) {
            return Err("constant enum variant belongs to another actual owner".into());
        }
        Ok(Read::Value(variant))
    }

    fn frozen_active_variant(&mut self, mut value: Constant, mut ty: TypeId, path: &[u64])
        -> Result<VariantId, Failure> {
        for (position, &index) in path.iter().enumerate() {
            self.charge(1)?;
            let reader = Reader::new(self.evaluator.facts());
            if matches!(reader.view(value)?, View::Image { .. }) {
                return self.image_active_variant(value, ty, &path[position..]);
            }
            ty = reader.field_types(ty, value)?.get(index)
                .ok_or("constant enum projection exceeds its stored owner")?;
            value = self.stored_field(value, index)?;
        }
        match Reader::new(self.evaluator.facts()).view(value)? {
            View::Product { variant: Some(variant), .. } => Ok(variant),
            View::Image { .. } => self.image_active_variant(value, ty, &[]),
            _ => Err("constant enum discriminant has no initialized active variant".into()),
        }
    }
}
