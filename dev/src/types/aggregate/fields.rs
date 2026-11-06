//! Semantic declaration fields reuse one full-argument fold, including lifetimes.
use super::{Definitions, Variant};
use crate::{
    support::retained::Growth,
    types::{GenericArgs, ItemId, Kind, Store, TypeId, substitute::Bound},
};
use std::cell::Cell;
struct Cost {
    work: Cell<usize>,
    bytes: Cell<usize>,
}
impl Cost {
    fn charge(&self, work: usize, bytes: usize) -> Result<(), &'static str> {
        let work = self
            .work
            .get()
            .checked_add(work)
            .filter(|&n| n <= 1_000_000)
            .ok_or("semantic field substitution work budget exceeded")?;
        let bytes = self
            .bytes
            .get()
            .checked_add(bytes)
            .filter(|&n| n <= 64 * 1024 * 1024)
            .ok_or("semantic field substitution storage budget exceeded")?;
        self.work.set(work);
        self.bytes.set(bytes);
        Ok(())
    }
}
pub(crate) struct SemanticFields<'a> {
    variants: &'a [Variant],
    variant: usize,
    field: usize,
    entered: bool,
    item: ItemId,
    arguments: GenericArgs,
    fold: Option<Bound>,
    cost: Cost,
}
impl Definitions {
    pub(crate) fn semantic_fields(
        &self,
        types: &Store,
        ty: TypeId,
    ) -> Result<super::SemanticFields<'_>, &'static str> {
        let Some(Kind::Adt(item, arguments)) = types.kind(ty) else {
            return Err("semantic fields require an actual nominal application");
        };
        let definition = self
            .get(item)
            .ok_or("semantic field declaration is pending")?;
        if definition.item != item {
            return Err("semantic field declaration changed its owner");
        }
        let cost = Cost {
            work: Cell::new(1),
            bytes: Cell::new(size_of::<SemanticFields<'_>>()),
        };
        types
            .arguments(arguments)
            .ok_or("semantic field arguments are missing")?;
        Ok(SemanticFields {
            variants: self.variants(definition),
            variant: 0,
            field: 0,
            entered: false,
            item,
            arguments,
            fold: None,
            cost,
        })
    }
}
impl SemanticFields<'_> {
    pub(crate) fn retained_bytes(&self) -> usize {
        self.cost.bytes.get()
    }
    pub(crate) fn work(&self) -> usize {
        self.cost.work.get()
    }
    pub(crate) fn reserve<T>(
        &self,
        values: &mut Vec<T>,
        additional: usize,
    ) -> Result<(), &'static str> {
        let growth = Growth::vector::<T>(values.len(), values.capacity(), additional);
        self.cost.charge(growth.work, growth.bytes)?;
        values
            .try_reserve(additional)
            .map_err(|_| "semantic field goal allocation failed")
    }
    pub(crate) fn next(&mut self, types: &mut Store) -> Result<Option<TypeId>, &'static str> {
        while let Some(variant) = self.variants.get(self.variant) {
            if !self.entered {
                self.cost.charge(1, 0)?;
                self.entered = true;
            }
            let fields = types
                .list(variant.fields)
                .ok_or("semantic declaration field list is missing")?;
            if self.field < fields.len() {
                let field = self.field;
                self.field += 1;
                return self.field(types, self.variant, field).map(Some);
            }
            self.variant += 1;
            self.field = 0;
            self.entered = false;
        }
        Ok(None)
    }
    /// Resolve one authenticated declaration coordinate with the same paid fold.
    pub(crate) fn field(
        &mut self,
        types: &mut Store,
        variant: usize,
        field: usize,
    ) -> Result<TypeId, &'static str> {
        self.cost.charge(1, 0)?;
        let variant = self
            .variants
            .get(variant)
            .ok_or("semantic field variant is missing")?;
        let ty = types
            .list(variant.fields)
            .and_then(|fields| fields.get(field))
            .copied()
            .ok_or("semantic declaration field coordinate is missing")?;
        if self.fold.is_none() {
            self.fold = Some(Bound::joint_charged(
                types,
                self.item,
                self.arguments,
                &|w, b| self.cost.charge(w, b),
            )?);
        }
        self.fold
            .as_mut()
            .ok_or("semantic fields lost their paid fold")?
            .apply_inside_observed::<true>(types, ty, 0, &|w, b| self.cost.charge(w, b))
    }
}
