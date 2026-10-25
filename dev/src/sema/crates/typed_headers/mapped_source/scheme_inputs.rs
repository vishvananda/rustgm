//! Authenticate the complete installed callable scheme, including nondefault late slots.
use super::{Catalog, DefId, DefKind, Graph, s};
use crate::sema::lifetimes::FunctionRegions;

impl Graph {
    pub(in crate::sema::crates::typed_headers) fn require_method_regions(
        &self,
        catalog: &Catalog,
        definition: DefId,
        scheme: &FunctionRegions,
        steps: &mut usize,
    ) -> Result<(), &'static str> {
        *steps = steps.saturating_add(1);
        let descriptor = self.descriptor(catalog)?;
        let id = catalog.declaration(definition)?;
        catalog.validate_declaration(id, definition, DefKind::Function)?;
        let (owner, row, _) = catalog.declaration_coordinate(id)?;
        if owner != descriptor.identity() {
            return Err("imported method scheme belongs to another actual source provider");
        }
        let rows = descriptor.rows();
        let Some(s::HeaderRef::Function(index)) = rows.header(row)? else {
            return Err("imported method scheme lacks its complete actual function row");
        };
        let raw = &rows.functions[index as usize];
        if scheme.total != raw.regions.total
            || scheme.captures != raw.regions.captures
            || !scheme.early.is_empty()
            || raw.regions.early.start != raw.regions.early.end
            || scheme.has_declared_late
        {
            return Err("imported method changed its exact supported callable region scheme");
        }
        let late = raw.regions.late.slice(&rows.region_slots)?;
        let bounds = raw.regions.bounds.slice(&rows.region_bounds)?;
        if late.len() != scheme.late.len() || bounds.len() != scheme.bounds.len() {
            return Err("imported method omitted actual late lifetime or region-bound inputs");
        }
        for (&slot, &actual) in late.iter().zip(&scheme.late) {
            *steps = steps.saturating_add(1);
            let slot = rows
                .slots
                .get(slot.0 as usize)
                .ok_or("imported method late slot is outside its actual formal table")?;
            if slot.owner != row
                || slot.kind != s::ParameterKind::Region
                || slot.index != u32::from(actual)
            {
                return Err("imported method late slot changed its actual owner/kind/ordinal");
            }
        }
        for (bound, &actual) in bounds.iter().zip(&scheme.bounds) {
            *steps = steps.saturating_add(1);
            if (
                self.region(bound.longer)
                    .map_err(|_| "invalid imported method bound")?,
                self.region(bound.shorter)
                    .map_err(|_| "invalid imported method bound")?,
            ) != actual
            {
                return Err("imported method changed its actual full region relation");
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy)]
pub(super) struct SignaturePort {
    pub owner: crate::support::crates::owner_catalog::DeclarationRef,
    pub kind: s::BinderKind,
    pub raw: crate::support::crates::generics::Signature,
}

impl Graph {
    pub(super) fn require_signature(
        &self,
        catalog: &Catalog,
        store: &crate::types::Store,
        expected: SignaturePort,
        actual: crate::types::TypeId,
        steps: &mut usize,
    ) -> Result<(), &'static str> {
        *steps = steps.saturating_add(1);
        self.require_store(store)?;
        let rows = self.descriptor(catalog)?.rows();
        let SignaturePort { owner, kind, raw } = expected;
        let frame = rows
            .binders
            .get(raw.binder.0 as usize)
            .ok_or("imported callable signature lacks its actual binder frame")?;
        let Some(crate::types::Kind::Function {
            bound_regions,
            parameters,
            result,
            unsafe_,
            convention,
        }) = store.kind(actual)
        else {
            return Err("imported callable signature is not an actual function type");
        };
        if frame.owner != owner
            || frame.kind != kind
            || bound_regions != frame.count
            || unsafe_ != raw.unsafe_
            || raw.variadic
            || raw.convention != crate::support::crates::generics::Convention::Rust
            || convention != crate::types::callable::Convention::RUST
            || self
                .ty(raw.result)
                .map_err(|_| "missing actual source result type")?
                != result
        {
            return Err("imported callable changed its full actual binder/signature contract");
        }
        let expected = raw.parameters.slice(&rows.roots)?;
        let actual = store
            .list(parameters)
            .ok_or("imported callable omitted its actual inputs")?;
        if actual.len() != expected.len() {
            return Err("imported callable signature omitted actual inputs");
        }
        for (&actual, &expected) in actual.iter().zip(expected) {
            *steps = steps.saturating_add(1);
            if actual
                != self
                    .ty(expected)
                    .map_err(|_| "missing actual source input type")?
            {
                return Err("imported callable changed an actual full parameter type");
            }
        }
        Ok(())
    }
}
