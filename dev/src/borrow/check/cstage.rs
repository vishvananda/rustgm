//! The concrete plan remains owned while its actual const eligibility is selected.
use super::{Environment, Error, Reused, loans, regions, structure};
use crate::{
    body::ir::Body,
    borrow::{
        destruction::{self, glue},
        loans::seal::Seal,
        order::Order,
        ownership::initialization,
    },
    types::{copy::Copies, destruction::regions::Regions},
};
use std::sync::Arc;
pub(crate) struct Initial<'body, 'slot> {
    facts: initialization::Facts<'body>,
    seal: Seal,
    order: Option<Order>,
    parameters: Option<Arc<crate::types::parameters::environment::Environment>>,
    slot: Option<&'slot mut Option<structure::Proved>>,
}
impl<'body> Initial<'body, '_> {
    pub(crate) fn body(&self) -> &'body Body {
        self.facts.body()
    }
}
impl Initial<'_, '_> {
    pub(crate) fn questions(
        &mut self,
        types: &mut crate::types::Store,
        definitions: &crate::types::aggregate::Definitions,
    ) -> Result<Option<&[crate::types::TypeId]>, &'static str> {
        self.facts.qualification_questions(types, definitions)
    }
}
pub(super) fn begin_qualification<S: std::hash::BuildHasher>(
    environment: &Environment<'_, S>,
    initial: &mut Initial<'_, '_>,
    contract: &regions::Body,
    (catalog, requirements): (
        &glue::Catalog<'_>,
        Option<&crate::body::drops::Requirements>,
    ),
    owner: Option<&Arc<()>>,
) -> Result<(), &'static str> {
    initial
        .seal
        .validate(environment, initial.body(), contract, owner)?;
    initial.facts.begin_qualification(
        environment.types,
        environment.definitions,
        &|item| catalog.custom(item),
        (initialization::Refinement::EligibilityOnly, requirements),
        &|ty, query| catalog.member_position(ty, query),
    )
}
pub(super) fn accept_qualification<S: std::hash::BuildHasher>(
    environment: &Environment<'_, S>,
    initial: &mut Initial<'_, '_>,
    contract: &regions::Body,
    answers: &crate::body::defining_cleanups::Qualification<'_>,
    owner: Option<&Arc<()>>,
) -> Result<(), &'static str> {
    initial
        .seal
        .validate(environment, initial.body(), contract, owner)?;
    initial.facts.accept_qualification(
        environment.types,
        environment.definitions,
        contract,
        answers,
    )
}
pub(crate) struct Prepared<'body, 'slot> {
    facts: initialization::Facts<'body>,
    plan: destruction::Plan,
    seal: Seal,
    order: Option<Order>,
    parameters: Option<Arc<crate::types::parameters::environment::Environment>>,
    slot: Option<&'slot mut Option<structure::Proved>>,
}
impl Prepared<'_, '_> {
    pub(super) fn body(&self) -> &Body {
        self.facts.body()
    }
    pub(crate) fn resolve_selected(
        &mut self,
        types: &mut crate::types::Store,
        definitions: &crate::types::aggregate::Definitions,
    ) -> Result<(), &'static str> {
        self.facts.resolve_selected(types, definitions)
    }
    pub(crate) fn active(&self) -> bool {
        self.plan
            .cleanups
            .iter()
            .any(|cleanup| cleanup.node.is_some())
    }
    pub(crate) fn eligibility(
        &self,
    ) -> impl Iterator<Item = Result<crate::body::defining_cleanups::Subject, &'static str>> + '_
    {
        let selected = self.facts.selected();
        let subjects = selected.map(crate::borrow::ownership::Selected::subjects);
        let full = subjects
            .and_then(Result::ok)
            .into_iter()
            .flatten()
            .copied()
            .map(Ok);
        let errors = subjects.and_then(Result::err).into_iter().map(Err);
        let whole = self
            .plan
            .cleanups
            .iter()
            .filter(move |_| selected.is_none())
            .filter_map(|cleanup| {
                cleanup.live_type.map(|ty| {
                    Ok(crate::body::defining_cleanups::Subject {
                        purpose: crate::body::defining_cleanups::Purpose::Body,
                        ty,
                        access: cleanup.node.is_some(),
                    })
                })
            });
        full.chain(errors).chain(whole)
    }
}
pub(super) fn prepare<'body, 'slot, S: std::hash::BuildHasher>(
    environment: &mut Environment<'_, S>,
    reused: &mut Reused,
    body: &'body Body,
    contract: &regions::Body,
    catalog: &mut glue::Catalog<'_>,
    structure: structure::Use<'body, 'slot>,
    owner: Option<Arc<()>>,
) -> Result<Prepared<'body, 'slot>, Error> {
    let expected = owner.clone();
    let initial = initialize_using(environment, body, contract, (structure, None), owner)?;
    select_initial(
        environment,
        reused,
        initial,
        contract,
        catalog,
        expected.as_ref(),
    )
}

