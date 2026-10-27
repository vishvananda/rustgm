//! Source receiver compatibility is a private ordinary trait proof, never a call grant.
use super::Error;
use crate::{
    resolve::{expansion::Report, model::DefId},
    sema::{
        compiler_items::headers::Trait,
        header_attributes,
        model::Declarations,
        traits::{assumptions::Assumptions, selection},
    },
    support::{bindings::LangItem, retained::Growth},
    types::{
        ItemId, Kind, Store, TypeId,
        arguments::Argument,
        infer::{Context, Instantiation},
        parameters::{
            ParameterId, ParameterKind,
            environment::Environment,
            predicates::{Bound, Predicate},
        },
        substitute,
    },
};
use std::sync::Arc;

struct Protocol {
    dispatch: ItemId,
    unsize: ItemId,
    meta: ItemId,
    target: TypeId,
    parameter: ParameterId,
}

pub(super) fn authenticate(
    report: &Report,
    facts: &Declarations,
    principal: DefId,
) -> Result<bool, Error> {
    let Some(catalog) = facts.source_bindings.as_deref() else {
        return Ok(false);
    };
    if catalog.language(LangItem::DispatchFromDyn).is_none() {
        return Ok(false);
    }
    catalog.require_provider_report(report)?;
    if !facts
        .generics
        .implementations
        .as_ref()
        .is_some_and(|registry| registry.headers_validated())
    {
        return Err(
            "source receiver compatibility requires the completed immutable impl epoch".into(),
        );
    }
    for role in [
        LangItem::DispatchFromDyn,
        LangItem::Unsize,
        LangItem::MetaSized,
    ] {
        let actual = catalog
            .header_role(role)?
            .ok_or("source receiver compatibility is missing its actual role")?;
        let mut steps = 0;
        let current = Trait::current(report, facts, actual.into(), &mut steps);
        header_attributes::charge_source_contract_query(facts, steps)
            .map_err(|_| Error::Invalid("source receiver authentication work exhausted"))?;
        current?;
        source_header(report, facts, actual.definition)?;
    }
    if facts.associated.callable(principal).is_none() {
        source_header(report, facts, principal)?;
        let owner = facts
            .associated
            .owner(principal)
            .and_then(|owner| facts.associated.get(owner))
            .ok_or("dyn source principal lost its actual owner")?;
        Trait::source_owner_with_bounds(report, facts, owner)?;
    }
    Ok(true)
}

fn protocol(facts: &Declarations) -> Result<Option<Protocol>, &'static str> {
    let Some(catalog) = facts.source_bindings.as_deref() else {
        return Ok(None);
    };
    let Some(dispatch) = catalog.header_role(LangItem::DispatchFromDyn)? else {
        return Ok(None);
    };
    let unsize = catalog
        .header_role(LangItem::Unsize)?
        .ok_or("DispatchFromDyn requires actual Unsize")?;
    let meta = catalog
        .header_role(LangItem::MetaSized)?
        .ok_or("DispatchFromDyn requires actual MetaSized")?;
    let parameters = facts
        .generics
        .parameters
        .get(&unsize.definition)
        .ok_or("Unsize has no declared target")?;
    let [parameter] = parameters.as_slice() else {
        return Err("Unsize must retain one actual target formal");
    };
    let Argument::Type(target) = parameter.argument else {
        return Err("Unsize target is not a type");
    };
    let Some(Kind::Parameter(parameter)) = facts.types.kind(target) else {
        return Err("Unsize target is not its genuine rigid declaration");
    };
    if facts.types.parameter(parameter).is_none_or(|value| {
        value.owner.0 != unsize.definition.0
            || value.index != 0
            || value.kind != ParameterKind::Type { sized: false }
    }) {
        return Err("Unsize target changed its actual declaration or PointeeSized contract");
    }
    Ok(Some(Protocol {
        dispatch: ItemId(dispatch.definition.0),
        unsize: ItemId(unsize.definition.0),
        meta: ItemId(meta.definition.0),
        target,
        parameter,
    }))
}

