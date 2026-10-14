//! Copy/Clone obligations under fixed assumptions, independent of storage layout.
use super::parameters::environment::Environment;
use super::{ItemId, Kind, Primitive, Store, TypeId, aggregate::Definitions};
use std::{collections::HashMap, hash::BuildHasher, sync::Arc};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Properties {
    pub copy: bool,
    pub clone_: bool,
}
impl Properties {
    const BOTH: Self = Self {
        copy: true,
        clone_: true,
    };
    fn intersect(self, other: Self) -> Self {
        Self {
            copy: self.copy && other.copy,
            clone_: self.clone_ && other.clone_,
        }
    }
}
#[derive(Clone, Copy, Default)]
enum State {
    #[default]
    Unseen,
    Active,
    Ready(Properties),
    Failed(&'static str),
}
struct Scope {
    environment: Arc<Environment>,
    // A body typically touches only a few types in the whole crate's store.
    states: HashMap<TypeId, State>,
}
/// One canonical store and immutable completed definitions/derive selections.
/// Appending canonical types is permitted. Nominal types never gain Copy from layout.
/// A symbolic session also owns one immutable parameter environment. Neither
/// positive nor negative results can be reused under different assumptions.
#[derive(Default)]
pub struct Copies {
    scope: Option<Box<Scope>>,
    states: Vec<State>,
    work: Vec<(TypeId, bool)>,
    pub steps: usize,
}
impl Copies {
    #[must_use]
    pub fn in_environment(environment: Arc<Environment>) -> Self {
        Self {
            scope: Some(Box::new(Scope {
                environment,
                states: HashMap::new(),
            })),
            ..Self::default()
        }
    }
    /// # Errors
    /// Rejects invalid type graphs and by-value cycles; caches failed queries too.
    pub fn demand<S: BuildHasher>(
        &mut self,
        types: &Store,
        definitions: &Definitions,
        selected: &HashMap<ItemId, Properties, S>,
        ty: TypeId,
    ) -> Result<Properties, &'static str> {
        match self.state(ty) {
            State::Ready(value) => return Ok(value),
            State::Failed(message) => return Err(message),
            _ => {}
        }
        if self.scope.is_none() {
            self.states.resize(types.len(), State::Unseen);
        }
        self.work.clear();
        self.work.push((ty, false));
        let result = self.run(types, definitions, selected);
        if let Err(message) = result {
            // A failed dependency also fails every suspended parent, without
            // leaving active entries that poison independent later queries.
            self.fail_active(message);
            self.work.clear();
            return Err(message);
        }
        self.get(ty)
    }
    fn state(&self, ty: TypeId) -> State {
        self.scope
            .as_ref()
            .map_or_else(
                || self.states.get(ty.0 as usize),
                |scope| scope.states.get(&ty),
            )
            .copied()
            .unwrap_or_default()
    }
    fn record(&mut self, ty: TypeId, state: State) {
        if let Some(scope) = &mut self.scope {
            scope.states.insert(ty, state);
        } else {
            self.states[ty.0 as usize] = state;
        }
    }
    fn fail_active(&mut self, message: &'static str) {
        let fail = |state: &mut State| {
            if matches!(state, State::Active) {
                *state = State::Failed(message);
            }
        };
        if let Some(scope) = &mut self.scope {
            scope.states.values_mut().for_each(fail);
        } else {
            self.states.iter_mut().for_each(fail);
        }
    }
    fn get(&self, ty: TypeId) -> Result<Properties, &'static str> {
        match self.state(ty) {
            State::Ready(value) => Ok(value),
            State::Failed(message) => Err(message),
            _ => Err("missing completed Copy obligation"),
        }
    }
    fn run<S: BuildHasher>(
        &mut self,
        types: &Store,
        definitions: &Definitions,
        selected: &HashMap<ItemId, Properties, S>,
    ) -> Result<(), &'static str> {
        while let Some((ty, finish)) = self.work.pop() {
            let kind = types.kind(ty).ok_or("invalid Copy type")?;
            let state = self.state(ty);
            match state {
                State::Ready(_) => continue,
                State::Failed(message) => return Err(message),
                State::Active if !finish => return Err("recursive by-value Copy obligation"),
                _ => {}
            }
            if !finish {
                self.record(ty, State::Active);
                self.work.push((ty, true));
                self.steps += 1;
            }
            let assumed = self
                .scope
                .as_ref()
                .map(|scope| scope.environment.assumed(ty));
            if assumed.is_some_and(|p| p.copy) {
                self.record(ty, State::Ready(Properties::BOTH));
                continue;
            }
            if matches!(kind, Kind::Adt(_, args) if types.region_args(args).is_none()) {
                definitions.shape(types, ty)?;
            }
            let initial = match kind {
                Kind::Parameter(id) => {
                    let abilities = self
                        .scope
                        .as_ref()
                        .ok_or("generic Copy checking requires a parameter environment")?
                        .environment
                        .abilities(id)
                        .ok_or("Copy parameter is outside the current environment")?;
                    Properties {
                        copy: abilities.copy,
                        clone_: abilities.clone_,
                    }
                }
                Kind::Adt(item, _) => selected.get(&item).copied().unwrap_or_default(),
                Kind::Primitive(Primitive::Str)
                | Kind::Slice(_)
                | Kind::Reference { mutable: true, .. } => Properties::default(),
                _ => Properties::BOTH,
            };
            let mut result = initial;
            let packed = matches!(kind, Kind::Adt(item, _)
                if definitions.get(item).is_some_and(|d| d.repr.packed.is_some()));
            if initial != Properties::default() {
                children(types, definitions, ty, |child| {
                    if finish {
                        let properties = self.get(child)?;
                        result = result.intersect(properties);
                        if packed {
                            result.clone_ &= properties.copy;
                        }
                    } else {
                        self.work.push((child, false));
                    }
                    Ok(())
                })?;
            }
            if finish {
                result.clone_ |= assumed.is_some_and(|p| p.clone_);
                self.record(ty, State::Ready(result));
            }
        }
        Ok(())
    }
}
fn children(
    types: &Store,
    definitions: &Definitions,
    ty: TypeId,
    mut visit: impl FnMut(TypeId) -> Result<(), &'static str>,
) -> Result<(), &'static str> {
    match types.kind(ty).ok_or("invalid Copy type")? {
        Kind::Array { element, .. } | Kind::ArrayTerm { element, .. } => visit(element)?,
        Kind::Tuple(list) => {
            for ty in types.list(list).ok_or("invalid Copy tuple")? {
                visit(*ty)?;
            }
        }
        Kind::Adt(_, arguments) => {
            for variant in definitions.shape(types, ty)?.variants() {
                for ty in types.list(variant.fields).ok_or("invalid Copy fields")? {
                    visit(*ty)?;
                }
            }
            // Derives impose bounds on type parameters even in pointer fields.
            // Packed Clone derives strengthen these bounds to Copy too.
            for argument in types.arguments(arguments).ok_or("invalid Copy arguments")? {
                if let super::arguments::Argument::Type(ty) = argument {
                    visit(ty)?;
                }
            }
        }
        // Reference and pointer abilities do not depend on their referent.
        _ => {}
    }
    Ok(())
}
