//! Associated result rules share capture-safe trait argument substitution.
mod headers;
mod opaque_bounds;
pub(in crate::sema) use opaque_bounds::prepare_outlives as prepare_opaque_outlives;
mod templates;
use super::assumptions::Assumptions;
use crate::support::hash::HashMap;
use crate::{
    resolve::model::DefId,
    sema::model::Declarations,
    types::{
        ItemId, Kind, TypeId,
        infer::{Context, TermId},
        parameters::{
            clauses::Clause,
            predicates::{Bound, Predicate},
        },
    },
};
use std::sync::Arc;

/// Pure mapped clauses, never caller permission or implementation proof.
#[derive(Default)]
pub(super) struct Cache {
    enabled: bool,
    entries: HashMap<(TypeId, ItemId), Arc<Assumptions>>,
    equalities: HashMap<(TypeId, ItemId), Arc<[Clause]>>,
    templates: HashMap<crate::types::outlives::ResultId, Arc<templates::Index>>,
    steps: usize,
}
impl Cache {
    pub(super) fn enable(&mut self) {
        self.enabled = true;
    }
    pub(super) fn invalidate(&mut self) {
        *self = Self::default();
    }
    fn get(&self, ty: TypeId, item: ItemId) -> Option<Arc<Assumptions>> {
        self.entries.get(&(ty, item)).cloned()
    }
    fn account(&mut self, cost: usize) -> Result<(), &'static str> {
        self.steps = self.steps.saturating_add(cost);
        if self
            .steps
            .saturating_add(self.entries.len())
            .saturating_add(self.equalities.len())
            .saturating_add(self.templates.len())
            > 1_000_000
        {
            return Err("associated result rule cache work budget exceeded");
        }
        Ok(())
    }
    fn insert(
        &mut self,
        ty: TypeId,
        item: ItemId,
        rules: Arc<Assumptions>,
        cost: usize,
    ) -> Result<(), &'static str> {
        if !self.enabled {
            return Ok(());
        }
        self.account(cost)?;
        self.entries.insert((ty, item), rules);
        Ok(())
    }
}

/// Declared projection, opaque and object promises share the scoped selector.
/// Projected subjects require their retained parent proof; object membership is
/// intrinsic to the complete object bounds. This index is not a caller environment.
pub(in crate::sema) fn traits(
    facts: &mut Declarations,
    ty: TypeId,
    item: ItemId,
) -> Result<Option<(Arc<Assumptions>, usize)>, String> {
    if let Some(rules) = facts
        .generics
        .traits
        .as_ref()
        .and_then(|contracts| contracts.result_rules.get(ty, item))
    {
        return Ok(Some((rules, 1)));
    }
    if let Some(Kind::Opaque(id, arguments)) = facts.types.kind(ty) {
        return opaque_bounds::traits(facts, ty, id, arguments, item).map(Some);
    }
    if matches!(facts.types.kind(ty), Some(Kind::Dynamic { .. })) {
        let (index, steps) = super::object_bounds::get(facts, ty)?;
        return Ok(Some((Arc::clone(&index.rules), steps)));
    }
    let Some(Kind::Projection(application)) = facts.types.kind(ty) else {
        return Ok(None);
    };
    let (templates, template_steps) = templates::for_result(facts, DefId(application.item.0))?;
    let parent = Predicate {
        ty: application.subject,
        bound: Bound::Trait(application.trait_application),
    };
    let mut result = Assumptions::default();
    let mut steps = template_steps;
    if let Some(predicates) = templates.candidates(item) {
        let mut fold = facts.trait_substitution(
            parent,
            Some((DefId(application.item.0), application.arguments)),
        )?;
        for &predicate in predicates {
            let mapped = fold.clause(&mut facts.types, predicate)?;
            result
                .insert_clause(&facts.types, mapped)
                .map_err(|e| e.to_string())?;
        }
        steps = steps
            .saturating_add(fold.steps)
            .saturating_add(predicates.len());
    }
    let result = Arc::new(result);
    facts
        .generics
        .traits
        .as_mut()
        .ok_or("missing result rule contracts")?
        .result_rules
        .insert(
            ty,
            item,
            Arc::clone(&result),
            steps.saturating_sub(template_steps),
        )?;
    Ok(Some((result, steps)))
}

/// Immutable result promises grant markers only to the declared result itself.
/// The requesting selector separately retains the parent application proof.
pub(in crate::sema) fn auto(
    facts: &mut Declarations,
    obligation: crate::types::auto_traits::Obligation,
) -> Result<(bool, usize), String> {
    let id = match facts.types.kind(obligation.ty) {
        Some(Kind::Projection(application)) => {
            crate::types::outlives::ResultId::Associated(application.item)
        }
        Some(Kind::Opaque(id, _)) => crate::types::outlives::ResultId::Opaque(id),
        Some(Kind::Dynamic { shape, .. }) => {
            return Ok((facts.types.dynamic_has_auto(shape, obligation.marker), 1));
        }
        _ => return Ok((false, 1)),
    };
    let (index, steps) = templates::for_declaration(facts, id)?;
    Ok((index.autos & (1 << obligation.marker as u8) != 0, steps))
}

