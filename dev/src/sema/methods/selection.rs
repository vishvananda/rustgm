//! Prefix-only candidate probes never consume result or call-argument constraints.
mod applicability;
mod paths;
use crate::{
    resolve::{
        expansion::Report,
        model::{DefId, ScopeId},
        names::NameId,
        package::Error,
        unit::NodeKey,
    },
    sema::{
        annotations::{Fields, Host},
        model::Declarations,
        traits::assumptions::Assumptions,
    },
    types::{
        ItemId, TypeId,
        infer::{self, Context, TermId},
        parameters::environment::Environment,
    },
};
pub(super) use applicability::applicable;
pub(super) use applicability::inference;
pub(in crate::sema) use paths::{
    OwnedCaller as PathCaller, Progress as PathProgress, Query as PathQuery,
};
use std::{collections::HashMap, sync::Arc};

#[derive(Clone, Copy)]
pub(in crate::sema) struct Caller<'a> {
    pub availability: crate::sema::traits::implementations::Availability,
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

pub(super) fn trait_groups(
    report: &Report,
    facts: &mut Declarations,
    source: &mut Context,
    prefix: TermId,
    site: (ScopeId, NameId),
    caller: Caller<'_>,
) -> Result<[Vec<DefId>; 2], &'static str> {
    if facts.methods.visible.is_none() {
        let index = super::visible::Index::new(report, facts)?;
        facts.methods.trait_visibility_steps = index.steps();
        facts.methods.visible = Some(Box::new(index));
    }
    let head = source.head(&facts.types, prefix).map_err(inference)?;
    let object = if matches!(head, Some(infer::Constructor::Dynamic { .. })) {
        let ty = source
            .resolved(&mut facts.types, prefix)
            .map_err(inference)?;
        let (index, steps) = crate::sema::traits::object_bounds::get(facts, ty)?;
        source.account_work(steps).map_err(inference)?;
        Some(index)
    } else {
        None
    };
    let parameter = match head {
        Some(infer::Constructor::Parameter(parameter)) => Some(parameter),
        _ => None,
    };
    let bound_traits = object.as_ref().map_or_else(
        || {
            parameter
                .zip(caller.traits)
                .map_or(&[][..], |(parameter, traits)| {
                    traits.parameter_traits(parameter)
                })
        },
        |object| object.traits.as_slice(),
    );
    let clone_bound = parameter
        .zip(caller.environment)
        .filter(|(parameter, environment)| {
            environment
                .abilities(*parameter)
                .is_some_and(|abilities| abilities.clone_)
        })
        .and(facts.cloning.as_deref())
        .map(|r| ItemId(r.canonical.definition.0));
    let with_clone;
    let bound_traits = if let Some(clone) = clone_bound {
        with_clone = bound_traits
            .iter()
            .copied()
            .chain(std::iter::once(clone))
            .collect::<Vec<_>>();
        with_clone.as_slice()
    } else {
        bound_traits
    };
    let index = facts
        .methods
        .visible
        .as_mut()
        .ok_or("missing trait visibility index")?;
    let before = index.steps();
    let groups = index.groups(report, site.0, site.1, bound_traits)?;
    facts.methods.trait_visibility_steps = facts
        .methods
        .trait_visibility_steps
        .saturating_add(index.steps().saturating_sub(before));
    if let Some(canonical) = report.resolution.builtin_traits.iteration()
        && groups
            .iter()
            .flatten()
            .any(|member| canonical.declarations.iter().any(|d| d.method == *member))
        && !facts
            .iteration
            .as_deref()
            .and_then(|r| r.ranges.as_deref())
            .is_some_and(|r| r.complete)
        && let Some(owner) = crate::sema::iteration::range_originals::requested_owner(
            report,
            &facts.types,
            source,
            prefix,
        )
        .map_err(inference)?
    {
        crate::sema::declarations::iteration::range_contracts::prepare(
            report,
            facts,
            owner,
            matches!(
                caller.availability,
                crate::sema::traits::implementations::Availability::Validated
            ),
        )
        .map_err(|_| "canonical range iteration preparation failed")?;
    }
    Ok(groups)
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
    // Prefix probes discover candidates; the actual body annotation owns its proofs.
    fn requirement(&mut self, _: NodeKey, _: crate::types::infer::predicates::Predicate) {}
    fn callable(&mut self, _: NodeKey) {}
    fn child(&mut self, _: NodeKey) {}
    fn pending_normalization(&self) -> bool {
        false
    }
    fn scope(
        &mut self,
        _: NodeKey,
        parent: crate::types::lifetime_proof::ScopeId,
        _: u32,
    ) -> Result<crate::types::lifetime_proof::ScopeId, Error> {
        Ok(parent)
    }
    fn deferred(
        &mut self,
        key: NodeKey,
        _: crate::types::lifetime_proof::ScopeId,
        _: crate::sema::alias_requirements::Expansion,
        _: &[crate::sema::generics::AnnotationArgument],
        _: &Fields,
    ) -> Result<(), Error> {
        Err(Declarations::error(
            self.report,
            key,
            "inferred alias proofs require delayed projection normalization",
        ))
    }
    fn normalization(
        &mut self,
        key: NodeKey,
        _: crate::types::lifetime_proof::ScopeId,
        _: TermId,
    ) -> Result<(), Error> {
        Err(Declarations::error(
            self.report,
            key,
            "method prefix requires completed projection normalization",
        ))
    }
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
            prepare_candidates(facts, key)?;
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

/// Header-time lookup may precede the whole-crate overlap pass. Its complete
/// bucket can build the same shape index without authorizing unchecked owners.
fn prepare_candidates(facts: &mut Declarations, key: (ItemId, NameId)) -> Result<(), &'static str> {
    if facts.methods.indices.contains_key(&key) {
        return Ok(());
    }
    let shapes = facts
        .methods
        .shapes
        .get_or_insert_with(|| Box::new(crate::types::substitute::Bound::shape()));
    let mut index = super::Candidates::default();
    for &member in facts
        .methods
        .inherent
        .get(&key)
        .ok_or("missing method bucket")?
    {
        let ty = facts
            .associated
            .schema(member)
            .ok_or("missing method owner schema")?
            .self_type;
        if facts.types.contains_parameters(ty) == Some(false) {
            let before = shapes.steps;
            let shape = shapes.apply(&mut facts.types, ty)?;
            facts.methods.selection_shape_steps = facts
                .methods
                .selection_shape_steps
                .saturating_add(shapes.steps.saturating_sub(before));
            index.closed.entry(shape).or_default().push(member);
        } else {
            index.open.push(member);
        }
    }
    facts.methods.indices.insert(key, index);
    Ok(())
}
