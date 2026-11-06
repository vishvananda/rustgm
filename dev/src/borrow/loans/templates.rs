//! Generic bodies share ordinary proofs with scoped moves and abstract cleanup.
mod staged;
use super::Analysis;
use crate::support::hash::HashMap;
use crate::{
    body::templates::Templates,
    borrow::{
        Error,
        destruction::symbolic,
        ownership::{BodyFacts, initialization},
        regions, storage,
    },
    types::{
        Store,
        copy::Copies,
        destruction::{NeedsDrop, regions::Regions},
        outlives::LiveRegions,
    },
};
pub(crate) use staged::{Initial, Prepared};

/// Prove moves, initialization, abstract destruction and loans in every template.
/// Provenance and promotion must already be checked on these retained bodies.
/// This result authorizes no specialization or native execution.
/// # Errors
/// Rejects missing contracts, invalid moves, destructor/loan conflicts and budgets.
pub fn verify<S: std::hash::BuildHasher>(
    environment: &mut crate::borrow::Environment<'_, S>,
    templates: &Templates,
    contracts: &regions::Analysis,
    custom: &[crate::types::ItemId],
    position: impl Fn(
        crate::types::TypeId,
        crate::body::drops::MemberQuery,
    ) -> Result<crate::body::drops::MemberPosition, &'static str>,
) -> Result<Vec<Analysis>, Error> {
    verify_iter(
        environment,
        templates.iter(),
        templates.len(),
        contracts,
        custom,
        position,
    )
}
/// The combined original component may include proof-only parents. Its rows are
/// checked identically and never enter the runtime template lookup catalog.
pub(crate) fn verify_iter<'a, S: std::hash::BuildHasher>(
    environment: &mut crate::borrow::Environment<'_, S>,
    templates: impl Iterator<Item = &'a crate::body::templates::Template>,
    count: usize,
    contracts: &regions::Analysis,
    custom: &[crate::types::ItemId],
    position: impl Fn(
        crate::types::TypeId,
        crate::body::drops::MemberQuery,
    ) -> Result<crate::body::drops::MemberPosition, &'static str>,
) -> Result<Vec<Analysis>, Error> {
    let error = |message: &str| Error {
        node: None,
        message: message.into(),
    };
    if contracts.bodies.len() != count {
        return Err(error(
            "template cleanup contracts have different body coverage",
        ));
    }
    let mut index = HashMap::with_capacity_and_hasher(
        contracts.bodies.len(),
        crate::support::hash::BuildFxHasher::default(),
    );
    for contract in &contracts.bodies {
        if index.insert(contract.key, contract).is_some() {
            return Err(error("duplicate template cleanup contract"));
        }
    }
    let mut verifier = Verifier::new(custom).map_err(error)?;
    let mut results = Vec::with_capacity(count);
    for template in templates {
        let body = &template.body;
        let error = |message: &str| Error {
            node: Some(body.source.into()),
            message: message.into(),
        };
        validate(template).map_err(error)?;
        let contract = index
            .remove(&body.key())
            .ok_or_else(|| error("template cleanup contract is missing or repeated"))?;
        results.push(verifier.body(
            environment,
            body,
            &template.destruction,
            &template.environment,
            contract,
            &position,
        )?);
    }
    if results.len() != count {
        return Err(error(
            "template cleanup iterator has different body coverage",
        ));
    }
    Ok(results)
}

