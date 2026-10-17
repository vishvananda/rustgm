//! Associated result rules share capture-safe trait argument substitution.
mod headers;
mod templates;
use super::assumptions::Assumptions;
use crate::{
    resolve::model::DefId,
    sema::model::Declarations,
    types::{
        ItemId, Kind, TypeId,
        infer::{Context, TermId},
        parameters::predicates::{Bound, Predicate},
    },
};
use std::{collections::HashMap, sync::Arc};

/// Pure mapped clauses, never caller permission or implementation proof.
#[derive(Default)]
pub(super) struct Cache {
    enabled: bool,
    entries: HashMap<(TypeId, ItemId), Arc<Assumptions>>,
    equalities: HashMap<(TypeId, ItemId), Arc<[Predicate]>>,
    templates: HashMap<DefId, Arc<templates::Index>>,
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

/// Only a proved parent projection can authorize these templates. The selector
/// retains that proof before using this query-local index; it is not an environment.
pub(super) fn traits(
    facts: &mut Declarations,
    ty: TypeId,
    item: ItemId,
) -> Result<Option<(Arc<Assumptions>, usize)>, String> {
    let Some(Kind::Projection(application)) = facts.types.kind(ty) else {
        return Ok(None);
    };
    if let Some(rules) = facts
        .generics
        .traits
        .as_ref()
        .and_then(|contracts| contracts.result_rules.get(ty, item))
    {
        return Ok(Some((rules, 1)));
    }
    let (templates, template_steps) = templates::for_result(facts, DefId(application.item.0))?;
    let parent = Predicate {
        ty: application.subject,
        bound: Bound::Trait(application.trait_application),
    };
    let mut result = Assumptions::default();
    let mut steps = template_steps;
    if let Some(predicates) = templates.candidates(item) {
        let mut fold = substitution(facts, parent)?;
        for &predicate in predicates {
            let mapped = fold.predicate(&mut facts.types, predicate)?;
            result
                .insert(&facts.types, mapped)
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

/// These equations describe a declared result, never evidence for forming its
/// parent application. Callers must retain that proof before selecting a value.
#[derive(Default)]
pub(super) struct Equations {
    pub clauses: Option<Arc<[Predicate]>>,
    pub steps: usize,
}

pub(super) fn equalities(
    facts: &mut Declarations,
    context: &mut Context,
    subject: TermId,
    item: ItemId,
) -> Result<Equations, String> {
    let Some(parent) = context
        .projection_item(&facts.types, subject)
        .map_err(|e| e.to_string())?
    else {
        return Ok(Equations::default());
    };
    let (templates, template_steps) = templates::for_result(facts, DefId(parent.0))?;
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
    let Some(Kind::Projection(application)) = facts.types.kind(ty) else {
        return Err("result equality subject lost its projection identity".into());
    };
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
    let mut fold = substitution(
        facts,
        Predicate {
            ty: application.subject,
            bound: Bound::Trait(application.trait_application),
        },
    )?;
    for &predicate in predicates {
        result.push(fold.predicate(&mut facts.types, predicate)?);
    }
    let steps = template_steps
        .saturating_add(fold.steps)
        .saturating_add(predicates.len());
    let result: Arc<[Predicate]> = result.into();
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
    use crate::{sema::generics::parameter_id, types::arguments::Argument};
    let Bound::Trait(id) = predicate.bound else {
        return Err("missing associated result trait");
    };
    let application = facts
        .types
        .trait_application(id)
        .ok_or("missing associated result application")?;
    let owner = DefId(application.item.0);
    let schema = facts
        .associated
        .schema(owner)
        .ok_or("missing associated result schema")?;
    let parameters = facts
        .generics
        .parameters
        .get(&owner)
        .map_or(&[][..], Vec::as_slice);
    let arguments = facts
        .types
        .arguments(application.arguments)
        .ok_or("missing associated result arguments")?;
    if parameters.len() != arguments.len() {
        return Err("associated result argument arity mismatch");
    }
    let mut mapping = vec![(
        parameter_id(&facts.types, Argument::Type(schema.self_type))?,
        Argument::Type(predicate.ty),
    )];
    let mut regions = Vec::new();
    for (formal, actual) in parameters.iter().zip(arguments) {
        if let Argument::Lifetime(_) = formal.argument {
            let Argument::Lifetime(region) = actual else {
                return Err("associated result lifetime argument kind mismatch");
            };
            regions.push(region);
        } else {
            mapping.push((parameter_id(&facts.types, formal.argument)?, actual));
        }
    }
    if regions.len() != schema.captures as usize {
        return Err("associated result lifetime arity mismatch");
    }
    crate::types::substitute::Bound::parameters_captured(&facts.types, &mapping, regions)
}

/// Complete finite result lifetime templates before scoped lifetime validation.
/// Recursive associated results remain demand-driven; only ordinary Self
/// supertraits contribute to the member's own promises.
pub(in crate::sema) fn prepare_outlives(facts: &mut Declarations) -> Result<(), String> {
    if facts.associated.types.is_empty() {
        return Ok(());
    }
    let direct = facts.associated.types.values().any(|contract| {
        contract
            .bounds
            .iter()
            .any(|p| matches!(p.bound, Bound::Outlives(_)))
    });
    let inherited = facts.generics.traits.as_ref().is_some_and(|contracts| {
        contracts.headers().any(|(_, header)| {
            header
                .predicates
                .iter()
                .any(|p| header.implies_self(p) && matches!(p.value.bound, Bound::Outlives(_)))
        })
    });
    if !direct && !inherited {
        return Ok(());
    }
    for index in 0..facts.associated.members().len() {
        let item = facts.associated.members()[index].definition;
        if !facts.associated.types.contains_key(&item) {
            continue;
        }
        let (templates, _) = templates::for_result(facts, item)?;
        facts
            .types
            .declare_projection_outlives(ItemId(item.0), &templates.outlives)?;
    }
    Ok(())
}
