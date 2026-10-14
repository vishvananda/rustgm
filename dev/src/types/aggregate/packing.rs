//! The packed declaration restriction follows named struct fields, before substitution.
use super::{Definitions, Kind};
use crate::types::{ItemId, Kind as Type, Store};
use std::collections::{HashMap, HashSet};

impl Definitions {
    /// Check the pinned language's declaration restriction once. It follows
    /// direct struct fields, including aliases already expanded by semantics;
    /// arrays, tuples, enums and substituted generic arguments do not add edges.
    /// This differs from an instance's numerical alignment and is not a layout.
    /// # Errors
    /// Rejects missing templates, forbidden containment and exhausted graph work.
    pub fn validate_packing(&mut self, types: &Store) -> Result<(), (ItemId, &'static str)> {
        if self.packing_checked {
            return Ok(());
        }
        if self
            .entries
            .iter()
            .any(|definition| definition.repr.packed.is_some())
        {
            self.packing_graph(types)?;
        }
        self.packing_checked = true;
        Ok(())
    }
    fn packing_graph(&self, types: &Store) -> Result<(), (ItemId, &'static str)> {
        let mut reverse: HashMap<ItemId, Vec<ItemId>> = HashMap::new();
        let mut aligned = HashSet::new();
        let mut work = Vec::new();
        let mut steps = 0_usize;
        for definition in &self.entries {
            if definition.kind != Kind::Struct {
                continue;
            }
            if definition.repr.align.is_some() {
                aligned.insert(definition.item);
                work.push(definition.item);
            }
            for variant in self.variants(definition) {
                for &ty in types
                    .list(variant.fields)
                    .ok_or((definition.item, "missing packed field templates"))?
                {
                    charge(&mut steps, definition.item)?;
                    if let Some(Type::Adt(item, _)) = types.kind(ty) {
                        let field = self
                            .get(item)
                            .ok_or((definition.item, "missing packed nominal field"))?;
                        if field.kind == Kind::Struct {
                            reverse.entry(item).or_default().push(definition.item);
                        }
                    }
                }
            }
        }
        while let Some(item) = work.pop() {
            charge(&mut steps, item)?;
            for &owner in reverse.get(&item).into_iter().flatten() {
                charge(&mut steps, owner)?;
                if aligned.insert(owner) {
                    work.push(owner);
                }
            }
        }
        if let Some(definition) = self
            .entries
            .iter()
            .find(|d| d.repr.packed.is_some() && aligned.contains(&d.item))
        {
            return Err((
                definition.item,
                "packed aggregate transitively contains an explicitly aligned type",
            ));
        }
        Ok(())
    }
    #[must_use]
    pub(crate) const fn packing_checked(&self) -> bool {
        self.packing_checked
    }
}
fn charge(steps: &mut usize, item: ItemId) -> Result<(), (ItemId, &'static str)> {
    *steps += 1;
    if *steps > 1_000_000 {
        return Err((item, "packed declaration work budget exceeded"));
    }
    Ok(())
}
