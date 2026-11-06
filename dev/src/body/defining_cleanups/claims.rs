//! Immutable promises refine defining demand; they grant no reads or const execution.
use super::{Declarations, DefId, Environment, Purpose, TypeId, Work};
use crate::{
    support::{bindings::LangItem, hash::HashMap},
    types::{
        Kind,
        parameters::{clauses::Clause, predicates::Bound},
    },
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Role {
    Absent,
    Pending(DefId),
    Validated(DefId),
}
impl Role {
    pub(super) fn current(
        facts: &Declarations,
        role: LangItem,
        work: &mut Work,
    ) -> Result<Self, &'static str> {
        work.charge(4)?;
        let Some(catalog) = facts.source_bindings.as_deref() else {
            return Ok(Self::Absent);
        };
        let Some(declaration) = catalog.language(role) else {
            return Ok(Self::Absent);
        };
        Ok(match catalog.validated_language(role) {
            Ok(Some(actual)) if actual.definition == declaration.definition => {
                Self::Validated(actual.definition)
            }
            _ => Self::Pending(declaration.definition),
        })
    }
    fn definition(self) -> Option<DefId> {
        match self {
            Self::Absent => None,
            Self::Pending(id) | Self::Validated(id) => Some(id),
        }
    }
    pub(super) fn require(self) -> Result<DefId, &'static str> {
        match self {
            Self::Validated(definition) => Ok(definition),
            Self::Absent => Err("defining cleanup requires its actual language declaration"),
            Self::Pending(_definition) => {
                Err("defining cleanup language declaration is not validated")
            }
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Claim {
    NoDrop,
    Selected(crate::support::constness::Constness),
    ConditionalDestruct,
    ProjectedDestruct,
    PendingCopy,
}
impl Claim {
    fn priority(self) -> u8 {
        match self {
            Self::PendingCopy => 0,
            Self::ConditionalDestruct | Self::ProjectedDestruct => 1,
            Self::NoDrop => 2,
            Self::Selected(_) => 3,
        }
    }
}
pub(super) struct Claims {
    values: HashMap<(TypeId, Purpose), Claim>,
    pub(super) unsupported: bool,
    selected: Option<crate::types::arena::Identity>,
}
impl Claims {
    pub(super) fn new(
        facts: &Declarations,
        environment: &Environment,
        role: Role,
        copy_role: Role,
        work: &mut Work,
    ) -> Result<Self, &'static str> {
        if Role::current(facts, LangItem::Destruct, work)? != role
            || Role::current(facts, LangItem::Copy, work)? != copy_role
        {
            return Err("defining cleanup claims changed their current language declarations");
        }
        let mut result = Self {
            values: HashMap::default(),
            unsupported: false,
            selected: None,
        };
        let mut clauses = environment.trait_clauses()?;
        loop {
            work.charge(1)?;
            let Some(clause) = clauses.next() else {
                break;
            };
            let Some((ty, claim)) = result.classify(facts, clause, role, copy_role, work)? else {
                continue;
            };
            result.insert(ty, Purpose::Body, claim, work)?;
        }
        Ok(result)
    }
    fn classify(
        &mut self,
        facts: &Declarations,
        clause: Clause,
        role: Role,
        copy_role: Role,
        work: &mut Work,
    ) -> Result<Option<(TypeId, Claim)>, &'static str> {
        let Clause::Predicate(predicate) = clause else {
            self.unsupported = true;
            return Ok(None);
        };
        let Some((application, effect)) = predicate.bound.trait_parts() else {
            return Ok(None);
        };
        work.charge(3)?;
        let application = facts
            .types
            .trait_application(application)
            .ok_or("invalid defining cleanup application")?;
        let copy = copy_role
            .definition()
            .is_some_and(|id| application.item.0 == id.0)
            && effect == crate::support::constness::Constness::Ordinary;
        let destruct = role
            .definition()
            .is_some_and(|id| application.item.0 == id.0)
            && matches!(
                predicate.bound,
                Bound::ConstTrait(_) | Bound::MaybeConstTrait(_)
            );
        if !copy && !destruct {
            return Ok(None);
        }
        let subject = facts
            .types
            .kind(predicate.ty)
            .ok_or("invalid defining cleanup subject")?;
        let projected = destruct && matches!(subject, Kind::Projection(_));
        if !(matches!(subject, Kind::Parameter(_)) || projected)
            || facts
                .types
                .arguments(application.arguments)
                .is_none_or(|args| args.len() != 0)
        {
            self.unsupported = true;
            return Ok(None);
        }
        let claim = if copy {
            if matches!(copy_role, Role::Validated(_)) {
                Claim::NoDrop
            } else {
                Claim::PendingCopy
            }
        } else if matches!(role, Role::Validated(_)) {
            if projected {
                self.require_projected_header(facts, predicate.ty, work)?;
                Claim::ProjectedDestruct
            } else {
                Claim::ConditionalDestruct
            }
        } else {
            return Ok(None);
        };
        Ok(Some((predicate.ty, claim)))
    }
    fn require_projected_header(
        &self,
        facts: &Declarations,
        ty: TypeId,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        work.charge(1)?;
        if self.values.get(&(ty, Purpose::Body)) == Some(&Claim::ProjectedDestruct) {
            return Ok(());
        }
        work.charge(12)?;
        let Some(Kind::Projection(application)) = facts.types.kind(ty) else {
            return Err("defining cleanup changed its projected subject");
        };
        let schema = facts
            .types
            .projection_schema(application.item)
            .ok_or("defining projection schema is pending")?;
        let trait_ = facts
            .types
            .trait_application(application.trait_application)
            .ok_or("defining projection has invalid trait inputs")?;
        if trait_.item != schema.trait_item
            || facts.associated.trait_owner(DefId(application.item.0))
                != Some(DefId(schema.trait_item.0))
            || facts.types.arguments(application.arguments).is_none()
            || facts.types.arguments(trait_.arguments).is_none()
        {
            return Err("defining projection changed its actual member or complete application");
        }
        facts
            .associated
            .require_projection_header(DefId(application.item.0))
            .map(|_| ())
    }
    pub(super) fn insert(
        &mut self,
        ty: TypeId,
        purpose: Purpose,
        claim: Claim,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        work.charge(1)?;
        if purpose != Purpose::Body && matches!(claim, Claim::NoDrop | Claim::PendingCopy) {
            return Err("Copy cleanup eligibility belongs only to the original body purpose");
        }
        if purpose == Purpose::OwnDrop && !matches!(claim, Claim::Selected(_)) {
            return Err("own Drop eligibility requires its actual selected application");
        }
        if let Some(old) = self.values.get_mut(&(ty, purpose)) {
            if claim.priority() > old.priority() {
                *old = claim;
            }
            return Ok(());
        }
        if crate::support::retained::new_map::<(TypeId, Purpose), Claim>(
            self.values.len().saturating_add(1),
        ) > 8 * 1024 * 1024
        {
            return Err("defining cleanup claims exceed storage budget");
        }
        if self.values.len() == self.values.capacity() {
            work.charge(self.values.len())?;
        }
        self.values
            .try_reserve(1)
            .map_err(|_| "defining cleanup claim allocation failed")?;
        self.values.insert((ty, purpose), claim);
        if self.retained_bytes() > 8 * 1024 * 1024 {
            return Err("defining cleanup claims exceed storage budget");
        }
        Ok(())
    }
    /// Retain a genuine scoped whole-type clause for this actual field demand.
    /// Body Copy/NoDrop and selected owner-only promises cannot be promoted.
    pub(super) fn retain_declared(
        &mut self,
        ty: TypeId,
        purpose: Purpose,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        if purpose == Purpose::Whole
            && self.get(ty, purpose).is_none()
            && let Some(claim @ (Claim::ConditionalDestruct | Claim::ProjectedDestruct)) =
                self.get(ty, Purpose::Body)
        {
            self.insert(ty, purpose, claim, work)?;
        }
        Ok(())
    }
    pub(super) fn select(
        &mut self,
        types: &mut crate::types::Store,
        ty: TypeId,
        purpose: Purpose,
        effect: crate::support::constness::Constness,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        self.insert(ty, purpose, Claim::Selected(effect), work)?;
        self.selected = Some(types.arena_identity());
        Ok(())
    }
    pub(super) fn require_selected(
        &self,
        types: &crate::types::Store,
        actual: crate::support::constness::Constness,
        expected: crate::support::constness::Constness,
    ) -> Result<(), &'static str> {
        if actual != expected
            || self
                .selected
                .as_ref()
                .is_none_or(|arena| !types.has_identity(arena))
        {
            return Err("selected const cleanup changed its actual effect or type arena");
        }
        Ok(())
    }
    pub(super) fn retain_selected(
        &mut self,
        types: &mut crate::types::Store,
        prior: &Self,
        effect: crate::support::constness::Constness,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        work.charge(prior.values.len())?;
        for (&(ty, purpose), &claim) in &prior.values {
            if let Claim::Selected(actual) = claim {
                prior.require_selected(types, actual, effect)?;
                self.select(types, ty, purpose, effect, work)?;
            }
        }
        Ok(())
    }
    pub(super) fn get(&self, ty: TypeId, purpose: Purpose) -> Option<Claim> {
        self.values.get(&(ty, purpose)).copied()
    }
    pub(super) fn retained_bytes(&self) -> usize {
        crate::support::retained::map(&self.values)
    }
    #[cfg(test)]
    pub(super) fn len(&self) -> usize {
        self.values.len()
    }
    #[cfg(test)]
    pub(super) fn projected_subjects(&self) -> impl Iterator<Item = TypeId> + '_ {
        self.values.iter().filter_map(|(&(ty, purpose), &claim)| {
            (purpose == Purpose::Body && claim == Claim::ProjectedDestruct).then_some(ty)
        })
    }
}