pub(super) fn prepare_using<'body, 'slot, S: std::hash::BuildHasher>(
    environment: &mut Environment<'_, S>,
    reused: &mut Reused,
    body: &'body Body,
    contract: &regions::Body,
    catalog: &mut glue::Catalog<'_>,
    (structure, future): (
        structure::Use<'body, 'slot>,
        Option<&'body crate::body::coroutines::Permit>,
    ),
    owner: Option<Arc<()>>,
) -> Result<Prepared<'body, 'slot>, Error> {
    if future.is_none() {
        return prepare(
            environment,
            reused,
            body,
            contract,
            catalog,
            structure,
            owner,
        );
    }
    let initial = initialize_using(
        environment,
        body,
        contract,
        (structure, future),
        owner.clone(),
    )?;
    select_initial(
        environment,
        reused,
        initial,
        contract,
        catalog,
        owner.as_ref(),
    )
}
pub(super) fn initialize_using<'body, 'slot, S: std::hash::BuildHasher>(
    environment: &mut Environment<'_, S>,
    body: &'body Body,
    contract: &regions::Body,
    (structure, future): (
        structure::Use<'body, 'slot>,
        Option<&'body crate::body::coroutines::Permit>,
    ),
    owner: Option<Arc<()>>,
) -> Result<Initial<'body, 'slot>, Error> {
    let parameters = super::super::permissions::check(environment, contract)?;
    let mut copies = parameters
        .as_ref()
        .map(|e| Copies::in_environment(Arc::clone(e)));
    let error = |message: &str| Error {
        node: Some(body.source.into()),
        message: message.into(),
    };
    let (shared, slot) = match structure {
        structure::Use::Shared(shared) => (Some(shared), None),
        structure::Use::Keep(slot) => (None, Some(slot)),
        structure::Use::Own => (None, None),
    };
    let order = if shared.is_some() {
        None
    } else {
        Order::new(body)
    };
    let mut view = super::super::copy_view::CopyView::checked(
        copies.as_mut().unwrap_or(environment.copies),
        body,
        contract,
        environment.source_copy,
    )
    .and_then(|view| view.generated(environment.types, body, future))
    .map_err(error)?;
    let init = if let Some(shared) = shared {
        initialization::analyze_shared(
            environment.types,
            environment.definitions,
            environment.derives,
            &mut view,
            body,
            shared.order,
            shared.storage,
        )
    } else {
        initialization::analyze_ordered(
            environment.types,
            environment.definitions,
            environment.derives,
            &mut view,
            body,
            order.as_ref(),
        )
    }
    .map_err(|e| Error {
        node: Some(e.node),
        message: e.message.into(),
    })?;
    Ok(Initial {
        facts: init,
        seal: Seal::new(environment, body, contract, owner),
        order,
        slot,
        parameters,
    })
}
pub(super) fn select_initial<'body, 'slot, S: std::hash::BuildHasher>(
    environment: &mut Environment<'_, S>,
    reused: &mut Reused,
    mut initial: Initial<'body, 'slot>,
    contract: &regions::Body,
    catalog: &mut glue::Catalog<'_>,
    owner: Option<&Arc<()>>,
) -> Result<Prepared<'body, 'slot>, Error> {
    let body = initial.body();
    let error = |message: &str| Error {
        node: Some(body.source.into()),
        message: message.into(),
    };
    initial
        .seal
        .validate(environment, body, contract, owner)
        .map_err(error)?;
    let init = &mut initial.facts;
    let defining = owner.is_some() && body.const_context;
    if defining {
        init.refine_variants(
            environment.types,
            environment.definitions,
            &|item| catalog.custom(item),
            initialization::Refinement::EligibilityOnly,
            &|ty, query| catalog.member_position(ty, query),
        )
        .map_err(error)?;
    }
    let plan = if defining {
        destruction::select_reusing_defining(catalog, init, &mut reused.transitions)
    } else {
        destruction::select_reusing(catalog, init, &mut reused.transitions)
    }
    .map_err(|e| Error {
        node: Some(e.node),
        message: e.message.into(),
    })?;
    Ok(Prepared {
        facts: initial.facts,
        plan,
        seal: initial.seal,
        order: initial.order,
        slot: initial.slot,
        parameters: initial.parameters,
    })
}

