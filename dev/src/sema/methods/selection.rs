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
    },
    types::{
        ItemId, TypeId,
        infer::{self, Context, Instantiation, TermId, arguments::Argument, properties::Query},
        parameters::{ParameterKind, environment::Environment, predicates::Bound},
    },
};
use std::{collections::HashMap, sync::Arc};

struct Prefix<'a> {
    report: &'a Report,
    facts: &'a mut Declarations,
    infer: Context,
    terms: HashMap<NodeKey, TermId>,
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
    environment: Option<&Arc<Environment>>,
) -> Result<DefId, Error> {
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
        let applicable = applicable(source.facts, &mut probe, prefix, owner, None, environment)
            .map_err(error)?;
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
    selected.ok_or_else(|| error("no applicable inherent method for this prefix"))
}

pub(super) fn candidates(
    facts: &mut Declarations,
    context: &mut Context,
    prefix: TermId,
    key: (ItemId, NameId),
) -> Result<Vec<DefId>, &'static str> {
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
    receiver: Option<TypeId>,
    environment: Option<&Arc<Environment>>,
) -> Result<bool, &'static str> {
    let target = facts
        .associated
        .schema(owner)
        .ok_or("missing candidate schema")?
        .self_type;
    let target = receiver.unwrap_or(target);
    let ids = facts.generics.parameter_ids(&facts.types, owner)?;
    let mut bindings = Vec::with_capacity(ids.len());
    let mut obligations = Vec::new();
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
    match context.equate_shape(&facts.types, target, prefix) {
        Ok(()) => {}
        Err(infer::Error::Mismatch | infer::Error::Occurs) => return Ok(false),
        Err(e) => return Err(inference(e)),
    }
    for predicate in facts.generics.predicates_in_scope(owner) {
        if matches!(predicate.value.bound, Bound::Trait(_)) {
            return Err("user trait candidate obligations require trait selection");
        }
        if matches!(predicate.value.bound, Bound::Outlives(_)) {
            continue;
        }
        let term = context
            .instantiate_with(&mut facts.types, predicate.value.ty, &mut substitution)
            .map_err(inference)?;
        obligations.push((term, predicate.value.bound));
    }
    if obligations.is_empty() {
        return Ok(true);
    }
    let environment = if let Some(environment) = environment {
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
fn inference(error: infer::Error) -> &'static str {
    match error {
        infer::Error::Budget => "method candidate work budget exceeded",
        infer::Error::Overflow => "method candidate arena exceeds 32 bits",
        _ => "invalid method candidate inference graph",
    }
}