pub(super) fn may_have_opaque_trait(
    facts: &Declarations,
    id: crate::types::opaque::OpaqueId,
    item: ItemId,
) -> bool {
    facts
        .generics
        .traits
        .as_ref()
        .and_then(|contracts| {
            contracts
                .result_rules
                .templates
                .get(&crate::types::outlives::ResultId::Opaque(id))
        })
        .is_none_or(|index| index.candidates(item).is_some())
}

/// These equations describe a declared result, never evidence for forming its
/// parent application. Callers must retain that proof before selecting a value.
#[derive(Default)]
pub(super) struct Equations {
    pub clauses: Option<Arc<[Clause]>>,
    pub steps: usize,
}

/// Inspect declaration metadata before mapping any growing caller input graph.
pub(super) fn has_equalities(
    facts: &mut Declarations,
    parent: ItemId,
    item: ItemId,
) -> Result<(bool, usize), String> {
    let (templates, steps) = templates::for_result(facts, DefId(parent.0))?;
    Ok((templates.equalities(item).is_some(), steps))
}

pub(super) fn equalities(
    facts: &mut Declarations,
    context: &mut Context,
    subject: TermId,
    item: ItemId,
) -> Result<Equations, String> {
    if facts.types.has_dynamic_shapes()
        && matches!(
            context
                .head(&facts.types, subject)
                .map_err(|e| e.to_string())?,
            Some(crate::types::infer::Constructor::Dynamic { .. })
        )
    {
        let ty = context
            .resolved(&mut facts.types, subject)
            .map_err(|e| e.to_string())?;
        let (index, steps) = super::object_bounds::get(facts, ty)?;
        return Ok(Equations {
            clauses: index.equalities.get(&item).cloned(),
            steps,
        });
    }
    let Some(parent) = context
        .result_identity(&facts.types, subject)
        .map_err(|e| e.to_string())?
    else {
        return Ok(Equations::default());
    };
    let (templates, template_steps) = templates::for_declaration(facts, parent)?;
    let Some(predicates) = templates.equalities(item) else {
        // Select declaration rules before resolving a possibly deep input graph.
        // The immutable index shares this miss without a mapped-rule allocation.
        return Ok(Equations {
            clauses: None,
            steps: template_steps,
        });
    };
    let ty = context
        .resolved(&mut facts.types, subject)
        .map_err(|e| e.to_string())?;
    if let Some(rules) = facts
        .generics
        .traits
        .as_ref()
        .and_then(|contracts| contracts.result_rules.equalities.get(&(ty, item)))
    {
        return Ok(Equations {
            clauses: Some(Arc::clone(rules)),
            steps: template_steps.saturating_add(1),
        });
    }
    let mut result = Vec::with_capacity(predicates.len());
    let mut fold = match facts.types.kind(ty) {
        Some(Kind::Projection(application)) => facts.trait_substitution(
            Predicate {
                ty: application.subject,
                bound: Bound::Trait(application.trait_application),
            },
            Some((DefId(application.item.0), application.arguments)),
        )?,
        Some(Kind::Opaque(id, arguments)) => facts
            .opaques
            .declarations
            .get(&id)
            .ok_or("missing opaque equality contract")?
            .substitution(&facts.types, arguments)?,
        _ => return Err("result equality subject lost its declaration identity".into()),
    };
    for &predicate in predicates {
        result.push(fold.clause(&mut facts.types, predicate)?);
    }
    let steps = template_steps
        .saturating_add(fold.steps)
        .saturating_add(predicates.len());
    let result: Arc<[Clause]> = result.into();
    let cache = &mut facts
        .generics
        .traits
        .as_mut()
        .ok_or("missing result equality contracts")?
        .result_rules;
    if cache.enabled {
        cache.account(steps.saturating_sub(template_steps))?;
        cache.equalities.insert((ty, item), Arc::clone(&result));
    }
    Ok(Equations {
        clauses: Some(result),
        steps,
    })
}

pub(in crate::sema) fn substitution(
    facts: &Declarations,
    predicate: Predicate,
) -> Result<crate::types::substitute::Bound, &'static str> {
    facts.trait_substitution(predicate, None)
}

/// Complete finite result lifetime templates before scoped lifetime validation.
/// Recursive associated results remain demand-driven; only ordinary Self
/// supertraits contribute to the member's own promises.
pub(in crate::sema) fn prepare_outlives(facts: &mut Declarations) -> Result<(), String> {
    if facts.associated.types.is_empty() {
        return Ok(());
    }
    let mut direct = false;
    for contract in facts.associated.types.values() {
        for &clause in &contract.bounds {
            direct |= clause.is_outlives(&facts.types)?;
        }
    }
    let mut inherited = false;
    if let Some(contracts) = &facts.generics.traits {
        for (_, header) in contracts.headers() {
            for predicate in &header.predicates {
                if header.implies_self(&facts.types, predicate)? {
                    inherited |= predicate.value.is_outlives(&facts.types)?;
                }
            }
        }
    }
    if !direct && !inherited {
        return Ok(());
    }
    for index in 0..facts.associated.members().len() {
        let item = facts.associated.members()[index].definition;
        if !facts.associated.types.contains_key(&item) {
            continue;
        }
        let (templates, _) = templates::for_result(facts, item)?;
        let captures = facts
            .associated
            .trait_owner(item)
            .and_then(|owner| facts.associated.schema(owner))
            .ok_or("missing associated lifetime owner schema")?
            .captures;
        facts
            .types
            .declare_projection_outlives(ItemId(item.0), captures, &templates.outlives)?;
    }
    Ok(())
}
