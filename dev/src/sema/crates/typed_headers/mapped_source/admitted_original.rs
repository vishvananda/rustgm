//! A callback-scoped provided original borrows the actual selected body authority.
use super::{Arc, Argument, Catalog, Declarations, DefId, Graph, Original, s};
pub(super) enum Mapping<'a> {
    Ordinary(&'a Graph),
    Selected {
        owner: &'a super::super::super::unpaid_provided::ProvidedOwners,
        store: &'a crate::types::Store,
        body: &'a super::super::super::unpaid_provided::OriginalBody<'a>,
        view: &'a crate::resolve::crates::unpaid_provided::RecipeView<'a>,
    },
}
impl Original<'_> {
    pub(in crate::sema::crates::typed_headers) fn selected<'a>(
        catalog: &'a Catalog,
        owner: &'a super::super::super::unpaid_provided::ProvidedOwners,
        facts: &'a Declarations,
        body: &'a super::super::super::unpaid_provided::OriginalBody<'a>,
        view: &'a crate::resolve::crates::unpaid_provided::RecipeView<'a>,
        definition: DefId,
        steps: &mut usize,
    ) -> Result<Original<'a>, &'static str> {
        super::charge(steps)?;
        let headers = facts
            .crates
            .as_deref()
            .and_then(|linkage| linkage.originals.as_deref())
            .ok_or("selected original lost its actual header owner")?;
        let header = headers
            .entries
            .get(&definition)
            .ok_or("selected original lost its actual installed header")?;
        let actual = facts.associated.imported_owner();
        if !matches!((actual, headers.source.as_ref()),
            (Some(crate::sema::crates::ImportedOwners::Provided(a)), Some(crate::sema::crates::ImportedOwners::Provided(b)))
            if std::ptr::eq(a.as_ref(), owner) && std::ptr::eq(b.as_ref(), owner))
        {
            return Err("selected original changed its actual installed owner allocation");
        }
        owner.require_store(&facts.types)?;
        owner.require_method_declaration(catalog, facts, definition, header, steps)?;
        let declaration = catalog.declaration(definition)?;
        let (provider, row, _) = catalog.declaration_coordinate(declaration)?;
        let rows = view
            .rows()
            .map_err(|_| "selected original lost its actual immutable rows")?;
        let Some(s::HeaderRef::Function(index)) = rows.header(row)? else {
            return Err("selected original lost its actual callable row");
        };
        let function = &rows.functions[index as usize];
        let s::FunctionBody::Original(reference) = function.body else {
            return Err("selected original cannot reconstruct an actual required method");
        };
        let actual = catalog.body_at(provider, reference)?;
        if provider != rows.descriptor().identity()
            || header.body != Some(catalog.origin(actual)?)
            || header
                .environment
                .as_ref()
                .is_none_or(|current| !Arc::ptr_eq(current, body.environment()))
            || header.constant
            || rows.originals[reference.0 as usize].context != s::CallContext::Runtime
        {
            return Err("selected original requires its actual runtime defining body environment");
        }
        let result = Original {
            mapping: Mapping::Selected {
                owner,
                store: &facts.types,
                body,
                view,
            },
            catalog,
            header,
            definition,
            body: actual,
            rows,
            reference,
        };
        result.check_locals(steps)?;
        Ok(result)
    }
    pub(super) fn descriptor(
        &self,
    ) -> Result<crate::support::crates::SourceDescriptor<'_>, &'static str> {
        match &self.mapping {
            Mapping::Ordinary(graph) => graph.descriptor(self.catalog),
            Mapping::Selected { owner, .. } => owner.descriptor(),
        }
    }
    pub(super) fn require_store(&self, store: &crate::types::Store) -> Result<(), &'static str> {
        match &self.mapping {
            Mapping::Ordinary(graph) => graph.require_store(store),
            Mapping::Selected { owner, .. } => owner.require_store(store),
        }
    }
    pub(super) fn role_origin(
        &self,
        role: crate::sema::compiler_items::Role,
    ) -> Result<(), &'static str> {
        match &self.mapping {
            Mapping::Ordinary(graph) => graph.role_origin(role),
            Mapping::Selected { owner, .. } => owner.role_origin(role),
        }
    }
    pub(super) fn slot(&self, reference: s::SlotRef) -> Result<Argument, &'static str> {
        match &self.mapping {
            Mapping::Ordinary(graph) => graph
                .slot(reference)
                .map_err(|_| "source original lost its actual formal slot"),
            Mapping::Selected { owner, .. } => owner.original_slot(reference),
        }
    }
}
