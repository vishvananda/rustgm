//! Shared structural field plans retain raw projection roots, never caller permission.
mod dependencies;
mod graph;
use super::{model::Declarations, projections::Concrete};
use crate::types::TypeId;
pub(crate) use graph::Registry;
use graph::{Recipe, RecipeId};
use std::collections::HashSet;
#[cfg(test)]
#[path = "../../tests/invariants/stored_fields.rs"]
mod tests;

/// One caller body and root lifetime scope. Successful calls must retain their
/// supplied proof in that caller before this set of completed recipes is reused.
#[derive(Default)]
pub(crate) struct Preparation {
    complete: HashSet<RecipeId>,
    steps: usize,
}
impl Preparation {
    fn charge(&mut self, count: usize) -> Result<(), &'static str> {
        self.steps = self
            .steps
            .checked_add(count)
            .filter(|steps| *steps <= 4_000_000)
            .ok_or("storage normalization work budget exceeded")?;
        Ok(())
    }

    pub(crate) fn prepare(
        &mut self,
        facts: &mut Declarations,
        root: TypeId,
        session: &mut Concrete<'_>,
    ) -> Result<(), &'static str> {
        let mut registry = facts.stored_fields.take().unwrap_or_default();
        let result = registry
            .demand(facts, root, session)
            .and_then(|recipe| self.prove(facts, &registry, recipe, session));
        facts.stored_fields = Some(registry);
        result
    }

    fn prove(
        &mut self,
        facts: &mut Declarations,
        registry: &Registry,
        root: RecipeId,
        session: &mut Concrete<'_>,
    ) -> Result<(), &'static str> {
        let mut work = vec![root];
        let mut seen = HashSet::new();
        while let Some(id) = work.pop() {
            self.charge(1)?;
            if id == RecipeId::EMPTY || self.complete.contains(&id) || !seen.insert(id) {
                continue;
            }
            match registry.recipe(id)? {
                Recipe::Field { raw, normalized } => {
                    let (result, steps) = session.normalize(facts, *raw)?;
                    self.charge(steps)?;
                    if result != *normalized {
                        return Err("independent storage normalization changed a completed field");
                    }
                }
                Recipe::Join(children) => {
                    self.charge(children.len())?;
                    work.extend(children.iter().rev().copied());
                }
            }
            if work.len().saturating_add(seen.len()) > 1_000_000 {
                return Err("storage proof pending graph budget exceeded");
            }
        }
        // Failed replay cannot authorize even a partially visited recipe.
        self.complete.extend(seen);
        Ok(())
    }
}
