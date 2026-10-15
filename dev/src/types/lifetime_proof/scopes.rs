//! Immutable binder plans can outlive a proof's move or scope remapping.
use super::{Obligations, ScopeId, quantified::Binder};
use std::sync::Arc;

/// Reuse one source-to-destination map while preserving empty and sibling binders.
pub(crate) fn map_scopes(
    bindings: &mut Vec<ScopeId>,
    root: ScopeId,
    needed: ScopeId,
    mut binder: impl FnMut(ScopeId) -> Result<(ScopeId, u32), &'static str>,
    mut create: impl FnMut(ScopeId, u32) -> Result<ScopeId, &'static str>,
) -> Result<ScopeId, &'static str> {
    if bindings.is_empty() {
        bindings.push(root);
    } else if bindings[0] != root {
        return Err("proof scope mapping has a different root");
    }
    while bindings.len() <= needed.0 as usize {
        let id = ScopeId(u32::try_from(bindings.len()).map_err(|_| "too many proof scopes")?);
        let (parent, count) = binder(id)?;
        let parent = *bindings
            .get(parent.0 as usize)
            .ok_or("missing mapped proof parent")?;
        bindings.push(create(parent, count)?);
    }
    bindings
        .get(needed.0 as usize)
        .copied()
        .ok_or("invalid mapped proof scope")
}

#[derive(Clone, Default)]
pub(crate) struct ScopePlan {
    scopes: Option<Arc<Vec<Binder>>>,
}
impl ScopePlan {
    /// A fresh query needs only its ancestor chain, never earlier sibling scopes.
    pub fn enter(
        &self,
        mut scope: ScopeId,
        destination: &mut Obligations,
    ) -> Result<ScopeId, &'static str> {
        let mut counts = Vec::with_capacity(usize::from(self.depth(scope)?));
        while scope != ScopeId::ROOT {
            let (parent, count) = self.binder(scope)?;
            counts.push(count);
            scope = parent;
        }
        let mut root = ScopeId::ROOT;
        for count in counts.into_iter().rev() {
            root = destination.scope(root, count)?;
        }
        Ok(root)
    }
    pub fn len(&self) -> usize {
        self.scopes.as_ref().map_or(0, |scopes| scopes.len())
    }
    pub fn binder(&self, scope: ScopeId) -> Result<(ScopeId, u32), &'static str> {
        let binder = scope
            .0
            .checked_sub(1)
            .and_then(|id| self.scopes.as_ref()?.get(id as usize))
            .ok_or("invalid retained proof scope")?;
        Ok((binder.parent, binder.count))
    }
    pub fn depth(&self, scope: ScopeId) -> Result<u16, &'static str> {
        if scope == ScopeId::ROOT {
            return Ok(0);
        }
        self.scopes
            .as_ref()
            .and_then(|scopes| scopes.get(scope.0 as usize - 1))
            .map(|binder| binder.depth)
            .ok_or("invalid retained proof scope")
    }
}
impl Obligations {
    pub(crate) fn scope_plan(&self) -> ScopePlan {
        ScopePlan {
            scopes: self.quantified.as_ref().map(|q| Arc::clone(&q.scopes)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fresh_queries_copy_only_their_actual_ancestors() -> Result<(), &'static str> {
        let mut source = Obligations::new(1);
        let outer = source.scope(ScopeId::ROOT, 0)?;
        for _ in 0..8192 {
            source.scope(outer, 3)?;
        }
        let chosen = source.scope(outer, 1)?;
        let mut destination = Obligations::new(1);
        let target = source.scope_plan().enter(chosen, &mut destination)?;
        assert_eq!(destination.scope_plan().len(), 2);
        assert_eq!(destination.scope_plan().depth(target)?, 2);
        let (parent, count) = destination.scope_plan().binder(target)?;
        assert_eq!(count, 1);
        assert_eq!(destination.scope_plan().binder(parent)?, (ScopeId::ROOT, 0));
        Ok(())
    }

    #[test]
    fn retained_plans_survive_scope_remapping_and_later_extension() -> Result<(), &'static str> {
        let mut source = Obligations::new(5);
        let outer = source.scope(ScopeId::ROOT, 1)?;
        let inner = source.scope(outer, 2)?;
        let plan = source.scope_plan();
        let mut destination = Obligations::new(5);
        destination.scope(ScopeId::ROOT, 3)?;
        destination.extend(source.clone())?;
        source.scope(inner, 4)?;
        assert_eq!(plan.len(), 2);
        assert_eq!(plan.binder(inner)?, (outer, 2));
        assert_eq!(plan.depth(inner)?, 2);
        assert_eq!(source.scope_plan().len(), 3);
        let merged = destination.scope_plan();
        assert_eq!(merged.len(), 3);
        assert_eq!(merged.binder(ScopeId(3))?, (ScopeId(2), 2));
        assert_eq!(merged.depth(ScopeId(3))?, 2);
        Ok(())
    }
}
