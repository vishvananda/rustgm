//! Shared destructor/field recipes; array length never expands the construction work.
mod cache;
use crate::{
    body::drops::Method as Declaration,
    resolve::model::DefId,
    sema::members::{FieldId, Members, Selection, VariantId},
    types::{ItemId, Kind, Store, TypeId, aggregate::Definitions, instance::Instance},
};
pub use cache::Cache;
use cache::{Key, State};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Id(pub u32);
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Range {
    pub start: u32,
    pub end: u32,
}
#[derive(Clone, Copy, Debug)]
pub enum Shape {
    /// Struct or tuple fields in declaration order, skipping fields without Drop.
    Fields(Range),
    /// Dispatch on the live enum variant before touching its fields.
    Variants(Range),
    /// Iterate in ascending index order, including zero-sized elements.
    Array { element: Id, length: u64 },
    /// Runtime element count comes from the selected wide reference.
    Slice { element: Id },
}
/// Generic recipes require a concrete method selection before emission.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Method {
    Definition(DefId),
    Unresolved(DefId),
    Instance(Instance),
}
#[derive(Clone, Copy)]
pub struct Recipe {
    /// A layout representative. Nominal recipes erase lifetime arguments only.
    pub ty: TypeId,
    /// Runs before the fields/elements selected by `kind`.
    pub method: Option<Method>,
    pub kind: Shape,
}
#[derive(Clone, Copy)]
pub struct Field {
    pub selection: Selection,
    pub recipe: Id,
}
#[derive(Clone, Copy)]
pub struct Variant {
    pub selection: VariantId,
    pub fields: Range,
}
/// Immutable recipes retained by the completed destruction proof.
#[derive(Default)]
pub struct Recipes {
    entries: Vec<Recipe>,
    fields: Vec<Field>,
    variants: Vec<Variant>,
    exposed: crate::support::hash::HashMap<TypeId, Option<Id>>,
}
impl Recipes {
    /// The selected element must share the catalog's exact canonical recipe key.
    pub(crate) fn slice(&self, types: &Store, id: Id) -> Result<(TypeId, Id), &'static str> {
        let recipe = self.get(id).ok_or("missing slice cleanup recipe")?;
        let (Some(Kind::Slice(element)), Shape::Slice { element: child }) =
            (types.kind(recipe.ty), recipe.kind)
        else {
            return Err("slice cleanup requires its exact slice recipe type");
        };
        let child_type = self.get(child).ok_or("missing slice cleanup child")?.ty;
        if recipe.method.is_some() || recipe_key(types, element)? != recipe_key(types, child_type)?
        {
            return Err("slice cleanup child belongs to another selected element");
        }
        Ok((element, child))
    }
    pub(crate) fn exposed(&self, ty: TypeId) -> Result<Option<Id>, &'static str> {
        self.exposed
            .get(&ty)
            .copied()
            .ok_or("object table lacks a proved destruction recipe")
    }
    #[must_use]
    pub fn recipes(&self) -> &[Recipe] {
        &self.entries
    }
    #[must_use]
    pub fn get(&self, id: Id) -> Option<&Recipe> {
        self.entries.get(id.0 as usize)
    }
    #[must_use]
    pub fn fields(&self, range: Range) -> Option<&[Field]> {
        self.fields.get(range.start as usize..range.end as usize)
    }
    #[must_use]
    pub fn variants(&self, range: Range) -> Option<&[Variant]> {
        self.variants.get(range.start as usize..range.end as usize)
    }
}
/// One immutable semantic environment and custom-destructor set. Completed
/// recipes are shared across every cleanup site and every function in the crate.
pub struct Catalog<'a> {
    types: &'a Store,
    definitions: &'a Definitions,
    members: &'a Members,
    cache: Cache,
}
impl<'a> Catalog<'a> {
    /// Retain a requested concrete type's recipe for indirect destruction.
    pub(crate) fn expose(&mut self, ty: TypeId) -> Result<Option<Id>, &'static str> {
        if let Some(&recipe) = self.cache.storage.exposed.get(&ty) {
            return Ok(recipe);
        }
        let recipe = self.demand(ty)?;
        self.cache.retain::<(TypeId, Option<Id>)>(2)?;
        self.cache.storage.exposed.insert(ty, recipe);
        Ok(recipe)
    }
    /// # Errors
    /// Rejects duplicate selected destructors and oversized method tables.
    pub fn new(
        types: &'a Store,
        definitions: &'a Definitions,
        members: &'a Members,
        methods: &[Declaration],
    ) -> Result<Self, &'static str> {
        Ok(Cache::new(methods)?.attach(types, definitions, members))
    }
    #[must_use]
    pub fn into_recipes(self) -> Recipes {
        self.cache.into_recipes()
    }
    /// Release semantic borrows while retaining completed recipes and query work.
    #[must_use]
    pub fn into_cache(self) -> Cache {
        self.cache
    }
    #[must_use]
    pub fn recipes(&self) -> &[Recipe] {
        self.cache.storage.recipes()
    }
    #[must_use]
    pub const fn steps(&self) -> usize {
        self.cache.steps
    }
    pub(crate) const fn types(&self) -> &Store {
        self.types
    }
    pub(crate) fn custom(&self, item: ItemId) -> bool {
        self.cache.methods.contains_key(&item)
    }
    #[must_use]
    pub fn get(&self, id: Id) -> Option<&Recipe> {
        self.cache.storage.get(id)
    }
    #[must_use]
    pub fn fields(&self, range: Range) -> Option<&[Field]> {
        self.cache.storage.fields(range)
    }
    #[must_use]
    pub fn variants(&self, range: Range) -> Option<&[Variant]> {
        self.cache.storage.variants(range)
    }
    /// Demand a complete, bounded recipe. None means the type needs no destruction.
    /// Nominal lifetime instantiations share code; references own no referent.
    /// # Errors
    /// Rejects invalid type/member facts, by-value cycles, unsupported unsized
    /// ownership and resource exhaustion. Failed dependencies remain memoized.
    pub fn demand(&mut self, ty: TypeId) -> Result<Option<Id>, &'static str> {
        let key = self.key(ty)?;
        if let Some(State::Ready(_) | State::Failed(_)) = self.cache.states.get(&key) {
            return self.result(ty);
        }
        self.cache.work.push((ty, false));
        if let Err(message) = self.run() {
            for state in self.cache.states.values_mut() {
                if matches!(state, State::Active) {
                    *state = State::Failed(message);
                }
            }
            self.cache.work.clear();
            self.cache
                .states
                .entry(key)
                .or_insert(State::Failed(message));
            return Err(message);
        }
        self.result(ty)
    }
    fn key(&self, ty: TypeId) -> Result<Key, &'static str> {
        recipe_key(self.types, ty)
    }
    fn result(&self, ty: TypeId) -> Result<Option<Id>, &'static str> {
        match self.cache.states.get(&self.key(ty)?) {
            Some(State::Ready(recipe)) => Ok(*recipe),
            Some(State::Failed(message)) => Err(message),
            _ => Err("missing completed destruction recipe"),
        }
    }
    fn run(&mut self) -> Result<(), &'static str> {
        while let Some((ty, finish)) = self.cache.work.pop() {
            self.cache.charge(1)?;
            let ty = self.types.storage_head(ty)?;
            let key = self.key(ty)?;
            match self.cache.states.get(&key) {
                Some(State::Ready(_)) => continue,
                Some(State::Failed(message)) => return Err(message),
                Some(State::Active) if !finish => {
                    return Err("recursive by-value destruction recipe");
                }
                _ => {}
            }
            if finish {
                let recipe = self.finish(ty)?;
                self.cache.states.insert(key, State::Ready(Some(recipe)));
                continue;
            }
            if !self.cache.needs.demand(self.types, self.definitions, ty)? {
                self.cache.retain::<(Key, State)>(2)?;
                self.cache.states.insert(key, State::Ready(None));
                continue;
            }
            self.cache.retain::<(Key, State)>(2)?;
            self.cache.states.insert(key, State::Active);
            self.cache.work.push((ty, true));
            match self
                .types
                .kind(ty)
                .ok_or("invalid destruction recipe type")?
            {
                Kind::Array { element, .. } | Kind::Slice(element) => self.schedule(element)?,
                Kind::Tuple(list) => self.schedule_fields(list)?,
                Kind::Closure { environment, .. } => {
                    let Some(Kind::Tuple(list)) = self.types.kind(environment) else {
                        return Err("closure destruction requires a tuple environment");
                    };
                    self.schedule_fields(list)?;
                }
                Kind::Adt(item, _) => {
                    if self.definitions.destruction(item)
                        == Some(crate::types::aggregate::FieldDestruction::Suppressed)
                    {
                        continue;
                    }
                    for variant in self.definitions.shape(self.types, ty)?.variants().rev() {
                        self.cache.charge(1)?;
                        self.schedule_fields(variant.fields)?;
                    }
                }
                _ => return Err("unsized or unsupported owned destruction type"),
            }
        }
        Ok(())
    }
    fn schedule_fields(&mut self, list: crate::types::ListId) -> Result<(), &'static str> {
        for &child in self
            .types
            .list(list)
            .ok_or("invalid destruction tuple")?
            .iter()
            .rev()
        {
            self.schedule(child)?;
        }
        Ok(())
    }
    fn schedule(&mut self, ty: TypeId) -> Result<(), &'static str> {
        self.cache.charge(1)?;
        self.cache.work.push((ty, false));
        Ok(())
    }
    fn field(&mut self, ty: TypeId, selection: Selection) -> Result<(), &'static str> {
        self.cache.charge(1)?;
        if let Some(recipe) = self.result(ty)? {
            self.cache.retain::<Field>(1)?;
            self.cache.storage.fields.push(Field { selection, recipe });
        }
        Ok(())
    }
    fn finish(&mut self, ty: TypeId) -> Result<Id, &'static str> {
        let mut method = None;
        let kind = match self
            .types
            .kind(ty)
            .ok_or("invalid destruction recipe type")?
        {
            Kind::Array { element, length } => Shape::Array {
                element: self
                    .result(element)?
                    .ok_or("missing array element destruction")?,
                length,
            },
            Kind::Slice(element) => Shape::Slice {
                element: self
                    .result(element)?
                    .ok_or("missing slice element destruction")?,
            },
            Kind::Tuple(list) => self.tuple_fields(list)?,
            Kind::Closure { environment, .. } => {
                let Some(Kind::Tuple(list)) = self.types.kind(environment) else {
                    return Err("closure destruction requires a tuple environment");
                };
                self.tuple_fields(list)?
            }
            Kind::Adt(item, args) => {
                method = self.cache.methods.get(&item).copied().map(|method| {
                    if self.types.region_args(args).is_some() {
                        Method::Definition(method)
                    } else {
                        Method::Unresolved(method)
                    }
                });
                self.nominal(ty, item)?
            }
            _ => return Err("invalid completed destruction type"),
        };
        self.cache.retain::<Recipe>(1)?;
        let id = Id(compact(self.cache.storage.entries.len())?);
        self.cache.storage.entries.push(Recipe { ty, method, kind });
        Ok(id)
    }
    fn tuple_fields(&mut self, list: crate::types::ListId) -> Result<Shape, &'static str> {
        let start = compact(self.cache.storage.fields.len())?;
        for (index, &child) in self
            .types
            .list(list)
            .ok_or("invalid destruction tuple")?
            .iter()
            .enumerate()
        {
            self.field(child, Selection::Tuple(compact(index)?))?;
        }
        Ok(Shape::Fields(Range {
            start,
            end: compact(self.cache.storage.fields.len())?,
        }))
    }
    fn nominal(&mut self, ty: TypeId, item: ItemId) -> Result<Shape, &'static str> {
        if self.definitions.destruction(item)
            == Some(crate::types::aggregate::FieldDestruction::Suppressed)
        {
            let start = compact(self.cache.storage.fields.len())?;
            return Ok(Shape::Fields(Range { start, end: start }));
        }
        let shape = self.definitions.shape(self.types, ty)?;
        let definition = shape.definition;
        let ids = self.members.variants(item);
        if ids.len() != self.definitions.variants(definition).len() {
            return Err("destruction variant facts disagree");
        }
        let start = compact(self.cache.storage.variants.len())?;
        for (ordinal, &id) in ids.iter().enumerate() {
            self.cache.charge(1)?;
            let variant = self
                .members
                .variants
                .get(id.0 as usize)
                .ok_or("invalid destruction variant")?;
            if variant.owner != item || variant.ordinal as usize != ordinal {
                return Err("destruction variant order disagrees");
            }
            let expected = self
                .types
                .list(
                    shape
                        .variant(ordinal)
                        .ok_or("missing destruction variant shape")?
                        .fields,
                )
                .ok_or("invalid destruction definition fields")?;
            if variant.end.checked_sub(variant.start).map(|n| n as usize) != Some(expected.len()) {
                return Err("destruction field facts disagree");
            }
            let declared = self
                .types
                .list(self.definitions.variants(definition)[ordinal].fields)
                .ok_or("invalid destruction declaration fields")?;
            if declared.len() != expected.len() {
                return Err("destruction instance has different field coverage");
            }
            let from = compact(self.cache.storage.fields.len())?;
            for index in variant.start..variant.end {
                let field = self
                    .members
                    .fields
                    .get(index as usize)
                    .ok_or("invalid destruction field")?;
                if field.ty != declared[(index - variant.start) as usize] {
                    return Err("destruction field facts disagree");
                }
                self.field(
                    expected[(index - variant.start) as usize],
                    Selection::Nominal(FieldId(index)),
                )?;
            }
            let fields = Range {
                start: from,
                end: compact(self.cache.storage.fields.len())?,
            };
            self.cache.retain::<Variant>(1)?;
            self.cache.storage.variants.push(Variant {
                selection: id,
                fields,
            });
        }
        if definition.kind == crate::types::aggregate::Kind::Struct {
            let variant = self
                .cache
                .storage
                .variants
                .get(start as usize)
                .ok_or("missing struct destruction fields")?;
            Ok(Shape::Fields(variant.fields))
        } else {
            Ok(Shape::Variants(Range {
                start,
                end: compact(self.cache.storage.variants.len())?,
            }))
        }
    }
}
fn compact(value: usize) -> Result<u32, &'static str> {
    u32::try_from(value).map_err(|_| "too many destruction recipe entries")
}
fn recipe_key(types: &Store, ty: TypeId) -> Result<Key, &'static str> {
    let ty = types.storage_head(ty)?;
    types
        .kind(ty)
        .map(|kind| match kind {
            Kind::Adt(item, args) if types.region_args(args).is_some() => Key::Nominal(item),
            _ => Key::Structural(ty),
        })
        .ok_or("invalid destruction recipe type")
}
