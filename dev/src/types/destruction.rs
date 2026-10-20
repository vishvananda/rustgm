//! Possible destruction requirements under immutable parameter assumptions.
pub mod regions;
use super::parameters::environment::{BuiltinQuery, Environment};
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
    builtins: Option<Box<BuiltinQuery>>,
    states: HashMap<TypeId, State>,
}
/// One canonical store, completed nominal definitions and immutable custom Drop set.
/// A symbolic query additionally owns one fixed parameter environment. `true`
/// means an instantiation may need destruction, not that it has selected drop glue.
pub struct NeedsDrop {
    scope: Option<Box<Scope>>,
    custom: Arc<HashSet<ItemId>>,
    states: Vec<State>,
    work: Vec<(TypeId, bool)>,
    pub steps: usize,
}
impl NeedsDrop {
    #[must_use]
    pub fn new(custom: impl IntoIterator<Item = ItemId>) -> Self {
        Self {
            scope: None,
            custom: Arc::new(custom.into_iter().collect()),
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
        Self::new(custom).scoped(environment)
    }
    /// Start independent caches while sharing the immutable destructor registry.
    #[must_use]
    pub fn scoped(&self, environment: Arc<Environment>) -> Self {
        Self {
            scope: Some(Box::new(Scope {
                environment,
                builtins: None,
                states: HashMap::new(),
            })),
            custom: Arc::clone(&self.custom),
            states: Vec::new(),
            work: Vec::new(),
            steps: 0,
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
            if matches!(kind, Kind::Dynamic { .. }) {
                self.record(ty, State::Ready(true));
                continue;
            }
            if let Some(scope) = &mut self.scope {
                let needed = if scope
                    .environment
                    .assumed_with(types, ty, &mut scope.builtins)?
                    .copy
                {
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
            if let Kind::Opaque(id, _) = kind {
                let copy = types
                    .opaque_declaration(id)
                    .ok_or("missing opaque declaration")?
                    .copies
                    .copy;
                if copy || self.scope.is_some() {
                    self.record(ty, State::Ready(!copy));
                    continue;
                }
            }
            if let Kind::Projection(application) = kind {
                if types
                    .projection_schema(application.item)
                    .is_some_and(|schema| schema.copies.copy)
                {
                    self.record(ty, State::Ready(false));
                    continue;
                }
                // A checked generic body can move or drop an opaque associated
                // result. Keep possible cleanup until instance normalization
                // selects its concrete type and actual destruction requirements.
                if self.scope.is_some() {
                    self.record(ty, State::Ready(true));
                    continue;
                }
                return Err("associated type destruction checking requires normalization");
            }
            if matches!(kind, Kind::Adt(_, args) if types.region_args(args).is_none()) {
                definitions.shape(types, ty)?;
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
            children(types, definitions, ty, |child| {
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
    ty: TypeId,
    mut visit: impl FnMut(TypeId) -> Result<(), &'static str>,
) -> Result<(), &'static str> {
    match types.kind(ty).ok_or("invalid destruction type")? {
        Kind::Opaque(..) => visit(types.storage_head(ty)?)?,
        Kind::Array { length: 0, .. } => {}
        Kind::Array { element, .. } | Kind::ArrayTerm { element, .. } | Kind::Slice(element) => {
            visit(element)?;
        }
        Kind::Tuple(list) => {
            for &ty in types.list(list).ok_or("invalid destruction tuple")? {
                visit(ty)?;
            }
        }
        Kind::Adt(_, _) => {
            for variant in definitions.shape(types, ty)?.variants() {
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

pub(crate) fn validate_method(
    types: &mut Store,
    instance: super::instance::Instance,
    owner: ItemId,
    target: TypeId,
) -> Result<(), &'static str> {
    let Some(Kind::FunctionItem {
        item, signature, ..
    }) = types.kind(instance.ty())
    else {
        return Err("destructor requires a selected callable");
    };
    if item != owner {
        return Err("destructor instance belongs to another method");
    }
    let Some(Kind::Function {
        parameters,
        result: TypeId::UNIT,
        bound_regions: 1,
        unsafe_: false,
    }) = types.kind(signature)
    else {
        return Err("selected destructor has an invalid signature");
    };
    let Some([receiver]) = types.list(parameters) else {
        return Err("selected destructor has an invalid parameter list");
    };
    let Some(Kind::Reference {
        mutable: true,
        region,
        referent,
    }) = types.kind(*receiver)
    else {
        return Err("selected destructor receiver must borrow its target mutably");
    };
    if region != super::RegionId::bound(0, 0)?
        || referent != super::substitute::Bound::shift(1).apply(types, target)?
    {
        return Err("selected destructor receiver disagrees with its concrete target");
    }
    Ok(())
}
