//! Prefix-only candidate probes never consume result or call-argument constraints.
use crate::{
    resolve::{
        expansion::Report,
        model::{Access, DefId, ScopeId},
        names::NameId,
        package::Error,
        unit::NodeKey,
    },
    sema::{
        annotations::{Fields, Host, Scratch},
        model::Declarations,
        traits::{
            assumptions::Assumptions,
            selection::{self, Selection},
        },
    },
    types::{
        ItemId, TypeId,
        infer::{self, Context, Instantiation, TermId, arguments::Argument, properties::Query},
        parameters::{ParameterKind, environment::Environment, predicates::Bound},
    },
};
use std::{collections::HashMap, sync::Arc};

#[derive(Clone, Copy)]
pub(in crate::sema) struct Caller<'a> {
    pub environment: Option<&'a Arc<Environment>>,
    pub traits: Option<&'a Assumptions>,
}

pub(super) struct Receiver {
    pub ty: TypeId,
    /// Bound-priority lookup must use the parameter that supplies that bound.
    pub subject: Option<TermId>,
}

struct Prefix<'a> {
    report: &'a Report,
    facts: &'a mut Declarations,
    infer: Context,
    terms: HashMap<NodeKey, TermId>,
}

pub(in crate::sema) fn choose_trait_path(
    report: &Report,
    facts: &mut Declarations,
    key: NodeKey,
    prefix: NodeKey,
    site: (ScopeId, NameId),
    caller: Caller<'_>,
) -> Result<DefId, Error> {
    let mut source = Prefix {
        report,
        facts,
        infer: Context::default(),
        terms: HashMap::new(),
    };
    let term = Scratch::default().infer(&mut source, prefix)?;
    let error = |message: &str| Declarations::error(report, key, message);
    let groups =
        trait_groups(report, source.facts, &mut source.infer, term, site, caller).map_err(error)?;
    for (group, candidates) in groups.into_iter().enumerate() {
        let mut selected = None;
        for method in candidates {
            let owner = source
                .facts
                .associated
                .trait_owner(method)
                .ok_or_else(|| error("missing trait candidate owner"))?;
            let mut probe = Context::default();
            let prefix = probe
                .import(&source.facts.types, &mut source.infer, term)
                .map_err(|e| error(&e.to_string()))?;
            let possible =
                applicable(source.facts, &mut probe, prefix, owner, None, caller).map_err(error)?;
            account_probe(source.facts, probe.steps).map_err(error)?;
            if possible && selected.replace(method).is_some() {
                return Err(error("multiple applicable trait methods"));
            }
        }
        if let Some(method) = selected {
            if group == 0 {
                source.facts.methods.bound_uses.insert(key);
            }
            return Ok(method);
        }
    }
    Err(error("no applicable trait method for this prefix"))
}

pub(super) fn trait_groups(
    report: &Report,
    facts: &mut Declarations,
    source: &mut Context,
    prefix: TermId,
    site: (ScopeId, NameId),
    caller: Caller<'_>,
) -> Result<[Vec<DefId>; 2], &'static str> {
    if facts.methods.visible.is_none() {
        facts.methods.visible = Some(Box::new(super::visible::Index::new(report, facts)?));
    }
    let parameter = match source.head(&facts.types, prefix).map_err(inference)? {
        Some(infer::Constructor::Parameter(parameter)) => Some(parameter),
        _ => None,
    };
    let bound_traits = parameter
        .zip(caller.traits)
        .map_or(&[][..], |(parameter, traits)| {
            traits.parameter_traits(parameter)
        });
    facts
        .methods
        .visible
        .as_mut()
        .ok_or("missing trait visibility index")?
        .groups(report, site.0, site.1, bound_traits)
}

pub(super) fn account_probe(facts: &mut Declarations, steps: usize) -> Result<(), &'static str> {
    facts.methods.selection_probes += 1;
    facts.methods.selection_steps = facts.methods.selection_steps.saturating_add(steps);
    if facts.methods.selection_steps > 1_000_000 {
        return Err("method selection work budget exceeded");
    }
    Ok(())
}
impl Host for Prefix<'_> {
    fn report(&self) -> &Report {
        self.report
    }
    fn facts(&self) -> &Declarations {
        self.facts
    }
    fn inference(&mut self) -> (&mut Declarations, &mut Context) {
        (self.facts, &mut self.infer)
    }
    fn cached(&self, key: NodeKey) -> Option<TermId> {
        self.terms.get(&key).copied()
    }
    fn save(&mut self, key: NodeKey, term: TermId) {
        self.terms.insert(key, term);
    }
    fn nominal(&mut self, _: TermId, _: Fields) {}
    fn callable(&mut self, _: NodeKey) {}
    fn child(&mut self, _: NodeKey) {}
}

