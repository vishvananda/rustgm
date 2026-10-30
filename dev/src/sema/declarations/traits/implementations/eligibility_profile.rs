//! Explicit source-only eligibility retains exact headers, not implementation permission.
use super::{Checker, DefId, Error, NodeKey, scalar_readiness::Pending};
use crate::{
    resolve::model::DefKind,
    sema::{
        associated::{Owner, OwnerKind},
        model::Declarations,
        traits::assumptions::Assumptions,
    },
    types::{
        Kind,
        parameters::{
            clauses::Clause,
            environment::Environment,
            predicates::{Bound, Predicate},
        },
    },
};
use std::sync::Arc;

#[derive(Clone, Copy, Debug)]
pub(super) enum Unsupported {
    SourceOwner,
    OpenSubject,
    Members,
    Requirements,
    ProofDebt,
}
pub(super) enum Eligibility {
    Ready(Plan),
    Unsupported(Unsupported),
}
pub(super) struct Plan {
    arena: crate::types::arena::Identity,
    member_profile: MemberProfile,
    owner_kind: OwnerKind,
    pub source: NodeKey,
    pub predicate: Predicate,
    pub requirements: Vec<Predicate>,
    pub members: Vec<DefId>,
    pub environment: Option<Arc<Environment>>,
    pub assumptions: Option<Arc<Assumptions>>,
}
enum MemberProfile {
    FunctionsOnly(NoAssociatedMembers),
    MethodsAndConstants(NoAssociatedTypes),
    Complete(CompleteMembers),
}
struct CompleteMembers {
    owners: [Owner; 2],
    counts: [usize; 2],
}
struct NoAssociatedTypes {
    owners: [Owner; 2],
    counts: [usize; 2],
}
struct NoAssociatedMembers {
    owners: [Owner; 2],
    counts: [usize; 2],
}
impl Plan {
    pub(super) fn member_check_work(&self) -> usize {
        self.members.len().saturating_add(6)
    }
    pub(super) fn require_no_associated(
        &self,
        facts: &Declarations,
        owner: DefId,
    ) -> Result<(), &'static str> {
        let MemberProfile::FunctionsOnly(seal) = &self.member_profile else {
            return Err("functions-only conformance requires its original member seal");
        };
        self.require_members(facts, owner, seal.owners, seal.counts)
    }
    pub(super) fn has_constants(&self) -> bool {
        matches!(self.member_profile, MemberProfile::MethodsAndConstants(_))
    }
    pub(super) fn require_constants(
        &self,
        facts: &Declarations,
        owner: DefId,
    ) -> Result<(), &'static str> {
        let MemberProfile::MethodsAndConstants(seal) = &self.member_profile else {
            return Err("constant conformance requires its actual member seal");
        };
        self.require_members(facts, owner, seal.owners, seal.counts)
    }
    pub(super) fn has_types(&self) -> bool {
        matches!(self.member_profile, MemberProfile::Complete(_))
    }
    pub(super) fn require_complete(
        &self,
        facts: &Declarations,
        owner: DefId,
    ) -> Result<[DefId; 2], &'static str> {
        let MemberProfile::Complete(seal) = &self.member_profile else {
            return Err("complete conformance requires its actual member seal");
        };
        self.require_members(facts, owner, seal.owners, seal.counts)?;
        Ok(seal.owners.map(|owner| owner.self_definition))
    }
    fn require_members(
        &self,
        facts: &Declarations,
        owner: DefId,
        owners: [Owner; 2],
        counts: [usize; 2],
    ) -> Result<(), &'static str> {
        self.require_current(facts, owner)?;
        let mut offset = 0;
        for (expected, count) in owners.iter().zip(counts) {
            let id = facts
                .associated
                .owner(expected.self_definition)
                .ok_or("functions-only conformance lost its indexed source owner")?;
            if facts.associated.get(id) != Some(expected) {
                return Err("functions-only conformance changed its actual owner kind");
            }
            let end = offset + count;
            if facts.associated.owner_members(id) != self.members.get(offset..end) {
                return Err("functions-only conformance changed its complete member bucket");
            }
            offset = end;
        }
        if offset != self.members.len() {
            return Err("functions-only conformance has incomplete member provenance");
        }
        Ok(())
    }

    pub(super) fn require_current(
        &self,
        facts: &Declarations,
        owner: DefId,
    ) -> Result<(), &'static str> {
        if !facts.types.has_identity(&self.arena) {
            return Err("scalar validation belongs to another actual type arena");
        }
        let header = facts
            .generics
            .implementations
            .as_ref()
            .and_then(|registry| registry.header(owner))
            .ok_or("scalar validation lost its actual implementation header")?;
        let actual = facts
            .associated
            .owner(owner)
            .and_then(|id| facts.associated.get(id))
            .ok_or("scalar validation lost its actual source owner")?;
        if header.owner != owner
            || header.source != Some(self.source)
            || header.predicate != self.predicate
            || actual.self_definition != owner
            || actual.source != Some(self.source)
            || actual.kind != self.owner_kind
        {
            return Err("scalar validation changed its actual owner or complete predicate");
        }
        let same = match (
            self.environment.as_ref(),
            facts.generics.environments.get(&owner),
        ) {
            (None, None) => true,
            (Some(expected), Some(actual)) => Arc::ptr_eq(expected, actual),
            _ => false,
        };
        if !same {
            return Err("scalar validation changed its immutable implementation environment");
        }
        Ok(())
    }
}

