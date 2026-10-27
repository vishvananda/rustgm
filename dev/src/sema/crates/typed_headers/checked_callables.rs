//! Selected callable declarations consume the same scheme/signature validators.
use super::{Borrowed, Catalog, Mapping, ProvidedOwners, RecipeView, ReservationWork, Store, s};
use crate::{
    resolve::{
        body_origin::DeclarationOrigin,
        model::{DefId, DefKind},
    },
    sema::{
        crates::typed_headers::{Header, mapped_source, signature_shapes},
        lifetimes::FunctionRegions,
        model::Declarations,
    },
    support::constness::Constness,
};
use std::sync::Arc;

impl ProvidedOwners {
    /// The receipt-backed source collector rejected tracking on every actual
    /// function before publishing these complete, immutable header columns.
    pub(in crate::sema::crates::typed_headers) fn caller_tracking_absent(
        &self,
        catalog: &Catalog,
        facts: &Declarations,
        definition: DefId,
        steps: &mut usize,
    ) -> Result<(), &'static str> {
        self.checked_call(catalog, &facts.types, steps, |_, work, _| {
            work.charge(8)
                .map_err(|_| "selected caller policy work budget exceeded")?;
            let id = catalog.declaration(definition)?;
            catalog.validate_declaration(id, definition, DefKind::Function)?;
            let (provider, row, edition) = catalog.declaration_coordinate(id)?;
            let descriptor = self.descriptor()?;
            if provider != descriptor.identity() || edition != descriptor.edition() {
                return Err("selected caller policy changed its actual provider coordinate");
            }
            let rows = descriptor.rows();
            let Some(s::HeaderRef::Function(index)) = rows.header(row)? else {
                return Err("selected caller policy lacks its actual function row");
            };
            let declaration = rows
                .declarations
                .get(row.0 as usize)
                .ok_or("selected caller policy declaration disappeared")?;
            let contract = self
                .data
                .columns()
                .functions
                .get(index as usize)
                .ok_or("selected caller policy lacks its completed source header")?;
            if contract.declaration != row || contract.declaration_site >= declaration.sites {
                return Err("selected caller policy changed its immutable header owner/site");
            }
            catalog.declaration_site(id, contract.declaration_site)?;
            Ok(())
        })
    }

    fn checked_call<T>(
        &self,
        catalog: &Catalog,
        store: &Store,
        steps: &mut usize,
        check: impl FnOnce(&RecipeView<'_>, &mut ReservationWork, &mut usize) -> Result<T, &'static str>,
    ) -> Result<T, &'static str> {
        let mut work = ReservationWork::default();
        let result = (|| {
            self.require_current(store, &mut work)?;
            if !std::ptr::eq(catalog, self.catalog.as_ref()) {
                return Err("selected callable changed its actual current catalog");
            }
            check(&self.view(), &mut work, steps)
        })();
        *steps = steps.saturating_add(work.steps);
        result
    }
    pub(in crate::sema::crates::typed_headers) fn require_method_regions(
        &self,
        catalog: &Catalog,
        store: &Store,
        definition: DefId,
        scheme: &FunctionRegions,
        steps: &mut usize,
    ) -> Result<(), &'static str> {
        self.checked_call(catalog, store, steps, |view, work, steps| {
            let mut mapped =
                Borrowed::retained(catalog, view, &self.reader, &self.maps, store, work)
                    .map_err(|_| "selected callable lost its actual checked mapping")?;
            mapped_source::require_method_regions(
                catalog,
                self.descriptor()?,
                definition,
                scheme,
                steps,
                |reference| mapped.region(reference),
            )
        })
    }
    pub(in crate::sema::crates::typed_headers) fn require_method_declaration(
        &self,
        catalog: &Catalog,
        facts: &Declarations,
        definition: DefId,
        header: &Header,
        steps: &mut usize,
    ) -> Result<(), &'static str> {
        self.checked_call(catalog, &facts.types, steps, |view, work, steps| {
            let id = catalog.declaration(definition)?;
            catalog.validate_declaration(id, definition, DefKind::Function)?;
            let (provider, row, _) = catalog.declaration_coordinate(id)?;
            let descriptor = self.descriptor()?;
            if provider != descriptor.identity()
                || header.declaration != DeclarationOrigin::Imported(id)
                || !self.function(catalog, definition, header.body, header.universals)?
            {
                return Err("selected callable changed its actual declaration/body presence");
            }
            let rows = descriptor.rows();
            let Some(s::HeaderRef::Function(index)) = rows.header(row)? else {
                return Err("selected callable lost its actual function row");
            };
            let raw = rows.functions[index as usize];
            let constant = match raw.constness {
                Constness::Ordinary => false,
                Constness::Const => true,
                Constness::Conditional => {
                    return Err(
                        "selected callable requires its actual declaration-effect producer",
                    );
                }
            };
            let (all, prefix) = self
                .scopes
                .inputs(view, catalog, &facts.generics, row, work)
                .map_err(|_| "selected callable requires its actual installed declaration scope")?;
            if !Arc::ptr_eq(all, &header.inputs)
                || header.declared != prefix
                || prefix != raw.declared_inputs as usize
                || header.constant != constant
            {
                return Err(
                    "selected callable changed its actual complete input Arc/prefix/effect",
                );
            }
            let mut mapped =
                Borrowed::retained(catalog, view, &self.reader, &self.maps, &facts.types, work)
                    .map_err(|_| "selected callable lost its actual checked signature mapping")?;
            for (kind, raw, actual) in [
                (
                    s::BinderKind::OriginalFunction,
                    raw.original,
                    header.signature,
                ),
                (s::BinderKind::PublicFunction, raw.callable, header.callable),
            ] {
                signature_shapes::require(
                    rows,
                    &facts.types,
                    signature_shapes::SignaturePort {
                        owner: row,
                        kind,
                        raw,
                    },
                    actual,
                    steps,
                    |reference| mapped.ty(reference),
                )?;
            }
            // Body readiness is separate: header.environment may still be None.
            Ok(())
        })
    }
}
