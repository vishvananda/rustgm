//! Contextual discovery partitions member identities before candidate copying.
use super::{Arc, DefId, HashMap, ItemContext, ItemId, NameId, Report, ScopeId};

#[derive(Default)]
pub(super) struct Index {
    pub(super) members: HashMap<DefId, HashMap<(NameId, ItemContext), DefId>>,
    pub(super) local: HashMap<ScopeId, HashMap<(NameId, ItemContext), Vec<DefId>>>,
    lexical: HashMap<(ScopeId, NameId, ItemContext, ItemContext), Arc<[DefId]>>,
    work: Vec<(ScopeId, ItemContext)>,
}

impl super::Index {
    pub(super) fn contextual_bounds(
        &mut self,
        report: &Report,
        traits: &[ItemId],
        name: NameId,
        mut context: ItemContext,
        result: &mut Vec<DefId>,
    ) -> Result<(), &'static str> {
        if self.contextual.is_none() {
            return Ok(());
        }
        while context != ItemContext::ROOT {
            self.account(traits.len().saturating_add(1))?;
            let index = self
                .contextual
                .as_ref()
                .ok_or("missing trait context index")?;
            for owner in traits {
                if let Some(&member) = index
                    .members
                    .get(&DefId(owner.0))
                    .and_then(|names| names.get(&(name, context)))
                {
                    result.push(member);
                }
            }
            context = report
                .item_context_parent(context)?
                .ok_or("nonroot trait method context has no parent")?;
        }
        Ok(())
    }

    pub(super) fn contextual_visible(
        &mut self,
        report: &Report,
        scope: ScopeId,
        name: NameId,
        original: ItemContext,
    ) -> Result<Arc<[DefId]>, &'static str> {
        self.contextual.get_or_insert_default().work.clear();
        let mut current = (Some(scope), original);
        let mut result: Arc<[DefId]> = Arc::from([]);
        while let (Some(scope), context) = current {
            self.account(1)?;
            let index = self
                .contextual
                .as_mut()
                .ok_or("missing trait context index")?;
            if let Some(cached) = index.lexical.get(&(scope, name, context, original)) {
                result = Arc::clone(cached);
                break;
            }
            index.work.push((scope, context));
            current = report.trait_scope_parent(scope, context)?;
            if current.0.is_none() && !self.prelude.is_empty() {
                let module = report.resolution.scopes[scope.0 as usize].module;
                if report.package.modules[module.0 as usize].implicit_prelude
                    && let Some(&member) = self.prelude.get(&name)
                {
                    self.account(1)?;
                    result = Arc::from([member]);
                }
            }
        }
        loop {
            let index = self
                .contextual
                .as_mut()
                .ok_or("missing trait context index")?;
            let Some((scope, context)) = index.work.pop() else {
                break;
            };
            self.index_scope(scope)?;
            let local = self.contextual_local(report, scope, name, original)?;
            if !local.is_empty() {
                self.account(local.len().saturating_add(result.len()))?;
                let mut combined = result.to_vec();
                combined.extend(local);
                combined.sort_unstable_by_key(|id| id.0);
                combined.dedup();
                result = Arc::from(combined);
            }
            self.contextual
                .as_mut()
                .ok_or("missing trait context index")?
                .lexical
                .insert((scope, name, context, original), Arc::clone(&result));
        }
        Ok(result)
    }

    pub(super) fn contextual_local(
        &mut self,
        report: &Report,
        scope: ScopeId,
        name: NameId,
        mut context: ItemContext,
    ) -> Result<Vec<DefId>, &'static str> {
        let count = self
            .local
            .get(&scope)
            .and_then(|names| names.get(&name))
            .map_or(0, Vec::len);
        self.account(count)?;
        let mut result = self
            .local
            .get(&scope)
            .and_then(|names| names.get(&name))
            .cloned()
            .unwrap_or_default();
        while context != ItemContext::ROOT {
            self.account(1)?;
            let count = self
                .contextual
                .as_ref()
                .and_then(|index| index.local.get(&scope))
                .and_then(|names| names.get(&(name, context)))
                .map_or(0, Vec::len);
            self.account(count)?;
            if let Some(members) = self
                .contextual
                .as_ref()
                .and_then(|index| index.local.get(&scope))
                .and_then(|names| names.get(&(name, context)))
            {
                result.extend_from_slice(members);
            }
            context = report
                .item_context_parent(context)?
                .ok_or("nonroot trait method context has no parent")?;
        }
        Ok(result)
    }
}
