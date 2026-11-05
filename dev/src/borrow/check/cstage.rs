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
    types::{TypeId, copy::Copies, destruction::regions::Regions},
};
use std::sync::Arc;
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
    pub(crate) fn eligibility(&self) -> impl Iterator<Item = Result<TypeId, &'static str>> + '_ {
        self.plan
            .cleanups
            .iter()
            .filter_map(|cleanup| cleanup.live_type.map(Ok))
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
    .map_err(|e| error(e.message))?;
    let plan = destruction::select_reusing(catalog, &init, &mut reused.transitions)
        .map_err(|e| error(e.message))?;
    Ok(Prepared {
        facts: init,
        plan,
        seal: Seal::new(environment, body, contract, owner),
        order,
        slot,
        parameters,
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
    loans::check_destruction(
        environment.types,
        environment.definitions,
        &mut reused.components,
        scoped.as_mut().unwrap_or(drop_regions),
        &prepared.facts,
        contract,
        &prepared.plan,
    )
    .map_err(|e| error(e.message))?;
    if let Some(slot) = prepared.slot
        && let std::borrow::Cow::Owned(storage) = prepared.facts.storage
    {
        *slot = Some(structure::Proved {
            order: prepared.order,
            storage,
        });
    }
    Ok(prepared
        .plan
        .cleanups
        .iter()
        .any(|c| c.node.is_some())
        .then(|| {
            let mut plan = prepared.plan;
            plan.shrink_to_fit();
            plan
        }))
}
