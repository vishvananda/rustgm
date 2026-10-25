//! Resolved names reserve candidate owners before their environments are elaborated.
use super::{Checker, DefId, DefKind, Error, NodeKey};
use crate::support::hash::{HashMap, HashSet};
use crate::{
    resolve::{
        model::{ItemContext, Namespace, Origin},
        names::NameId,
    },
    sema::{associated::OwnerKind, methods::buckets::Bucket},
    types::{ItemId, subject_head::Head},
};
#[cfg(test)]
#[path = "../../../tests/invariants/inherent_headers.rs"]
mod tests;

#[derive(Default)]
pub(super) struct Pending {
    owners: HashMap<DefId, Members>,
    buckets: HashMap<(Option<Head>, NameId), HashSet<DefId>>,
}

struct Members {
    head: Option<Head>,
    all_marked: bool,
    declarations: Vec<(NameId, DefId)>,
}

impl Pending {
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
        self.contains_head(Head::Nominal(nominal), name)
    }

    pub(super) fn contains_head(&self, head: Head, name: NameId) -> bool {
        self.buckets.contains_key(&(Some(head), name)) || self.buckets.contains_key(&(None, name))
    }

    pub(super) fn dependencies(&self, nominal: ItemId, name: NameId) -> Vec<DefId> {
        self.head_dependencies(Head::Nominal(nominal), name)
    }

    pub(super) fn head_dependencies(&self, head: Head, name: NameId) -> Vec<DefId> {
        let mut owners: Vec<_> = [Some(head), None]
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
                        head: heads.shape(target).unwrap_or_default(),
                        all_marked: true,
                        declarations: Vec::new(),
                    },
                );
            }
        }
        for member in self.facts.associated.members() {
            let owner = self.facts.associated.get(member.owner).ok_or_else(|| {
                Error::at(crate::resolve::unit::FileId(0), 0, "invalid method owner")
            })?;
            let definition = &self.report.resolution.definitions[member.definition.0 as usize];
            if !matches!(definition.kind, DefKind::Function | DefKind::Const) {
                continue;
            }
            match owner.kind {
                OwnerKind::Trait { .. } => {
                    trait_names.insert(definition.name);
                }
                OwnerKind::Impl {
                    target,
                    trait_type: None,
                    ..
                } => {
                    // A raw head is only an exclusion hint, never a ready schema.
                    let _ = target;
                    pending.insert(
                        owner.self_definition,
                        definition.name,
                        member.definition,
                        marked.contains(&member.definition),
                    );
                }
                OwnerKind::Impl { .. } => {}
                OwnerKind::ImportedImpl(_) => {
                    self.facts
                        .associated
                        .require_imported(*owner)
                        .map_err(Error::unlocated)?;
                    return Err(Error::unlocated(
                        "imported inherent/member lookup requires its wider profile",
                    ));
                }
            }
        }
        self.inherent_headers = pending;
        self.facts.methods.trait_names = trait_names;
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
        if members.head.is_some_and(|reserved| reserved != head) {
            return Err(self.error(source, "inherent owner disagrees with reserved target head"));
        }
        for (name, member) in members.declarations {
            let context = self.report.definition_item_context(member)?;
            let bucket = Bucket::declared(head, name)
                .ok_or_else(|| Error::unlocated("invalid inherent target head"))?;
            if matches!(bucket, Bucket::Structural(..))
                && !self.structural_representative(owner, name, member, context)?
            {
                continue;
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
        if declaration.namespace != Namespace::Value
            || declaration.name != name
            || !matches!(declaration.kind, DefKind::Function | DefKind::Const)
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
            .get_key((declaration.scope, name, Namespace::Value, context))
            .ok_or_else(error)?;
        let target = definitions
            .get(binding.target.0 as usize)
            .ok_or_else(error)?;
        if binding.origin != Origin::Direct
            || binding.ambiguous
            || target.scope != declaration.scope
            || target.namespace != Namespace::Value
            || target.name != name
            || !matches!(target.kind, DefKind::Function | DefKind::Const)
            || associated.owner(binding.target) != Some(actual_owner)
            || self.report.definition_item_context(binding.target)? != context
        {
            return Err(error());
        }
        Ok(binding.target == member)
    }

    pub(super) fn index_inherent_methods(&mut self) -> Result<(), Error> {
        if !self.inherent_headers.owners.is_empty() || !self.inherent_headers.buckets.is_empty() {
            return Err(Error::at(
                crate::resolve::unit::FileId(0),
                0,
                "unfinished inherent member headers",
            ));
        }
        // Demand order may differ from source order. Freeze ordering only after
        // every owner has published, independently of hash iteration order.
        self.facts.methods.freeze_inherent_names();
        self.inherent_headers = Pending::default();
        Ok(())
    }
}
