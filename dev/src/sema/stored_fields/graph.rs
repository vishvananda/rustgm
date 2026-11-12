//! Append-only structural recipes remain stable while other queries publish completed nodes.
use super::{Declarations, TypeId, dependencies::Reader};
use crate::support::hash::{HashMap, HashSet};
use crate::types::{ItemId, Kind, ListId};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) struct RecipeId(usize);
impl RecipeId {
    pub const EMPTY: Self = Self(0);
}
pub(super) enum Recipe {
    Field { raw: TypeId, normalized: TypeId },
    Join(Vec<RecipeId>),
}
#[derive(Default)]
pub(crate) struct Registry {
    roots: HashMap<TypeId, RecipeId>,
    /// Parameterized roots whose premise-free build completed their subtree.
    premise_free: HashSet<TypeId>,
    fields: HashMap<TypeId, RecipeId>,
    pub(super) recipes: Vec<Recipe>,
    steps: usize,
    retained: usize,
}
impl Registry {
    pub(crate) fn work(&self) -> crate::sema::model::StoredFieldWork {
        crate::sema::model::StoredFieldWork {
            steps: self.steps,
            recipes: self.recipes.len(),
            retained_bytes: self.retained,
        }
    }

    /// The completed recipe published for this root, if any.
    pub(super) fn root(&self, ty: TypeId) -> Option<RecipeId> {
        self.roots.get(&ty).copied()
    }
    pub(crate) fn contains_projection(&self, ty: TypeId) -> Option<bool> {
        self.roots.get(&ty).map(|&id| id != RecipeId::EMPTY)
    }
    pub(super) fn recipe(&self, id: RecipeId) -> Result<&Recipe, &'static str> {
        id.0.checked_sub(1)
            .and_then(|index| self.recipes.get(index))
            .ok_or("invalid storage proof recipe")
    }
    pub(super) fn charge(&mut self, count: usize) -> Result<(), &'static str> {
        self.steps = self
            .steps
            .checked_add(count)
            .filter(|steps| *steps <= 4_000_000)
            .ok_or("storage dependency work budget exceeded")?;
        Ok(())
    }
    fn retain(&mut self, bytes: usize) -> Result<(), &'static str> {
        self.retained = self
            .retained
            .checked_add(bytes)
            .filter(|bytes| *bytes <= 64_000_000)
            .ok_or("storage proof recipes exceed 64 MB budget")?;
        Ok(())
    }
    fn push(&mut self, recipe: Recipe) -> Result<RecipeId, &'static str> {
        let children = match &recipe {
            Recipe::Join(children) => children.len(),
            Recipe::Field { .. } => 0,
        };
        self.retain(
            size_of::<Recipe>().saturating_add(children.saturating_mul(size_of::<RecipeId>())),
        )?;
        self.recipes.push(recipe);
        Ok(RecipeId(self.recipes.len()))
    }
    fn field(&mut self, input: TypeId, normalized: TypeId) -> Result<RecipeId, &'static str> {
        if let Some(&id) = self.fields.get(&input) {
            return if matches!(self.recipe(id)?, Recipe::Field { normalized: prior, .. } if *prior == normalized)
            {
                Ok(id)
            } else {
                Err("independent storage normalization changed a completed field")
            };
        }
        self.retain(64)?;
        let id = self.push(Recipe::Field {
            raw: input,
            normalized,
        })?;
        self.fields.insert(input, id);
        Ok(id)
    }
    fn join(&mut self, mut children: Vec<RecipeId>) -> Result<RecipeId, &'static str> {
        self.charge(children.len())?;
        let mut seen = HashSet::default();
        children.retain(|&id| id != RecipeId::EMPTY && seen.insert(id));
        match children.as_slice() {
            [] => Ok(RecipeId::EMPTY),
            [child] => Ok(*child),
            _ => self.push(Recipe::Join(children)),
        }
    }
    fn publish(
        &mut self,
        facts: &mut Declarations,
        finish: Finish,
        premise_free: bool,
    ) -> Result<(), &'static str> {
        if let Some(fields) = finish.shape {
            facts
                .aggregates
                .complete_storage(&facts.types, finish.ty, fields)?;
        }
        // Each child finished first, so this subtree needs no premise-free
        // re-entry.
        if premise_free
            && facts.types.contains_parameters(finish.ty) == Some(true)
            && self.premise_free.insert(finish.ty)
        {
            self.retain(64)?;
        }
        // Another query may have completed this root while its field resolver
        // was suspended. Its stable recipe already names the same checked shape.
        if self.roots.contains_key(&finish.ty) {
            return Ok(());
        }
        let mut recipes = finish.recipes;
        for child in finish.children {
            recipes.push(
                *self
                    .roots
                    .get(&child)
                    .ok_or("unfinished storage dependency")?,
            );
        }
        let recipe = self.join(recipes)?;
        self.retain(64)?;
        self.roots.insert(finish.ty, recipe);
        Ok(())
    }
}
struct Finish {
    ty: TypeId,
    children: Vec<TypeId>,
    recipes: Vec<RecipeId>,
    shape: Option<Vec<ListId>>,
}
enum Work {
    Enter(TypeId),
    Finish(Finish),
}
pub(super) enum Progress {
    Complete(RecipeId),
    Normalize(TypeId),
    Nominal(ItemId),
}
pub(super) struct Build {
    root: TypeId,
    /// Parameterized nominals' projected fields normalize too, only in a
    /// premise-free session.
    premise_free: bool,
    started: bool,
    work: Vec<Work>,
    active: HashSet<TypeId>,
    reader: Option<(TypeId, Reader)>,
}
/// A published root needs no further traversal. An ordinary build publishes a
/// parameterized root without normalizing its nested projected fields, so a
/// premise-free build reuses one only after completing its subtree once.
fn reusable(premise_free: bool, registry: &Registry, facts: &Declarations, ty: TypeId) -> bool {
    registry.roots.contains_key(&ty)
        && (!premise_free
            || facts.types.contains_parameters(ty) != Some(true)
            || registry.premise_free.contains(&ty))
}
impl Build {
    pub fn new(root: TypeId) -> Self {
        Self::with(root, false)
    }
    pub fn premise_free(root: TypeId) -> Self {
        Self::with(root, true)
    }
    fn with(root: TypeId, premise_free: bool) -> Self {
        Self {
            root,
            premise_free,
            started: false,
            work: Vec::new(),
            active: HashSet::default(),
            reader: None,
        }
    }
    pub fn normalized(&mut self, facts: &Declarations, normal: TypeId) -> Result<(), &'static str> {
        self.reader
            .as_mut()
            .ok_or("storage graph has no pending field")?
            .1
            .normalized(facts, normal)
    }
    pub fn advance(
        &mut self,
        registry: &mut Registry,
        facts: &mut Declarations,
    ) -> Result<Progress, &'static str> {
        let premise_free = self.premise_free;
        if !self.started {
            if let Some(&recipe) = registry.roots.get(&self.root)
                && reusable(premise_free, registry, facts, self.root)
            {
                return Ok(Progress::Complete(recipe));
            }
            self.work.push(Work::Enter(self.root));
            self.started = true;
        }
        loop {
            if let Some((ty, reader)) = &mut self.reader {
                // Finish an outstanding normalization before reusing interleaved
                // structural publication; its caller evidence must not be dropped.
                if !reader.pending() && reusable(premise_free, registry, facts, *ty) {
                    self.active.remove(ty);
                    self.reader = None;
                    continue;
                }
                if let Some(raw) = reader.advance(registry, facts)? {
                    return Ok(Progress::Normalize(raw));
                }
                let (ty, reader) = self
                    .reader
                    .take()
                    .ok_or("missing completed storage reader")?;
                let dependencies = reader.finish();
                let recipes = dependencies
                    .fields
                    .into_iter()
                    .map(|(raw, normal)| registry.field(raw, normal))
                    .collect::<Result<Vec<_>, _>>()?;
                let finish = Finish {
                    ty,
                    children: dependencies.children,
                    recipes,
                    shape: dependencies.shape,
                };
                let start = self.work.len();
                self.work
                    .extend(finish.children.iter().rev().copied().map(Work::Enter));
                self.work.insert(start, Work::Finish(finish));
            } else {
                let Some(task) = self.work.pop() else {
                    break;
                };
                registry.charge(1)?;
                match task {
                    Work::Enter(ty) => {
                        if reusable(premise_free, registry, facts, ty) {
                            continue;
                        }
                        if let Some(Kind::Adt(item, _)) = facts.types.kind(ty)
                            && facts.aggregates.get(item).is_none()
                        {
                            self.work.push(Work::Enter(ty));
                            return Ok(Progress::Nominal(item));
                        }
                        if !self.active.insert(ty) {
                            return Err("cycle in storage dependencies");
                        }
                        self.reader = Some((ty, Reader::new(registry, facts, ty, premise_free)?));
                    }
                    Work::Finish(finish) => {
                        self.active.remove(&finish.ty);
                        registry.publish(facts, finish, premise_free)?;
                    }
                }
            }
            if self.work.len().saturating_add(self.active.len()) > 1_000_000 {
                return Err("storage dependency pending graph budget exceeded");
            }
        }
        registry
            .roots
            .get(&self.root)
            .copied()
            .map(Progress::Complete)
            .ok_or("missing storage dependency root")
    }
}