/// Reusable ownership, cleanup and loan work for one demanded checked CFG.
pub(crate) struct Verifier {
    custom: crate::support::hash::HashSet<crate::types::ItemId>,
    needed: NeedsDrop,
    drops: Regions,
    components: LiveRegions,
    owner: Option<std::sync::Arc<()>>,
}
impl Verifier {
    pub(crate) fn new(custom: &[crate::types::ItemId]) -> Result<Self, &'static str> {
        Ok(Self {
            custom: custom.iter().copied().collect(),
            needed: NeedsDrop::new(custom.iter().copied()),
            drops: Regions::new(custom.iter().copied())?,
            components: LiveRegions::default(),
            owner: None,
        })
    }
    pub(crate) fn body<S: std::hash::BuildHasher>(
        &mut self,
        environment: &mut crate::borrow::Environment<'_, S>,
        body: &crate::body::ir::Body,
        destruction: &crate::body::drops::Requirements,
        parameters: &std::sync::Arc<crate::types::parameters::environment::Environment>,
        contract: &regions::Body,
        position: &impl Fn(
            crate::types::TypeId,
            crate::body::drops::MemberQuery,
        ) -> Result<crate::body::drops::MemberPosition, &'static str>,
    ) -> Result<Analysis, Error> {
        let prepared = self.prepare_body(
            environment,
            body,
            destruction,
            parameters,
            contract,
            (initialization::Refinement::Physical, position),
        )?;
        self.finish_initial(environment, prepared, contract)
    }
    pub(crate) fn begin<'a, S: std::hash::BuildHasher>(
        &mut self,
        environment: &mut crate::borrow::Environment<'_, S>,
        body: &'a crate::body::ir::Body,
        parameters: &std::sync::Arc<crate::types::parameters::environment::Environment>,
        contract: &regions::Body,
    ) -> Result<Initial<'a>, Error> {
        staged::require_arenas(environment, body)?;
        self.owner.get_or_insert_with(|| std::sync::Arc::new(()));
        self.initialize(environment, body, parameters, contract)
    }
    fn prepare_body<'a, S: std::hash::BuildHasher>(
        &mut self,
        environment: &mut crate::borrow::Environment<'_, S>,
        body: &'a crate::body::ir::Body,
        destruction: &crate::body::drops::Requirements,
        parameters: &std::sync::Arc<crate::types::parameters::environment::Environment>,
        contract: &regions::Body,
        (mode, position): (
            initialization::Refinement,
            &impl Fn(
                crate::types::TypeId,
                crate::body::drops::MemberQuery,
            ) -> Result<crate::body::drops::MemberPosition, &'static str>,
        ),
    ) -> Result<Prepared<'a>, Error> {
        let initial = self.initialize(environment, body, parameters, contract)?;
        self.select_initial(
            environment,
            initial,
            destruction,
            contract,
            (mode, position),
        )
    }
    pub(crate) fn initialize<'a, S: std::hash::BuildHasher>(
        &mut self,
        environment: &mut crate::borrow::Environment<'_, S>,
        body: &'a crate::body::ir::Body,
        parameters: &std::sync::Arc<crate::types::parameters::environment::Environment>,
        contract: &regions::Body,
    ) -> Result<Initial<'a>, Error> {
        let mut defining = crate::body::defining_cleanups::Work::new(1_000_000);
        if let Some(state) = contract.builtins.as_deref().and_then(|r| r.defining()) {
            state
                .check_environment(parameters, &mut defining)
                .map_err(|message| Error {
                    node: Some(body.source.into()),
                    message: message.into(),
                })?;
        }
        let parameters = crate::borrow::permissions::check(environment, contract)?
            .unwrap_or_else(|| std::sync::Arc::clone(parameters));
        let mut copies = Copies::in_environment(std::sync::Arc::clone(&parameters));
        let mut copy_view = crate::borrow::copy_view::CopyView::checked(
            &mut copies,
            body,
            contract,
            environment.source_copy,
        )
        .map_err(|message| Error {
            node: Some(body.source.into()),
            message: message.into(),
        })?;
        let order = crate::borrow::order::Order::new(body);
        let facts = initialization::analyze_ordered(
            environment.types,
            environment.definitions,
            environment.derives,
            &mut copy_view,
            body,
            order.as_ref(),
        )
        .map_err(|e| Error {
            node: Some(e.node),
            message: e.message.into(),
        })?;
        Ok(Initial::new(
            environment,
            facts,
            parameters,
            contract,
            self.owner.clone(),
            defining.steps,
        ))
    }
    pub(crate) fn select_initial<'a, S: std::hash::BuildHasher>(
        &mut self,
        environment: &mut crate::borrow::Environment<'_, S>,
        mut initial: Initial<'a>,
        destruction: &crate::body::drops::Requirements,
        contract: &regions::Body,
        (mode, position): (
            initialization::Refinement,
            &impl Fn(
                crate::types::TypeId,
                crate::body::drops::MemberQuery,
            ) -> Result<crate::body::drops::MemberPosition, &'static str>,
        ),
    ) -> Result<Prepared<'a>, Error> {
        initial
            .require(environment, contract, self.owner.as_ref())
            .map_err(|message| Error {
                node: Some(initial.body().source.into()),
                message: message.into(),
            })?;
        let body = initial.body();
        let parameters = &initial.parameters;
        let facts = &mut initial.facts;
        let plan = if no_cleanup(destruction) {
            facts
                .destructor_moves(environment.types, |item| self.custom.contains(&item))
                .map_err(|e| Error {
                    node: Some(e.node),
                    message: e.message.into(),
                })?;
            None
        } else {
            let mut needed = self.needed.scoped(std::sync::Arc::clone(parameters));
            let select = if matches!(mode, initialization::Refinement::Defining)
                && environment.source_copy
                && (body.const_context
                    || contract
                        .builtins
                        .as_deref()
                        .is_some_and(|r| r.defining().is_some()))
            {
                symbolic::select_defining
            } else {
                symbolic::select
            };
            Some(
                select(
                    environment.types,
                    environment.definitions,
                    &mut needed,
                    facts,
                    |item| self.custom.contains(&item),
                    position,
                )
                .map_err(|e| Error {
                    node: Some(e.node),
                    message: e.message.into(),
                })?,
            )
        };
        Ok(initial.select(plan))
    }
}

