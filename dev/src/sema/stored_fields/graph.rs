//! Append-only structural recipes remain stable while other queries publish completed nodes.
use super::{Declarations, TypeId, dependencies::Reader};
use crate::support::hash::{HashMap, HashSet};
use crate::types::{ItemId, Kind, ListId};

/// One registry serves every storage proof of a session, so its work is
/// bounded by a coarse whole-graph runaway guard rather than by what one
/// proof may spend.
const WORK_LIMIT: usize = 1 << 30;

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
            .filter(|steps| *steps <= WORK_LIMIT)
            .ok_or("storage dependency work exceeds its whole-session runaway limit")?;
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
    /// Publish one strongly connected component of the dependency graph. Its
    /// types reach one another, as a list node reaches itself through the
    /// pointer argument of its link, so they share one recipe: the fields of
    /// every member and the roots of every dependency outside it.
    fn publish(
        &mut self,
        facts: &mut Declarations,
        mut members: Vec<Finish>,
        premise_free: bool,
    ) -> Result<(), &'static str> {
        if owns_itself(&members) {
            return Err("cycle in storage dependencies");
        }
        for member in &mut members {
            if let Some(fields) = member.shape.take() {
                facts
                    .aggregates
                    .complete_storage(&facts.types, member.ty, fields)?;
            }
            // Each dependency outside the component finished first, so this
            // subtree needs no premise-free re-entry.
            if premise_free
                && facts.types.contains_parameters(member.ty) == Some(true)
                && self.premise_free.insert(member.ty)
            {
                self.retain(64)?;
            }
        }
        // Another query may have completed these roots while a field resolver
        // was suspended. Their stable recipes already name the same shapes.
        if members
            .iter()
            .all(|member| self.roots.contains_key(&member.ty))
        {
            return Ok(());
        }
        let inside = members
            .iter()
            .map(|member| member.ty)
            .collect::<HashSet<_>>();
        let mut recipes = Vec::new();
        for member in &mut members {
            recipes.append(&mut member.recipes);
            self.charge(member.children.len())?;
            for child in member
                .children
                .iter()
                .filter(|child| !inside.contains(child))
            {
                recipes.push(
                    *self
                        .roots
                        .get(child)
                        .ok_or("unfinished storage dependency")?,
                );
            }
        }
        let recipe = self.join(recipes)?;
        for member in &members {
            if !self.roots.contains_key(&member.ty) {
                self.retain(64)?;
                self.roots.insert(member.ty, recipe);
            }
        }
        Ok(())
    }
}
/// Whether a component's members own one another's storage in a cycle, as
/// an infinitely sized type does. A nominal's arguments are never owned
/// storage: they are reached through pointers, if at all.
fn owns_itself(members: &[Finish]) -> bool {
    if let [member] = members {
        return member
            .children
            .iter()
            .take(member.owned)
            .any(|&child| child == member.ty);
    }
    let positions = members
        .iter()
        .zip(0..)
        .map(|(member, position)| (member.ty, position))
        .collect::<HashMap<_, usize>>();
    let owned = |member: &Finish| {
        member
            .children
            .iter()
            .take(member.owned)
            .filter_map(|child| positions.get(child).copied())
            .collect::<Vec<_>>()
    };
    let mut incoming = vec![0_usize; members.len()];
    for member in members {
        for position in owned(member) {
            incoming[position] += 1;
        }
    }
    let mut ready = (0..members.len())
        .filter(|&position| incoming[position] == 0)
        .collect::<Vec<_>>();
    let mut removed = 0;
    while let Some(position) = ready.pop() {
        removed += 1;
        for next in owned(&members[position]) {
            incoming[next] -= 1;
            if incoming[next] == 0 {
                ready.push(next);
            }
        }
    }
    removed < members.len()
}
struct Finish {
    ty: TypeId,
    children: Vec<TypeId>,
    owned: usize,
    recipes: Vec<RecipeId>,
    shape: Option<Vec<ListId>>,
}
/// A type on the depth-first path: its discovery index, the lowest index
/// that its dependencies reach, and where the members of its component that
/// already finished begin.
struct Entry {
    ty: TypeId,
    index: usize,
    low: usize,
    deferred: usize,
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
    /// The types whose dependencies are being read, outermost first.
    path: Vec<Entry>,
    /// The discovery index of each type on the path or in `deferred`.
    indices: HashMap<TypeId, usize>,
    /// Finished types whose component's first type is still on the path.
    deferred: Vec<Finish>,
    discovered: usize,
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
    fn enter(&mut self, ty: TypeId) {
        let index = self.discovered;
        self.discovered += 1;
        self.indices.insert(ty, index);
        self.path.push(Entry {
            ty,
            index,
            low: index,
            deferred: self.deferred.len(),
        });
    }
    /// Abandon a type that another query published while it was being read.
    fn leave(&mut self, ty: TypeId) -> Result<(), &'static str> {
        let entry = self
            .path
            .pop()
            .filter(|entry| entry.ty == ty && entry.low == entry.index)
            .ok_or("storage dependency path lost its reader")?;
        self.indices.remove(&entry.ty);
        Ok(())
    }
    /// A type whose dependencies all finished: the members of its component
    /// once it is that component's first type, else nothing yet.
    fn finish(&mut self, finish: Finish) -> Result<Option<Vec<Finish>>, &'static str> {
        let entry = self
            .path
            .pop()
            .filter(|entry| entry.ty == finish.ty)
            .ok_or("storage dependency path lost its type")?;
        if entry.low < entry.index {
            let parent = self
                .path
                .last_mut()
                .ok_or("storage dependency component has no first type")?;
            parent.low = parent.low.min(entry.low);
            self.deferred.push(finish);
            return Ok(None);
        }
        let mut members = self.deferred.split_off(entry.deferred);
        members.push(finish);
        for member in &members {
            self.indices.remove(&member.ty);
        }
        Ok(Some(members))
    }
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
            path: Vec::new(),
            indices: HashMap::default(),
            deferred: Vec::new(),
            discovered: 0,
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
                    let ty = *ty;
                    self.reader = None;
                    self.leave(ty)?;
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
                    owned: dependencies.owned,
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
                        // A type met again before its component completed
                        // belongs to the component of the type that met it.
                        if let Some(&index) = self.indices.get(&ty) {
                            let top = self
                                .path
                                .last_mut()
                                .ok_or("storage dependency has no reader")?;
                            top.low = top.low.min(index);
                            continue;
                        }
                        if let Some(Kind::Adt(item, _)) = facts.types.kind(ty)
                            && facts.aggregates.get(item).is_none()
                        {
                            self.work.push(Work::Enter(ty));
                            return Ok(Progress::Nominal(item));
                        }
                        self.enter(ty);
                        self.reader = Some((ty, Reader::new(registry, facts, ty, premise_free)?));
                    }
                    Work::Finish(finish) => {
                        if let Some(members) = self.finish(finish)? {
                            registry.publish(facts, members, premise_free)?;
                        }
                    }
                }
            }
            if self.work.len().saturating_add(self.indices.len()) > 1_000_000 {
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
