//! Specialization arguments open in the same arena and catalog as the impl headers.
use super::{Context, Error, Graph, Headers, error, s};
use crate::{
    sema::{crates::typed_headers::recipe_cursor, specialization::ImportedParent},
    types::{Store, arguments::Argument},
};

impl Graph {
    pub(in crate::sema) fn specialization(&self) -> &[ImportedParent] {
        &self.specialization
    }

    pub(in crate::sema::crates::typed_headers) fn remap_specialization(
        &mut self,
        context: &Context<'_>,
        store: &mut Store,
        budget: &mut Headers,
    ) -> Result<(), Error> {
        let rows = context.descriptor.rows();
        let Some(ancestry) = &rows.cold().specialization else {
            return Ok(());
        };
        self.require_store(store).map_err(error)?;
        budget.reserve(
            ancestry
                .parents
                .len()
                .saturating_mul(size_of::<ImportedParent>()),
        )?;
        let mut parents = Vec::with_capacity(ancestry.parents.len());
        for edge in &ancestry.parents {
            let child = context.definition(edge.child, budget)?;
            let parent = context.definition(edge.parent, budget)?;
            let raw = edge.arguments.slice(&rows.arguments).map_err(error)?;
            budget.charge(raw.len().saturating_add(1))?;
            budget.reserve(raw.len().saturating_mul(size_of::<Argument>()))?;
            let arguments = raw
                .iter()
                .map(|&argument| self.parent_argument(rows, argument, store, budget))
                .collect::<Result<Vec<_>, _>>()?;
            let growth = store.argument_interning_growth(&arguments);
            budget.charge(growth.work)?;
            budget.reserve(growth.bytes)?;
            parents.push(ImportedParent {
                child,
                parent,
                arguments: store.intern_arguments(&arguments).map_err(error)?,
            });
        }
        budget.observed.vector(&parents);
        self.specialization = parents;
        Ok(())
    }

    fn parent_argument(
        &self,
        rows: s::ValidatedView<'_>,
        raw: s::Argument,
        store: &mut Store,
        budget: &mut Headers,
    ) -> Result<Argument, Error> {
        match raw {
            s::Argument::Type(ty) => self.ty(ty).map(Argument::Type),
            s::Argument::Region(region) => self.region(region).map(Argument::Lifetime),
            s::Argument::Const(term) => recipe_cursor::remap_const_argument(
                rows,
                store,
                term,
                budget,
                |store, input, budget| match input {
                    recipe_cursor::ConstInput::Slot(slot) => self.slot(slot),
                    recipe_cursor::ConstInput::Variant(value) => {
                        recipe_cursor::const_variant(rows, store, self.ty(value.ty)?, value, budget)
                    }
                },
            ),
        }
    }
}
