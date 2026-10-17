//! Persistent proof dependencies omit field paths with no projection work.
use super::{Concrete, Declarations, TypeId, dependencies};
use std::collections::{HashMap, HashSet};

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
    fields: HashMap<TypeId, RecipeId>,
    pub(super) recipes: Vec<Recipe>,
    steps: usize,
    retained: usize,
}
impl Registry {
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
    pub(super) fn demand(
        &mut self,
        facts: &mut Declarations,
        root: TypeId,
        session: &mut Concrete<'_>,
    ) -> Result<RecipeId, &'static str> {
        if let Some(&recipe) = self.roots.get(&root) {
            return Ok(recipe);
        }
        let mut build = Build::default();
        let recipe = build.walk(self, facts, root, session)?;
        for (ty, fields) in build.shapes {
            facts
                .aggregates
                .complete_storage(&facts.types, ty, fields)?;
        }
        // Publish only after the entire structural graph completed. No caller's
        // proof is retained here; a new scope replays the raw field recipes.
        self.roots.extend(build.roots);
        self.fields.extend(build.fields);
        self.recipes.extend(build.recipes);
        Ok(recipe)
    }
}

#[derive(Default)]
struct Build {
    roots: HashMap<TypeId, RecipeId>,
    fields: HashMap<TypeId, RecipeId>,
    recipes: Vec<Recipe>,
    shapes: Vec<(TypeId, Vec<crate::types::ListId>)>,
}
enum Work {
    Enter(TypeId),
    Finish {
        ty: TypeId,
        children: Vec<TypeId>,
        recipes: Vec<RecipeId>,
    },
}
impl Build {
    fn root(&self, registry: &Registry, ty: TypeId) -> Option<RecipeId> {
        self.roots
            .get(&ty)
            .or_else(|| registry.roots.get(&ty))
            .copied()
    }
    fn push(&mut self, registry: &mut Registry, recipe: Recipe) -> Result<RecipeId, &'static str> {
        let children = match &recipe {
            Recipe::Join(children) => children.len(),
            Recipe::Field { .. } => 0,
        };
        registry.retain(
            size_of::<Recipe>().saturating_add(children.saturating_mul(size_of::<RecipeId>())),
        )?;
        self.recipes.push(recipe);
        Ok(RecipeId(registry.recipes.len() + self.recipes.len()))
    }
    fn field(
        &mut self,
        registry: &mut Registry,
        ty: TypeId,
        normalized: TypeId,
    ) -> Result<RecipeId, &'static str> {
        if let Some(&id) = self.fields.get(&ty).or_else(|| registry.fields.get(&ty)) {
            return Ok(id);
        }
        registry.retain(64)?;
        let id = self.push(
            registry,
            Recipe::Field {
                raw: ty,
                normalized,
            },
        )?;
        self.fields.insert(ty, id);
        Ok(id)
    }
    fn join(
        &mut self,
        registry: &mut Registry,
        mut children: Vec<RecipeId>,
    ) -> Result<RecipeId, &'static str> {
        registry.charge(children.len())?;
        let mut seen = HashSet::new();
        children.retain(|&id| id != RecipeId::EMPTY && seen.insert(id));
        match children.as_slice() {
            [] => Ok(RecipeId::EMPTY),
            [child] => Ok(*child),
            _ => self.push(registry, Recipe::Join(children)),
        }
    }
    fn walk(
        &mut self,
        registry: &mut Registry,
        facts: &mut Declarations,
        root: TypeId,
        session: &mut Concrete<'_>,
    ) -> Result<RecipeId, &'static str> {
        let mut work = vec![Work::Enter(root)];
        let mut active = HashSet::new();
        while let Some(task) = work.pop() {
            registry.charge(1)?;
            match task {
                Work::Enter(ty) => {
                    if self.root(registry, ty).is_some() {
                        continue;
                    }
                    if !active.insert(ty) {
                        return Err("cycle in storage dependencies");
                    }
                    registry.retain(64)?;
                    let dependencies = dependencies::read(registry, facts, ty, session)?;
                    let recipes = dependencies
                        .fields
                        .into_iter()
                        .map(|(raw, normalized)| self.field(registry, raw, normalized))
                        .collect::<Result<Vec<_>, _>>()?;
                    if let Some(fields) = dependencies.shape {
                        self.shapes.push((ty, fields));
                    }
                    let start = work.len();
                    work.extend(dependencies.children.iter().rev().copied().map(Work::Enter));
                    work.insert(
                        start,
                        Work::Finish {
                            ty,
                            children: dependencies.children,
                            recipes,
                        },
                    );
                }
                Work::Finish {
                    ty,
                    children,
                    mut recipes,
                } => {
                    for child in children {
                        recipes.push(
                            self.root(registry, child)
                                .ok_or("unfinished storage dependency")?,
                        );
                    }
                    let recipe = self.join(registry, recipes)?;
                    active.remove(&ty);
                    self.roots.insert(ty, recipe);
                }
            }
            if work.len().saturating_add(self.roots.len()) > 1_000_000 {
                return Err("storage dependency pending graph budget exceeded");
            }
        }
        self.root(registry, root)
            .ok_or("missing storage dependency root")
    }
}
