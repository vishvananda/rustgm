//! Copy/Clone obligations under fixed assumptions, independent of storage layout.
use super::parameters::environment::{BuiltinQuery, Environment};
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
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Arguments {
    #[default]
    All,
    FieldsOnly,
}
/// Declared implementation conditions are separate from computed abilities.
/// Source derives require all type arguments; a validated library declaration
/// can instead carry its actual field-only contract.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Declaration {
    properties: Properties,
    arguments: Arguments,
}
impl Declaration {
    #[must_use]
    pub const fn derived(properties: Properties) -> Self {
        Self {
            properties,
            arguments: Arguments::All,
        }
    }
    #[must_use]
    pub const fn properties(self) -> Properties {
        self.properties
    }
    #[must_use]
    pub const fn has_argument_bounds(self) -> bool {
        matches!(self.arguments, Arguments::All)
    }
    pub(crate) const fn fields_only_clone() -> Self {
        Self {
            properties: Properties {
                copy: false,
                clone_: true,
            },
            arguments: Arguments::FieldsOnly,
        }
    }
}
impl From<Properties> for Declaration {
    fn from(properties: Properties) -> Self {
        Self::derived(properties)
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
    environment: Option<Arc<Environment>>,
    builtins: Option<Box<BuiltinQuery>>,
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
    declaration: bool,
    intercrate: Option<Box<super::parameters::shape::Shapes>>,
    states: Vec<State>,
    work: Vec<(TypeId, bool)>,
    pub steps: usize,
}
impl Copies {
    #[must_use]
    pub fn in_environment(environment: Arc<Environment>) -> Self {
        Self {
            scope: Some(Box::new(Scope {
                environment: Some(environment),
                builtins: None,
                states: HashMap::new(),
            })),
            ..Self::default()
        }
    }
    /// Check a derive's field templates under its generated bounds. Nested
    /// nominals use their conditional implementation contracts; the caller
    /// must separately validate every selected derive before publishing facts.
    #[must_use]
    pub fn for_derive(environment: Option<Arc<Environment>>) -> Self {
        Self {
            scope: Some(Box::new(Scope {
                environment,
                builtins: None,
                states: HashMap::new(),
            })),
            declaration: true,
            ..Self::default()
        }
    }
    /// Possibility in a compatible crate world, never permission to copy a value.
    /// Local derive contracts still constrain their arguments. Foreign type heads
    /// may acquire implementations; references retain the referent's locality.
    pub(crate) fn for_coherence(environment: Arc<Environment>) -> Self {
        let mut query = Self::for_derive(Some(Arc::clone(&environment)));
        query.intercrate = Some(Box::new(super::parameters::shape::Shapes::new(environment)));
        query
    }
    /// # Errors
    /// Rejects invalid type graphs and by-value cycles; caches failed queries too.
    pub fn demand<S: BuildHasher>(
        &mut self,
        types: &Store,
        definitions: &Definitions,
        selected: &HashMap<ItemId, Declaration, S>,
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
        selected: &HashMap<ItemId, Declaration, S>,
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
            if let Some(value) = self.intercrate_properties(types, definitions, ty, kind)? {
                self.record(ty, State::Ready(value));
                continue;
            }
            let assumed =
                self.scope
                    .as_mut()
                    .and_then(|scope| {
                        scope.environment.as_ref().map(|environment| {
                            environment.assumed_with(types, ty, &mut scope.builtins)
                        })
                    })
                    .transpose()?;
            if assumed.is_some_and(|p| p.copy) {
                self.record(ty, State::Ready(Properties::BOTH));
                continue;
            }
            if !self.declaration
                && matches!(kind, Kind::Adt(_, args) if types.region_args(args).is_none())
            {
                definitions.shape(types, ty)?;
            }
            let initial = self.initial_properties(types, selected, kind, assumed.is_some())?;
            let mut result = initial;
            let packed = matches!(kind, Kind::Adt(item, _)
                if definitions.get(item).is_some_and(|d| d.repr.packed.is_some()));
            if initial != Properties::default() {
                let arguments = match kind {
                    Kind::Adt(item, _) => {
                        selected.get(&item).is_none_or(|d| d.has_argument_bounds())
                    }
                    _ => true,
                };
                children(
                    types,
                    definitions,
                    ty,
                    !self.declaration || !arguments,
                    arguments,
                    |child| {
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
                    },
                )?;
            }
            if finish {
                result.clone_ |= assumed.is_some_and(|p| p.clone_);
                self.record(ty, State::Ready(result));
            }
        }
        Ok(())
    }
    fn initial_properties<S: BuildHasher>(
        &self,
        types: &Store,
        selected: &HashMap<ItemId, Declaration, S>,
        kind: Kind,
        assumed: bool,
    ) -> Result<Properties, &'static str> {
        Ok(match kind {
            Kind::Opaque(id, _) => {
                types
                    .opaque_declaration(id)
                    .ok_or("invalid opaque declaration")?
                    .copies
            }
            Kind::Projection(application) => {
                self.projected_copies(types, application.item, assumed)?
            }
            Kind::Parameter(id) => {
                let abilities = self
                    .scope
                    .as_ref()
                    .and_then(|scope| scope.environment.as_ref())
                    .ok_or("generic Copy checking requires a parameter environment")?
                    .abilities(id)
                    .ok_or("Copy parameter is outside the current environment")?;
                Properties {
                    copy: abilities.copy,
                    clone_: abilities.clone_,
                }
            }
            Kind::Adt(item, _) => selected
                .get(&item)
                .copied()
                .unwrap_or_default()
                .properties(),
            Kind::Primitive(Primitive::Str)
            | Kind::Slice(_)
            | Kind::Dynamic { .. }
            | Kind::Reference { mutable: true, .. } => Properties::default(),
            _ => Properties::BOTH,
        })
    }
    fn projected_copies(
        &self,
        types: &Store,
        item: ItemId,
        assumed: bool,
    ) -> Result<Properties, &'static str> {
        let mut promises = types
            .projection_schema(item)
            .ok_or("missing associated type declaration")?
            .copies;
        if let Some(environment) = self.scope.as_ref().and_then(|s| s.environment.as_ref()) {
            let extra = environment.projection_promises(item).copies;
            promises.copy |= extra.copy;
            promises.clone_ |= extra.clone_;
        }
        if promises == Properties::default() && !assumed {
            return Err("associated type Copy checking requires normalization");
        }
        Ok(promises)
    }
    fn intercrate_properties(
        &mut self,
        types: &Store,
        definitions: &Definitions,
        ty: TypeId,
        kind: Kind,
    ) -> Result<Option<Properties>, &'static str> {
        let Some(shapes) = &mut self.intercrate else {
            return Ok(None);
        };
        if !shapes.demand(types, ty)?.sized {
            return Ok(Some(Properties::default()));
        }
        match kind {
            Kind::Adt(..) | Kind::Parameter(_) | Kind::Opaque(..) | Kind::Closure { .. } => {
                Ok(None)
            }
            Kind::Reference {
                mutable: true,
                mut referent,
                ..
            } => loop {
                self.steps += 1;
                if self.steps > 1_000_000 {
                    return Err("coherence locality work budget exceeded");
                }
                match types.kind(referent).ok_or("invalid coherence referent")? {
                    Kind::Reference {
                        referent: inner, ..
                    } => referent = inner,
                    Kind::Adt(item, _) if definitions.get(item).is_some() => {
                        return Ok(Some(Properties::default()));
                    }
                    _ => return Ok(Some(Properties::BOTH)),
                }
            },
            _ => Ok(Some(Properties::BOTH)),
        }
    }
}
fn children(
    types: &Store,
    definitions: &Definitions,
    ty: TypeId,
    fields: bool,
    argument_bounds: bool,
    mut visit: impl FnMut(TypeId) -> Result<(), &'static str>,
) -> Result<(), &'static str> {
    match types.kind(ty).ok_or("invalid Copy type")? {
        Kind::Closure { environment, .. } => visit(environment)?,
        Kind::Array { element, .. } | Kind::ArrayTerm { element, .. } => visit(element)?,
        Kind::Tuple(list) => {
            for ty in types.list(list).ok_or("invalid Copy tuple")? {
                visit(*ty)?;
            }
        }
        Kind::Adt(_, arguments) => {
            if fields {
                for variant in definitions.shape(types, ty)?.variants() {
                    for ty in types.list(variant.fields).ok_or("invalid Copy fields")? {
                        visit(*ty)?;
                    }
                }
            }
            // Derives impose bounds on type parameters even in pointer fields.
            // Packed Clone derives strengthen these bounds to Copy too.
            if argument_bounds {
                for argument in types.arguments(arguments).ok_or("invalid Copy arguments")? {
                    if let super::arguments::Argument::Type(ty) = argument {
                        visit(ty)?;
                    }
                }
            }
        }
        // Reference and pointer abilities do not depend on their referent.
        _ => {}
    }
    Ok(())
}
