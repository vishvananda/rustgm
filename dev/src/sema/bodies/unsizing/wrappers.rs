//! Whole-value wrapper coercions retain their actual ordinary trait obligation.
use super::{Checker, Constructor, Error, Justification, NodeId, Pending, TermId};
use crate::{
    sema::model::AdjustmentKind,
    support::{bindings::LangItem, retained::Growth},
    types::{
        ItemId,
        infer::{arguments::Argument, predicates::Predicate},
        unsizing::Conversion,
    },
};

impl Checker<'_> {
    pub(in crate::sema::bodies) fn validate_nominal_goal(
        &mut self,
        index: usize,
        solved: &crate::types::infer::Solved,
        predicates: &[crate::types::parameters::predicates::Predicate],
        evidence: &crate::support::hash::HashMap<usize, crate::sema::traits::selection::Source>,
    ) -> Result<(), Error> {
        let Some(pending) = self.state.unsizing_adjustments.get(&index) else {
            return Ok(());
        };
        let Justification::NominalGoal(goal) = pending.justification else {
            if matches!(pending.conversion, Conversion::Nominal { .. }) {
                return Err(self.error(
                    self.state.body.node,
                    "nominal coercion lost its actual trait goal",
                ));
            }
            return Ok(());
        };
        let adjustment = self.state.adjustments.get(index).ok_or_else(|| {
            self.error(
                self.state.body.node,
                "nominal coercion adjustment disappeared",
            )
        })?;
        let node = adjustment.node;
        self.facts
            .charge_caller_inputs(1, 0)
            .map_err(|e| self.error(node, e))?;
        let Conversion::Nominal { source, target } = pending.conversion else {
            return Err(self.error(node, "nominal trait goal belongs to a different conversion"));
        };
        let predicate = predicates
            .get(goal as usize)
            .ok_or_else(|| self.error(node, "nominal coercion goal was not solved"))?;
        let source = solved.types[source.0 as usize];
        let target = solved.types[target.0 as usize];
        if self
            .state
            .call_types
            .obligations
            .get(goal as usize)
            .is_none_or(|(site, _)| *site != node)
            || source != solved.types[adjustment.actual.0 as usize]
            || target != solved.types[adjustment.expected.0 as usize]
            || predicate.ty != source
            || !evidence.contains_key(&(goal as usize))
        {
            return Err(self.error(
                node,
                "nominal coercion changed its actual body/goal/application",
            ));
        }
        let Some((id, crate::support::constness::Constness::Ordinary)) =
            predicate.bound.trait_parts()
        else {
            return Err(self.error(node, "nominal coercion lost its ordinary canonical trait"));
        };
        let application = self
            .facts
            .types
            .trait_application(id)
            .ok_or_else(|| self.error(node, "nominal coercion goal application is missing"))?;
        if crate::sema::compiler_items::unsizing_role(
            self.facts,
            crate::resolve::model::DefId(application.item.0),
        )
        .map_err(|e| self.error(node, e))?
            != Some(LangItem::CoerceUnsized)
            || self
                .facts
                .types
                .arguments(application.arguments)
                .is_none_or(|mut a| {
                    a.next() != Some(crate::types::arguments::Argument::Type(target))
                        || a.next().is_some()
                })
        {
            return Err(self.error(node, "nominal coercion goal changed its actual role/target"));
        }
        if let Some(crate::sema::traits::selection::Source::Implementation { owner, arguments }) =
            evidence.get(&(goal as usize))
        {
            selected_schema(self.facts, *owner, *arguments, source, target)
                .map_err(|e| self.error(node, e))?;
        }
        Ok(())
    }
}

