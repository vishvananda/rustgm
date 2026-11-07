//! Resolved names reserve candidate owners before their environments are elaborated.
use super::{Checker, DefId, DefKind, Error, NodeKey};
use crate::support::hash::{HashMap, HashSet};
use crate::{
    resolve::{
        model::{ItemContext, Namespace, Origin},
        names::NameId,
    },
    sema::{
        associated::OwnerKind,
        methods::buckets::{Bucket, Target},
    },
    types::{ItemId, Store, subject_head::Head},
};
#[cfg(test)]
#[path = "../../../tests/invariants/inherent_headers.rs"]
mod tests;

#[derive(Default)]
pub(super) struct Pending {
    owners: HashMap<DefId, Members>,
    buckets: HashMap<(Option<Target>, NameId), HashSet<DefId>>,
}

struct Members {
    head: Option<Target>,
    all_marked: bool,
    declarations: Vec<(NameId, DefId)>,
}

impl Pending {
    fn alias_growth(&self, owner: DefId, name: NameId) -> crate::support::retained::Growth {
        use crate::support::retained::Growth;
        let Some(members) = self.owners.get(&owner) else {
            return Growth::default();
        };
        let mut growth = Growth::vector::<(NameId, DefId)>(
            members.declarations.len(),
            members.declarations.capacity(),
            1,
        );
        let key = (members.head, name);
        growth.include(Growth::map::<(Option<Target>, NameId), HashSet<DefId>>(
            self.buckets.len(),
            self.buckets.capacity(),
            usize::from(!self.buckets.contains_key(&key)),
        ));
        let (length, capacity) = self
            .buckets
            .get(&key)
            .map_or((0, 0), |set| (set.len(), set.capacity()));
        growth.include(Growth::map::<DefId, ()>(length, capacity, 1));
        growth
    }
    fn insert(&mut self, owner: DefId, name: NameId, member: DefId, marked: bool) {
        if let Some(members) = self.owners.get_mut(&owner) {
            members.all_marked &= marked;
            members.declarations.push((name, member));
            self.buckets
                .entry((members.head, name))
                .or_default()
                .insert(owner);
        }
    }

    pub(super) fn permitted(&self, owner: DefId, requested_internal: bool) -> bool {
        requested_internal
            && self
                .owners
                .get(&owner)
                .is_some_and(|members| members.all_marked)
    }

    #[cfg(test)]
    fn contains(&self, nominal: ItemId, name: NameId) -> bool {
        self.contains_target(Target::Shape(Head::Nominal(nominal)), name)
    }

    pub(super) fn contains_head(&self, store: &Store, head: Head, name: NameId) -> bool {
        Target::of(store, head).is_some_and(|target| self.contains_target(target, name))
    }
    fn contains_target(&self, target: Target, name: NameId) -> bool {
        self.buckets.contains_key(&(Some(target), name)) || self.buckets.contains_key(&(None, name))
    }

    pub(super) fn dependencies(&self, nominal: ItemId, name: NameId) -> Vec<DefId> {
        self.target_dependencies(Target::Shape(Head::Nominal(nominal)), name)
    }

    pub(super) fn head_dependencies(&self, store: &Store, head: Head, name: NameId) -> Vec<DefId> {
        Target::of(store, head)
            .map_or_else(Vec::new, |target| self.target_dependencies(target, name))
    }
    fn target_dependencies(&self, target: Target, name: NameId) -> Vec<DefId> {
        let mut owners: Vec<_> = [Some(target), None]
            .into_iter()
            .flat_map(|head| {
                self.buckets
                    .get(&(head, name))
                    .into_iter()
                    .flatten()
                    .copied()
            })
            .collect();
        owners.sort_unstable_by_key(|id| id.0);
        owners
    }

    fn remove(&mut self, owner: DefId) -> Option<Members> {
        let members = self.owners.remove(&owner)?;
        for &(name, _) in &members.declarations {
            let key = (members.head, name);
            if let Some(bucket) = self.buckets.get_mut(&key) {
                bucket.remove(&owner);
                if bucket.is_empty() {
                    self.buckets.remove(&key);
                }
            }
        }
        Some(members)
    }
}

