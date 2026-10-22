//! Shared structural field plans retain raw projection roots, never caller permission.
mod dependencies;
mod graph;
use super::{model::Declarations, projections::Concrete};
use crate::support::hash::HashSet;
use crate::types::{ItemId, TypeId};
pub(crate) use graph::Registry;
use graph::{Recipe, RecipeId};
#[cfg(test)]
#[path = "../../tests/invariants/stored_fields.rs"]
mod tests;

/// One caller body and root lifetime scope. Successful queries must retain their
/// supplied proof in that caller before completed recipes can be reused.
#[derive(Default)]
pub(crate) struct Preparation {
    complete: HashSet<RecipeId>,
    steps: usize,
}
/// A structural walk and caller proof keep their exact frontier across waits.
pub(crate) struct Query {
    graph: Option<graph::Build>,
    work: Vec<RecipeId>,
    seen: HashSet<RecipeId>,
    field: Option<(TypeId, TypeId)>,
    complete: bool,
}
pub(crate) enum Progress {
    Complete,
    Normalize(TypeId),
    Nominal(ItemId),
}
impl Query {
    pub fn new(root: TypeId) -> Self {
        Self {
            graph: Some(graph::Build::new(root)),
            work: Vec::new(),
            seen: HashSet::default(),
            field: None,
            complete: false,
        }
    }
    fn normalized(
        &mut self,
        facts: &mut Declarations,
        normal: TypeId,
        steps: usize,
    ) -> Result<(), &'static str> {
        if let Some(graph) = &mut self.graph {
            graph.normalized(facts, normal)?;
            facts
                .stored_fields
                .as_mut()
                .ok_or("missing storage registry")?
                .charge(steps)?;
        } else {
            let (_, expected) = self
                .field
                .take()
                .ok_or("storage proof has no pending field")?;
            if normal != expected {
                return Err("independent storage normalization changed a completed field");
            }
        }
        Ok(())
    }
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
    /// Whether a published root needs no further field proof in this body.
    /// This is exactly the immediate completion of [`Self::prepare`], which
    /// then neither normalizes nor adds caller evidence.
    pub(crate) fn completed(
        &mut self,
        facts: &Declarations,
        root: TypeId,
    ) -> Result<bool, &'static str> {
        let Some(recipe) = facts.stored_fields.as_ref().and_then(|r| r.root(root)) else {
            return Ok(false);
        };
        if recipe == RecipeId::EMPTY || self.complete.contains(&recipe) {
            self.charge(1)?;
            return Ok(true);
        }
        Ok(false)
    }
    pub(crate) fn prepare(
        &mut self,
        facts: &mut Declarations,
        root: TypeId,
        session: &mut Concrete<'_>,
    ) -> Result<(), &'static str> {
        let mut query = Query::new(root);
        loop {
            match self.advance(facts, &mut query)? {
                Progress::Complete => return Ok(()),
                Progress::Nominal(_) => return Err("missing aggregate field template"),
                Progress::Normalize(raw) => {
                    let (normal, steps) = session.normalize(facts, raw)?;
                    self.normalized(facts, &mut query, normal, steps)?;
                }
            }
        }
    }
    pub(crate) fn normalized(
        &mut self,
        facts: &mut Declarations,
        query: &mut Query,
        normal: TypeId,
        steps: usize,
    ) -> Result<(), &'static str> {
        self.charge(steps)?;
        query.normalized(facts, normal, steps)
    }
    pub(crate) fn advance(
        &mut self,
        facts: &mut Declarations,
        query: &mut Query,
    ) -> Result<Progress, &'static str> {
        if query.complete {
            return Ok(Progress::Complete);
        }
        if let Some(graph) = &mut query.graph {
            // Restore the shared registry before a resolver or child declaration
            // runs. A query owns its traversal, never the global registry.
            let mut registry = facts.stored_fields.take().unwrap_or_default();
            let progress = graph.advance(&mut registry, facts);
            facts.stored_fields = Some(registry);
            match progress? {
                graph::Progress::Normalize(raw) => return Ok(Progress::Normalize(raw)),
                graph::Progress::Nominal(item) => return Ok(Progress::Nominal(item)),
                graph::Progress::Complete(recipe) => {
                    query.graph = None;
                    if recipe == RecipeId::EMPTY || self.complete.contains(&recipe) {
                        self.charge(1)?;
                        query.complete = true;
                        return Ok(Progress::Complete);
                    }
                    query.work.push(recipe);
                }
            }
        }
        if let Some((raw, _)) = query.field {
            return Ok(Progress::Normalize(raw));
        }
        let registry = facts
            .stored_fields
            .as_ref()
            .ok_or("missing storage proof registry")?;
        while let Some(id) = query.work.pop() {
            self.charge(1)?;
            if id == RecipeId::EMPTY || self.complete.contains(&id) || !query.seen.insert(id) {
                continue;
            }
            match registry.recipe(id)? {
                Recipe::Field { raw, normalized } => {
                    query.field = Some((*raw, *normalized));
                    return Ok(Progress::Normalize(*raw));
                }
                Recipe::Join(children) => {
                    self.charge(children.len())?;
                    query.work.extend(children.iter().rev().copied());
                }
            }
            if query.work.len().saturating_add(query.seen.len()) > 1_000_000 {
                return Err("storage proof pending graph budget exceeded");
            }
        }
        // A failed or suspended replay authorizes none of its visited prefix.
        self.complete.extend(query.seen.drain());
        query.complete = true;
        Ok(Progress::Complete)
    }
}
