//! Immutable family input contracts and complete applications, separate from result promises.
use super::{Budget, Implied, Pair, Scan, free, requirements::Demand};
use crate::support::hash::HashMap;
use crate::types::{
    ItemId, Store,
    arguments::Argument,
    parameters::{
        ParameterId,
        clauses::Clause,
        predicates::{Bound, Predicate},
    },
    projection::Application,
    substitute::families,
};
#[cfg(test)]
#[path = "../../../tests/invariants/input_contracts/lifetimes.rs"]
mod tests;

pub(crate) struct Inputs {
    pub owner: ItemId,
    pub subject: ParameterId,
    pub captures: u32,
    pub parent: Box<[Argument]>,
    pub own: Box<[Argument]>,
}
struct Template {
    inputs: Inputs,
    conditions: Conditions,
}
#[derive(Default)]
pub(super) struct Conditions {
    clauses: Vec<Predicate>,
    regions: Vec<Pair>,
}
#[derive(Default)]
pub(crate) struct Families {
    templates: HashMap<ItemId, Template>,
    instances: HashMap<Application, Conditions>,
    implied: Implied,
    budget: Budget<'static>,
}
impl Families {
    pub(crate) fn insert(
        &mut self,
        types: &mut Store,
        member: ItemId,
        inputs: Inputs,
        clauses: &[Clause],
        regions: &[Pair],
    ) -> Result<(), &'static str> {
        if self.templates.contains_key(&member) {
            return Err("family input lifetime contract is already complete");
        }
        self.budget.work(
            inputs
                .parent
                .len()
                .saturating_add(inputs.own.len())
                .saturating_add(clauses.len())
                .saturating_add(regions.len())
                .saturating_add(1),
        )?;
        // Trait Self has its own synthetic declaration owner, separate from
        // the declaration owning explicit parent arguments.
        if types
            .parameter(inputs.subject)
            .is_none_or(|p| !matches!(p.kind, crate::types::parameters::ParameterKind::Type { .. }))
            || types
                .projection_schema(member)
                .is_none_or(|s| s.trait_item != inputs.owner)
        {
            return Err("family input schema has a different parent");
        }
        let mut conditions = Conditions::default();
        for &clause in clauses {
            if let Some(predicate) = self.implied.outlives(types, clause, 0)? {
                conditions.clauses.push(predicate);
            }
        }
        conditions.regions.extend_from_slice(regions);
        self.templates
            .insert(member, Template { inputs, conditions });
        Ok(())
    }

    pub(super) fn is_empty(&self) -> bool {
        self.templates.is_empty()
    }

    pub(super) fn steps(&self) -> usize {
        self.implied.steps().saturating_add(self.budget.0)
    }

    pub(super) fn prepare(
        &mut self,
        types: &mut Store,
        application: Application,
        budget: &mut Budget<'_>,
    ) -> Result<(), &'static str> {
        if self.instances.contains_key(&application) {
            return Ok(());
        }
        let Some(template) = self.templates.get(&application.item) else {
            return Ok(());
        };
        let parent = types
            .trait_application(application.trait_application)
            .ok_or("invalid family input parent")?;
        let schema = &template.inputs;
        if parent.item != schema.owner {
            return Err("family input application has a different parent");
        }
        budget.work(
            schema
                .parent
                .len()
                .saturating_add(schema.own.len())
                .saturating_add(template.conditions.clauses.len())
                .saturating_add(template.conditions.regions.len())
                .saturating_add(1),
        )?;
        let mut mapping = families::mappings(
            types,
            families::Application {
                owner: parent.item,
                parent: parent.arguments,
                member: application.item,
                own: application.arguments,
            },
            (schema.parent.iter().copied(), schema.own.iter().copied()),
            schema.captures,
        )?;
        mapping
            .parameters
            .push((schema.subject, Argument::Type(application.subject)));
        let mut fold = mapping.finish(types)?;
        let mut conditions = Conditions::default();
        for &predicate in &template.conditions.clauses {
            conditions.clauses.push(fold.predicate(types, predicate)?);
        }
        for &(a, b) in &template.conditions.regions {
            conditions
                .regions
                .push((fold.region(a, 0)?, fold.region(b, 0)?));
        }
        budget.work(fold.steps)?;
        self.instances.insert(application, conditions);
        Ok(())
    }

    pub(super) fn instance(
        &self,
        application: Application,
    ) -> Result<Option<&Conditions>, &'static str> {
        if !self.templates.contains_key(&application.item) {
            return Ok(None);
        }
        self.instances
            .get(&application)
            .map(Some)
            .ok_or("family input lifetime requirements were not prepared")
    }
}

impl Scan<'_> {
    pub(super) fn family_demands(
        &mut self,
        types: &Store,
        bounds: &super::Bounds,
        demand: &mut impl Demand,
    ) -> Result<(), &'static str> {
        let Some(families) = bounds.projected.as_ref().and_then(|p| p.families.as_ref()) else {
            return Ok(());
        };
        for index in 0..self.family_uses.len() {
            let (application, depth) = self.family_uses[index];
            self.budget.charge()?;
            let Some(conditions) = families.instance(application)? else {
                continue;
            };
            for &predicate in &conditions.clauses {
                self.budget.charge()?;
                let Bound::Outlives(shorter) = predicate.bound else {
                    return Err("family lifetime contract contains another predicate kind");
                };
                let Some(shorter) = free(shorter, depth)? else {
                    continue;
                };
                if depth != 0
                    && self
                        .components
                        .captures(types, predicate.ty, depth, &mut self.budget)?
                {
                    continue;
                }
                demand.require(
                    self,
                    types,
                    predicate.ty,
                    shorter,
                    depth,
                    super::ComponentKind::TypeBound,
                )?;
            }
            for &(a, b) in &conditions.regions {
                self.budget.charge()?;
                if let Some(pair) = free(a, depth)?.zip(free(b, depth)?) {
                    self.record(pair)?;
                }
            }
        }
        Ok(())
    }
}
