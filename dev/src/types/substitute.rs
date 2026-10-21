//! Substitute a template binder and its parameters without capturing nested binders.
mod equation_closure;
pub(crate) use equation_closure::RegionEquations;
mod export;
pub(crate) mod families;
mod items;
pub use export::ExportSource;
pub(crate) mod lookup;
mod normalize;
mod rewrite;
pub use normalize::{Alias, Normalizer, Request, Resolution, Step};
mod parameters;
mod predicates;
mod region_mapping;
mod universals;
use super::{Kind, ListId, RegionId, Store, TypeId, arguments::Argument};
use std::collections::HashMap;

type Key = (TypeId, u16);
const WORK_LIMIT: usize = 1_000_000;

enum Action {
    Remove,
    Shift(u16),
    Lower(u16),
    Items(Box<items::Replacements>),
    Universals(Box<universals::Mapping>),
    Regions(Box<region_mapping::Mapping>),
    Shape { preserve_static: bool },
    Normalize,
}

/// A substitution cache belongs to one store and one argument environment.
/// Reuse it across the parameters/result/fields of that same instantiation.
pub struct Bound {
    arguments: Vec<RegionId>,
    action: Action,
    parameters: Option<Box<parameters::Bindings>>,
    completed: HashMap<Key, TypeId>,
    applications: Option<Box<predicates::Applications>>,
    work: Vec<(Key, bool)>,
    children: Vec<TypeId>,
    regions: Vec<RegionId>,
    generic: Vec<Argument>,
    pub steps: usize,
}

impl Bound {
    #[must_use]
    pub fn new(arguments: Vec<RegionId>) -> Self {
        Self {
            arguments,
            action: Action::Remove,
            parameters: None,
            completed: HashMap::new(),
            applications: None,
            work: Vec::new(),
            children: Vec::new(),
            regions: Vec::new(),
            generic: Vec::new(),
            steps: 0,
        }
    }

    pub(crate) fn shift(amount: u16) -> Self {
        let mut bound = Self::new(Vec::new());
        bound.action = Action::Shift(amount);
        bound
    }

    /// Region-free inference graphs are unchanged only by lifetime-only folds.
    pub(crate) fn regions_only(&self) -> bool {
        self.parameters.is_none()
            && matches!(
                self.action,
                Action::Remove
                    | Action::Shift(_)
                    | Action::Lower(_)
                    | Action::Universals(_)
                    | Action::Regions(_)
            )
    }

    /// Remove surrounding scopes only after proving that none of their locals occur.
    pub(crate) fn lower(amount: u16) -> Self {
        let mut bound = Self::new(Vec::new());
        bound.action = Action::Lower(amount);
        bound
    }

    /// An index key for lifetime-insensitive shape equality, never a usable type.
    pub(crate) fn shape() -> Self {
        let mut bound = Self::new(Vec::new());
        bound.action = Action::Shape {
            preserve_static: false,
        };
        bound
    }

    pub(crate) fn auto_shape() -> Self {
        let mut bound = Self::new(Vec::new());
        bound.action = Action::Shape {
            preserve_static: true,
        };
        bound
    }

    // Erasure has no lexical context. Repeated callable tails must not acquire
    // a separate structural cache entry for every enclosing binder depth.
    fn cache_depth(&self, depth: u16) -> u16 {
        if matches!(self.action, Action::Shape { .. }) {
            0
        } else {
            depth
        }
    }