pub(super) fn compatible(
    facts: &mut Declarations,
    method: DefId,
    principal: DefId,
    self_type: TypeId,
) -> Result<Option<bool>, Error> {
    let Some(protocol) = protocol(facts)? else {
        return Ok(None);
    };
    let prepared = prepare(facts, method, principal, self_type, &protocol)?;
    let mut context = Context::default();
    let mut substitution = Instantiation::new(&mut context, &facts.types, &[])
        .map_err(|_| Error::Invalid("invalid receiver proof instantiation"))?;
    let goal = context
        .instantiate_predicate(&mut facts.types, prepared.goal, &mut substitution)
        .map_err(|_| Error::Invalid("invalid receiver dispatch goal"))?;
    let caller = selection::Caller {
        assumptions: &prepared.assumptions,
        environment: Some(&prepared.environment),
        first_region: prepared.first,
        retain_sized: false,
    };
    let result = selection::select(facts, &mut context, &goal, caller);
    header_attributes::charge_source_contract_query(facts, context.steps)
        .map_err(|_| Error::Invalid("source receiver proof work exhausted"))?;
    match result.map_err(|_| {
        Error::Invalid("source receiver dispatch proof failed or remains unfinished")
    })? {
        selection::Selection::Proven(_) => Ok(Some(true)),
        selection::Selection::Absent => Ok(Some(false)),
        selection::Selection::Refuted(refutation) => {
            refutation.reject()?;
            Ok(Some(false))
        }
        selection::Selection::Deferred => {
            Err("source receiver dispatch proof is still pending".into())
        }
    }
}

struct Prepared {
    assumptions: Assumptions,
    environment: Arc<Environment>,
    goal: Predicate,
    first: u32,
}

fn prepare(
    facts: &mut Declarations,
    method: DefId,
    principal: DefId,
    self_type: TypeId,
    protocol: &Protocol,
) -> Result<Prepared, &'static str> {
    let environment = facts
        .generics
        .environments
        .get(&method)
        .cloned()
        .ok_or("source receiver lacks its completed method environment")?;
    let Some(Kind::FunctionItem { signature, .. }) = facts
        .definitions
        .get(method.0 as usize)
        .copied()
        .flatten()
        .and_then(|ty| facts.types.kind(ty))
    else {
        return Err("source receiver lacks its actual method signature");
    };
    let payments = header_attributes::source_payments(facts.must_use_contracts.as_deref())
        .map_err(|_| "source receiver lacks its actual source payer")?;
    let pay = |steps, bytes| payments.charge(steps, bytes);
    let (environment, signature, mut opening) =
        opened_method(&mut facts.types, &environment, signature, protocol, &pay)?;
    let (receiver, target) = translated_receiver(
        &mut facts.types,
        signature,
        self_type,
        protocol.target,
        &pay,
    )?;
    let canonical = *facts
        .generics
        .trait_self_bounds
        .get(&principal)
        .ok_or("principal lost its actual canonical Self application")?;
    let Some((application, crate::support::constness::Constness::Ordinary)) =
        canonical.bound.trait_parts()
    else {
        return Err("principal canonical application changed kind");
    };
    if canonical.ty != self_type
        || facts
            .types
            .trait_application(application)
            .is_none_or(|value| value.item != ItemId(principal.0))
    {
        return Err("principal canonical application changed its actual owner");
    }
    pay(1, 0)?;
    let canonical = if facts
        .types
        .trait_application_has_free_regions(application)
        .ok_or("invalid principal region summary")?
    {
        opening.predicate_charged(&mut facts.types, canonical, &pay)?
    } else {
        canonical
    };
    let target_principal = Predicate {
        ty: protocol.target,
        bound: canonical.bound,
    };
    let seeds = [
        predicate(
            &mut facts.types,
            self_type,
            protocol.unsize,
            &[Argument::Type(protocol.target)],
            &pay,
        )?,
        target_principal,
        predicate(&mut facts.types, protocol.target, protocol.meta, &[], &pay)?,
    ];
    let assumptions = implied(
        &mut facts.types,
        facts
            .generics
            .traits
            .as_mut()
            .ok_or("receiver lacks completed trait contracts")?,
        seeds,
        &pay,
    )?;
    let original = Assumptions::from_environment_charged(&facts.types, Some(&environment), &pay)?;
    let assumptions = original.extended(&facts.types, &assumptions, &pay)?;
    let goal = predicate(
        &mut facts.types,
        receiver,
        protocol.dispatch,
        &[Argument::Type(target)],
        &pay,
    )?;
    Ok(Prepared {
        assumptions,
        environment,
        goal,
        first: opening
            .next_region()
            .ok_or("receiver lost its opened region scope")?,
    })
}

