//! Immutable binder plans can outlive a proof's move or scope remapping.
use super::{Obligations, ScopeId, quantified::Binder};
use std::sync::Arc;

#[derive(Clone, Default)]
pub(crate) struct ScopePlan {
    scopes: Option<Arc<Vec<Binder>>>,
}
impl ScopePlan {
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
