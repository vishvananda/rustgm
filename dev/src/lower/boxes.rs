//! A box's content lives behind the pointer its leading fields hold:
//! alloc's `Box(Unique(NonNull(*const T is !null)), A)`. Code generation
//! finds that pointer by its exact field path in each box's layout.
use super::context::Context;
use crate::{
    sema::members::Selection,
    types::{Kind, TypeId},
};

/// The leading-field path a box may take to its pointer.
const DEPTH: usize = 8;

impl Context<'_> {
    /// The byte offset, within a box of `ty`, of the pointer to its content.
    /// # Errors
    /// Rejects a type that is no box, and a box whose leading fields do not
    /// lead to a pointer to exactly its content.
    pub(super) fn box_pointer_offset(&mut self, ty: TypeId) -> Result<u64, &'static str> {
        let content = crate::sema::compiler_items::boxes::content(self.types, self.aggregates, ty)?
            .ok_or("box dereference requires an owned box place")?;
        let mut current = ty;
        let mut offset = 0u64;
        for _ in 0..DEPTH {
            match self.types.kind(current) {
                Some(Kind::Pointer { pointee, .. }) if pointee == content => return Ok(offset),
                Some(Kind::Pattern { base, .. }) => current = base,
                Some(Kind::Adt(item, _)) => {
                    let layout = self.layout(current)?;
                    let [variant] = self.members.variants(item) else {
                        return Err("box pointer path requires leading struct fields");
                    };
                    let first = self
                        .members
                        .variants
                        .get(variant.0 as usize)
                        .filter(|variant| variant.start < variant.end)
                        .ok_or("box pointer path requires a leading field")?
                        .start;
                    let (field, at) = self.selected_field(
                        &layout,
                        Selection::Nominal(crate::sema::members::FieldId(first)),
                    )?;
                    offset = offset
                        .checked_add(at)
                        .ok_or("box pointer offset overflow")?;
                    current = field;
                }
                _ => return Err("box leading fields do not hold a pointer to its content"),
            }
        }
        Err("box pointer path exceeds its leading-field depth")
    }
}
