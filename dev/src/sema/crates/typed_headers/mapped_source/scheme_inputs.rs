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
        require_regions(
            catalog,
            descriptor,
            definition,
            scheme,
            steps,
            |reference| {
                self.region(reference)
                    .map_err(|_| "invalid imported method bound")
            },
        )
    }
}

pub(in crate::sema::crates::typed_headers) fn require_regions(
    catalog: &Catalog,
    descriptor: crate::support::crates::SourceDescriptor<'_>,
    definition: DefId,
    scheme: &FunctionRegions,
    steps: &mut usize,
    mut region: impl FnMut(s::RegionRef) -> Result<crate::types::RegionId, &'static str>,
) -> Result<(), &'static str> {
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
            region(bound.longer).map_err(|_| "invalid imported method bound")?,
            region(bound.shorter).map_err(|_| "invalid imported method bound")?,
        ) != actual
        {
            return Err("imported method changed its actual full region relation");
        }
    }
    Ok(())
}

pub(super) use super::super::signature_shapes::SignaturePort;

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
        super::super::signature_shapes::require(rows, store, expected, actual, steps, |reference| {
            self.ty(reference).map_err(|_| "missing actual source type")
        })
    }
}
