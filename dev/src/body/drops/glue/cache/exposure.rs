//! One lazy exact exposure family, with a shared query/recipe capacity budget.
use super::{Cache, Key, State};
use crate::{
    body::drops::glue::{Field, Recipe, Recipes, Variant},
    support::{hash::HashMap, retained},
    types::{Store, TypeId, aggregate::Definitions, destruction::NeedsDrop},
};
const LIMIT: usize = 64_000_000;
const WORK: usize = 1_000_000;

impl Cache {
    pub(crate) fn exposed_empty(
        types: &mut Store,
        definitions: &mut Definitions,
    ) -> Result<Self, &'static str> {
        let needs = NeedsDrop::for_exposure(types, definitions, LIMIT - size_of::<Self>())?;
        let mut cache = Self {
            methods: HashMap::default(),
            needs,
            states: HashMap::default(),
            storage: Recipes::default(),
            work: Vec::new(),
            retained: 0,
            steps: 0,
        };
        cache.reserve_exposure(0)?;
        Ok(cache)
    }
    pub(in crate::body::drops::glue) fn require_environment(
        &self,
        types: &Store,
        definitions: &Definitions,
    ) -> Result<(), &'static str> {
        let exact = self.needs.catalogue_bytes()?.is_some();
        self.needs
            .require_catalogue_environment(types, definitions)?;
        if exact && self.steps > WORK {
            return Err("exact exposure cache previously failed");
        }
        Ok(())
    }
    pub(in crate::body::drops::glue) fn requirement(
        &mut self,
        types: &Store,
        definitions: &Definitions,
        root: TypeId,
    ) -> Result<bool, &'static str> {
        let remaining = if self.needs.catalogue_bytes()?.is_some() {
            LIMIT
                .checked_sub(self.exposure_bytes(0))
                .ok_or("exposure recipes exhausted query storage")?
        } else {
            LIMIT
        };
        self.needs
            .demand_catalogue(types, definitions, root, remaining)
    }
    pub(in crate::body::drops::glue) fn push_work(
        &mut self,
        value: (TypeId, bool),
    ) -> Result<(), &'static str> {
        if self.needs.catalogue_bytes()?.is_some() {
            self.reserve_exposure(1)?;
        }
        self.work.push(value);
        Ok(())
    }
    pub(in crate::body::drops::glue) fn fail(&mut self, key: Key, message: &'static str) {
        let exact = matches!(self.needs.catalogue_bytes(), Ok(Some(_)));
        if exact {
            self.steps = self.steps.saturating_add(self.states.len()).max(WORK + 1);
        }
        for state in self.states.values_mut() {
            if matches!(state, State::Active) {
                *state = State::Failed(message);
            }
        }
        self.work.clear();
        if let Some(state) = self.states.get_mut(&key) {
            *state = State::Failed(message);
        } else if !exact {
            self.states.insert(key, State::Failed(message));
        }
    }
    pub(super) fn reserve_exposure(&mut self, extra: usize) -> Result<(), &'static str> {
        let bytes = retained::sum([
            self.exposure_bytes(extra),
            self.needs.catalogue_bytes()?.unwrap_or(0),
        ]);
        if bytes > LIMIT {
            self.steps = self.steps.max(WORK + 1);
            return Err("exact exposure recipes exceed 64 MB budget");
        }
        self.retained = self.retained.max(bytes);
        Ok(())
    }
    fn exposure_bytes(&self, extra: usize) -> usize {
        retained::sum([
            size_of::<Self>(),
            retained::map(&self.methods),
            map_growth::<Key, State>(self.states.len(), self.states.capacity(), extra),
            vector_growth::<Recipe>(
                self.storage.entries.len(),
                self.storage.entries.capacity(),
                extra,
            ),
            vector_growth::<Field>(
                self.storage.fields.len(),
                self.storage.fields.capacity(),
                extra,
            ),
            vector_growth::<Variant>(
                self.storage.variants.len(),
                self.storage.variants.capacity(),
                extra,
            ),
            map_growth::<TypeId, Option<super::super::Id>>(
                self.storage.exposed.len(),
                self.storage.exposed.capacity(),
                extra,
            ),
            vector_growth::<(TypeId, bool)>(self.work.len(), self.work.capacity(), extra),
        ])
    }
}
fn map_growth<K, V>(len: usize, capacity: usize, extra: usize) -> usize {
    let need = len.saturating_add(extra);
    let slots = if need > capacity {
        capacity.saturating_add(need.saturating_mul(4))
    } else {
        capacity
    };
    slots.saturating_mul(size_of::<(K, V)>())
}
fn vector_growth<T>(len: usize, capacity: usize, extra: usize) -> usize {
    let need = len.saturating_add(extra);
    let slots = if need > capacity {
        capacity.saturating_add(capacity.saturating_mul(2).max(4).max(need))
    } else {
        capacity
    };
    slots.saturating_mul(size_of::<T>())
}
