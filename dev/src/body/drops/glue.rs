//! Shared destructor/field recipes; array length never expands the construction work.
use crate::{
    body::drops::Method,
    resolve::model::DefId,
    sema::members::{FieldId, Members, Selection, VariantId},
    types::{ItemId, Kind, Store, TypeId, aggregate::Definitions, destruction::NeedsDrop},
};
use std::collections::HashMap;

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
}
#[derive(Clone, Copy)]
pub struct Recipe {
    /// A layout representative. Nominal recipes erase lifetime arguments only.
    pub ty: TypeId,
    /// Runs before the fields/elements selected by `kind`.
    pub method: Option<DefId>,
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Key {
    Nominal(ItemId),
    Structural(TypeId),
}
#[derive(Clone, Copy)]
enum State {
    Active,
    Ready(Option<Id>),
    Failed(&'static str),
}
/// Immutable recipes retained by the completed destruction proof.
#[derive(Default)]
pub struct Recipes {
    entries: Vec<Recipe>,
    fields: Vec<Field>,
    variants: Vec<Variant>,
}
impl Recipes {
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
    methods: HashMap<ItemId, DefId>,
    needs: NeedsDrop,
    states: HashMap<Key, State>,
    storage: Recipes,
    work: Vec<(TypeId, bool)>,
    retained: usize,
    pub steps: usize,
}
impl<'a> Catalog<'a> {
    /// # Errors
    /// Rejects duplicate selected destructors and oversized method tables.
    pub fn new(
        types: &'a Store,
        definitions: &'a Definitions,
        members: &'a Members,
        methods: &[Method],
    ) -> Result<Self, &'static str> {
        let retained = methods
            .len()
            .checked_mul(2 * (size_of::<(ItemId, DefId)>() + size_of::<ItemId>()))
            .filter(|n| *n <= 64_000_000)
            .ok_or("destruction methods exceed 64 MB budget")?;
        let mut selected = HashMap::new();
        for method in methods {
            if selected.insert(method.item, method.function).is_some() {
                return Err("duplicate destructor recipe method");
            }
        }
        let needs = NeedsDrop::new(selected.keys().copied());
        Ok(Self {
            types,
            definitions,
            members,
            methods: selected,
            needs,
            states: HashMap::new(),
            storage: Recipes::default(),
            work: Vec::new(),
            retained,
            steps: 0,
        })
    }
    #[must_use]
    pub fn into_recipes(self) -> Recipes {
        self.storage
    }
    #[must_use]
    pub fn recipes(&self) -> &[Recipe] {
        self.storage.recipes()
    }
    pub(crate) const fn types(&self) -> &Store {
        self.types
    }
    pub(crate) fn custom(&self, item: ItemId) -> bool {
        self.methods.contains_key(&item)
    }
    #[must_use]
    pub fn get(&self, id: Id) -> Option<&Recipe> {
        self.storage.get(id)
    }
    #[must_use]
    pub fn fields(&self, range: Range) -> Option<&[Field]> {
        self.storage.fields(range)
    }
    #[must_use]
    pub fn variants(&self, range: Range) -> Option<&[Variant]> {
        self.storage.variants(range)
    }
    /// Demand a complete, bounded recipe. None means the type needs no destruction.
    /// Nominal lifetime instantiations share code; references own no referent.
    /// # Errors
    /// Rejects invalid type/member facts, by-value cycles, unsupported unsized
    /// ownership and resource exhaustion. Failed dependencies remain memoized.
    pub fn demand(&mut self, ty: TypeId) -> Result<Option<Id>, &'static str> {
        let key = self.key(ty)?;
        if let Some(State::Ready(_) | State::Failed(_)) = self.states.get(&key) {
            return self.result(ty);
        }
        self.work.push((ty, false));
        if let Err(message) = self.run() {
            for state in self.states.values_mut() {
                if matches!(state, State::Active) {
                    *state = State::Failed(message);
                }
            }
            self.work.clear();
            self.states.entry(key).or_insert(State::Failed(message));
            return Err(message);
        }
        self.result(ty)
    }
    fn key(&self, ty: TypeId) -> Result<Key, &'static str> {
        if matches!(self.types.kind(ty), Some(Kind::Adt(_, args)) if self.types.region_args(args).is_none())
        {
            // The lifetime-only key below cannot identify type/const instances,
            // and method DefIds do not yet select specialized destructor bodies.
            return Err("generic destruction recipes require selected instances");
        }
        self.types
            .kind(ty)
            .map(|kind| match kind {
                Kind::Adt(item, _) => Key::Nominal(item),
                _ => Key::Structural(ty),
            })
            .ok_or("invalid destruction recipe type")
    }
    fn result(&self, ty: TypeId) -> Result<Option<Id>, &'static str> {
        match self.states.get(&self.key(ty)?) {
            Some(State::Ready(recipe)) => Ok(*recipe),
            Some(State::Failed(message)) => Err(message),
            _ => Err("missing completed destruction recipe"),
        }
    }
    fn charge(&mut self, count: usize) -> Result<(), &'static str> {
        self.steps = self
            .steps
            .checked_add(count)
            .filter(|n| *n <= 1_000_000)
            .ok_or("destruction recipe work budget exceeded")?;
        Ok(())
    }
    fn retain<T>(&mut self, count: usize) -> Result<(), &'static str> {
        self.retained = count
            .checked_mul(size_of::<T>())
            .and_then(|n| self.retained.checked_add(n))
            .filter(|n| *n <= 64_000_000)
            .ok_or("destruction recipes exceed 64 MB budget")?;
        Ok(())
    }
    fn run(&mut self) -> Result<(), &'static str> {
        while let Some((ty, finish)) = self.work.pop() {
            self.charge(1)?;
            let key = self.key(ty)?;
            match self.states.get(&key) {
                Some(State::Ready(_)) => continue,
                Some(State::Failed(message)) => return Err(message),
                Some(State::Active) if !finish => {
                    return Err("recursive by-value destruction recipe");
                }
                _ => {}
            }
            if finish {
                let recipe = self.finish(ty)?;
                self.states.insert(key, State::Ready(Some(recipe)));
                continue;
            }
            if !self.needs.demand(self.types, self.definitions, ty)? {
                self.retain::<(Key, State)>(2)?;
                self.states.insert(key, State::Ready(None));
                continue;
            }
            self.retain::<(Key, State)>(2)?;
            self.states.insert(key, State::Active);
            self.work.push((ty, true));
            match self
                .types
                .kind(ty)
                .ok_or("invalid destruction recipe type")?
            {
                Kind::Array { element, .. } => self.schedule(element)?,
                Kind::Tuple(list) => {
                    for &child in self
                        .types
                        .list(list)
                        .ok_or("invalid destruction tuple")?
                        .iter()
                        .rev()
                    {
                        self.schedule(child)?;
                    }
                }
                Kind::Adt(item, _) => {
                    for &id in self.members.variants(item).iter().rev() {
                        self.charge(1)?;
                        for field in self
                            .members
                            .fields(id)
                            .ok_or("invalid destruction fields")?
                            .iter()
                            .rev()
                        {
                            self.schedule(field.ty)?;
                        }
                    }
                }
                _ => return Err("unsized or unsupported owned destruction type"),
            }
        }
        Ok(())
    }
    fn schedule(&mut self, ty: TypeId) -> Result<(), &'static str> {
        self.charge(1)?;
        self.work.push((ty, false));
        Ok(())
    }
    fn field(&mut self, ty: TypeId, selection: Selection) -> Result<(), &'static str> {
        self.charge(1)?;
        if let Some(recipe) = self.result(ty)? {
            self.retain::<Field>(1)?;
            self.storage.fields.push(Field { selection, recipe });
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
            Kind::Tuple(list) => {
                let start = compact(self.storage.fields.len())?;
                for (index, &child) in self
                    .types
                    .list(list)
                    .ok_or("invalid destruction tuple")?
                    .iter()
                    .enumerate()
                {
                    self.field(child, Selection::Tuple(compact(index)?))?;
                }
                Shape::Fields(Range {
                    start,
                    end: compact(self.storage.fields.len())?,
                })
            }
            Kind::Adt(item, _) => {
                method = self.methods.get(&item).copied();
                self.nominal(item)?
            }
            _ => return Err("invalid completed destruction type"),
        };
        self.retain::<Recipe>(1)?;
        let id = Id(compact(self.storage.entries.len())?);
        self.storage.entries.push(Recipe { ty, method, kind });
        Ok(id)
    }
    fn nominal(&mut self, item: ItemId) -> Result<Shape, &'static str> {
        let definition = self
            .definitions
            .get(item)
            .ok_or("missing destruction definition")?;
        let ids = self.members.variants(item);
        if ids.len() != self.definitions.variants(definition).len() {
            return Err("destruction variant facts disagree");
        }
        let start = compact(self.storage.variants.len())?;
        for (ordinal, &id) in ids.iter().enumerate() {
            self.charge(1)?;
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
                .list(self.definitions.variants(definition)[ordinal].fields)
                .ok_or("invalid destruction definition fields")?;
            if variant.end.checked_sub(variant.start).map(|n| n as usize) != Some(expected.len()) {
                return Err("destruction field facts disagree");
            }
            let from = compact(self.storage.fields.len())?;
            for index in variant.start..variant.end {
                let field = self
                    .members
                    .fields
                    .get(index as usize)
                    .ok_or("invalid destruction field")?;
                if field.ty != expected[(index - variant.start) as usize] {
                    return Err("destruction field facts disagree");
                }
                self.field(field.ty, Selection::Nominal(FieldId(index)))?;
            }
            let fields = Range {
                start: from,
                end: compact(self.storage.fields.len())?,
            };
            self.retain::<Variant>(1)?;
            self.storage.variants.push(Variant {
                selection: id,
                fields,
            });
        }
        if definition.kind == crate::types::aggregate::Kind::Struct {
            let variant = self
                .storage
                .variants
                .get(start as usize)
                .ok_or("missing struct destruction fields")?;
            Ok(Shape::Fields(variant.fields))
        } else {
            Ok(Shape::Variants(Range {
                start,
                end: compact(self.storage.variants.len())?,
            }))
        }
    }
}
fn compact(value: usize) -> Result<u32, &'static str> {
    u32::try_from(value).map_err(|_| "too many destruction recipe entries")
}
