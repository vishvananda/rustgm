//! Possible destruction requirements under immutable parameter assumptions.
mod bounded;
pub mod regions;
use super::parameters::environment::{BuiltinQuery, Environment};
use super::{ItemId, Kind, Store, TypeId, aggregate::Definitions};
use crate::support::hash::{HashMap, HashSet};
use std::sync::Arc;

#[derive(Clone, Copy, Default)]
enum State {
    #[default]
    Unseen,
    Active,
    Ready(bool),
    Failed(&'static str),
}
struct Scope {
    environment: Option<Arc<Environment>>,
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
    /// Concrete queries retain only visited types and cannot assume symbolic cleanup.
    #[allow(dead_code)] // The source needs_drop consumer awaits validated source roles.
    pub(crate) fn concrete_sparse(custom: impl IntoIterator<Item = ItemId>) -> Self {
        let mut query = Self::new(custom);
        query.scope = Some(Box::new(Scope {
            environment: None,
            builtins: None,
            states: HashMap::default(),
        }));
        query
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
                environment: Some(environment),
                builtins: None,
                states: HashMap::default(),
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
        self.demand_inside::<false>(types, definitions, ty, usize::MAX)
    }
    fn demand_inside<const BOUNDED: bool>(
        &mut self,
        types: &Store,
        definitions: &Definitions,
        ty: TypeId,
        limit: usize,
    ) -> Result<bool, &'static str> {
        if matches!(self.state(ty), State::Ready(_) | State::Failed(_)) {
            return self.get(ty);
        }
        types.kind(ty).ok_or("invalid destruction type")?;
        if self.scope.is_none() {
            self.states.resize(types.len(), State::Unseen);
        }
        self.push::<BOUNDED>((ty, false), limit)?;
        if let Err(message) = self.run::<BOUNDED>(types, definitions, limit) {
            let fail = |state: &mut State| {
                if matches!(state, State::Active) {
                    *state = State::Failed(message);
                }
            };
            if let Some(scope) = &mut self.scope {
                if BOUNDED {
                    self.steps = self.steps.saturating_add(scope.states.len());
                }
                scope.states.values_mut().for_each(fail);
            } else {
                self.states.iter_mut().for_each(fail);
            }
            if !BOUNDED || !matches!(self.state(ty), State::Unseen) {
                self.record::<BOUNDED>(ty, State::Failed(message), limit)?;
            }
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
    fn symbolic(&self) -> bool {
        self.scope
            .as_ref()
            .is_some_and(|scope| scope.environment.is_some())
    }
    fn exact_heads(&self) -> bool {
        self.scope
            .as_ref()
            .is_some_and(|scope| scope.environment.is_none())
    }
    fn record<const BOUNDED: bool>(
        &mut self,
        ty: TypeId,
        state: State,
        limit: usize,
    ) -> Result<(), &'static str> {
        if BOUNDED {
            if let Some(existing) = self.scope.as_mut().and_then(|s| s.states.get_mut(&ty)) {
                *existing = state;
                return Ok(());
            }
            self.retain::<true>(1, 0, limit)?;
        }
        match &mut self.scope {
            Some(scope) => {
                scope.states.insert(ty, state);
            }
            None => self.states[ty.0 as usize] = state,
        }
        Ok(())
    }
    fn get(&self, ty: TypeId) -> Result<bool, &'static str> {
        match self.state(ty) {
            State::Ready(value) => Ok(value),
            State::Failed(message) => Err(message),
            _ => Err("missing destruction requirement"),
        }
    }
    fn charge<const BOUNDED: bool>(&mut self) -> Result<(), &'static str> {
        let next = self
            .steps
            .checked_add(1)
            .ok_or("destruction query work budget exceeded")?;
        if BOUNDED {
            self.steps = next;
        }
        if next > 1_000_000 {
            return Err("destruction query work budget exceeded");
        }
        self.steps = next;
        Ok(())
    }
    fn nominal_requirement(
        &self,
        definitions: &Definitions,
        item: ItemId,
    ) -> Result<Option<bool>, &'static str> {
        let policy = definitions
            .destruction(item)
            .ok_or("missing destruction aggregate")?;
        Ok(match (self.custom.contains(&item), policy) {
            (true, _) => Some(true),
            (_, super::aggregate::FieldDestruction::Suppressed) => Some(false),
            _ => None,
        })
    }
    fn assumed_requirement(
        &mut self,
        types: &Store,
        ty: TypeId,
        kind: Kind,
    ) -> Result<Option<bool>, &'static str> {
        if let Some(scope) = &mut self.scope
            && let Some(environment) = &scope.environment
        {
            if environment
                .assumed_with(types, ty, &mut scope.builtins)?
                .copy
            {
                return Ok(Some(false));
            }
            if let Kind::Parameter(id) = kind {
                return Ok(Some(
                    !environment
                        .abilities(id)
                        .ok_or("destruction parameter is outside its environment")?
                        .copy,
                ));
            }
        } else if matches!(kind, Kind::Parameter(_) | Kind::ArrayTerm { .. }) {
            return Err("generic destruction checking requires a parameter environment");
        }
        Ok(None)
    }
    fn run<const BOUNDED: bool>(
        &mut self,
        types: &Store,
        definitions: &Definitions,
        limit: usize,
    ) -> Result<(), &'static str> {
        while let Some((ty, finish)) = self.work.pop() {
            self.charge::<BOUNDED>()?;
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
                self.record::<BOUNDED>(ty, State::Ready(true), limit)?;
                continue;
            }
            if let Some(needed) = self.assumed_requirement(types, ty, kind)? {
                self.record::<BOUNDED>(ty, State::Ready(needed), limit)?;
                continue;
            }
            if let Kind::Opaque(id, _) = kind {
                let copy = types
                    .opaque_declaration(id)
                    .ok_or("missing opaque declaration")?
                    .copies
                    .copy;
                if (copy && !self.exact_heads()) || self.symbolic() {
                    self.record::<BOUNDED>(ty, State::Ready(!copy), limit)?;
                    continue;
                }
            }
            if let Kind::Projection(application) = kind {
                if !self.exact_heads()
                    && types
                        .projection_schema(application.item)
                        .is_some_and(|schema| schema.copies.copy)
                {
                    self.record::<BOUNDED>(ty, State::Ready(false), limit)?;
                    continue;
                }
                // A checked generic body can move or drop an opaque associated
                // result. Keep possible cleanup until instance normalization
                // selects its concrete type and actual destruction requirements.
                if self.symbolic() {
                    self.record::<BOUNDED>(ty, State::Ready(true), limit)?;
                    continue;
                }
                return Err("associated type destruction checking requires normalization");
            }
            if matches!(kind, Kind::Adt(_, args) if types.region_args(args).is_none()) {
                definitions.shape(types, ty)?;
            }
            if let Kind::Adt(item, _) = kind
                && let Some(needed) = self.nominal_requirement(definitions, item)?
            {
                self.record::<BOUNDED>(ty, State::Ready(needed), limit)?;
                continue;
            }
            let mut needed = false;
            if !finish {
                self.record::<BOUNDED>(ty, State::Active, limit)?;
                self.push::<BOUNDED>((ty, true), limit)?;
            }
            children(types, definitions, ty, |child| {
                self.charge::<BOUNDED>()?;
                if finish {
                    needed |= self.get(child)?;
                } else {
                    self.push::<BOUNDED>((child, false), limit)?;
                }
                Ok(())
            })?;
            if finish {
                self.record::<BOUNDED>(ty, State::Ready(needed), limit)?;
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
        Kind::Closure { environment, .. } => visit(environment)?,
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
        convention: super::callable::Convention::RUST,
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

#[cfg(test)]
#[path = "../../tests/invariants/concrete_destruction.rs"]
mod tests;
