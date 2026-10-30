//! Ordinary originals share the selected allocator, never selected authorization.
use super::{Prepared, Ready, SignaturePair, error, numbering_records};
use crate::{
    resolve::{
        crates::{ReservationWork, unpaid_provided::RecipeView},
        model::DefId,
        package::Error,
    },
    sema::{crates::typed_headers::Header, model::Declarations},
    support::crates::generics::source as s,
    types::ItemId,
};
use std::sync::Arc;

pub(in crate::sema::crates::typed_headers) struct OrdinaryRecords {
    prepared: Prepared,
}
pub(in crate::sema::crates::typed_headers) struct OrdinaryDomain {
    record: Arc<numbering_records::RecordedDomain>,
}
impl OrdinaryRecords {
    #[cfg(test)]
    pub(in crate::sema::crates::typed_headers) fn completed(&self) -> usize {
        self.prepared
            .ready
            .iter()
            .filter(|row| row.is_some())
            .count()
    }
    pub(in crate::sema::crates::typed_headers) fn new(
        view: &RecipeView<'_>,
        work: &mut ReservationWork,
    ) -> Result<Self, Error> {
        Ok(Self {
            prepared: Prepared::new(view, work)?,
        })
    }
    pub(in crate::sema::crates::typed_headers) fn prepare(
        &mut self,
        view: &RecipeView<'_>,
        facts: &Declarations,
        definition: DefId,
        header: &Header,
        work: &mut ReservationWork,
    ) -> Result<OrdinaryDomain, Error> {
        let expected = self.expected(view, facts, definition, header, work)?;
        let rows = view.rows()?;
        let index = expected.pair.function_row;
        let ready = &mut self.prepared.ready[index as usize];
        if ready.is_some_and(|previous| previous != expected) {
            return Err(error(
                "ordinary recording changed its exact current scope or signature",
            ));
        }
        if self.prepared.recordings.is_none() {
            self.prepared.recordings = Some(Box::new(numbering_records::Owner::new(rows, work)?));
        }
        let recordings = self
            .prepared
            .recordings
            .as_deref_mut()
            .ok_or_else(|| error("ordinary recording lost its prepaid owner"))?;
        recordings.complete(view, &facts.types, &expected, work)?;
        let record = recordings.selected_record(view, &facts.types, &expected, work)?;
        work.charge(1)?;
        *ready = Some(expected);
        Ok(OrdinaryDomain {
            record: Arc::clone(record),
        })
    }
    fn expected(
        &self,
        view: &RecipeView<'_>,
        facts: &Declarations,
        definition: DefId,
        header: &Header,
        work: &mut ReservationWork,
    ) -> Result<Ready, Error> {
        work.charge(12)?;
        let rows = view.rows()?;
        let catalog = facts
            .crates
            .as_deref()
            .and_then(|linkage| linkage.origin_catalog())
            .ok_or_else(|| error("ordinary recording lost its actual catalog"))?;
        let declaration = catalog.declaration(definition).map_err(error)?;
        let (provider, owner, _) = catalog.declaration_coordinate(declaration).map_err(error)?;
        let Some(s::HeaderRef::Function(index)) = rows.header(owner).map_err(error)? else {
            return Err(error("ordinary recording requires its actual function"));
        };
        let raw = rows.functions[index as usize];
        let s::FunctionBody::Original(body) = raw.body else {
            return Err(error("ordinary recording cannot invent an absent original"));
        };
        let actual_body = view.body(body, work)?;
        let registration = facts
            .lifetimes
            .callables
            .registration(ItemId(definition.0))
            .ok_or_else(|| error("ordinary recording lacks its actual callable registration"))?;
        original_formals(view, facts, body, registration.1, work)?;
        if provider != rows.descriptor().identity()
            || view.declaration(owner, work)? != declaration
            || catalog.origin(actual_body).map_err(error)?
                != header
                    .body
                    .ok_or_else(|| error("ordinary recording lost its header body"))?
            || registration.0.signature != header.callable
            || registration.0.parameters != 0
        {
            return Err(error(
                "ordinary recording changed its current header or callable owner",
            ));
        }
        Ok(Ready {
            pair: SignaturePair {
                declaration,
                body: Some(actual_body),
                original: header.signature,
                callable: header.callable,
                function_row: index,
            },
            header: raw,
            original: facts
                .types
                .kind(header.signature)
                .ok_or_else(|| error("ordinary original signature disappeared"))?,
            callable: facts
                .types
                .kind(header.callable)
                .ok_or_else(|| error("ordinary callable signature disappeared"))?,
            item: ItemId(definition.0),
            formals: registration.1.len(),
            scope: self.prepared.scope(rows, raw.scope)?,
        })
    }
    pub(in crate::sema::crates::typed_headers) fn get(
        &self,
        view: &RecipeView<'_>,
        facts: &Declarations,
        definition: DefId,
        header: &Header,
        work: &mut ReservationWork,
    ) -> Result<OrdinaryDomain, Error> {
        let expected = self.expected(view, facts, definition, header, work)?;
        if self.prepared.ready[expected.pair.function_row as usize] != Some(expected) {
            return Err(error(
                "ordinary original requires its actual pending current domain",
            ));
        }
        let record = self
            .prepared
            .recordings
            .as_deref()
            .ok_or_else(|| error("ordinary original requires its actual pending recording"))?
            .selected_record(view, &facts.types, &expected, work)?;
        work.charge(1)?;
        Ok(OrdinaryDomain {
            record: Arc::clone(record),
        })
    }
}
fn original_formals(
    view: &RecipeView<'_>,
    facts: &Declarations,
    body: crate::support::crates::owner_catalog::BodyRef,
    actual: &[crate::types::parameters::ParameterId],
    work: &mut ReservationWork,
) -> Result<(), Error> {
    let rows = view.rows()?;
    let input = rows
        .body_inputs
        .get(body.0 as usize)
        .filter(|input| input.body == body)
        .ok_or_else(|| error("ordinary recording lost its complete original formal domain"))?;
    work.charge(input.parameters.len().saturating_add(1))?;
    if input.parameters.len() != actual.len() {
        return Err(error(
            "ordinary recording changed its full original formal arity",
        ));
    }
    for (&slot, &id) in input.parameters.iter().zip(actual) {
        let slot = rows.slots.get(slot.0 as usize).ok_or_else(|| {
            error("ordinary recording original formal is outside its actual table")
        })?;
        let parameter = facts
            .types
            .parameter(id)
            .ok_or_else(|| error("ordinary recording lost its actual canonical formal"))?;
        let s::ParameterKind::Type { sized } = slot.kind else {
            return Err(error(
                "ordinary recording original formal is not an actual type",
            ));
        };
        if parameter.owner != ItemId(view.definition(slot.owner, work)?.0)
            || parameter.index != slot.index
            || parameter.kind
                != (crate::types::parameters::ParameterKind::Type {
                    sized: sized.is_some(),
                })
        {
            return Err(error(
                "ordinary recording changed its actual full formal owner or order",
            ));
        }
    }
    Ok(())
}
impl OrdinaryDomain {
    pub(in crate::sema::crates::typed_headers) fn raw(
        &self,
    ) -> Result<crate::support::crates::owner_catalog::BodyRef, Error> {
        match self.record.expected().header.body {
            s::FunctionBody::Original(body) => Ok(body),
            s::FunctionBody::Absent => Err(error("ordinary domain cannot invent an absent body")),
        }
    }
    pub(in crate::sema::crates::typed_headers) fn environment(
        &self,
    ) -> Option<&Arc<crate::types::parameters::environment::Environment>> {
        self.record.ordinary_environment.get()
    }
    pub(in crate::sema::crates::typed_headers) fn publish_environment(
        &self,
        environment: &Arc<crate::types::parameters::environment::Environment>,
        work: &mut ReservationWork,
    ) -> Result<(), Error> {
        work.charge(1)?;
        if let Some(current) = self.environment() {
            if !Arc::ptr_eq(current, environment) {
                return Err(error(
                    "ordinary original changed its actual constructed environment",
                ));
            }
            return Ok(());
        }
        self.record
            .ordinary_environment
            .set(Arc::clone(environment))
            .map_err(|_| error("ordinary original environment was already published"))
    }
    pub(in crate::sema::crates::typed_headers) fn recorded_region(
        &self,
        view: &RecipeView<'_>,
        reference: s::RegionRef,
        work: &mut ReservationWork,
    ) -> Result<Option<crate::types::RegionId>, Error> {
        self.record.recorded_region(view, reference, work)
    }
    pub(in crate::sema::crates::typed_headers) fn body(
        &self,
    ) -> Result<crate::resolve::body_origin::ImportedBodyId, Error> {
        self.record
            .expected()
            .pair
            .body
            .ok_or_else(|| error("ordinary domain lost its actual body"))
    }
    pub(in crate::sema::crates::typed_headers) fn require(
        &self,
        view: &RecipeView<'_>,
        store: &crate::types::Store,
        callables: &crate::types::callable::Templates,
        work: &mut ReservationWork,
    ) -> Result<(), Error> {
        work.charge(3)?;
        if callables
            .registration(self.record.expected().item)
            .is_none_or(|(template, parameters)| {
                template.signature != self.record.expected().pair.callable
                    || template.parameters != 0
                    || parameters.len() != self.record.expected().formals
            })
        {
            return Err(error(
                "ordinary domain changed its actual callable registration",
            ));
        }
        self.record.require(
            &numbering_records::DomainStage::new(view, store, self.record.expected(), work)?,
            store,
            work,
        )
    }
    pub(in crate::sema::crates::typed_headers) fn variable(
        &self,
        view: &RecipeView<'_>,
        store: &crate::types::Store,
        slot: u32,
        work: &mut ReservationWork,
    ) -> Result<crate::types::RegionId, Error> {
        self.record
            .region(view, store, self.record.expected(), slot, work)
    }
    pub(in crate::sema::crates::typed_headers) fn region(
        &self,
        view: &RecipeView<'_>,
        store: &crate::types::Store,
        reference: s::RegionRef,
        work: &mut ReservationWork,
    ) -> Result<Option<crate::types::RegionId>, Error> {
        let region = self.record.recorded_region(view, reference, work)?;
        if let Some(s::Region::Variable { slot, .. }) =
            view.rows()?.regions.get(reference.0 as usize)
        {
            return self.variable(view, store, *slot, work).map(Some);
        }
        Ok(region)
    }
    pub(in crate::sema::crates::typed_headers) fn normalization_interval(
        &self,
        view: &RecipeView<'_>,
        store: &crate::types::Store,
        index: usize,
        input: &s::NormalizationInput,
        work: &mut ReservationWork,
    ) -> Result<std::ops::Range<u32>, Error> {
        self.record.require(
            &numbering_records::DomainStage::new(view, store, self.record.expected(), work)?,
            store,
            work,
        )?;
        self.record.normalization_interval(view, index, input, work)
    }
}