impl Checker<'_> {
    pub(super) fn early_impl_plan(
        &mut self,
        owner: DefId,
        source: NodeKey,
        predicate: Predicate,
        pending: &mut Pending,
    ) -> Result<Eligibility, Error> {
        pending
            .charge(&mut self.facts, 1)
            .map_err(|e| self.error(source, e))?;
        let actual = self
            .facts
            .associated
            .owner(owner)
            .and_then(|id| self.facts.associated.get(id))
            .copied();
        if actual.is_none_or(|actual| {
            actual.self_definition != owner
                || actual.source != Some(source)
                || !matches!(
                    actual.kind,
                    OwnerKind::Impl {
                        polarity: crate::sema::associated::ImplPolarity::Positive,
                        ..
                    }
                )
        }) {
            return Ok(Eligibility::Unsupported(Unsupported::SourceOwner));
        }
        let owner_kind = actual
            .ok_or_else(|| self.error(source, "source owner disappeared"))?
            .kind;
        if !self.early_source_subject(owner, predicate.ty, pending)?
            || (matches!(
                self.facts.types.kind(predicate.ty),
                Some(Kind::Primitive(_))
            ) && predicate
                .contains_parameters(&self.facts.types)
                .map_err(|e| self.error(source, e))?)
            || predicate
                .has_free_regions(&self.facts.types)
                .map_err(|e| self.error(source, e))?
            || self.trait_region_count(owner) != 0
        {
            return Ok(Eligibility::Unsupported(Unsupported::OpenSubject));
        }
        let Some((application, _)) = predicate.bound.trait_parts() else {
            return Ok(Eligibility::Unsupported(Unsupported::Requirements));
        };
        let Some(trait_owner) = self.scalar_trait_owner(source, application)? else {
            return Ok(Eligibility::Unsupported(Unsupported::SourceOwner));
        };
        let owners = [
            actual.ok_or_else(|| self.error(source, "source owner disappeared"))?,
            trait_owner,
        ];
        let members =
            self.scalar_source_members([owner, trait_owner.self_definition], source, pending)?;
        pending
            .charge(&mut self.facts, members.len().saturating_add(4))
            .map_err(|e| self.error(source, e))?;
        let Some(member_profile) = self.scalar_member_profile(source, owners, &members)? else {
            return Ok(Eligibility::Unsupported(Unsupported::Members));
        };
        let requirements = self.scalar_requirements(owner, source, predicate, pending)?;
        let Some(requirements) = requirements else {
            return Ok(Eligibility::Unsupported(Unsupported::Requirements));
        };
        let arena = pending
            .arena_for(&mut self.facts)
            .map_err(|e| self.error(source, e))?;
        let environment = self.facts.generics.environments.get(&owner).cloned();
        let assumptions =
            self.early_source_assumptions(source, predicate.ty, environment.as_deref())?;
        Ok(Eligibility::Ready(Plan {
            arena,
            member_profile,
            owner_kind,
            source,
            predicate,
            requirements,
            members,
            environment,
            assumptions,
        }))
    }

    fn early_source_assumptions(
        &self,
        source: NodeKey,
        subject: crate::types::TypeId,
        environment: Option<&Environment>,
    ) -> Result<Option<Arc<Assumptions>>, Error> {
        let producer = self.report.lint_producer().ok_or_else(|| {
            self.error(
                source,
                "generic conformance requires its actual source payer",
            )
        })?;
        let pay = |work, bytes| {
            producer
                .diagnostic_payload(source, work, bytes)
                .map_err(|e| e.message)
        };
        pay(1, size_of::<Option<Arc<Assumptions>>>()).map_err(|e| self.error(source, e))?;
        if !matches!(self.facts.types.kind(subject), Some(Kind::Parameter(_))) {
            return Ok(None);
        }
        pay(1, size_of::<Assumptions>() + 2 * size_of::<usize>())
            .map_err(|e| self.error(source, e))?;
        let assumptions =
            Assumptions::from_environment_charged(&self.facts.types, environment, &pay)
                .map_err(|e| self.error(source, e))?;
        Ok(Some(Arc::new(assumptions)))
    }

    fn early_source_subject(
        &mut self,
        owner: DefId,
        subject: crate::types::TypeId,
        pending: &mut Pending,
    ) -> Result<bool, Error> {
        let count = self
            .facts
            .generics
            .parameters
            .get(&owner)
            .map_or(0, Vec::len);
        pending
            .charge(&mut self.facts, count.saturating_add(1))
            .map_err(Error::unlocated)?;
        match self.facts.types.kind(subject) {
            Some(Kind::Primitive(_)) => Ok(self
                .facts
                .generics
                .parameters
                .get(&owner)
                .is_none_or(Vec::is_empty)
                && self
                    .facts
                    .generics
                    .declared_predicates_in_scope(owner)
                    .next()
                    .is_none()),
            Some(Kind::Parameter(id)) => {
                if self
                    .facts
                    .types
                    .parameter(id)
                    .is_none_or(|p| p.owner.0 != owner.0)
                {
                    return Ok(false);
                }
                let Some(environment) = self.facts.generics.environments.get(&owner) else {
                    return Ok(false);
                };
                Ok(self.facts.generics.parameters.get(&owner).is_some_and(|parameters| {
                    parameters.iter().any(|p| p.argument == crate::types::arguments::Argument::Type(subject))
                    && parameters.iter().enumerate().all(|(index, parameter)| {
                        matches!(parameter.argument, crate::types::arguments::Argument::Type(ty)
                            if matches!(self.facts.types.kind(ty), Some(Kind::Parameter(id))
                                if self.facts.types.parameter(id).is_some_and(|p| p.owner.0 == owner.0 && p.index as usize == index)
                                    && environment.abilities(id).is_some()))
                    })
                }))
            }
            _ => Ok(false),
        }
    }

    fn scalar_member_profile(
        &self,
        source: NodeKey,
        owners: [Owner; 2],
        members: &[DefId],
    ) -> Result<Option<MemberProfile>, Error> {
        let mut constants = false;
        let mut types = false;
        for member in members {
            match self.report.resolution.definitions[member.0 as usize].kind {
                DefKind::Function => {}
                DefKind::Const => constants = true,
                DefKind::TypeAlias => types = true,
                _ => return Ok(None),
            }
        }
        let mut counts = [0; 2];
        for (slot, actual) in counts.iter_mut().zip(owners) {
            let id = self
                .facts
                .associated
                .owner(actual.self_definition)
                .ok_or_else(|| self.error(source, "source member owner disappeared"))?;
            *slot = self
                .facts
                .associated
                .owner_members(id)
                .ok_or_else(|| self.error(source, "source member bucket disappeared"))?
                .len();
        }
        Ok(Some(if types {
            MemberProfile::Complete(CompleteMembers { owners, counts })
        } else if constants {
            MemberProfile::MethodsAndConstants(NoAssociatedTypes { owners, counts })
        } else {
            MemberProfile::FunctionsOnly(NoAssociatedMembers { owners, counts })
        }))
    }

    fn scalar_trait_owner(
        &self,
        source: NodeKey,
        application: crate::types::traits::ApplicationId,
    ) -> Result<Option<Owner>, Error> {
        let item = self
            .facts
            .types
            .trait_application(application)
            .ok_or_else(|| self.error(source, "missing scalar trait application"))?
            .item;
        Ok(self.facts.associated.owner(DefId(item.0))
            .and_then(|id| self.facts.associated.get(id))
            .filter(|owner| owner.source.is_some() && matches!(owner.kind, OwnerKind::Trait { definition, .. } if definition.0 == item.0))
            .copied())
    }

    fn scalar_source_members(
        &mut self,
        owners: [DefId; 2],
        source: NodeKey,
        pending: &mut Pending,
    ) -> Result<Vec<DefId>, Error> {
        let mut result = Vec::new();
        for owner in owners {
            pending
                .charge(&mut self.facts, 1)
                .map_err(|e| self.error(source, e))?;
            if !pending.members.contains_key(&owner) {
                let id = self.facts.associated.owner(owner).ok_or_else(|| {
                    self.error(source, "scalar member owner is not actual source")
                })?;
                let count = self
                    .facts
                    .associated
                    .owner_members(id)
                    .ok_or_else(|| self.error(source, "scalar member inventory is missing"))?
                    .len();
                pending
                    .charge(&mut self.facts, count)
                    .map_err(|e| self.error(source, e))?;
                let values = self
                    .facts
                    .associated
                    .owner_members(id)
                    .ok_or_else(|| self.error(source, "scalar member inventory disappeared"))?
                    .to_vec();
                pending
                    .charge(&mut self.facts, count.saturating_add(1))
                    .map_err(|e| self.error(source, e))?;
                pending
                    .methods
                    .prepare_source_owner(self.report, &self.facts, owner)
                    .map_err(|e| self.error(source, e))?;
                pending.members.insert(owner, values);
            }
            let count = pending
                .members
                .get(&owner)
                .ok_or_else(|| self.error(source, "scalar source member index is missing"))?
                .len();
            pending
                .charge(&mut self.facts, count)
                .map_err(|e| self.error(source, e))?;
            result
                .try_reserve_exact(count)
                .map_err(|_| self.error(source, "scalar member allocation failed"))?;
            result.extend_from_slice(
                pending
                    .members
                    .get(&owner)
                    .ok_or_else(|| self.error(source, "scalar source member index disappeared"))?,
            );
        }
        Ok(result)
    }

    fn scalar_requirements(
        &mut self,
        owner: DefId,
        source: NodeKey,
        predicate: Predicate,
        pending: &mut Pending,
    ) -> Result<Option<Vec<Predicate>>, Error> {
        let requirements = self
            .facts
            .generics
            .traits
            .as_mut()
            .ok_or_else(|| Error::unlocated("missing scalar trait requirements"))?
            .requirements(&mut self.facts.types, predicate)
            .map_err(|e| self.error(source, e))?;
        for &ty in &requirements.types {
            pending
                .charge(&mut self.facts, 1)
                .map_err(|e| self.error(source, e))?;
            if !matches!(self.facts.types.kind(ty), Some(Kind::Primitive(_)))
                && !matches!(self.facts.types.kind(ty), Some(Kind::Parameter(id))
                    if self.facts.types.parameter(id).is_some_and(|p| p.owner.0 == owner.0))
            {
                return Ok(None);
            }
        }
        let mut direct = Vec::new();
        for &clause in requirements.required.iter().chain(&requirements.implied) {
            pending
                .charge(&mut self.facts, 1)
                .map_err(|e| self.error(source, e))?;
            let Clause::Predicate(value) = clause else {
                return Ok(None);
            };
            if matches!(
                value.bound,
                Bound::Outlives(_)
                    | Bound::ProjectionEqual(_)
                    | Bound::ConstTrait(_)
                    | Bound::MaybeConstTrait(_)
            ) || value
                .has_free_regions(&self.facts.types)
                .map_err(|e| self.error(source, e))?
            {
                return Ok(None);
            }
            direct.push(value);
        }
        Ok(Some(direct))
    }
}
