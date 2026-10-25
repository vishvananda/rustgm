//! Resolved names reserve candidate owners before their environments are elaborated.
use super::{Checker, DefId, DefKind, Error, NodeKey};
use crate::support::hash::{HashMap, HashSet};
use crate::{
    resolve::names::NameId,
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

    pub(super) fn permitted(&self, owner: DefId, empty_request: bool) -> bool {
        self.owners.get(&owner).is_some_and(|members| {
            if members.declarations.is_empty() {
                empty_request
            } else {
                members.all_marked
            }
        })
    }

    pub(super) fn contains(&self, nominal: ItemId, name: NameId) -> bool {
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
            self.facts.methods.publish_inherent(
                Bucket::declared(head, name)
                    .ok_or_else(|| Error::unlocated("invalid inherent target head"))?,
                context,
                member,
            );
        }
        Ok(())
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