fn predicate(
    store: &mut Store,
    subject: TypeId,
    item: ItemId,
    arguments: &[Argument],
    charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
) -> Result<Predicate, &'static str> {
    let arguments = store.intern_arguments_observed::<true>(arguments, charge)?;
    let application = store.intern_trait_application_observed::<true>(
        crate::types::traits::Application { item, arguments },
        charge,
    )?;
    Ok(Predicate {
        ty: subject,
        bound: Bound::Trait(application),
    })
}

fn source_header(report: &Report, facts: &Declarations, definition: DefId) -> Result<(), Error> {
    let view = facts
        .header_attribute_contracts(report, definition)
        .map_err(|_| Error::Invalid("source receiver changed its completed source header"))?;
    if view.origin.source().is_none()
        || matches!(view.must_use, header_attributes::ContractKnowledge::Unknown)
        || matches!(view.inline, header_attributes::ContractKnowledge::Unknown)
    {
        return Err("source receiver requires its actual completed local header".into());
    }
    Ok(())
}

fn opened_method(
    types: &mut Store,
    environment: &Arc<Environment>,
    signature: TypeId,
    protocol: &Protocol,
    charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
) -> Result<(Arc<Environment>, TypeId, substitute::Bound), &'static str> {
    let mut uses = crate::types::parameters::uses::Uses::default();
    uses.collect_counted(types, signature, &|growth| {
        charge(growth.work, growth.bytes)
    })?;
    if environment.abilities(protocol.parameter).is_some()
        || uses.parameters.contains(&protocol.parameter)
        || uses.constants.contains(&protocol.parameter)
    {
        return Err(
            "receiver rigid target already occurs in the complete method signature or environment",
        );
    }
    let mut opening = substitute::Bound::open_regions_charged(1, charge)?;
    let signature = opening.signature_charged(types, signature, 0, charge)?;
    let environment = environment.opened_regions_charged(types, &mut opening, charge)?;
    let arena = types
        .existing_arena_identity()
        .ok_or("receiver query lacks its authenticated source arena")?;
    let environment =
        environment.with_rigid_query_parameter(types, arena, protocol.parameter, charge)?;
    Ok((environment, signature, opening))
}

fn translated_receiver(
    types: &mut Store,
    signature: TypeId,
    self_type: TypeId,
    target: TypeId,
    charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
) -> Result<(TypeId, TypeId), &'static str> {
    let Some(Kind::Function { parameters, .. }) = types.kind(signature) else {
        return Err("opened receiver signature changed kind");
    };
    let receiver = *types
        .list(parameters)
        .and_then(|types| types.first())
        .ok_or("source receiver signature has no receiver")?;
    let Some(Kind::Parameter(self_parameter)) = types.kind(self_type) else {
        return Err("source receiver Self lost its declared parameter");
    };
    let mut replace = substitute::Bound::parameters_captured_charged(
        types,
        &[(self_parameter, Argument::Type(target))],
        Vec::new(),
        charge,
    )?;
    let Argument::Type(target) =
        replace.apply_argument_charged(types, Argument::Type(receiver), charge)?
    else {
        return Err("receiver substitution changed kind");
    };
    Ok((receiver, target))
}

fn implied(
    types: &mut Store,
    contracts: &mut crate::sema::traits::Contracts,
    seeds: [Predicate; 3],
    pay: &impl Fn(usize, usize) -> Result<(), &'static str>,
) -> Result<Vec<Predicate>, &'static str> {
    let mut assumptions = Vec::new();
    for (index, seed) in seeds.into_iter().enumerate() {
        let growth = Growth::vector::<Predicate>(assumptions.len(), assumptions.capacity(), 1);
        pay(growth.work + 1, growth.bytes)?;
        assumptions.push(seed);
        // The real method environment already retains all inherited clauses.
        // U:Principal is the exact hypothetical application, not permission to
        // copy its entire supertrait graph for each receiver. Pointer dispatch
        // needs only the actual Unsize and metadata-sizing input contracts.
        if index == 1 {
            continue;
        }
        let requirements = contracts.requirements_counted(types, seed, pay)?;
        for &clause in &requirements.implied {
            if let crate::types::parameters::clauses::Clause::Predicate(value) = clause
                && value.bound.trait_parts().is_some()
            {
                let growth =
                    Growth::vector::<Predicate>(assumptions.len(), assumptions.capacity(), 1);
                pay(growth.work + 1, growth.bytes)?;
                assumptions.push(value);
            }
        }
    }
    Ok(assumptions)
}