#[derive(Clone, Copy)]
pub(super) enum RegionUse {
    Initial,
    Reproved,
}
pub(super) fn finish<S: std::hash::BuildHasher>(
    environment: &mut Environment<'_, S>,
    reused: &mut Reused,
    drop_regions: &mut Regions,
    prepared: Prepared<'_, '_>,
    contract: &regions::Body,
    owner: Option<&Arc<()>>,
    regions: RegionUse,
) -> Result<Option<destruction::Plan>, Error> {
    finish_sealed(
        environment,
        reused,
        drop_regions,
        prepared,
        contract,
        owner,
        regions,
    )
    .map(|(plan, _)| plan)
}

/// Preserve the same checked seal when an affine owner must move after this proof.
pub(super) fn finish_sealed<S: std::hash::BuildHasher>(
    environment: &mut Environment<'_, S>,
    reused: &mut Reused,
    drop_regions: &mut Regions,
    prepared: Prepared<'_, '_>,
    contract: &regions::Body,
    owner: Option<&Arc<()>>,
    regions: RegionUse,
) -> Result<(Option<destruction::Plan>, Seal), Error> {
    let error = |message: &str| Error {
        node: Some(prepared.body().source.into()),
        message: message.into(),
    };
    prepared
        .seal
        .validate(environment, prepared.body(), contract, owner)
        .map_err(error)?;
    let parameters = match regions {
        RegionUse::Initial => prepared.parameters.clone(),
        RegionUse::Reproved => super::super::permissions::check(environment, contract)?,
    };
    let mut scoped = parameters.map(|p| drop_regions.scoped(p));
    if owner.is_none() {
        loans::require_no_const_cleanup(
            prepared.body(),
            destruction::schedule::Schedule::concrete(&prepared.plan),
        )
        .map_err(|e| Error {
            node: Some(e.node),
            message: e.message.into(),
        })?;
    }
    let schedule = destruction::schedule::Schedule::concrete(&prepared.plan)
        .with_selected(prepared.body(), prepared.facts.selected())
        .map_err(error)?;
    // Only this exact sealed source-original plan reaches concrete native loans.
    // The public concrete-plan API retains its independent CTFE fence.
    loans::check_schedule(
        environment.types,
        environment.definitions,
        &mut reused.components,
        scoped.as_mut().unwrap_or(drop_regions),
        &prepared.facts,
        contract,
        schedule,
    )
    .map_err(|e| Error {
        node: Some(e.node),
        message: e.message.into(),
    })?;
    if let Some(slot) = prepared.slot
        && let std::borrow::Cow::Owned(storage) = prepared.facts.storage
    {
        *slot = Some(structure::Proved {
            order: prepared.order,
            storage,
        });
    }
    let plan = prepared
        .plan
        .cleanups
        .iter()
        .any(|c| c.node.is_some())
        .then(|| {
            let mut plan = prepared.plan;
            plan.shrink_to_fit();
            plan
        });
    Ok((plan, prepared.seal))
}
