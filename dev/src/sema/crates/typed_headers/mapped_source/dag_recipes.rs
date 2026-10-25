//! Ordinary Graph preparation preserves its eager ordinal order in the shared cursor.
use super::{Context, Declarations, Error, Headers, RegionId, TypeId};
use crate::{resolve::body_origin::ImportedDeclarationId, types::arguments::Argument};

pub(super) fn prepare(
    context: &Context<'_>,
    facts: &mut Declarations,
    slots: &[Argument],
    regions: &[RegionId],
    root: ImportedDeclarationId,
    budget: &mut Headers,
) -> Result<Vec<TypeId>, Error> {
    super::super::recipe_cursor::prepare_functions(context, facts, slots, regions, root, budget)
}