impl Checker<'_> {
    /// Drain the same exact bucket used by ordinary qualified-path lookup.
    /// Publication remains owned by each completed implementation header.
    pub(super) fn prepare_inherent_bucket(&mut self, nominal: ItemId, name: NameId) {
        for owner in self
            .inherent_headers
            .dependencies(nominal, name)
            .into_iter()
            .rev()
        {
            self.work.push(super::Work::ImplHeader(
                owner,
                super::impl_headers::Phase::Begin,
            ));
        }
    }

    pub(super) fn reserve_inherent_headers(&mut self) -> Result<(), Error> {
        let mut heads = super::heads::Heads::new(self.report, &self.facts);
        let mut pending = Pending::default();
        let mut trait_names = HashSet::default();
        let marked = self.incoherent_members()?;
        for index in 0..self.facts.associated.owners().len() {
            let Some(owner) = self
                .facts
                .associated
                .source_at(index)
                .map_err(Error::unlocated)?
            else {
                continue;
            };
            if let OwnerKind::Impl {
                target,
                trait_type: None,
                ..
            } = owner.kind
            {
                pending.owners.insert(
                    owner.self_definition,
                    Members {
                        head: heads
                            .shape(target)
                            .unwrap_or_default()
                            .and_then(|head| Target::of(&self.facts.types, head)),
                        all_marked: true,
                        declarations: Vec::new(),
                    },
                );
            }
        }
        for index in 0..self.facts.associated.members().len() {
            let member = self.facts.associated.members()[index];
            let owner = self.facts.associated.get(member.owner).ok_or_else(|| {
                Error::at(crate::resolve::unit::FileId(0), 0, "invalid method owner")
            })?;
            let definition = &self.report.resolution.definitions[member.definition.0 as usize];
            if !matches!(
                definition.kind,
                DefKind::Function | DefKind::Const | DefKind::TypeAlias
            ) {
                continue;
            }
            match owner.kind {
                OwnerKind::Trait { .. } => {
                    if definition.kind != DefKind::TypeAlias {
                        trait_names.insert(definition.name);
                    }
                }
                OwnerKind::Impl {
                    target,
                    trait_type: None,
                    ..
                } => {
                    // A raw head is only an exclusion hint, never a ready schema.
                    let _ = target;
                    let name = definition.name;
                    let owner = owner.self_definition;
                    if definition.kind == DefKind::TypeAlias {
                        let growth = pending.alias_growth(owner, name);
                        self.charge_inherent_type(growth.work.saturating_add(1), growth.bytes)?;
                    }
                    pending.insert(
                        owner,
                        name,
                        member.definition,
                        marked.contains(&member.definition),
                    );
                }
                OwnerKind::Impl { .. } => {}
                OwnerKind::ImportedImpl(_) => {
                    self.facts.body_work.canonical_steps =
                        self.facts.body_work.canonical_steps.saturating_add(8);
                    self.facts
                        .associated
                        .require_imported(*owner)
                        .map_err(Error::unlocated)?;
                    if let Some(crate::sema::crates::ImportedOwners::Ordinary(graph)) =
                        self.facts.associated.imported_owner()
                    {
                        // A real trait impl is not an inherent lookup candidate.
                        // Its complete member conformance remains pending.
                        if graph
                            .feature_trait_implementation(&self.facts.types, *owner)
                            .map_err(Error::unlocated)?
                            .is_some()
                        {
                            continue;
                        }
                    }
                    return Err(Error::unlocated(
                        "imported inherent/member lookup requires its wider profile",
                    ));
                }
                OwnerKind::ImportedInherent(_) => {
                    self.publish_imported_inherent(*owner, member.definition, definition.name)?;
                }
            }
        }
        self.inherent_headers = pending;
        self.facts.methods.trait_names = trait_names;
        Ok(())
    }

    /// An imported inherent impl arrives with its completed, re-authenticated
    /// Self schema, so each method joins the ordinary nominal bucket at once.
    /// Lookup then uses exactly the local candidate and receiver rules.
    fn publish_imported_inherent(
        &mut self,
        owner: crate::sema::associated::Owner,
        member: DefId,
        name: NameId,
    ) -> Result<(), Error> {
        self.facts.body_work.canonical_steps =
            self.facts.body_work.canonical_steps.saturating_add(1);
        self.facts
            .associated
            .require_imported(owner)
            .map_err(Error::unlocated)?;
        if self.report.resolution.definitions[member.0 as usize].kind != DefKind::Function {
            return Err(Error::unlocated(
                "imported inherent associated constants and types require a wider profile",
            ));
        }
        let schema = self
            .facts
            .associated
            .schema(owner.self_definition)
            .ok_or_else(|| Error::unlocated("imported inherent owner lost its Self schema"))?;
        let head = self
            .facts
            .types
            .kind(schema.self_type)
            .map(Head::canonical)
            .ok_or_else(|| Error::unlocated("imported inherent owner lost its Self type"))?;
        let Some(bucket @ Bucket::Nominal(..)) = Bucket::declared(&self.facts.types, head, name)
        else {
            return Err(Error::unlocated(
                "imported inherent impl Self is not a nominal of its own provider",
            ));
        };
        // The member's storage was prepaid when its owner was installed; the
        // bucket growth is canonical work, as for the trait-impl owner above.
        let growth = self
            .facts
            .methods
            .inherent_growth(bucket, ItemContext::ROOT);
        self.facts.body_work.canonical_steps = self
            .facts
            .body_work
            .canonical_steps
            .saturating_add(growth.work);
        self.facts
            .methods
            .publish_inherent(bucket, ItemContext::ROOT, member);
        Ok(())
    }

    pub(super) fn publish_inherent_members(
        &mut self,
        owner: DefId,
        source: NodeKey,
    ) -> Result<(), Error> {
        let Some(members) = self.inherent_headers.remove(owner) else {
            return Ok(());
        };
        let schema = self
            .facts
            .associated
            .schema(owner)
            .ok_or_else(|| self.error(source, "missing inherent owner schema"))?;
        let actual = self
            .facts
            .types
            .kind(schema.self_type)
            .ok_or_else(|| self.error(source, "missing inherent target"))?;
        let head = Head::canonical(actual);
        let target = Target::of(&self.facts.types, head)
            .ok_or_else(|| self.error(source, "inherent owner has no actual principal or shape"))?;
        if members.head.is_some_and(|reserved| reserved != target) {
            return Err(self.error(source, "inherent owner disagrees with reserved target head"));
        }
        for (name, member) in members.declarations {
            let context = self.report.definition_item_context(member)?;
            let bucket = Bucket::declared(&self.facts.types, head, name)
                .ok_or_else(|| Error::unlocated("invalid inherent target head"))?;
            if matches!(bucket, Bucket::Structural(Target::Shape(_), _))
                && !self.structural_representative(owner, name, member, context)?
            {
                continue;
            }
            if self.report.resolution.definitions[member.0 as usize].kind == DefKind::TypeAlias {
                let growth = self.facts.methods.inherent_growth(bucket, context);
                self.charge_inherent_type(growth.work.saturating_add(1), growth.bytes)?;
            }
            self.facts.methods.publish_inherent(bucket, context, member);
        }
        Ok(())
    }

    /// The resolver retains every declaration, but an impl's exact name binding
    /// supplies one lookup representative. Other impl owners remain candidates.
    fn structural_representative(
        &self,
        owner: DefId,
        name: NameId,
        member: DefId,
        context: ItemContext,
    ) -> Result<bool, Error> {
        let error = || Error::unlocated("invalid structural member binding identity");
        let associated = &self.facts.associated;
        let actual_owner = associated.owner(owner).ok_or_else(error)?;
        let implementation = associated.get(actual_owner).ok_or_else(error)?;
        if implementation.self_definition != owner
            || !matches!(
                implementation.kind,
                OwnerKind::Impl {
                    trait_type: None,
                    ..
                }
            )
            || associated.owner(member) != Some(actual_owner)
        {
            return Err(error());
        }
        let definitions = &self.report.resolution.definitions;
        let declaration = definitions.get(member.0 as usize).ok_or_else(error)?;
        let namespace = if declaration.kind == DefKind::TypeAlias {
            Namespace::Type
        } else {
            Namespace::Value
        };
        if declaration.namespace != namespace
            || declaration.name != name
            || !matches!(
                declaration.kind,
                DefKind::Function | DefKind::Const | DefKind::TypeAlias
            )
            || definitions
                .get(owner.0 as usize)
                .and_then(|owner| owner.members)
                != Some(declaration.scope)
        {
            return Err(error());
        }
        let binding = self
            .report
            .resolution
            .bindings
            .get_key((declaration.scope, name, namespace, context))
            .ok_or_else(error)?;
        let target = definitions
            .get(binding.target.0 as usize)
            .ok_or_else(error)?;
        if binding.origin != Origin::Direct
            || binding.ambiguous
            || target.scope != declaration.scope
            || target.namespace != Namespace::Value
            || target.name != name
            || !matches!(
                target.kind,
                DefKind::Function | DefKind::Const | DefKind::TypeAlias
            )
            || associated.owner(binding.target) != Some(actual_owner)
            || self.report.definition_item_context(binding.target)? != context
        {
            return Err(error());
        }
        Ok(binding.target == member)
    }

    pub(super) fn index_inherent_methods(&mut self) -> Result<(), Error> {
        // An analysis-only census keeps the owners of its isolated failed
        // headers pending, so a later lookup of their names waits on them and
        // is recorded as blocked. Ordinary checking isolates nothing.
        let census = self.census.as_deref();
        let isolated = |owner: &DefId| census.is_some_and(|ledger| ledger.isolates_header(*owner));
        if !self.inherent_headers.owners.keys().all(isolated)
            || !self
                .inherent_headers
                .buckets
                .values()
                .flatten()
                .all(isolated)
        {
            return Err(Error::at(
                crate::resolve::unit::FileId(0),
                0,
                "unfinished inherent member headers",
            ));
        }
        // Demand order may differ from source order. Freeze ordering only after
        // every owner has published, independently of hash iteration order.
        self.facts.methods.freeze_inherent_names();
        if self.inherent_headers.owners.is_empty() {
            self.inherent_headers = Pending::default();
        }
        Ok(())
    }
}
