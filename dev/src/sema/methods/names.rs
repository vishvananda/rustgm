//! Member identity is partitioned before shape and applicability work.
use super::{Candidates, Methods};
use crate::{
    resolve::{
        expansion::Report,
        model::{Access, DefId, ItemContext, MethodCall, ScopeId},
        names::NameId,
        package::{Error, ModuleId},
        unit::NodeKey,
    },
    support::hash::HashMap,
    syntax::ast::Kind,
    types::ItemId,
};

type Key = (ItemId, NameId);
#[cfg(test)]
#[path = "../../../tests/invariants/inherent_name_partitions.rs"]
mod tests;

/// One lazy lookup owner keeps new hygiene state out of ordinary crate records.
#[derive(Default)]
pub(super) struct Index {
    pub(super) visible: Option<super::visible::Index>,
    pub(super) contextual: Option<Box<Contextual>>,
}

#[derive(Default)]
pub(super) struct Contextual {
    pub(super) inherent: HashMap<Key, HashMap<ItemContext, Vec<DefId>>>,
    indices: HashMap<(ItemId, NameId, ItemContext), Candidates>,
    steps: usize,
}

#[derive(Clone, Copy)]
pub(in crate::sema) struct Use {
    pub context: ItemContext,
    pub caller: ModuleId,
}

pub(super) struct Cursor(ItemContext);
impl Cursor {
    pub fn new(context: ItemContext) -> Self {
        Self(context)
    }

    pub fn next(
        &mut self,
        methods: &mut Methods,
        report: &Report,
        key: Key,
    ) -> Result<Option<ItemContext>, &'static str> {
        while self.0 != ItemContext::ROOT {
            methods.account_name_lookup(1)?;
            let current = self.0;
            self.0 = report
                .item_context_parent(current)?
                .ok_or("nonroot method context has no parent")?;
            if methods.inherent_members(key, current).is_some() {
                return Ok(Some(current));
            }
        }
        Ok(None)
    }
}

impl Use {
    pub(super) fn retain_traits(
        self,
        report: &Report,
        facts: &crate::sema::model::Declarations,
        groups: &mut [Vec<DefId>; 2],
    ) -> Result<(), &'static str> {
        for group in groups {
            let mut selected = 0;
            for index in 0..group.len() {
                let member = group[index];
                let owner = facts
                    .associated
                    .trait_owner(member)
                    .ok_or("trait candidate has no actual owner")?;
                if self.permits(report, owner, member)? {
                    group[selected] = member;
                    selected += 1;
                }
            }
            group.truncate(selected);
        }
        Ok(())
    }

    pub fn path(report: &Report, suffix: NodeKey, scope: ScopeId) -> Result<Self, Error> {
        let offset = report
            .package
            .unit(suffix.file)
            .configured
            .ast
            .node(suffix.node)
            .span
            .start;
        Self::at(report, suffix, offset, scope)
    }

    pub fn receiver(report: &Report, key: NodeKey, site: MethodCall) -> Result<Self, Error> {
        if let Some(node) = site.method {
            return Self::path(report, NodeKey { node, ..key }, site.scope);
        }
        let Kind::MethodCall { name, .. } = report
            .package
            .unit(key.file)
            .configured
            .ast
            .node(key.node)
            .kind
        else {
            return Err(Error::unlocated("method lookup has no actual identifier"));
        };
        Self::at(report, key, name.start, site.scope)
    }

    fn at(report: &Report, key: NodeKey, offset: u32, scope: ScopeId) -> Result<Self, Error> {
        let caller = report
            .resolution
            .scopes
            .get(scope.0 as usize)
            .ok_or_else(|| Error::unlocated("method lookup has no actual caller scope"))?
            .module;
        Ok(Self {
            context: report.package.item_context(key.file, offset)?,
            caller,
        })
    }

    pub fn matching(
        self,
        report: &Report,
        container: DefId,
        member: DefId,
    ) -> Result<Option<ModuleId>, &'static str> {
        let adjusted = report.adjust_item(container, self.context, self.caller)?;
        let declared = report
            .definition_item_context(member)
            .map_err(|_| "invalid declared method context")?;
        if adjusted.context != declared {
            return Ok(None);
        }
        Ok(Some(adjusted.privacy))
    }

    pub fn permits(
        self,
        report: &Report,
        container: DefId,
        member: DefId,
    ) -> Result<bool, &'static str> {
        let Some(privacy) = self.matching(report, container, member)? else {
            return Ok(false);
        };
        let definition = report
            .resolution
            .definitions
            .get(member.0 as usize)
            .ok_or("unknown method declaration")?;
        Ok(match definition.access {
            Access::Public => true,
            Access::Within(ancestor) => report.package.is_descendant(privacy, ancestor),
            Access::ExternalPrivate => false,
        })
    }
}

