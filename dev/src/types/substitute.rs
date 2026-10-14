//! Substitute a template binder and its parameters without capturing nested binders.
mod parameters;
mod universals;
use super::{Kind, ListId, RegionId, Store, TypeId, arguments::Argument};
use std::collections::HashMap;

type Key = (TypeId, u16);
const WORK_LIMIT: usize = 1_000_000;

enum Action {
    Remove,
    Shift(u16),
    Universals(Box<universals::Mapping>),
}

/// A substitution cache belongs to one store and one argument environment.
/// Reuse it across the parameters/result/fields of that same instantiation.
pub struct Bound {
    arguments: Vec<RegionId>,
    action: Action,
    parameters: Option<Box<parameters::Bindings>>,
    completed: HashMap<Key, TypeId>,
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
        let key = (ty, depth);
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
        })
    }

    fn result(&self, store: &Store, key: Key) -> Result<Option<TypeId>, &'static str> {
        let escaping = store
            .escaping_binders(key.0)
            .ok_or("invalid substitution type")?;
        let closed = if matches!(self.action, Action::Universals(_)) {
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
        self.work.push(((ty, depth), false));
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
            Kind::Adt(_, arguments) => {
                self.argument_dependencies(store, arguments, depth)?;
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
        if let Action::Universals(mapping) = &mut self.action {
            return mapping.region(region, depth);
        }
        let Some((binder, index)) = region.bound_parts() else {
            return Ok(region);
        };
        if binder < depth {
            return Ok(region);
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
        Ok(match kind {
            Kind::Never | Kind::Primitive(_) | Kind::Parameter(_) => kind,
            Kind::Adt(item, args) => Kind::Adt(item, self.arguments(store, args, depth)?),
            Kind::Tuple(list) => Kind::Tuple(self.list(store, list, depth)?),
            Kind::Array { element, length } => Kind::Array {
                element: self.child(store, element, depth)?,
                length,
            },
            Kind::ArrayTerm { element, length } => Kind::ArrayTerm {
                element: self.child(store, element, depth)?,
                length: self.constant(store, length)?,
            },
            Kind::Slice(element) => Kind::Slice(self.child(store, element, depth)?),
            Kind::Reference {
                mutable,
                region,
                referent,
            } => Kind::Reference {
                mutable,
                region: self.region(region, depth)?,
                referent: self.child(store, referent, depth)?,
            },
            Kind::Pointer { mutable, pointee } => Kind::Pointer {
                mutable,
                pointee: self.child(store, pointee, depth)?,
            },
            Kind::Function {
                bound_regions,
                parameters,
                result,
                unsafe_,
            } => {
                let depth = nested(depth)?;
                Kind::Function {
                    bound_regions,
                    parameters: self.list(store, parameters, depth)?,
                    result: self.child(store, result, depth)?,
                    unsafe_,
                }
            }
            Kind::FunctionItem {
                item,
                signature,
                arguments,
            } => Kind::FunctionItem {
                item,
                arguments: self.arguments(store, arguments, depth)?,
                signature: self.child(store, signature, depth)?,
            },
        })
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
