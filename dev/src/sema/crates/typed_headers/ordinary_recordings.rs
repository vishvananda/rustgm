//! Ordinary originals share the selected allocator, never selected authorization.
use super::{Prepared, Ready, SignaturePair, error, numbering_records};
use crate::sema::crates::typed_headers::mapped_source::slot_bindings;
use crate::{
    resolve::{
        crates::{ReservationWork, unpaid_provided::RecipeView},
        model::DefId,
        package::Error,
    },
    sema::{crates::typed_headers::Header, model::Declarations},
    support::{
        crates::{generics::source as s, owner_catalog::BodyRef},
        hash::HashMap,
        retained,
    },
    types::ItemId,
};
use numbering_records::{ClosureHeader, InitializerHeader, RecordedDomain};
use std::sync::Arc;

pub(in crate::sema::crates::typed_headers) struct OrdinaryRecords {
    prepared: Prepared,
    initializers: HashMap<BodyRef, Arc<RecordedDomain>>,
    closures: HashMap<BodyRef, Arc<RecordedDomain>>,
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
        work.reserve(size_of::<Self>().saturating_sub(size_of::<Prepared>()))?;
        Ok(Self {
            prepared: Prepared::new(view, work)?,
            initializers: HashMap::default(),
            closures: HashMap::default(),
        })
    }

    pub(in crate::sema::crates::typed_headers) fn prepare_closure(
        &mut self,
        view: &RecipeView<'_>,
        facts: &Declarations,
        expected: ClosureHeader,
        work: &mut ReservationWork,
    ) -> Result<OrdinaryDomain, Error> {
        work.charge(1)?;
        let stage = numbering_records::DomainStage::closure(view, &facts.types, expected, work)?;
        if let Some(record) = self.closures.get(&expected.raw) {
            record.require(&stage, &facts.types, work)?;
            return Ok(OrdinaryDomain {
                record: Arc::clone(record),
            });
        }
        let (_, formals) = facts
            .generics
            .callable_parameters(&facts.types, expected.owner)
            .map_err(error)?;
        original_formals(view, facts, expected.raw, &formals, work)?;
        let before = retained::map(&self.closures);
        let growth = retained::Growth::map::<BodyRef, Arc<RecordedDomain>>(
            self.closures.len(),
            self.closures.capacity(),
            1,
        );
        work.charge(growth.work)?;
        work.reserve(growth.bytes)?;
        let record = Arc::new(stage.record(view, work)?);
        self.closures.insert(expected.raw, Arc::clone(&record));
        work.retained = work
            .retained
            .saturating_sub(growth.bytes)
            .saturating_add(retained::map(&self.closures).saturating_sub(before));
        Ok(OrdinaryDomain { record })
    }

    pub(in crate::sema::crates::typed_headers) fn closure(
        &self,
        view: &RecipeView<'_>,
        facts: &Declarations,
        expected: ClosureHeader,
        work: &mut ReservationWork,
    ) -> Result<OrdinaryDomain, Error> {
        work.charge(1)?;
        let record = self
            .closures
            .get(&expected.raw)
            .ok_or_else(|| error("ordinary closure requires its actual numbering domain"))?;
        let stage = numbering_records::DomainStage::closure(view, &facts.types, expected, work)?;
        record.require(&stage, &facts.types, work)?;
        Ok(OrdinaryDomain {
            record: Arc::clone(record),
        })
    }

    pub(in crate::sema::crates::typed_headers) fn domain_for_raw(
        &self,
        view: &RecipeView<'_>,
        body: BodyRef,
    ) -> Result<Option<OrdinaryDomain>, Error> {
        let rows = view.rows()?;
        let Some(owner) = rows.body_owners().get(body.0 as usize) else {
            return Ok(None);
        };
        let record = match owner.kind {
            crate::support::crates::owner_catalog::BodyKind::Function => {
                if let Some(s::HeaderRef::Function(index)) =
                    rows.header(owner.declaration).map_err(error)?
                {
                    self.prepared
                        .recordings
                        .as_deref()
                        .and_then(|recordings| recordings.entry(index))
                        .cloned()
                } else {
                    None
                }
            }
            crate::support::crates::owner_catalog::BodyKind::Constant => {
                self.initializers.get(&body).cloned()
            }
            crate::support::crates::owner_catalog::BodyKind::Closure => {
                self.closures.get(&body).cloned()
            }
            crate::support::crates::owner_catalog::BodyKind::Anonymous
            | crate::support::crates::owner_catalog::BodyKind::Static => None,
        };
        Ok(record.map(|record| OrdinaryDomain { record }))
    }

    pub(in crate::sema::crates::typed_headers) fn prepare_initializer(
        &mut self,
        view: &RecipeView<'_>,
        facts: &Declarations,
        expected: InitializerHeader,
        work: &mut ReservationWork,
    ) -> Result<OrdinaryDomain, Error> {
        work.charge(1)?;
        let stage =
            numbering_records::DomainStage::initializer(view, &facts.types, expected, work)?;
        if let Some(record) = self.initializers.get(&expected.raw) {
            record.require(&stage, &facts.types, work)?;
            return Ok(OrdinaryDomain {
                record: Arc::clone(record),
            });
        }
        let (_, formals) = facts
            .generics
            .callable_parameters(&facts.types, expected.definition)
            .map_err(error)?;
        original_formals(view, facts, expected.raw, &formals, work)?;
        let before = retained::map(&self.initializers);
        let growth = retained::Growth::map::<BodyRef, Arc<RecordedDomain>>(
            self.initializers.len(),
            self.initializers.capacity(),
            1,
        );
        work.charge(growth.work)?;
        work.reserve(growth.bytes)?;
        let record = Arc::new(stage.record(view, work)?);
        self.initializers.insert(expected.raw, Arc::clone(&record));
        work.retained = work
            .retained
            .saturating_sub(growth.bytes)
            .saturating_add(retained::map(&self.initializers).saturating_sub(before));
        Ok(OrdinaryDomain { record })
    }

    pub(in crate::sema::crates::typed_headers) fn initializer(
        &self,
        view: &RecipeView<'_>,
        facts: &Declarations,
        expected: InitializerHeader,
        work: &mut ReservationWork,
    ) -> Result<OrdinaryDomain, Error> {
        work.charge(1)?;
        let record = self
            .initializers
            .get(&expected.raw)
            .ok_or_else(|| error("ordinary initializer requires its actual numbering domain"))?;
        let stage =
            numbering_records::DomainStage::initializer(view, &facts.types, expected, work)?;
        record.require(&stage, &facts.types, work)?;
        Ok(OrdinaryDomain {
            record: Arc::clone(record),
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
            || !registered_callable(&facts.types, view, raw, header.callable, registration.0)
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
/// A callable's registration is its public signature, except that a
/// version-thirteen inherent method's template, rebuilt at staging from the
/// same header, leads with its captured slots as early parameters and keeps
/// only its own late slots in its frame.
fn registered_callable(
    store: &crate::types::Store,
    view: &RecipeView<'_>,
    function: s::FunctionHeader,
    callable: crate::types::TypeId,
    template: crate::types::callable::Template,
) -> bool {
    use crate::types::Kind;
    let early = function
        .regions
        .early
        .end
        .saturating_sub(function.regions.early.start);
    let late = function
        .regions
        .late
        .end
        .saturating_sub(function.regions.late.start);
    if function.regions.captures == 0
        && early == 0
        && function.regions.bounds.start == function.regions.bounds.end
        && late == function.regions.total
    {
        return template.signature == callable && template.parameters == 0;
    }
    let (
        Some(Kind::Function {
            bound_regions,
            parameters,
            unsafe_,
            convention,
            ..
        }),
        Some(Kind::Function {
            parameters: public,
            unsafe_: public_unsafe,
            convention: public_convention,
            ..
        }),
    ) = (store.kind(template.signature), store.kind(callable))
    else {
        return false;
    };
    (function.regions.captures == 0
        || view.rows().is_ok_and(|rows| {
            super::staged_headers::scheme_ports::associated_captures(rows.descriptor(), &function)
        }))
        && template.parameters == function.regions.captures.saturating_add(early)
        && bound_regions == late
        && store.list(parameters).map(<[_]>::len) == store.list(public).map(<[_]>::len)
        && unsafe_ == public_unsafe
        && convention == public_convention
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
        .get(rows.original_index(body).map_err(error)?)
        .filter(|input| input.body == body)
        .ok_or_else(|| error("ordinary recording lost its complete original formal domain"))?;
    work.charge(input.parameters.len().saturating_add(1))?;
    if input.parameters.len() != actual.len() {
        return Err(error(
            "ordinary recording changed its full original formal arity",
        ));
    }
    let actual = scope_order(facts, actual, work)?;
    for (&slot, &id) in input.parameters.iter().zip(&actual) {
        let slot = rows.slots.get(slot.0 as usize).ok_or_else(|| {
            error("ordinary recording original formal is outside its actual table")
        })?;
        let parameter = facts
            .types
            .parameter(id)
            .ok_or_else(|| error("ordinary recording lost its actual canonical formal"))?;
        let kind = match slot.kind {
            s::ParameterKind::Type { sized } => crate::types::parameters::ParameterKind::Type {
                sized: sized.is_some(),
            },
            // Version thirteen: a const formal keeps its complete position.
            s::ParameterKind::Const { ty } => crate::types::parameters::ParameterKind::Const {
                ty: slot_bindings::find_const_type(
                    &facts.types,
                    s::const_formal_type(&rows.types, ty).map_err(error)?,
                    |row| view.definition(row, work),
                )?,
            },
            s::ParameterKind::Region => {
                return Err(error(
                    "ordinary recording original formal is not an actual type",
                ));
            }
        };
        if parameter.owner != ItemId(view.definition(slot.owner, work)?.0)
            || parameter.index != slot_bindings::type_index(&rows.declarations, slot)
            || parameter.kind != kind
        {
            return Err(error(
                "ordinary recording changed its actual full formal owner or order",
            ));
        }
    }
    Ok(())
}
/// A callable registration lists its outermost scope's formals first; a body
/// contract lists them as `parameters_in_scope` does, its own scope first and
/// then each enclosing scope. Regroup by owner, keeping each owner's order.
fn scope_order(
    facts: &Declarations,
    registered: &[crate::types::parameters::ParameterId],
    work: &mut ReservationWork,
) -> Result<Vec<crate::types::parameters::ParameterId>, Error> {
    let owner = |id: crate::types::parameters::ParameterId| {
        facts
            .types
            .parameter(id)
            .map(|parameter| parameter.owner)
            .ok_or_else(|| error("ordinary recording lost its actual canonical formal"))
    };
    work.charge(registered.len().saturating_add(1))?;
    let mut ordered = Vec::with_capacity(registered.len());
    let mut end = registered.len();
    while let Some(&last) = end.checked_sub(1).and_then(|index| registered.get(index)) {
        let group = owner(last)?;
        let mut start = end - 1;
        while let Some(&previous) = start.checked_sub(1).and_then(|index| registered.get(index))
            && owner(previous)? == group
        {
            start -= 1;
        }
        ordered.extend_from_slice(&registered[start..end]);
        end = start;
    }
    Ok(ordered)
}
impl OrdinaryDomain {
    pub(in crate::sema::crates::typed_headers) fn raw(
        &self,
    ) -> Result<crate::support::crates::owner_catalog::BodyRef, Error> {
        let Some(expected) = self.record.expected_function() else {
            return Ok(self.record.raw());
        };
        match expected.header.body {
            s::FunctionBody::Original(body) => Ok(body),
            s::FunctionBody::CompilerOperation(_) => Err(error(
                "compiler-operation execution requires genuine future state ownership",
            )),
            s::FunctionBody::Absent | s::FunctionBody::Intrinsic(_) => {
                Err(error("ordinary domain cannot invent an absent body"))
            }
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
        self.record.expected_function().map_or_else(
            || Ok(self.record.body()),
            |expected| {
                expected
                    .pair
                    .body
                    .ok_or_else(|| error("ordinary domain lost its actual body"))
            },
        )
    }
    pub(in crate::sema::crates::typed_headers) fn require(
        &self,
        view: &RecipeView<'_>,
        store: &crate::types::Store,
        callables: &crate::types::callable::Templates,
        work: &mut ReservationWork,
    ) -> Result<(), Error> {
        work.charge(3)?;
        if let Some(expected) = self.record.expected_function()
            && callables
                .registration(expected.item)
                .is_none_or(|(template, parameters)| {
                    !registered_callable(
                        store,
                        view,
                        expected.header,
                        expected.pair.callable,
                        template,
                    ) || parameters.len() != expected.formals
                })
        {
            return Err(error(
                "ordinary domain changed its actual callable registration",
            ));
        }
        self.record.require_current(view, store, work)
    }
    pub(in crate::sema::crates::typed_headers) fn variable(
        &self,
        view: &RecipeView<'_>,
        store: &crate::types::Store,
        slot: u32,
        work: &mut ReservationWork,
    ) -> Result<crate::types::RegionId, Error> {
        self.record.require_current(view, store, work)?;
        self.record.slot(slot, work)
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
    pub(in crate::sema::crates::typed_headers) fn normalization_intervals(
        &self,
        view: &RecipeView<'_>,
        store: &crate::types::Store,
        index: usize,
        input: &s::NormalizationInput,
        work: &mut ReservationWork,
    ) -> Result<
        Vec<(
            crate::support::crates::generics::Range,
            std::ops::Range<u32>,
        )>,
        Error,
    > {
        self.record.require_current(view, store, work)?;
        self.record
            .normalization_intervals(view, index, input, work)
    }
    #[cfg(test)]
    pub(in crate::sema::crates::typed_headers) fn normalization_interval(
        &self,
        view: &RecipeView<'_>,
        store: &crate::types::Store,
        index: usize,
        input: &s::NormalizationInput,
        work: &mut ReservationWork,
    ) -> Result<std::ops::Range<u32>, Error> {
        self.record.require_current(view, store, work)?;
        self.record.normalization_interval(view, index, input, work)
    }
}