pub(crate) fn no_cleanup(requirements: &crate::body::drops::Requirements) -> bool {
    !requirements.locals.contains(&true)
        && !requirements.values.contains(&true)
        && !requirements.projections.contains(&true)
        && requirements.sites.is_empty()
}

pub(crate) fn validate(template: &crate::body::templates::Template) -> Result<(), &'static str> {
    let body = &template.body;
    if template.destruction.locals.len() != body.locals.len()
        || template.destruction.values.len() != body.values.len()
        || template.destruction.projections.len() != body.projections.len()
    {
        return Err("template loan destruction facts have different body coverage");
    }
    Ok(())
}

/// Check each selected template using its scoped moves and proved provenance.
/// Possible destruction still requires the separate abstract cleanup timeline.
/// The result cannot authorize concrete specialization or execution.
/// # Errors
/// Rejects incomplete/foreign facts, pending cleanup, loan conflicts and budgets.
pub fn check(
    types: &Store,
    components: &mut LiveRegions,
    templates: &Templates,
    contracts: &regions::Analysis,
    ownership: &[BodyFacts],
) -> Result<Vec<Analysis>, Error> {
    if ownership.len() != templates.len() || contracts.bodies.len() != templates.len() {
        return Err(Error {
            node: templates.bodies.first().map(|t| t.body.source.root_site()),
            message: "template loan facts have different body coverage".into(),
        });
    }
    let mut index = HashMap::with_capacity_and_hasher(
        contracts.bodies.len(),
        crate::support::hash::BuildFxHasher::default(),
    );
    for contract in &contracts.bodies {
        if index.insert(contract.key, contract).is_some() {
            return Err(Error {
                node: Some(contract.source.into()),
                message: "duplicate template loan contract".into(),
            });
        }
    }
    let mut results = Vec::with_capacity(templates.len());
    for (ordinal, template) in templates.iter().enumerate() {
        let body = &template.body;
        let error = |message: &str| Error {
            node: Some(body.source.into()),
            message: message.into(),
        };
        validate(template).map_err(error)?;
        if template.destruction.locals.contains(&true)
            || template.destruction.values.contains(&true)
            || template
                .destruction
                .sites
                .iter()
                .any(|site| matches!(site.trigger, crate::body::drops::Trigger::Store(_)))
        {
            return Err(error(
                "generic loan checking requires abstract cleanup for possible destruction",
            ));
        }
        let contract = index
            .remove(&body.key())
            .ok_or_else(|| error("template loan contract is missing or repeated"))?;
        let storage = storage::check(body).map_err(|e| Error {
            node: Some(e.node),
            message: e.message.into(),
        })?;
        results.push(
            super::check(
                types,
                components,
                body,
                contract,
                &ownership[ordinal],
                &storage,
            )
            .map_err(|e| Error {
                node: Some(e.node),
                message: e.message.into(),
            })?,
        );
    }
    Ok(results)
}