    /// Substitute a type written inside the removed binder. Bound arguments are
    /// interpreted in the destination context and shifted beneath nested binders.
    ///
    /// # Errors
    /// Rejects missing arguments, invalid IDs, compact overflow and exhausted work.
    pub fn apply(&mut self, store: &mut Store, ty: TypeId) -> Result<TypeId, &'static str> {
        self.apply_inside(store, ty, 0)
    }
    pub(crate) fn apply_inside(
        &mut self,
        store: &mut Store,
        ty: TypeId,
        depth: u16,
    ) -> Result<TypeId, &'static str> {
        let key = (ty, self.cache_depth(depth));
        if let Some(result) = self.result(store, key)? {
            return Ok(result);
        }
        self.work.push((key, false));
        let result = self
            .drain(store)
            .and_then(|()| self.result(store, key)?.ok_or("missing substituted root"));
        self.work.clear();
        result
    }
    /// Transform a signature's parameter/result roots and publish a replacement
    /// binder. Arguments must already be expressed inside that replacement binder.
    ///
    /// # Errors
    /// Rejects non-signatures and the same invalid data/resource limits as `apply`.
    pub fn signature(
        &mut self,
        store: &mut Store,
        ty: TypeId,
        bound_regions: u32,
    ) -> Result<TypeId, &'static str> {
        let Some(Kind::Function {
            parameters,
            result,
            unsafe_,
            convention,
            ..
        }) = store.kind(ty)
        else {
            return Err("substitution requires a function signature");
        };
        let count = store
            .list(parameters)
            .ok_or("missing signature parameters")?
            .len();
        let mut children = Vec::with_capacity(count);
        for index in 0..count {
            let ty = store
                .list(parameters)
                .ok_or("missing signature parameter list")?[index];
            children.push(self.apply(store, ty)?);
        }
        let result = self.apply(store, result)?;
        let parameters = store.intern_list(&children)?;
        store.intern(Kind::Function {
            bound_regions,
            parameters,
            result,
            unsafe_,
            convention,
        })
    }

    fn result(&self, store: &Store, key: Key) -> Result<Option<TypeId>, &'static str> {
        let key = (key.0, self.cache_depth(key.1));
        if matches!(self.action, Action::Normalize) {
            return if store
                .contains_projections(key.0)
                .ok_or("invalid normalization type")?
            {
                Ok(self.completed.get(&key).copied())
            } else {
                Ok(Some(key.0))
            };
        }
        if matches!(self.action, Action::Items(_)) {
            return Ok(self.completed.get(&key).copied());
        }
        let escaping = store
            .escaping_binders(key.0)
            .ok_or("invalid substitution type")?;
        let closed = if matches!(self.action, Action::Shape { .. }) {
            store.has_free_regions(key.0, 0) == Some(false)
                && store.contains_late_callables(key.0) == Some(false)
        } else if matches!(self.action, Action::Universals(_) | Action::Regions(_)) {
            store.has_free_regions(key.0, key.1) == Some(false)
        } else {
            escaping <= key.1
        };
        if closed && !(self.parameters.is_some() && store.contains_parameters(key.0) == Some(true))
        {
            return Ok(Some(key.0));
        }
        Ok(self.completed.get(&key).copied())
    }

    fn child(&self, store: &Store, ty: TypeId, depth: u16) -> Result<TypeId, &'static str> {
        self.result(store, (ty, depth))?
            .ok_or("missing substituted dependency")
    }

    fn drain(&mut self, store: &mut Store) -> Result<(), &'static str> {
        while let Some((key, finish)) = self.work.pop() {
            self.steps += 1;
            if self.steps > WORK_LIMIT {
                return Err("lifetime substitution work budget exceeded");
            }
            if self.result(store, key)?.is_some() {
                continue;
            }
            let kind = store.kind(key.0).ok_or("invalid substitution type")?;
            if matches!(self.action, Action::Items(_)) && self.replace_item(store, key, finish)? {
                continue;
            }
            if let Kind::Parameter(id) = kind
                && let Some(replacement) = self.parameter(store, id, key.1)?
            {
                self.completed.insert(key, replacement);
                continue;
            }
            if finish {
                let kind = self.rebuild(store, kind, key.1)?;
                self.completed.insert(key, store.intern(kind)?);
            } else {
                self.work.push((key, true));
                self.dependencies(store, kind, key.1)?;
            }
        }
        Ok(())
    }

    fn queue(&mut self, ty: TypeId, depth: u16) -> Result<(), &'static str> {
        if self.work.len().saturating_add(self.steps) >= WORK_LIMIT {
            return Err("lifetime substitution work budget exceeded");
        }
        self.work.push(((ty, self.cache_depth(depth)), false));
        Ok(())
    }

    fn dependencies(&mut self, store: &Store, kind: Kind, depth: u16) -> Result<(), &'static str> {
        let list = match kind {
            Kind::Tuple(list) => Some((list, depth)),
            Kind::Function {
                parameters, result, ..
            } => {
                let depth = nested(depth)?;
                self.queue(result, depth)?;
                Some((parameters, depth))
            }
            Kind::Array { element: ty, .. }
            | Kind::ArrayTerm { element: ty, .. }
            | Kind::Slice(ty)
            | Kind::Reference { referent: ty, .. }
            | Kind::Pointer { pointee: ty, .. } => {
                self.queue(ty, depth)?;
                None
            }
            Kind::FunctionItem {
                signature,
                arguments,
                ..
            } => {
                self.queue(signature, depth)?;
                self.argument_dependencies(store, arguments, depth)?;
                None
            }
            Kind::Closure {
                signature,
                environment,
                arguments,
                ..
            } => {
                self.queue(signature, depth)?;
                self.queue(environment, depth)?;
                self.argument_dependencies(store, arguments, depth)?;
                None
            }
            Kind::Adt(_, arguments) | Kind::Opaque(_, arguments) => {
                self.argument_dependencies(store, arguments, depth)?;
                None
            }
            Kind::Dynamic { arguments, .. } => {
                self.argument_dependencies(store, arguments, nested(depth)?)?;
                None
            }
            Kind::Projection(application) => {
                self.queue(application.subject, depth)?;
                let trait_ = store
                    .trait_application(application.trait_application)
                    .ok_or("invalid substituted projection trait")?;
                self.argument_dependencies(store, trait_.arguments, depth)?;
                self.argument_dependencies(store, application.arguments, depth)?;
                None
            }
            Kind::Never | Kind::Primitive(_) | Kind::Parameter(_) => None,
        };
        if let Some((list, depth)) = list {
            for ty in store.list(list).ok_or("invalid substitution child list")? {
                self.queue(*ty, depth)?;
            }
        }
        Ok(())
    }

    fn argument_dependencies(
        &mut self,
        store: &Store,
        args: super::GenericArgs,
        depth: u16,
    ) -> Result<(), &'static str> {
        if store.region_args(args).is_some() {
            return Ok(());
        }
        let arguments = store.arguments(args).ok_or("invalid generic arguments")?;
        self.account_arguments(arguments.len())?;
        for argument in arguments {
            if let Argument::Type(ty) = argument {
                self.queue(ty, depth)?;
            }
        }
        Ok(())
    }

    pub(crate) fn region(
        &mut self,
        region: RegionId,
        depth: u16,
    ) -> Result<RegionId, &'static str> {
        if matches!(self.action, Action::Items(_) | Action::Normalize) {
            return Ok(region);
        }
        if let Action::Shape { preserve_static } = self.action {
            return Ok(if preserve_static && region != RegionId::STATIC {
                RegionId(1)
            } else {
                RegionId::STATIC
            });
        }
        if let Action::Universals(mapping) = &mut self.action {
            return mapping.region(region, depth);
        }
        if let Action::Regions(mapping) = &mut self.action {
            return mapping.region(region, depth, &mut self.steps);
        }
        let Some((binder, index)) = region.bound_parts() else {
            return Ok(region);
        };
        if binder < depth {
            return Ok(region);
        }
        if let Action::Lower(amount) = self.action {
            if binder - depth < amount {
                return Err("cannot lower a captured local lifetime");
            }
            return RegionId::bound(binder - amount, index);
        }
        if let Action::Shift(amount) = self.action {
            return RegionId::bound(
                binder
                    .checked_add(amount)
                    .ok_or("lifetime binder depth overflow")?,
                index,
            );
        }
        if binder > depth {
            return RegionId::bound(binder - 1, index);
        }
        let argument = *self
            .arguments
            .get(index as usize)
            .ok_or("missing lifetime argument")?;
        if let Some((binder, index)) = argument.bound_parts() {
            RegionId::bound(
                binder
                    .checked_add(depth)
                    .ok_or("lifetime binder depth overflow")?,
                index,
            )
        } else {
            Ok(argument)
        }
    }

    fn list(
        &mut self,
        store: &mut Store,
        list: ListId,
        depth: u16,
    ) -> Result<ListId, &'static str> {
        self.children.clear();
        for ty in store.list(list).ok_or("invalid substitution list")? {
            self.children.push(self.child(store, *ty, depth)?);
        }
        store.intern_list(&self.children)
    }

    fn rebuild(&mut self, store: &mut Store, kind: Kind, depth: u16) -> Result<Kind, &'static str> {
        rewrite::rebuild(&mut rewrite::Mutable { bound: self, store }, kind, depth)
    }
    fn arguments(
        &mut self,
        store: &mut Store,
        args: super::GenericArgs,
        depth: u16,
    ) -> Result<super::GenericArgs, &'static str> {
        if let Some(arguments) = store.region_args(args) {
            self.account_arguments(arguments.len())?;
            self.regions.clear();
            for region in arguments {
                let region = self.region(*region, depth)?;
                self.regions.push(region);
            }
            return store.intern_regions(&self.regions);
        }
        let arguments = store.arguments(args).ok_or("invalid generic arguments")?;
        self.account_arguments(arguments.len())?;
        self.generic.clear();
        for argument in arguments {
            let argument = match argument {
                Argument::Lifetime(region) => Argument::Lifetime(self.region(region, depth)?),
                Argument::Type(ty) => Argument::Type(self.child(store, ty, depth)?),
                Argument::Const(id) => Argument::Const(self.constant(store, id)?),
            };
            self.generic.push(argument);
        }
        store.intern_arguments(&self.generic)
    }

    fn account_arguments(&mut self, count: usize) -> Result<(), &'static str> {
        self.steps = self.steps.saturating_add(count);
        if self.steps.saturating_add(self.work.len()) > WORK_LIMIT {
            return Err("lifetime substitution work budget exceeded");
        }
        Ok(())
    }
}

fn nested(depth: u16) -> Result<u16, &'static str> {
    depth
        .checked_add(1)
        .filter(|depth| *depth < 0x8000)
        .ok_or("lifetime binder nesting exceeds 32767")
}
