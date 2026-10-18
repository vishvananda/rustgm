//! Resolved names reserve candidate owners before their environments are elaborated.
use super::{Checker, DefId, DefKind, Error, NodeKey, Type};
use crate::{resolve::names::NameId, sema::associated::OwnerKind, types::ItemId};
use std::collections::{HashMap, HashSet};
#[cfg(test)]
#[path = "../../../tests/invariants/inherent_headers.rs"]
mod tests;

#[derive(Default)]
pub(super) struct Pending {
    owners: HashMap<DefId, Members>,
    buckets: HashMap<(Option<ItemId>, NameId), HashSet<DefId>>,
}

struct Members {
    head: Option<ItemId>,
    declarations: Vec<(NameId, DefId)>,
}

impl Pending {
    fn insert(&mut self, owner: DefId, head: Option<ItemId>, name: NameId, member: DefId) {
        self.owners
            .entry(owner)
            .or_insert_with(|| Members {
                head,
                declarations: Vec::new(),
            })
            .declarations
            .push((name, member));
        self.buckets.entry((head, name)).or_default().insert(owner);
    }

    pub(super) fn contains(&self, nominal: ItemId, name: NameId) -> bool {
        self.buckets.contains_key(&(Some(nominal), name))
            || self.buckets.contains_key(&(None, name))
    }

    pub(super) fn dependencies(&self, nominal: ItemId, name: NameId) -> Vec<DefId> {
        let mut owners: Vec<_> = [Some(nominal), None]
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
    pub(super) fn reserve_inherent_headers(&mut self) -> Result<(), Error> {
        let mut heads = super::heads::Heads::new(self.report, &self.facts);
        let mut pending = Pending::default();
        let mut trait_names = HashSet::new();
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
                    let head = pending.owners.get(&owner.self_definition).map_or_else(
                        || heads.target(target).unwrap_or_default(),
                        |members| members.head,
                    );
                    pending.insert(
                        owner.self_definition,
                        head,
                        definition.name,
                        member.definition,
                    );
                }
                OwnerKind::Impl { .. } => {}
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
        let Some(Type::Adt(nominal, _)) = self.facts.types.kind(schema.self_type) else {
            return Err(self.error(source, "missing inherent nominal target"));
        };
        if members.head.is_some_and(|head| head != nominal) {
            return Err(self.error(
                source,
                "inherent owner disagrees with reserved nominal head",
            ));
        }
        for (name, member) in members.declarations {
            self.facts
                .methods
                .inherent
                .entry((nominal, name))
                .or_default()
                .push(member);
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
        for bucket in self.facts.methods.inherent.values_mut() {
            bucket.sort_unstable_by_key(|id| id.0);
        }
        self.inherent_headers = Pending::default();
        Ok(())
    }
}
