//! Immutable promises refine defining demand; they grant no reads or const execution.
use super::{Declarations, DefId, Environment, TypeId, Work};
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
        }
    }
}
pub(super) struct Claims {
    values: HashMap<TypeId, Claim>,
    pub(super) unsupported: bool,
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
        };
        let mut clauses = environment.trait_clauses();
        loop {
            work.charge(1)?;
            let Some(clause) = clauses.next() else {
                break;
            };
            let Some((ty, claim)) = result.classify(facts, clause, role, copy_role, work)? else {
                continue;
            };
            result.insert(ty, claim, work)?;
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
        if self.values.get(&ty) == Some(&Claim::ProjectedDestruct) {
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
    fn insert(&mut self, ty: TypeId, claim: Claim, work: &mut Work) -> Result<(), &'static str> {
        work.charge(1)?;
        if let Some(old) = self.values.get_mut(&ty) {
            if claim.priority() > old.priority() {
                *old = claim;
            }
            return Ok(());
        }
        if crate::support::retained::new_map::<TypeId, Claim>(self.values.len().saturating_add(1))
            > 8 * 1024 * 1024
        {
            return Err("defining cleanup claims exceed storage budget");
        }
        if self.values.len() == self.values.capacity() {
            work.charge(self.values.len())?;
        }
        self.values
            .try_reserve(1)
            .map_err(|_| "defining cleanup claim allocation failed")?;
        self.values.insert(ty, claim);
        if self.retained_bytes() > 8 * 1024 * 1024 {
            return Err("defining cleanup claims exceed storage budget");
        }
        Ok(())
    }
    pub(super) fn get(&self, ty: TypeId) -> Option<Claim> {
        self.values.get(&ty).copied()
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
        self.values
            .iter()
            .filter_map(|(&ty, &claim)| (claim == Claim::ProjectedDestruct).then_some(ty))
    }
}
