//! Both policy phases traverse the same resolved public aliases and module scopes.
use super::{Draft, Error};
use crate::{
    resolve::model::{Access, DefId, DefKind, Resolution, ScopeId},
    support::{hash::HashSet, retained},
};

pub(crate) trait Budget {
    fn charge(&mut self, steps: usize, bytes: usize) -> Result<(), Error>;
}
impl Budget for Draft {
    fn charge(&mut self, steps: usize, bytes: usize) -> Result<(), Error> {
        self.charge(steps, bytes)
    }
}

pub(crate) fn visit<B: Budget>(
    facts: &Resolution,
    budget: &mut B,
    mut declaration: impl FnMut(DefId, &mut B) -> Result<(), Error>,
) -> Result<HashSet<ScopeId>, Error> {
    let root = *facts
        .module_scopes
        .first()
        .ok_or_else(|| Error::unlocated("library root scope is absent"))?;
    let mut seen = HashSet::default();
    let mut pending = Vec::new();
    enqueue(root, &mut seen, &mut pending, budget)?;
    while let Some(scope) = pending.pop() {
        budget.charge(1, 0)?;
        for (_, _, binding) in facts.bindings.scope(scope) {
            budget.charge(1, 0)?;
            if binding.access != Access::Public {
                continue;
            }
            if binding.ambiguous {
                return Err(Error::unlocated("ambiguous public library binding"));
            }
            declaration(binding.target, budget)?;
            let definition = &facts.definitions[binding.target.0 as usize];
            if matches!(definition.kind, DefKind::Module | DefKind::Trait)
                && let Some(members) = definition.members
            {
                enqueue(members, &mut seen, &mut pending, budget)?;
            }
        }
    }
    Ok(seen)
}
fn enqueue<B: Budget>(
    scope: ScopeId,
    seen: &mut HashSet<ScopeId>,
    pending: &mut Vec<ScopeId>,
    budget: &mut B,
) -> Result<(), Error> {
    budget.charge(1, 0)?;
    if seen.contains(&scope) {
        return Ok(());
    }
    let table = retained::Growth::map::<ScopeId, ()>(seen.len(), seen.capacity(), 1);
    let queue = retained::Growth::vector::<ScopeId>(pending.len(), pending.capacity(), 1);
    budget.charge(
        table.work.saturating_add(queue.work),
        table.bytes.saturating_add(queue.bytes),
    )?;
    seen.try_reserve(1)
        .map_err(|_| Error::unlocated("library public scope allocation failed"))?;
    pending
        .try_reserve(1)
        .map_err(|_| Error::unlocated("library public work allocation failed"))?;
    seen.insert(scope);
    pending.push(scope);
    Ok(())
}
