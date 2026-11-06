//! Nominal conversions use ordinary implementation selection and its caller debts.
mod transfer_layout;
use super::{Declarations, Dispatcher, Failure, Preparation};
use crate::{
    resolve::{body_origin::DeclarationOrigin, model::DefId},
    support::bindings::LangItem,
    types::{
        ItemId, TypeId,
        arguments::Argument,
        parameters::predicates::{Bound, Predicate},
    },
};

impl Dispatcher {
    pub(crate) fn nominal_coercion(
        &mut self,
        facts: &mut Declarations,
        source: TypeId,
        target: TypeId,
        mut preparation: Option<&mut dyn Preparation>,
    ) -> Result<(), Failure> {
        facts.charge_caller_inputs(
            1,
            size_of::<crate::support::hash::HashSet<(TypeId, TypeId)>>(),
        )?;
        let mut next = Some((source, target));
        let mut seen = crate::support::hash::HashSet::default();
        while let Some((source, target)) = next.take() {
            let growth = crate::support::retained::Growth::map::<(TypeId, TypeId), ()>(
                seen.len(),
                seen.capacity(),
                1,
            );
            facts.charge_caller_inputs(growth.work.saturating_add(1), growth.bytes)?;
            seen.try_reserve(1)
                .map_err(|_| "nominal conversion traversal allocation failed")?;
            if !seen.insert((source, target)) {
                return Err("cycle in nominal physical conversion applications".into());
            }
            let owner = if let Some(service) = &mut preparation {
                self.selected_nominal(facts, source, target, Some(&mut **service))?
            } else {
                self.selected_nominal(facts, source, target, None)?
            };
            let cached = facts
                .unsizing_rules
                .as_deref()
                .ok_or("nominal conversion lacks its declaration recipe owner")?
                .nominal_plan(&facts.types, source, target)?;
            let step = if let Some(plan) = cached {
                if plan.owner != owner {
                    return Err(
                        "nominal physical plan belongs to another selected implementation".into(),
                    );
                }
                plan.step
            } else {
                let plan = transfer_layout::build(facts, owner, source, target)?;
                let step = plan.step;
                let pay = crate::sema::header_attributes::caller_payments(
                    facts.must_use_contracts.as_deref(),
                    true,
                )
                .map_err(Failure::Preparation)?;
                facts
                    .unsizing_rules
                    .as_deref_mut()
                    .ok_or("nominal conversion recipe owner disappeared")?
                    .retain_nominal_plan(&facts.types, plan, &|w, b| pay.charge(w, b))?;
                step
            };
            if let crate::types::unsizing::CoercionStep::Nested { source, target } = step {
                next = Some((source, target));
            }
        }
        Ok(())
    }
    fn selected_nominal(
        &mut self,
        facts: &mut Declarations,
        source: TypeId,
        target: TypeId,
        preparation: Option<&mut dyn Preparation>,
    ) -> Result<ItemId, Failure> {
        let role = facts
            .source_bindings
            .as_deref()
            .ok_or("nominal dispatch requires its actual source catalog")?
            .validated_role(LangItem::CoerceUnsized)?
            .ok_or("nominal dispatch requires its actual CoerceUnsized role")?;
        if !matches!(role.origin, DeclarationOrigin::Source(_)) {
            return Err("imported nominal coercion requires its declaration/plan transport".into());
        }
        let values = [Argument::Type(target)];
        let mut growth = facts.types.argument_interning_growth(&values);
        growth.include(facts.types.trait_application_growth());
        facts.charge_caller_inputs(growth.work.saturating_add(1), growth.bytes)?;
        let arguments = facts.types.intern_arguments(&values)?;
        let application =
            facts
                .types
                .intern_trait_application(crate::types::traits::Application {
                    item: ItemId(role.definition.0),
                    arguments,
                })?;
        let predicate = Predicate {
            ty: source,
            bound: Bound::Trait(application),
        };
        let before = self.normalize.steps;
        let normalized = self.normalize.predicate(&mut facts.types, predicate)?;
        self.context
            .account_work(self.normalize.steps.saturating_sub(before))
            .map_err(|e| e.to_string())?;
        let (owner, arguments, proof) = self.implementation_using(facts, predicate, preparation)?;
        let target = match normalized.bound {
            Bound::Trait(id) => facts
                .types
                .trait_application(id)
                .and_then(|application| facts.types.arguments(application.arguments))
                .and_then(|mut args| match (args.next(), args.next()) {
                    (Some(Argument::Type(ty)), None) => Some(ty),
                    _ => None,
                })
                .ok_or("nominal dispatch lost its full target application")?,
            _ => return Err("nominal dispatch changed its ordinary trait goal".into()),
        };
        validate_schema(facts, owner, arguments, normalized.ty, target)?;
        self.extend_evidence(proof)?;
        Ok(ItemId(owner.0))
    }
}

fn validate_schema(
    facts: &mut Declarations,
    owner: DefId,
    arguments: crate::types::GenericArgs,
    source: TypeId,
    target: TypeId,
) -> Result<(), Failure> {
    if !facts
        .generics
        .implementations
        .as_ref()
        .is_some_and(|r| r.is_validated(owner))
    {
        return Err("nominal dispatch requires its validated selected implementation".into());
    }
    let pay =
        crate::sema::header_attributes::caller_payments(facts.must_use_contracts.as_deref(), true)
            .map_err(Failure::Preparation)?;
    let (mut fold, _) =
        facts.selected_implementation_substitution(owner, arguments, &|w, b| pay.charge(w, b))?;
    let schema = facts
        .unsizing_rules
        .as_deref()
        .ok_or("nominal dispatch lacks its declaration recipe owner")?
        .nominal_schema(&facts.types, ItemId(owner.0))?
        .ok_or("nominal dispatch lacks its actual selected declaration recipe")?;
    if schema.endpoints(&mut facts.types, &mut fold, &|w, b| pay.charge(w, b))? != (source, target)
    {
        return Err("nominal dispatch changed its selected full application/schema".into());
    }
    Ok(())
}
