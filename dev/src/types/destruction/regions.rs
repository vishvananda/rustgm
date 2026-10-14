//! Drop checking follows owned fields; references do not own their referents.
use crate::{
    support::intern_slice::Slices,
    types::{ItemId, Kind, RegionId, Store, TypeId, aggregate::Definitions},
};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Key {
    Type(TypeId),
    /// Field templates retain declaration-bound lifetime slots.
    Nominal(ItemId),
}
#[derive(Clone, Copy)]
enum State {
    Active,
    Ready(u32),
    Failed(&'static str),
}
/// One immutable type/definition environment and custom-destructor set. Nominal
/// templates are analyzed once; uses substitute only their required lifetime slots.
/// This profile has no may-dangle opt-out: custom Drop requires every argument.
pub struct Regions {
    custom: HashSet<ItemId>,
    states: HashMap<Key, State>,
    lists: Slices<RegionId>,
    work: Vec<(Key, bool)>,
    retained: usize,
    pub steps: usize,
}
impl Regions {
    /// # Errors
    /// Rejects an oversized selected-destructor set.
    pub fn new(custom: impl IntoIterator<Item = ItemId>) -> Result<Self, &'static str> {
        let mut query = Self {
            custom: HashSet::new(),
            states: HashMap::new(),
            lists: Slices::default(),
            work: Vec::new(),
            retained: 0,
            steps: 0,
        };
        for item in custom {
            query.charge(1)?;
            query.retain(2 * size_of::<ItemId>())?;
            query.custom.insert(item);
        }
        Ok(query)
    }
    /// Non-static regions needed to destroy this type, independent of runtime
    /// field pruning. The original type supplies actual arguments, never an erased
    /// glue representative. Results are deterministic and shared canonical slices.
    /// # Errors
    /// Rejects invalid identities, recursive owned requirements and resource limits.
    pub fn get(
        &mut self,
        types: &Store,
        definitions: &Definitions,
        ty: TypeId,
    ) -> Result<&[RegionId], &'static str> {
        let key = Key::Type(ty);
        if !matches!(
            self.states.get(&key),
            Some(State::Ready(_) | State::Failed(_))
        ) {
            self.work.push((key, false));
            if let Err(message) = self.run(types, definitions) {
                for state in self.states.values_mut() {
                    if matches!(state, State::Active) {
                        *state = State::Failed(message);
                    }
                }
                self.work.clear();
                self.states.entry(key).or_insert(State::Failed(message));
                return Err(message);
            }
        }
        self.result(key)
    }
    fn result(&self, key: Key) -> Result<&[RegionId], &'static str> {
        match self.states.get(&key) {
            Some(State::Ready(id)) => self.lists.get(*id).ok_or("invalid drop region list"),
            Some(State::Failed(message)) => Err(message),
            _ => Err("missing drop region dependency"),
        }
    }
    fn charge(&mut self, cost: usize) -> Result<(), &'static str> {
        self.steps = self
            .steps
            .checked_add(cost)
            .filter(|n| *n <= 1_000_000)
            .ok_or("drop region query work budget exceeded")?;
        Ok(())
    }
    fn retain(&mut self, bytes: usize) -> Result<(), &'static str> {
        self.retained = self
            .retained
            .checked_add(bytes)
            .filter(|n| *n <= 64_000_000)
            .ok_or("drop region query exceeds 64 MB budget")?;
        Ok(())
    }
    fn run(&mut self, types: &Store, definitions: &Definitions) -> Result<(), &'static str> {
        while let Some((key, finish)) = self.work.pop() {
            self.charge(1)?;
            match self.states.get(&key) {
                Some(State::Ready(_)) => continue,
                Some(State::Failed(message)) => return Err(message),
                Some(State::Active) if !finish => {
                    return Err("recursive owned drop region requirement");
                }
                _ => {}
            }
            if finish {
                let mut values = self.finish(types, definitions, key)?;
                self.publish(key, &mut values)?;
                continue;
            }
            self.retain(2 * size_of::<(Key, State)>())?;
            self.states.insert(key, State::Active);
            self.work.push((key, true));
            dependencies(types, definitions, &self.custom, key, |child| {
                self.steps = self
                    .steps
                    .checked_add(1)
                    .filter(|n| *n <= 1_000_000)
                    .ok_or("drop region query work budget exceeded")?;
                self.work.push((child, false));
                Ok(())
            })?;
        }
        Ok(())
    }
    fn finish(
        &mut self,
        types: &Store,
        definitions: &Definitions,
        key: Key,
    ) -> Result<Vec<RegionId>, &'static str> {
        let mut values = Vec::new();
        if let Key::Type(ty) = key
            && let Some(Kind::Adt(item, args)) = types.kind(ty)
        {
            let arguments = types
                .region_args(args)
                .ok_or("invalid drop region arguments")?;
            if self.custom.contains(&item) {
                self.append(&mut values, arguments)?;
                return Ok(values);
            }
            let count = self.result(Key::Nominal(item))?.len();
            self.charge(count)?;
            for slot in self.result(Key::Nominal(item))? {
                let Some((0, index)) = slot.bound_parts() else {
                    return Err("drop field template has a foreign lifetime");
                };
                values.push(
                    *arguments
                        .get(index as usize)
                        .ok_or("missing drop lifetime argument")?,
                );
            }
        } else {
            self.collect(types, definitions, key, &mut values)?;
        }
        Ok(values)
    }
    fn append(
        &mut self,
        values: &mut Vec<RegionId>,
        regions: &[RegionId],
    ) -> Result<(), &'static str> {
        self.charge(regions.len())?;
        values.extend_from_slice(regions);
        Ok(())
    }
    fn collect(
        &mut self,
        types: &Store,
        definitions: &Definitions,
        key: Key,
        values: &mut Vec<RegionId>,
    ) -> Result<(), &'static str> {
        let mut cost = 0_usize;
        dependencies(types, definitions, &self.custom, key, |child| {
            let regions = self.result(child)?;
            cost = cost
                .checked_add(regions.len() + 1)
                .filter(|n| *n <= 1_000_000)
                .ok_or("drop region union work budget exceeded")?;
            values.extend_from_slice(regions);
            Ok(())
        })?;
        self.charge(cost)
    }
    fn publish(&mut self, key: Key, values: &mut Vec<RegionId>) -> Result<(), &'static str> {
        self.charge(
            values
                .len()
                .saturating_mul(values.len().max(1).ilog2() as usize + 2),
        )?;
        values.retain(|r| *r != RegionId::STATIC);
        values.sort_unstable_by_key(|r| r.0);
        values.dedup();
        self.retain(
            values
                .len()
                .saturating_mul(size_of::<RegionId>())
                .saturating_add(64),
        )?;
        let list = self.lists.intern(values)?;
        self.states.insert(key, State::Ready(list));
        Ok(())
    }
}
fn dependencies(
    types: &Store,
    definitions: &Definitions,
    custom: &HashSet<ItemId>,
    key: Key,
    mut visit: impl FnMut(Key) -> Result<(), &'static str>,
) -> Result<(), &'static str> {
    match key {
        Key::Nominal(item) => {
            let definition = definitions
                .get(item)
                .ok_or("missing drop region definition")?;
            for variant in definitions.variants(definition) {
                for &ty in types
                    .list(variant.fields)
                    .ok_or("invalid drop region fields")?
                {
                    visit(Key::Type(ty))?;
                }
            }
        }
        Key::Type(ty) => match types.kind(ty).ok_or("invalid drop region type")? {
            Kind::Parameter(_) => {
                return Err("generic drop checking requires a parameter environment");
            }
            Kind::Adt(item, _) => {
                definitions
                    .get(item)
                    .ok_or("missing drop region definition")?;
                if !custom.contains(&item) {
                    visit(Key::Nominal(item))?;
                }
            }
            Kind::Array { length: 0, .. } => {}
            Kind::Array { element, .. } | Kind::Slice(element) => visit(Key::Type(element))?,
            Kind::Tuple(list) => {
                for &ty in types.list(list).ok_or("invalid drop region tuple")? {
                    visit(Key::Type(ty))?;
                }
            }
            _ => {}
        },
    }
    Ok(())
}
