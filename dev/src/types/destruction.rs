//! Possible destruction requirements under immutable parameter assumptions.
pub mod regions;
use super::parameters::environment::Environment;
use super::{ItemId, Kind, Store, TypeId, aggregate::Definitions};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

#[derive(Clone, Copy, Default)]
enum State {
    #[default]
    Unseen,
    Active,
    Ready(bool),
    Failed(&'static str),
}
struct Scope {
    environment: Arc<Environment>,
    states: HashMap<TypeId, State>,
}
/// One canonical store, completed nominal definitions and immutable custom Drop set.
/// A symbolic query additionally owns one fixed parameter environment. `true`
/// means an instantiation may need destruction, not that it has selected drop glue.
pub struct NeedsDrop {
    scope: Option<Box<Scope>>,
    custom: HashSet<ItemId>,
    states: Vec<State>,
    work: Vec<(TypeId, bool)>,
    pub steps: usize,
}
impl NeedsDrop {
    #[must_use]
    pub fn new(custom: impl IntoIterator<Item = ItemId>) -> Self {
        Self {
            scope: None,
            custom: custom.into_iter().collect(),
            states: Vec::new(),
            work: Vec::new(),
            steps: 0,
        }
    }
    /// Copy assumptions prove absence of destruction. Unconstrained parameters
    /// and unknown array lengths conservatively preserve possible cleanup.
    #[must_use]
    pub fn in_environment(
        custom: impl IntoIterator<Item = ItemId>,
        environment: Arc<Environment>,
    ) -> Self {
        Self {
            scope: Some(Box::new(Scope {
                environment,
                states: HashMap::new(),
            })),
            ..Self::new(custom)
        }
    }
    /// # Errors
    /// Rejects invalid identities, by-value cycles and exhausted query work.
    pub fn demand(
        &mut self,
        types: &Store,
        definitions: &Definitions,
        ty: TypeId,
    ) -> Result<bool, &'static str> {
        if matches!(self.state(ty), State::Ready(_) | State::Failed(_)) {
            return self.get(ty);
        }
        types.kind(ty).ok_or("invalid destruction type")?;
        if self.scope.is_none() {
            self.states.resize(types.len(), State::Unseen);
        }
        self.work.push((ty, false));
        if let Err(message) = self.run(types, definitions) {
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
            self.record(ty, State::Failed(message));
            self.work.clear();
            return Err(message);
        }
        self.get(ty)
    }
    fn state(&self, ty: TypeId) -> State {
        match &self.scope {
            Some(scope) => scope.states.get(&ty),
            None => self.states.get(ty.0 as usize),
        }
        .copied()
        .unwrap_or_default()
    }
    fn record(&mut self, ty: TypeId, state: State) {
        match &mut self.scope {
            Some(scope) => {
                scope.states.insert(ty, state);
            }
            None => self.states[ty.0 as usize] = state,
        }
    }
    fn get(&self, ty: TypeId) -> Result<bool, &'static str> {
        match self.state(ty) {
            State::Ready(value) => Ok(value),
            State::Failed(message) => Err(message),
            _ => Err("missing destruction requirement"),
        }
    }
    fn charge(&mut self) -> Result<(), &'static str> {
        self.steps = self
            .steps
            .checked_add(1)
            .filter(|n| *n <= 1_000_000)
            .ok_or("destruction query work budget exceeded")?;
        Ok(())
    }
    fn run(&mut self, types: &Store, definitions: &Definitions) -> Result<(), &'static str> {
        while let Some((ty, finish)) = self.work.pop() {
            self.charge()?;
            match self.state(ty) {
                State::Ready(_) => continue,
                State::Failed(message) => return Err(message),
                State::Active if !finish => {
                    return Err("recursive by-value destruction requirement");
                }
                _ => {}
            }
            let kind = types.kind(ty).ok_or("invalid destruction type")?;
            if let Some(scope) = &self.scope {
                let needed = if scope.environment.assumed(ty).copy {
                    Some(false)
                } else if let Kind::Parameter(id) = kind {
                    Some(
                        !scope
                            .environment
                            .abilities(id)
                            .ok_or("destruction parameter is outside its environment")?
                            .copy,
                    )
                } else {
                    None
                };
                if let Some(needed) = needed {
                    self.record(ty, State::Ready(needed));
                    continue;
                }
            } else if matches!(kind, Kind::Parameter(_) | Kind::ArrayTerm { .. }) {
                return Err("generic destruction checking requires a parameter environment");
            }
            if matches!(kind, Kind::Adt(_, args) if types.region_args(args).is_none()) {
                return Err("generic destruction checking requires instantiated fields");
            }
            if let Kind::Adt(item, _) = kind {
                definitions
                    .get(item)
                    .ok_or("missing destruction aggregate")?;
                if self.custom.contains(&item) {
                    self.record(ty, State::Ready(true));
                    continue;
                }
            }
            let mut needed = false;
            if !finish {
                self.record(ty, State::Active);
                self.work.push((ty, true));
            }
            children(types, definitions, kind, |child| {
                self.charge()?;
                if finish {
                    needed |= self.get(child)?;
                } else {
                    self.work.push((child, false));
                }
                Ok(())
            })?;
            if finish {
                self.record(ty, State::Ready(needed));
            }
        }
        Ok(())
    }
}
fn children(
    types: &Store,
    definitions: &Definitions,
    kind: Kind,
    mut visit: impl FnMut(TypeId) -> Result<(), &'static str>,
) -> Result<(), &'static str> {
    match kind {
        Kind::Array { length: 0, .. } => {}
        Kind::Array { element, .. } | Kind::ArrayTerm { element, .. } | Kind::Slice(element) => {
            visit(element)?;
        }
        Kind::Tuple(list) => {
            for &ty in types.list(list).ok_or("invalid destruction tuple")? {
                visit(ty)?;
            }
        }
        Kind::Adt(item, _) => {
            let definition = definitions
                .get(item)
                .ok_or("missing destruction aggregate")?;
            for variant in definitions.variants(definition) {
                for &ty in types
                    .list(variant.fields)
                    .ok_or("invalid destruction fields")?
                {
                    visit(ty)?;
                }
            }
        }
        // References and raw pointers do not own their referents.
        _ => {}
    }
    Ok(())
}
