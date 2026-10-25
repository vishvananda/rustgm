//! Demand normal source scalar conformance before executable constant work.
use super::{
    Checker, DefId, Error, Work,
    eligibility_profile::{Eligibility, Plan, Unsupported},
};
use crate::{support::hash::HashMap, types::Kind};
#[derive(Clone, Copy)]
pub(in crate::sema::declarations) enum Phase {
    Try,
    Begin,
    Signatures,
    Requirements,
}
#[derive(Default)]
pub(in crate::sema::declarations) struct Pending {
    arena: Option<crate::types::arena::Identity>,
    pub(super) plans: HashMap<DefId, Plan>,
    pub(super) active: HashMap<DefId, super::prerequisite_queries::Attempt>,
    pub(super) skipped: HashMap<DefId, Unsupported>,
    pub(super) members: HashMap<DefId, Vec<DefId>>,
    pub(super) methods: super::methods::Members,
    pub(super) pairs: super::implementation_pairs::Pairs,
    pub(super) overlap_steps: usize,
    pub(super) work: usize,
}
impl Pending {
    pub(super) fn skip(
        &mut self,
        facts: &mut crate::sema::model::Declarations,
        owner: DefId,
        reason: Unsupported,
    ) -> Result<(), &'static str> {
        let arena = self.arena_for(facts)?;
        let source = facts
            .associated
            .owner(owner)
            .and_then(|id| facts.associated.get(id));
        if let Some(source) = source {
            let registry = facts
                .generics
                .implementations
                .as_mut()
                .ok_or("early profile lost its implementation registry")?;
            let (header_source, predicate) = registry
                .header(owner)
                .map(|h| (h.source, h.predicate))
                .ok_or("early profile lost its source header")?;
            if source.self_definition != owner
                || source.source != header_source
                || !matches!(source.kind, crate::sema::associated::OwnerKind::Impl { .. })
            {
                return Err("early profile changed its actual source owner");
            }
            registry.mark_early_unavailable(
                &facts.types,
                &arena,
                owner,
                header_source,
                predicate,
            )?;
        }
        self.skipped.insert(owner, reason);
        Ok(())
    }
    pub(super) fn arena_for(
        &mut self,
        facts: &mut crate::sema::model::Declarations,
    ) -> Result<crate::types::arena::Identity, &'static str> {
        self.charge(facts, 1)?;
        if let Some(arena) = &self.arena {
            if !facts.types.has_identity(arena) {
                return Err("scalar scheduling belongs to another actual type arena");
            }
            return Ok(arena.clone());
        }
        let arena = facts.types.arena_identity();
        self.arena = Some(arena.clone());
        Ok(arena)
    }
    pub(super) fn charge(
        &mut self,
        facts: &mut crate::sema::model::Declarations,
        amount: usize,
    ) -> Result<(), &'static str> {
        self.work = self.work.saturating_add(amount);
        facts.body_work.canonical_steps = facts.body_work.canonical_steps.saturating_add(amount);
        if self.work > 1_000_000 {
            return Err("implementation prerequisite work budget exceeded");
        }
        Ok(())
    }
}
impl Checker<'_> {
    pub(in crate::sema::declarations) fn prepare_early_implementations(
        &mut self,
        steps: &mut usize,
        limit: usize,
    ) -> Result<(), Error> {
        if self.report.package.source_core().is_none() {
            return Ok(());
        }
        let mut pending = Pending::default();
        pending
            .charge(&mut self.facts, 2)
            .map_err(Error::unlocated)?;
        let catalog = self.facts.source_bindings.as_deref().ok_or_else(|| {
            Error::unlocated("source implementation preparation has no current catalog")
        })?;
        catalog
            .require_provider_report(self.report)
            .map_err(Error::unlocated)?;
        let copy = catalog
            .validated_language(crate::support::bindings::LangItem::Copy)
            .map_err(Error::unlocated)?;
        let Some(copy) = copy else {
            return Ok(());
        };
        let count = self
            .facts
            .generics
            .implementations
            .as_ref()
            .map_or(0, |r| r.header_count());
        pending
            .charge(&mut self.facts, count)
            .map_err(Error::unlocated)?;
        let Some(registry) = self.facts.generics.implementations.as_ref() else {
            return Ok(());
        };
        let mut owners = registry
            .headers()
            .filter_map(|header| {
                let (application, _) = header.predicate.bound.trait_parts()?;
                let trait_ = self.facts.types.trait_application(application)?;
                (trait_.item.0 == copy.definition.0
                    && matches!(
                        self.facts.types.kind(header.predicate.ty),
                        Some(Kind::Primitive(_))
                    ))
                .then_some(header.owner)
            })
            .collect::<Vec<_>>();
        if owners.is_empty() {
            return Ok(());
        }
        owners.sort_unstable_by_key(|owner| owner.0);
        self.append_implementation_pairs(&mut pending)?;
        self.early_implementations = Some(Box::new(pending));
        for owner in owners {
            self.work.push(Work::ValidateImpl(owner, Phase::Try));
            self.drain(steps, limit)?;
        }
        Ok(())
    }
    pub(super) fn ensure_scalar_plan(
        &mut self,
        owner: DefId,
        pending: &mut Pending,
    ) -> Result<bool, Error> {
        pending
            .charge(&mut self.facts, 1)
            .map_err(Error::unlocated)?;
        if pending.skipped.contains_key(&owner) {
            return Ok(false);
        }
        if pending.plans.contains_key(&owner)
            || pending.active.contains_key(&owner)
            || self
                .facts
                .generics
                .implementations
                .as_ref()
                .is_some_and(|r| r.is_validated(owner))
        {
            return Ok(true);
        }
        let (source, predicate) = self
            .facts
            .generics
            .implementations
            .as_ref()
            .and_then(|r| r.header(owner))
            .map(|h| (h.source, h.predicate))
            .ok_or_else(|| {
                Error::unlocated("implementation validation requires its actual header")
            })?;
        match self.early_impl_plan(owner, source, predicate, pending)? {
            Eligibility::Ready(plan) => {
                pending.plans.insert(owner, plan);
                Ok(true)
            }
            Eligibility::Unsupported(reason) => {
                pending
                    .skip(&mut self.facts, owner, reason)
                    .map_err(Error::unlocated)?;
                Ok(false)
            }
        }
    }
    pub(in crate::sema::declarations) fn advance_impl_validation(
        &mut self,
        owner: DefId,
        phase: Phase,
    ) -> Result<(), Error> {
        if self
            .facts
            .generics
            .implementations
            .as_ref()
            .is_some_and(|r| r.is_validated(owner))
        {
            return Ok(());
        }
        let mut pending = self.early_implementations.take().ok_or_else(|| {
            Error::unlocated("implementation validation has no source scheduling context")
        })?;
        let result = self.advance_scalar_impl(owner, phase, &mut pending);
        self.early_implementations = Some(pending);
        result
    }
    fn advance_scalar_impl(
        &mut self,
        owner: DefId,
        phase: Phase,
        pending: &mut Pending,
    ) -> Result<(), Error> {
        if pending.skipped.contains_key(&owner) {
            return if matches!(phase, Phase::Try) {
                Ok(())
            } else {
                Err(Error::unlocated(
                    "implementation validation is outside the early scalar profile",
                ))
            };
        }
        match phase {
            Phase::Try | Phase::Begin => {
                if pending.active.contains_key(&owner) {
                    return Err(Error::unlocated(
                        "cycle in implementation validation dependencies",
                    ));
                }
                if !self.ensure_scalar_plan(owner, pending)? {
                    return if matches!(phase, Phase::Try) {
                        Ok(())
                    } else {
                        Err(Error::unlocated(
                            "implementation validation is outside the early scalar profile",
                        ))
                    };
                }
                self.append_implementation_pairs(pending)?;
                let count = pending.pairs.count(Some(owner));
                pending
                    .charge(&mut self.facts, count)
                    .map_err(Error::unlocated)?;
                for peer in pending.pairs.peers(owner) {
                    if !self.ensure_scalar_plan(peer, pending)? {
                        pending
                            .skip(&mut self.facts, owner, Unsupported::Requirements)
                            .map_err(Error::unlocated)?;
                        return Ok(());
                    }
                }
                let plan = pending
                    .plans
                    .remove(&owner)
                    .ok_or_else(|| Error::unlocated("scalar implementation has no exact plan"))?;
                pending
                    .charge(&mut self.facts, 4)
                    .map_err(|e| self.error(plan.source, e))?;
                plan.require_current(&self.facts, owner)
                    .map_err(|e| self.error(plan.source, e))?;
                self.work.push(Work::ValidateImpl(owner, Phase::Signatures));
                self.work.extend(
                    plan.members
                        .iter()
                        .rev()
                        .map(|&m| Work::Definition(m, false)),
                );
                pending
                    .active
                    .insert(owner, super::prerequisite_queries::Attempt::new(plan));
            }
            Phase::Signatures | Phase::Requirements => {
                self.scalar_impl_requirements(owner, pending)?;
            }
        }
        Ok(())
    }
    pub(super) fn append_implementation_pairs(
        &mut self,
        pending: &mut Pending,
    ) -> Result<(), Error> {
        let added = self
            .facts
            .generics
            .implementations
            .as_mut()
            .map_or_else(Vec::new, |r| r.take_overlap_pairs());
        pending
            .charge(&mut self.facts, added.len())
            .map_err(Error::unlocated)?;
        pending.pairs.append(added);
        Ok(())
    }
    pub(super) fn check_implementation_pairs(
        &mut self,
        owner: Option<DefId>,
        pending: &mut Pending,
    ) -> Result<(), Error> {
        self.append_implementation_pairs(pending)?;
        pending
            .charge(&mut self.facts, pending.pairs.count(owner))
            .map_err(Error::unlocated)?;
        for (a, owner_a, b, owner_b) in pending.pairs.take(owner) {
            let source = self
                .facts
                .generics
                .implementations
                .as_ref()
                .and_then(|r| r.header(owner_b))
                .map(|h| h.source)
                .ok_or_else(|| Error::unlocated("missing scalar coherence owner"))?;
            let before = pending.overlap_steps;
            let result = self.validate_impl_pair(
                source,
                (a, owner_a),
                (b, owner_b),
                &mut pending.overlap_steps,
            );
            pending
                .charge(
                    &mut self.facts,
                    pending.overlap_steps.saturating_sub(before),
                )
                .map_err(|e| self.error(source, e))?;
            result?;
        }
        Ok(())
    }
    pub(super) fn complete_scalar_impl(
        &mut self,
        owner: DefId,
        plan: &Plan,
        pending: &mut Pending,
    ) -> Result<(), Error> {
        pending
            .charge(&mut self.facts, 4)
            .map_err(|e| self.error(plan.source, e))?;
        plan.require_current(&self.facts, owner)
            .map_err(|e| self.error(plan.source, e))?;
        self.check_implementation_pairs(Some(owner), pending)?;
        let before = pending.overlap_steps;
        pending
            .charge(&mut self.facts, plan.member_check_work())
            .map_err(|e| self.error(plan.source, e))?;
        let result = self.validate_impl_owner(
            owner,
            plan.source,
            plan.predicate,
            super::validation::Conformance::FunctionsOnly {
                methods: &pending.methods,
                plan,
            },
            &mut pending.overlap_steps,
        );
        pending
            .charge(
                &mut self.facts,
                pending.overlap_steps.saturating_sub(before),
            )
            .map_err(|e| self.error(plan.source, e))?;
        result
    }
}