pub(in crate::sema) fn choose(
    report: &Report,
    facts: &mut Declarations,
    key: NodeKey,
    prefix: NodeKey,
    bucket: (ItemId, NameId),
    scope: ScopeId,
    caller: Caller<'_>,
) -> Result<Option<DefId>, Error> {
    let mut source = Prefix {
        report,
        facts,
        infer: Context::default(),
        terms: HashMap::new(),
    };
    let term = Scratch::default().infer(&mut source, prefix)?;
    let error = |message: &str| Declarations::error(report, key, message);
    let candidates = candidates(source.facts, &mut source.infer, term, bucket).map_err(error)?;
    source.facts.methods.selection_steps = source
        .facts
        .methods
        .selection_steps
        .saturating_add(source.infer.steps);
    let module = report.resolution.scopes[scope.0 as usize].module;
    let mut selected = None;
    for method in candidates {
        if let Access::Within(ancestor) = report.resolution.definitions[method.0 as usize].access
            && !report.package.is_descendant(module, ancestor)
        {
            continue;
        }
        let owner = source
            .facts
            .associated
            .owner(method)
            .and_then(|id| source.facts.associated.get(id))
            .ok_or_else(|| error("missing method candidate owner"))?
            .self_definition;
        let mut probe = Context::default();
        let prefix = probe
            .import(&source.facts.types, &mut source.infer, term)
            .map_err(|e| error(&e.to_string()))?;
        let applicable =
            applicable(source.facts, &mut probe, prefix, owner, None, caller).map_err(error)?;
        source.facts.methods.selection_probes += 1;
        source.facts.methods.selection_steps = source
            .facts
            .methods
            .selection_steps
            .saturating_add(probe.steps);
        if source.facts.methods.selection_steps > 1_000_000 {
            return Err(error("method selection work budget exceeded"));
        }
        if applicable && selected.replace(method).is_some() {
            return Err(error("multiple applicable inherent methods"));
        }
    }
    Ok(selected)
}

pub(super) fn candidates(
    facts: &mut Declarations,
    context: &mut Context,
    prefix: TermId,
    key: (ItemId, NameId),
) -> Result<Vec<DefId>, &'static str> {
    let bucket = facts
        .methods
        .inherent
        .get(&key)
        .ok_or("missing method bucket")?;
    if bucket.len() == 1 {
        return Ok(bucket.clone());
    }
    match context.resolved(&mut facts.types, prefix) {
        Ok(ty) if facts.types.contains_parameters(ty) == Some(false) => {
            let shapes = facts
                .methods
                .shapes
                .as_mut()
                .ok_or("missing method shape cache")?;
            let before = shapes.steps;
            let ty = shapes.apply(&mut facts.types, ty)?;
            let steps = shapes.steps.saturating_sub(before);
            facts.methods.selection_shape_steps =
                facts.methods.selection_shape_steps.saturating_add(steps);
            context.account_work(steps).map_err(inference)?;
            let index = facts
                .methods
                .indices
                .get(&key)
                .ok_or("missing method candidate index")?;
            let mut candidates = index.open.clone();
            candidates.extend(index.closed.get(&ty).into_iter().flatten().copied());
            candidates.sort_unstable_by_key(|id| id.0);
            Ok(candidates)
        }
        Ok(_) | Err(infer::Error::Unresolved) => facts
            .methods
            .inherent
            .get(&key)
            .cloned()
            .ok_or("missing method bucket"),
        Err(_) => Err("invalid method prefix inference graph"),
    }
}

