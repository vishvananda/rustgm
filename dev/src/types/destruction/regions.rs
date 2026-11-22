//! Drop checking follows owned fields; references do not own their referents.
mod cycles;
use crate::support::hash::{HashMap, HashSet};
use crate::types::{
    GenericArgs, ItemId, Kind, RegionId, Store, TypeId,
    aggregate::Definitions,
    arguments::Argument,
    outlives::{
        Parts, TypeComponents,
        storage::{Lists, Values},
    },
    parameters::environment::{BuiltinQuery, Environment},
};
use std::sync::Arc;

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
/// A checked destructor can relax individual arguments while preserving owned field requirements.
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
    pub(crate) fn has_destructor(&self, item: ItemId) -> bool {
        self.custom.contains(&item)
    }

    /// # Errors
    /// Rejects an oversized selected-destructor set.
    pub fn new(custom: impl IntoIterator<Item = ItemId>) -> Result<Self, &'static str> {
        let mut query = Self {
            custom: Arc::default(),
            environment: None,
            builtins: None,
            states: HashMap::default(),
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
            states: HashMap::default(),
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
            let result = self.run(types, definitions).and_then(|finished| {
                if finished {
                    Ok(())
                } else {
                    self.clear_active(None);
                    self.resolve_cycles(types, definitions, key)
                }
            });
            if let Err(message) = result {
                self.clear_active(Some(message));
                self.states.entry(key).or_insert(State::Failed(message));
                return Err(message);
            }
        }
        self.result(key)
    }
    fn clear_active(&mut self, failure: Option<&'static str>) {
        for (key, finish) in self.work.drain(..) {
            if finish && let Some(state @ State::Active) = self.states.get_mut(&key) {
                if let Some(message) = failure {
                    *state = State::Failed(message);
                } else {
                    self.states.remove(&key);
                }
            }
        }
    }
    fn requirements<'a>(
        &'a self,
        key: Key,
        cycle: Option<&'a cycles::Graph>,
    ) -> Result<Parts<'a>, &'static str> {
        match self.states.get(&key) {
            Some(State::Ready(id)) => self.lists.get(*id).ok_or("invalid drop region list"),
            Some(State::Failed(message)) => Err(message),
            _ => cycle
                .and_then(|graph| graph.parts(key))
                .ok_or("missing drop region dependency"),
        }
    }
    fn result(&self, key: Key) -> Result<Parts<'_>, &'static str> {
        self.requirements(key, None)
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
    fn run(&mut self, types: &Store, definitions: &Definitions) -> Result<bool, &'static str> {
        while let Some(&(key, finish)) = self.work.last() {
            self.charge(1)?;
            self.work.pop();
            match self.states.get(&key) {
                Some(State::Ready(_)) => continue,
                Some(State::Failed(message)) => return Err(message),
                Some(State::Active) if !finish => {
                    return Ok(false);
                }
                _ => {}
            }
            if finish {
                let built = self
                    .finish(types, definitions, key, None)
                    .and_then(|mut values| self.publish(key, &mut values));
                if built.is_err() {
                    self.work.push((key, true));
                }
                built?;
                continue;
            }
            if let Some(mut values) = self.terminal(types, definitions, key)? {
                self.retain(2 * size_of::<(Key, State)>())?;
                self.publish(key, &mut values)?;
                continue;
            }
            self.retain(2 * size_of::<(Key, State)>())?;
            self.states.insert(key, State::Active);
            self.work.push((key, true));
            dependencies(types, definitions, &self.custom, key, |child, _| {
                self.steps = self
                    .steps
                    .checked_add(1)
                    .filter(|n| *n <= 1_000_000)
                    .ok_or("drop region query work budget exceeded")?;
                self.work.push((child, false));
                Ok(())
            })?;
        }
        Ok(true)
    }
    fn terminal(
        &mut self,
        types: &Store,
        definitions: &Definitions,
        key: Key,
    ) -> Result<Option<Values>, &'static str> {
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
                    return Ok(Some(values));
                }
            } else if matches!(kind, Kind::Parameter(_) | Kind::ArrayTerm { .. }) {
                return Err("generic drop checking requires a parameter environment");
            }
            if matches!(kind, Kind::Adt(item, _) if !self.custom.contains(&item)
            && definitions.destruction(item) == Some(crate::types::aggregate::FieldDestruction::Suppressed))
            {
                return Ok(Some(Values::default()));
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
                    return Ok(Some(values));
                }
                return Err("associated type drop regions require normalization");
            }
        }
        Ok(None)
    }
    fn argument_inputs(&mut self, types: &Store, ty: TypeId) -> Result<Values, &'static str> {
        let components = self.components.get_or_insert_with(Default::default);
        let parts = components.drop_inputs(types, ty, self.environment.is_some())?;
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
        let mut values = Values::default();
        values.extend(parts);
        self.charge(values.parts().len())?;
        Ok(values)
    }
    fn unrelaxed_inputs(
        &mut self,
        types: &Store,
        arguments: GenericArgs,
        relaxed: &[u32],
        values: &mut Values,
    ) -> Result<(), &'static str> {
        let arguments = types.arguments(arguments).ok_or("invalid drop arguments")?;
        self.charge(arguments.len())?;
        for (index, argument) in (0_u32..).zip(arguments) {
            if relaxed.binary_search(&index).is_ok() {
                continue;
            }
            match argument {
                Argument::Lifetime(region) => values.regions.push(region),
                Argument::Type(ty) => values.extend(self.argument_inputs(types, ty)?.parts()),
                Argument::Const(_) => {}
            }
        }
        Ok(())
    }
    fn finish(
        &mut self,
        types: &Store,
        definitions: &Definitions,
        key: Key,
        cycle: Option<&cycles::Graph>,
    ) -> Result<Values, &'static str> {
        let mut values = Values::default();
        if let Key::Type(ty) = key
            && matches!(types.kind(ty), Some(Kind::Dynamic { .. }))
        {
            return self.argument_inputs(types, ty);
        }
        if let Key::Type(ty) = key
            && let Some(Kind::Adt(item, args)) = types.kind(ty)
        {
            let custom = self.custom.contains(&item);
            let relaxed = if custom {
                definitions.drop_eyepatches(item)
            } else {
                &[]
            };
            if custom && !relaxed.is_empty() {
                self.unrelaxed_inputs(types, args, relaxed, &mut values)?;
            }
            if types.region_args(args).is_none() {
                if custom && relaxed.is_empty() {
                    return self.argument_inputs(types, ty);
                }
                self.collect(types, definitions, key, cycle, &mut values)?;
                return Ok(values);
            }
            let arguments = types
                .region_args(args)
                .ok_or("invalid drop region arguments")?;
            if custom && relaxed.is_empty() {
                self.charge(arguments.len())?;
                values.regions.extend_from_slice(arguments);
                return Ok(values);
            }
            let count = self.requirements(Key::Nominal(item), cycle)?.len();
            self.charge(count)?;
            let parts = self.requirements(Key::Nominal(item), cycle)?;
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
            self.collect(types, definitions, key, cycle, &mut values)?;
        }
        Ok(values)
    }
    fn collect(
        &mut self,
        types: &Store,
        definitions: &Definitions,
        key: Key,
        cycle: Option<&cycles::Graph>,
        values: &mut Values,
    ) -> Result<(), &'static str> {
        let mut cost = 0_usize;
        dependencies(types, definitions, &self.custom, key, |child, _| {
            let regions = self.requirements(child, cycle)?;
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
    mut visit: impl FnMut(Key, bool) -> Result<(), &'static str>,
) -> Result<(), &'static str> {
    match key {
        Key::Nominal(item) => {
            let definition = definitions
                .get(item)
                .ok_or("missing drop region definition")?;
            if definitions.destruction(item)
                != Some(crate::types::aggregate::FieldDestruction::Suppressed)
            {
                fields(
                    types,
                    definitions.variants(definition).iter().copied(),
                    &mut visit,
                )?;
            }
        }
        Key::Type(ty) => match types.kind(ty).ok_or("invalid drop region type")? {
            Kind::Future {
                identity,
                arguments,
            } => {
                for &field in types
                    .list(types.future_application(identity, arguments)?.captures)
                    .ok_or("missing future drop captures")?
                {
                    visit(Key::Type(field), false)?;
                }
            }
            Kind::Closure { environment, .. } => visit(Key::Type(environment), false)?,
            Kind::Opaque(id, _) => {
                if !types
                    .opaque_declaration(id)
                    .is_some_and(|declaration| declaration.copies.copy)
                {
                    visit(Key::Type(types.storage_head(ty)?), false)?;
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
                let custom = custom.contains(&item);
                if !custom || !definitions.drop_eyepatches(item).is_empty() {
                    if custom && let Some(content) = definitions.box_content(types, ty)? {
                        visit(Key::Type(content), true)?;
                    }
                    if let Some(target) = definitions.phantom_argument(types, ty)? {
                        visit(Key::Type(target), true)?;
                        return Ok(());
                    }
                    if types.region_args(args).is_some() {
                        visit(Key::Nominal(item), false)?;
                    } else {
                        fields(types, definitions.shape(types, ty)?.variants(), &mut visit)?;
                    }
                }
            }
            Kind::Array { length: 0, .. } => {}
            Kind::Array { element, .. }
            | Kind::ArrayTerm { element, .. }
            | Kind::Pattern { base: element, .. }
            | Kind::Slice(element) => visit(Key::Type(element), false)?,
            Kind::Tuple(list) => {
                for &ty in types.list(list).ok_or("invalid drop region tuple")? {
                    visit(Key::Type(ty), false)?;
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
    visit: &mut impl FnMut(Key, bool) -> Result<(), &'static str>,
) -> Result<(), &'static str> {
    for variant in variants {
        for &ty in types
            .list(variant.fields)
            .ok_or("invalid drop region fields")?
        {
            visit(Key::Type(ty), false)?;
        }
    }
    Ok(())
}
