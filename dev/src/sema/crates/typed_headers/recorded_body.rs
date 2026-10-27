//! A stack lookup borrows the actual selected body, defining Arc and recording.
use super::{
    Arc, Body, Declarations, Demand, Environment, Error, Owner, Prepared, Ready, RecipeView,
    ReservationWork, error, numbering_records, originals,
};
use crate::{
    support::crates::generics::source as s,
    types::{RegionId, Store},
};

pub(in crate::sema::crates::typed_headers) struct OriginalBody<'a> {
    owner: &'a Owner,
    prepared: &'a Prepared,
    expected: Ready,
    body: crate::resolve::body_origin::ImportedBodyId,
    environment: &'a Arc<Environment>,
    record: &'a numbering_records::RecordedDomain,
}
impl OriginalBody<'_> {
    pub(in crate::sema::crates::typed_headers) fn environment(&self) -> &Arc<Environment> {
        self.environment
    }
    pub(in crate::sema::crates::typed_headers) fn require(
        &self,
        view: &RecipeView<'_>,
        facts: &Declarations,
        work: &mut ReservationWork,
    ) -> Result<(), Error> {
        self.prepared.require_in(
            &facts.lifetimes.callables,
            view,
            &facts.types,
            self.expected.header.declaration,
            self.expected.pair,
            work,
        )?;
        let body = self
            .owner
            .bodies
            .get(self.expected.pair.function_row as usize)
            .and_then(Option::as_deref)
            .ok_or_else(|| error("selected original lost its actual defining body"))?;
        let origin = match body {
            Body::Ready { environment, .. } if Arc::ptr_eq(environment, self.environment) => {
                originals::Origin::Declaration
            }
            Body::Contract { product, .. }
                if Arc::ptr_eq(&product.environment, self.environment) =>
            {
                product.seal
            }
            Body::Numbered { product, .. }
                if Arc::ptr_eq(&product.environment, self.environment) =>
            {
                originals::Origin::Numbered
            }
            _ => {
                return Err(error(
                    "selected original changed its actual defining environment",
                ));
            }
        };
        if self.owner.cached(
            view,
            facts,
            self.prepared,
            &originals::BodyDemand {
                expected: self.expected,
                origin,
            },
            work,
        )? != Some(Demand::Ready(()))
        {
            return Err(error(
                "selected original requires its pending defining environment",
            ));
        }
        self.record(view, &facts.types, work)?;
        Ok(())
    }
    pub(in crate::sema::crates::typed_headers) const fn body(
        &self,
    ) -> crate::resolve::body_origin::ImportedBodyId {
        self.body
    }
    pub(in crate::sema::crates::typed_headers) fn require_cursor(
        &self,
        view: &RecipeView<'_>,
        store: &Store,
        callables: &crate::types::callable::Templates,
        work: &mut ReservationWork,
    ) -> Result<(), Error> {
        self.prepared.require_in(
            callables,
            view,
            store,
            self.expected.header.declaration,
            self.expected.pair,
            work,
        )?;
        let record = self.record(view, store, work)?;
        if let Some(Body::Numbered { scope, product, .. }) = self
            .owner
            .bodies
            .get(self.expected.pair.function_row as usize)
            .and_then(Option::as_deref)
        {
            let declaration = self
                .owner
                .scopes
                .get(*scope)
                .and_then(std::cell::OnceCell::get)
                .ok_or_else(|| error("selected original lost its actual declaration"))?;
            self.prepared
                .recordings
                .as_deref()
                .ok_or_else(|| error("selected original lost its actual recording owner"))?
                .require_neutral(
                    &numbering_records::Request {
                        view,
                        store,
                        expected: &self.expected,
                        declaration: &declaration.environment,
                    },
                    &product.recording,
                    self.environment,
                    work,
                )?;
        }
        // Keep the selected record borrow tied to this immutable body lookup.
        let _ = record;
        Ok(())
    }
    fn record(
        &self,
        view: &RecipeView<'_>,
        store: &Store,
        work: &mut ReservationWork,
    ) -> Result<&Arc<numbering_records::RecordedDomain>, Error> {
        let actual = self
            .prepared
            .recordings
            .as_deref()
            .ok_or_else(|| error("selected original requires its actual recorded domain"))?
            .selected_record(view, store, &self.expected, work)?;
        if !std::ptr::eq(actual.as_ref(), self.record) {
            return Err(error(
                "selected original changed its immutable record allocation",
            ));
        }
        Ok(actual)
    }
    pub(in crate::sema::crates::typed_headers) fn parameter(
        &self,
        view: &RecipeView<'_>,
        reference: s::SlotRef,
        work: &mut ReservationWork,
    ) -> Result<crate::types::parameters::ParameterId, Error> {
        let leaf = self
            .owner
            .scopes
            .get(self.expected.scope.0)
            .and_then(std::cell::OnceCell::get)
            .map(Box::as_ref)
            .ok_or_else(|| error("selected original lost its actual complete formal scope"))?;
        self.owner
            .original_parameter(view.rows()?, self.prepared, leaf, reference, work)
    }
    pub(in crate::sema::crates::typed_headers) fn region_recorded(
        &self,
        view: &RecipeView<'_>,
        reference: s::RegionRef,
        work: &mut ReservationWork,
    ) -> Result<Option<RegionId>, Error> {
        self.record.recorded_region(view, reference, work)
    }
    pub(in crate::sema::crates::typed_headers) fn region(
        &self,
        view: &RecipeView<'_>,
        store: &Store,
        reference: s::RegionRef,
        work: &mut ReservationWork,
    ) -> Result<Option<RegionId>, Error> {
        work.charge(1)?;
        match view.rows()?.regions.get(reference.0 as usize) {
            Some(s::Region::Variable { body, slot }) => {
                if view.body(*body, work)?
                    != self
                        .expected
                        .pair
                        .body
                        .ok_or_else(|| error("selected original lost its actual body identity"))?
                {
                    return Err(error(
                        "selected original region belongs to another actual body",
                    ));
                }
                self.record(view, store, work)?
                    .region(view, store, &self.expected, *slot, work)
                    .map(Some)
            }
            Some(_) => Ok(None),
            None => Err(error(
                "selected original region is outside its actual domain",
            )),
        }
    }
}
impl Prepared {
    pub(in crate::sema::crates::typed_headers) fn prepare_original_roots(
        &mut self,
        view: RecipeView<'_>,
        state: &mut super::State,
        facts: &mut Declarations,
        declaration: crate::support::crates::owner_catalog::DeclarationRef,
        work: &mut ReservationWork,
    ) -> Result<Demand<()>, Error> {
        use crate::sema::crates::typed_headers::recipe_cursor::{Recorded, Reserved};
        let rows = view.rows()?;
        let Some(s::HeaderRef::Function(index)) = rows.header(declaration).map_err(error)? else {
            return Err(error(
                "selected original roots require their actual function",
            ));
        };
        let expected = self
            .ready
            .get(index as usize)
            .copied()
            .flatten()
            .ok_or_else(|| error("selected original roots require their actual full callable"))?;
        self.require_in(
            &facts.lifetimes.callables,
            &view,
            &facts.types,
            declaration,
            expected.pair,
            work,
        )?;
        self.record_selected(&view, &facts.types, &expected, work)?;
        let body = self.original_body(&view, facts, declaration, work)?;
        let definition = view.definition(declaration, work)?;
        let header = facts
            .crates
            .as_deref()
            .and_then(|linkage| linkage.originals.as_deref())
            .and_then(|headers| headers.entries.get(&definition))
            .ok_or_else(|| error("selected original roots lost their actual header"))?;
        if let Some(current) = &header.environment {
            if !Arc::ptr_eq(current, body.environment()) {
                return Err(error(
                    "selected original roots changed their actual completed body Arc",
                ));
            }
            return Ok(Demand::Ready(()));
        }
        let s::FunctionBody::Original(raw) = expected.header.body else {
            return Err(error(
                "selected original roots require their actual raw original",
            ));
        };
        let raw = &rows.originals[raw.0 as usize];
        work.charge(
            raw.locals
                .len()
                .saturating_add(raw.values.len())
                .saturating_add(1),
        )?;
        let mut port = Recorded {
            port: Reserved::new(view, &state.maps, &facts.types, work)?,
            body: &body,
            callables: &facts.lifetimes.callables,
        };
        for root in raw
            .locals
            .iter()
            .map(|local| local.ty)
            .chain(raw.values.iter().map(|value| value.ty))
            .chain(std::iter::once(raw.result))
        {
            body.require(port.port.recipe_view(), facts, work)?;
            if let Demand::Blocked(dependency) =
                state
                    .cursor
                    .demand(&mut port, &mut facts.types, root, work)?
            {
                return Ok(Demand::Blocked(dependency));
            }
        }
        body.require(port.port.recipe_view(), facts, work)?;
        let Some(crate::sema::crates::ImportedOwners::Provided(mapping)) =
            facts.associated.imported_owner()
        else {
            return Err(error(
                "selected body requests lost their actual mapped owner",
            ));
        };
        let mut applications = mapping.application_writer(
            &body.owner.applications,
            port.port.recipe_view(),
            &facts.types,
            work,
        )?;
        let mut inputs = super::ports::Adapter {
            port,
            cursor: &mut state.cursor,
        };
        super::body_requests::prepare(
            raw.body,
            &mut inputs,
            &mut applications,
            &mut facts.types,
            work,
        )?;
        work.charge(1)?;
        let environment = Arc::clone(body.environment());
        let header = facts
            .crates
            .as_deref_mut()
            .and_then(|linkage| linkage.originals.as_deref_mut())
            .and_then(|headers| headers.entries.get_mut(&definition))
            .ok_or_else(|| {
                error("selected original roots lost their actual header before publication")
            })?;
        header.environment = Some(environment);
        Ok(Demand::Ready(()))
    }
    pub(in crate::sema::crates::typed_headers) fn original_body(
        &self,
        view: &RecipeView<'_>,
        facts: &Declarations,
        owner: crate::support::crates::owner_catalog::DeclarationRef,
        work: &mut ReservationWork,
    ) -> Result<OriginalBody<'_>, Error> {
        work.charge(2)?;
        let Some(s::HeaderRef::Function(index)) = view.rows()?.header(owner).map_err(error)? else {
            return Err(error(
                "selected original requires its actual function header",
            ));
        };
        let expected = self
            .ready
            .get(index as usize)
            .copied()
            .flatten()
            .ok_or_else(|| {
                error("selected original requires its actual full callable readiness")
            })?;
        let owner = self
            .environments
            .as_deref()
            .ok_or_else(|| error("selected original requires its actual defining owner"))?;
        let body = owner
            .bodies
            .get(index as usize)
            .and_then(Option::as_deref)
            .ok_or_else(|| error("selected original requires its pending defining body"))?;
        let environment = match body {
            Body::Ready { environment, .. } => environment,
            Body::Contract { product, .. } => &product.environment,
            Body::Numbered { product, .. } => &product.environment,
            Body::Blocked { .. } => {
                return Err(error("selected original defining body remains pending"));
            }
        };
        let record = self
            .recordings
            .as_deref()
            .ok_or_else(|| error("selected original requires its actual recorded domain"))?
            .selected_record(view, &facts.types, &expected, work)?;
        let result = OriginalBody {
            owner,
            prepared: self,
            expected,
            body: expected
                .pair
                .body
                .ok_or_else(|| error("selected original lost its actual body identity"))?,
            environment,
            record,
        };
        result.require(view, facts, work)?;
        Ok(result)
    }
}