fn selected_schema(
    facts: &mut crate::sema::model::Declarations,
    owner: crate::resolve::model::DefId,
    arguments: crate::types::GenericArgs,
    source: crate::types::TypeId,
    target: crate::types::TypeId,
) -> Result<(), &'static str> {
    if !facts
        .generics
        .implementations
        .as_ref()
        .is_some_and(|r| r.is_validated(owner))
    {
        return Err("nominal coercion requires its actual validated implementation");
    }
    let pay =
        crate::sema::header_attributes::caller_payments(facts.must_use_contracts.as_deref(), true)
            .map_err(|_| "nominal coercion selected application lacks its source payer")?;
    let (mut fold, _) =
        facts.selected_implementation_substitution(owner, arguments, &|w, b| pay.charge(w, b))?;
    let schema = facts
        .unsizing_rules
        .as_deref()
        .ok_or("nominal coercion declaration recipe owner is missing")?
        .nominal_schema(&facts.types, ItemId(owner.0))?
        .ok_or("nominal coercion requires its actual source declaration recipe")?;
    let (from, to) = schema.endpoints(&mut facts.types, &mut fold, &|w, b| pay.charge(w, b))?;
    if from != source || to != target {
        return Err("nominal coercion selected arguments changed its complete source/target");
    }
    Ok(())
}

impl Checker<'_> {
    pub(in crate::sema::bodies) fn source_nominal_coercion(
        &mut self,
        node: NodeId,
        actual: TermId,
        expected: TermId,
    ) -> Result<bool, Error> {
        if self.facts.source_bindings.is_none() {
            return Ok(false);
        }
        let (
            Some(Constructor::Adt(owner, _) | Constructor::AdtArgs(owner, _)),
            Some(Constructor::Adt(other, _) | Constructor::AdtArgs(other, _)),
        ) = (self.head(node, actual)?, self.head(node, expected)?)
        else {
            return Ok(false);
        };
        if owner != other {
            return Ok(false);
        }
        let Some(catalog) = self.facts.source_bindings.as_deref() else {
            return Ok(false);
        };
        let Some(role) = catalog
            .validated_role(LangItem::CoerceUnsized)
            .map_err(|e| self.error(node, e))?
        else {
            return Ok(false);
        };
        if !matches!(
            role.origin,
            crate::resolve::body_origin::DeclarationOrigin::Source(_)
        ) {
            return Err(self.error(
                node,
                "imported nominal coercion requires authenticated declaration/plan transport",
            ));
        }
        catalog
            .require_provider_report(self.report)
            .map_err(|e| self.error(node, e))?;
        if !self
            .state
            .infer
            .unsizing_mismatch(&self.facts.types, actual, expected)
            .map_err(|e| self.error(node, e))?
        {
            return Ok(false);
        }
        let goal = self.retain_nominal_goal(node, role.definition, actual, expected)?;
        self.state.unsizing_adjustments.insert(
            self.state.adjustments.len(),
            Pending {
                view: expected,
                conversion: Conversion::Nominal {
                    source: actual,
                    target: expected,
                },
                justification: Justification::NominalGoal(goal),
                relations: Vec::new(),
            },
        );
        self.record_adjustment(node, expected, AdjustmentKind::Unsize)?;
        Ok(true)
    }
    fn retain_nominal_goal(
        &mut self,
        node: NodeId,
        role: crate::resolve::model::DefId,
        actual: TermId,
        expected: TermId,
    ) -> Result<u32, Error> {
        let index = self.state.call_types.obligations.len();
        let goal = u32::try_from(index)
            .map_err(|_| self.error(node, "nominal coercion goal index overflow"))?;
        let mut growth = Growth::vector::<(NodeId, Predicate)>(
            index,
            self.state.call_types.obligations.capacity(),
            1,
        );
        let pending = &self.state.unsizing_adjustments;
        growth.include(Growth::map::<usize, Pending>(
            pending.len(),
            pending.capacity(),
            1,
        ));
        self.facts
            .charge_caller_inputs(
                growth.work.saturating_add(1),
                growth.bytes.saturating_add(size_of::<Argument>()),
            )
            .map_err(|e| self.error(node, e))?;
        self.state
            .infer
            .account_work(growth.work.saturating_add(1))
            .map_err(|e| self.error(node, e))?;
        self.state
            .call_types
            .obligations
            .try_reserve(1)
            .map_err(|_| self.error(node, "nominal coercion goal allocation failed"))?;
        self.state
            .unsizing_adjustments
            .try_reserve(1)
            .map_err(|_| self.error(node, "nominal coercion adjustment allocation failed"))?;
        self.state.call_types.obligations.push((
            node,
            Predicate::trait_application(actual, ItemId(role.0), vec![Argument::Type(expected)]),
        ));
        Ok(goal)
    }
}
