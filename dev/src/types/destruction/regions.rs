//! Drop checking follows owned fields; references do not own their referents.
use crate::types::{
    ItemId, Kind, RegionId, Store, TypeId,
    aggregate::Definitions,
    outlives::{
        Parts, TypeComponents,
        storage::{Lists, Values},
    },
    parameters::environment::{BuiltinQuery, Environment},
};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

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
    custom: Arc<HashSet<ItemId>>,
    environment: Option<Arc<Environment>>,
    builtins: Option<Box<BuiltinQuery>>,
    states: HashMap<Key, State>,
    lists: Lists,
    work: Vec<(Key, bool)>,
    retained: usize,
    components: Option<Box<TypeComponents>>,
    pub steps: usize,
}
impl Regions {
    /// # Errors
    /// Rejects an oversized selected-destructor set.
    pub fn new(custom: impl IntoIterator<Item = ItemId>) -> Result<Self, &'static str> {
        let mut query = Self {
            custom: Arc::default(),
            environment: None,
            builtins: None,
            states: HashMap::new(),
            lists: Lists::default(),
            work: Vec::new(),
            retained: 0,
            components: None,
            steps: 0,
        };
        for item in custom {
            query.charge(1)?;
            query.retain(2 * size_of::<ItemId>())?;
            Arc::make_mut(&mut query.custom).insert(item);
        }
        Ok(query)
    }
    /// Fresh query caches for one immutable parameter environment.
    #[must_use]
    pub fn scoped(&self, environment: Arc<Environment>) -> Self {
        Self {
            custom: Arc::clone(&self.custom),
            environment: Some(environment),
            builtins: None,
            states: HashMap::new(),
            lists: Lists::default(),
            work: Vec::new(),
            retained: 0,
            components: None,
            steps: 0,
        }
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
        if types.contains_parameters(ty) == Some(true) {
            return Err("generic drop checking requires symbolic components");
        }
        let parts = self.symbolic(types, definitions, ty)?;
        if !parts.parameters.is_empty() {
            return Err("closed drop type contains opaque field components");
        }
        Ok(parts.regions)
    }
    /// Preserve opaque owned parameters alongside named destructor lifetimes.
    /// # Errors
    /// Rejects foreign parameters, unsupported nominal fields, cycles and budgets.
    pub fn symbolic(
        &mut self,
        types: &Store,
        definitions: &Definitions,
        ty: TypeId,
    ) -> Result<Parts<'_>, &'static str> {
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
    fn result(&self, key: Key) -> Result<Parts<'_>, &'static str> {
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
            if let Key::Type(ty) = key {
                let kind = types.kind(ty).ok_or("invalid drop region type")?;
                if let Some(environment) = &self.environment {
                    let assumed = environment.assumed_with(types, ty, &mut self.builtins)?;
                    let copy = if let Kind::Parameter(id) = kind {
                        environment
                            .abilities(id)
                            .ok_or("drop parameter is outside its environment")?
                            .copy
                            || assumed.copy
                    } else {
                        assumed.copy
                    };
                    if copy || matches!(kind, Kind::Parameter(_)) {
                        let mut values = Values::default();
                        if !copy && let Kind::Parameter(id) = kind {
                            values.parameters.push(id);
                        }
                        self.retain(2 * size_of::<(Key, State)>())?;
                        self.publish(key, &mut values)?;
                        continue;
                    }
                } else if matches!(kind, Kind::Parameter(_) | Kind::ArrayTerm { .. }) {
                    return Err("generic drop checking requires a parameter environment");
                }
                let symbolic_result = match kind {
                    Kind::Projection(application) => !types
                        .projection_schema(application.item)
                        .is_some_and(|schema| schema.copies.copy),
                    Kind::Opaque(id, _) => {
                        self.environment.is_some()
                            && !types
                                .opaque_declaration(id)
                                .is_some_and(|declaration| declaration.copies.copy)
                    }
                    _ => false,
                };
                if symbolic_result {
                    if let Some(environment) = &self.environment {
                        let components = self.components.get_or_insert_with(Default::default);
                        let parts = components.live_value(types, ty)?;
                        if parts
                            .parameters
                            .iter()
                            .any(|p| environment.abilities(*p).is_none())
                        {
                            return Err("drop projection input is outside its environment");
                        }
                        let mut values = Values::default();
                        values.extend(parts);
                        self.charge(values.parts().len())?;
                        self.retain(2 * size_of::<(Key, State)>())?;
                        self.publish(key, &mut values)?;
                        continue;
                    }
                    return Err("associated type drop regions require normalization");
                }
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
    ) -> Result<Values, &'static str> {
        let mut values = Values::default();
        if let Key::Type(ty) = key
            && matches!(types.kind(ty), Some(Kind::Dynamic { .. }))
        {
            let components = self.components.get_or_insert_with(Default::default);
            values.extend(components.drop_inputs(types, ty)?);
            self.charge(values.parts().len())?;
            return Ok(values);
        }
        if let Key::Type(ty) = key
            && let Some(Kind::Adt(item, args)) = types.kind(ty)
        {
            if types.region_args(args).is_none() {
                if self.custom.contains(&item) {
                    let components = self.components.get_or_insert_with(Default::default);
                    let parts = components.drop_inputs(types, ty)?;
                    if let Some(environment) = &self.environment {
                        if parts
                            .parameters
                            .iter()
                            .any(|p| environment.abilities(*p).is_none())
                        {
                            return Err("drop parameter is outside its environment");
                        }
                    } else if !parts.parameters.is_empty() {
                        return Err("generic drop checking requires a parameter environment");
                    }
                    values.extend(parts);
                    self.charge(values.parts().len())?;
                } else {
                    self.collect(types, definitions, key, &mut values)?;
                }
                return Ok(values);
            }
            let arguments = types
                .region_args(args)
                .ok_or("invalid drop region arguments")?;
            if self.custom.contains(&item) {
                self.charge(arguments.len())?;
                values.regions.extend_from_slice(arguments);
                return Ok(values);
            }
            let count = self.result(Key::Nominal(item))?.len();
            self.charge(count)?;
            let parts = self.result(Key::Nominal(item))?;
            if !parts.parameters.is_empty() {
                return Err("generic drop checking requires instantiated fields");
            }
            for slot in parts.regions {
                let Some((0, index)) = slot.bound_parts() else {
                    return Err("drop field template has a foreign lifetime");
                };
                values.regions.push(
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
    fn collect(
        &mut self,
        types: &Store,
        definitions: &Definitions,
        key: Key,
        values: &mut Values,
    ) -> Result<(), &'static str> {
        let mut cost = 0_usize;
        dependencies(types, definitions, &self.custom, key, |child| {
            let regions = self.result(child)?;
            cost = cost
                .checked_add(regions.len() + 1)
                .filter(|n| *n <= 1_000_000)
                .ok_or("drop region union work budget exceeded")?;
            values.extend(regions);
            Ok(())
        })?;
        self.charge(cost)
    }
    fn publish(&mut self, key: Key, values: &mut Values) -> Result<(), &'static str> {
        self.charge(
            values
                .parts()
                .len()
                .saturating_mul(values.parts().len().max(1).ilog2() as usize + 2),
        )?;
        values.canonicalize();
        self.retain(
            values
                .parts()
                .len()
                .saturating_mul(size_of::<RegionId>())
                .saturating_add(if values.parameters.is_empty() {
                    64
                } else {
                    192
                }),
        )?;
        let list = self.lists.intern(values.parts())?;
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
            fields(
                types,
                definitions.variants(definition).iter().copied(),
                &mut visit,
            )?;
        }
        Key::Type(ty) => match types.kind(ty).ok_or("invalid drop region type")? {
            Kind::Opaque(id, _) => {
                if !types
                    .opaque_declaration(id)
                    .is_some_and(|declaration| declaration.copies.copy)
                {
                    visit(Key::Type(types.storage_head(ty)?))?;
                }
            }
            Kind::Parameter(_) => {
                return Err("generic drop checking requires a parameter environment");
            }
            Kind::Adt(item, args) => {
                definitions
                    .get(item)
                    .ok_or("missing drop region definition")?;
                if types.region_args(args).is_none() {
                    definitions.shape(types, ty)?;
                }
                if !custom.contains(&item) {
                    if types.region_args(args).is_some() {
                        visit(Key::Nominal(item))?;
                    } else {
                        fields(types, definitions.shape(types, ty)?.variants(), &mut visit)?;
                    }
                }
            }
            Kind::Array { length: 0, .. } => {}
            Kind::Array { element, .. }
            | Kind::ArrayTerm { element, .. }
            | Kind::Slice(element) => visit(Key::Type(element))?,
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

fn fields(
    types: &Store,
    variants: impl Iterator<Item = crate::types::aggregate::Variant>,
    visit: &mut impl FnMut(Key) -> Result<(), &'static str>,
) -> Result<(), &'static str> {
    for variant in variants {
        for &ty in types
            .list(variant.fields)
            .ok_or("invalid drop region fields")?
        {
            visit(Key::Type(ty))?;
        }
    }
    Ok(())
}