impl Methods {
    pub(in crate::sema) fn publish_inherent(
        &mut self,
        key: Key,
        context: ItemContext,
        member: DefId,
    ) {
        // Keep the coarse key as a presence hint, even for a solely opaque name.
        let ordinary = self.inherent.entry(key).or_default();
        if context == ItemContext::ROOT {
            ordinary.push(member);
            self.indices.remove(&key);
        } else {
            let contextual = self
                .names
                .get_or_insert_default()
                .contextual
                .get_or_insert_default();
            contextual
                .inherent
                .entry(key)
                .or_default()
                .entry(context)
                .or_default()
                .push(member);
            contextual.indices.remove(&(key.0, key.1, context));
        }
    }

    pub(in crate::sema) fn freeze_inherent_names(&mut self) {
        for members in self.inherent.values_mut() {
            members.sort_unstable_by_key(|member| member.0);
        }
        if let Some(contextual) = self.contextual_mut() {
            for members in contextual
                .inherent
                .values_mut()
                .flat_map(HashMap::values_mut)
            {
                members.sort_unstable_by_key(|member| member.0);
            }
        }
    }

    pub(in crate::sema) fn has_contextual_inherent(&self, key: Key) -> bool {
        self.contextual()
            .is_some_and(|index| index.inherent.contains_key(&key))
    }

    pub(in crate::sema) fn inherent_groups(
        &self,
    ) -> impl Iterator<Item = (Key, ItemContext, &[DefId])> {
        self.inherent
            .iter()
            .map(|(&key, members)| (key, ItemContext::ROOT, members.as_slice()))
            .chain(
                self.contextual()
                    .into_iter()
                    .flat_map(|index| &index.inherent)
                    .flat_map(|(&key, names)| {
                        names
                            .iter()
                            .map(move |(&context, members)| (key, context, members.as_slice()))
                    }),
            )
    }

    pub(super) fn inherent_members(&self, key: Key, context: ItemContext) -> Option<&[DefId]> {
        if context == ItemContext::ROOT {
            self.inherent.get(&key).map(Vec::as_slice)
        } else {
            self.contextual()?
                .inherent
                .get(&key)?
                .get(&context)
                .map(Vec::as_slice)
        }
    }

    pub(super) fn candidate_index(&self, key: Key, context: ItemContext) -> Option<&Candidates> {
        if context == ItemContext::ROOT {
            self.indices.get(&key)
        } else {
            self.contextual()?.indices.get(&(key.0, key.1, context))
        }
    }

    pub(in crate::sema) fn retain_candidate_index(
        &mut self,
        key: Key,
        context: ItemContext,
        index: Candidates,
    ) -> Result<(), &'static str> {
        if context == ItemContext::ROOT {
            self.indices.insert(key, index);
        } else {
            let contextual = self
                .contextual_mut()
                .ok_or("contextual method index has no declarations")?;
            if !contextual
                .inherent
                .get(&key)
                .is_some_and(|names| names.contains_key(&context))
            {
                return Err("contextual method index changed its declared identity");
            }
            contextual.indices.insert((key.0, key.1, context), index);
        }
        Ok(())
    }

    pub(super) fn account_name_lookup(&mut self, amount: usize) -> Result<(), &'static str> {
        if let Some(index) = self.contextual_mut() {
            index.steps = index.steps.saturating_add(amount);
            if index.steps > 1_000_000 {
                return Err("method identity lookup work budget exceeded");
            }
        }
        Ok(())
    }

    fn contextual(&self) -> Option<&Contextual> {
        self.names.as_deref()?.contextual.as_deref()
    }

    fn contextual_mut(&mut self) -> Option<&mut Contextual> {
        self.names.as_deref_mut()?.contextual.as_deref_mut()
    }
}