pub(super) fn applicable(
    facts: &mut Declarations,
    context: &mut Context,
    prefix: TermId,
    owner: DefId,
    receiver: Option<Receiver>,
    caller: Caller<'_>,
) -> Result<bool, &'static str> {
    let target = facts
        .associated
        .schema(owner)
        .ok_or("missing candidate schema")?
        .self_type;
    let subject = target;
    let target = receiver.as_ref().map_or(target, |r| r.ty);
    let ids = if facts.associated.trait_owner(owner).is_some() {
        facts.generics.callable_parameters(&facts.types, owner)?.1
    } else {
        facts.generics.parameter_ids(&facts.types, owner)?
    };
    let mut bindings = Vec::with_capacity(ids.len());
    let mut obligations = Vec::new();
    let mut traits = Vec::new();
    for id in ids {
        let argument = context
            .parameter_variable(&facts.types, id)
            .map_err(inference)?;
        if let Argument::Type(term) = argument
            && matches!(
                facts.types.parameter(id).map(|p| p.kind),
                Some(ParameterKind::Type { sized: true })
            )
        {
            obligations.push((term, Bound::Sized));
        }
        bindings.push((id, argument));
    }
    let mut substitution =
        Instantiation::new(context, &facts.types, &bindings).map_err(inference)?;
    let target = context
        .instantiate_with(&mut facts.types, target, &mut substitution)
        .map_err(inference)?;
    if let Some(actual) = receiver.and_then(|r| r.subject) {
        let subject = context
            .instantiate_with(&mut facts.types, subject, &mut substitution)
            .map_err(inference)?;
        match context.equate_shape(&facts.types, subject, actual) {
            Ok(()) => {}
            Err(infer::Error::Mismatch | infer::Error::Occurs) => return Ok(false),
            Err(error) => return Err(inference(error)),
        }
    }
    match context.equate_shape(&facts.types, target, prefix) {
        Ok(()) => {}
        Err(infer::Error::Mismatch | infer::Error::Occurs) => return Ok(false),
        Err(e) => return Err(inference(e)),
    }
    for predicate in facts.generics.predicates_in_scope(owner) {
        if matches!(predicate.value.bound, Bound::Trait(_)) {
            traits.push(
                context
                    .instantiate_predicate(&mut facts.types, predicate.value, &mut substitution)
                    .map_err(inference)?,
            );
            continue;
        }
        if matches!(predicate.value.bound, Bound::Outlives(_)) {
            continue;
        }
        let term = context
            .instantiate_with(&mut facts.types, predicate.value.ty, &mut substitution)
            .map_err(inference)?;
        obligations.push((term, predicate.value.bound));
    }
    if !trait_requirements(facts, context, &traits, caller)? {
        return Ok(false);
    }
    if obligations.is_empty() {
        return Ok(true);
    }
    let environment = if let Some(environment) = caller.environment {
        Arc::clone(environment)
    } else {
        Arc::new(Environment::new(&facts.types, &[])?)
    };
    let mut properties = Query::new(environment);
    for (term, bound) in obligations {
        let value = properties.demand(
            context,
            &mut facts.types,
            &facts.aggregates,
            &facts.derives,
            term,
        )?;
        let possible = match bound {
            Bound::Sized => value.sized,
            Bound::Copy => value.abilities.copy,
            Bound::Clone => value.abilities.clone_,
            Bound::Outlives(_) => return Err("unexpected candidate region predicate"),
            Bound::Trait(_) => {
                return Err("user trait candidate obligations require trait selection");
            }
        };
        if !possible {
            return Ok(false);
        }
    }
    Ok(true)
}
fn trait_requirements(
    facts: &mut Declarations,
    context: &mut Context,
    goals: &[infer::predicates::Predicate],
    caller: Caller<'_>,
) -> Result<bool, &'static str> {
    if goals.is_empty() {
        return Ok(true);
    }
    let empty = Assumptions::default();
    let assumptions = caller.traits.unwrap_or(&empty);
    let mut first_region = u32::try_from(facts.body_regions.len())
        .ok()
        .and_then(|n| n.checked_add(1))
        .ok_or("method trait region identity space exhausted")?;
    let mut pending: Vec<_> = goals.iter().collect();
    while !pending.is_empty() {
        let before = pending.len();
        let mut next = Vec::new();
        for goal in pending {
            match selection::select(
                facts,
                context,
                goal,
                selection::Caller {
                    assumptions,
                    environment: caller.environment,
                    first_region,
                },
            )
            .map_err(|_| "invalid or exhausted method trait selection")?
            {
                Selection::Absent => return Ok(false),
                Selection::Deferred => next.push(goal),
                Selection::Proven(proof) => first_region = proof.regions.end,
            }
        }
        if next.len() == before {
            // A candidate is possible; the selected ordinary callable must prove
            // its bounds after all arguments and result constraints are known.
            return Ok(true);
        }
        pending = next;
    }
    Ok(true)
}
fn inference(error: infer::Error) -> &'static str {
    match error {
        infer::Error::Budget => "method candidate work budget exceeded",
        infer::Error::Overflow => "method candidate arena exceeds 32 bits",
        _ => "invalid method candidate inference graph",
    }
}
