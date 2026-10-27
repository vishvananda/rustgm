//! Shared type graphs are visited once while accumulating rigid parameter uses.
mod dag_parts;
mod multiplicities;
mod sole;
use super::ParameterId;
use crate::support::{hash::HashSet, retained::Growth};
use crate::types::{Kind, Store, TypeId, arguments::Argument};
pub(crate) use multiplicities::Multiplicities;
pub(crate) use sole::Sole;
#[derive(Default)]
pub struct Uses {
    opaque_projections: bool,
    pub parameters: HashSet<ParameterId>,
    pub constants: HashSet<ParameterId>,
    seen: HashSet<TypeId>,
    applications: HashSet<crate::types::traits::ApplicationId>,
    work: Vec<TypeId>,
    pub steps: usize,
}
impl Uses {
    /// Inputs of a non-injective projection do not constrain implementation parameters.
    #[must_use]
    pub fn constraining() -> Self {
        Self {
            opaque_projections: true,
            ..Self::default()
        }
    }
    /// Accumulate type and const parameter occurrences from completed types in one store.
    /// # Errors
    /// Rejects malformed canonical identities and exhausted traversal resources.
    pub fn collect(&mut self, store: &Store, root: TypeId) -> Result<(), &'static str> {
        self.collect_observed::<false>(store, root, &|_| Ok(()))
    }
    pub(crate) fn collect_counted(
        &mut self,
        store: &Store,
        root: TypeId,
        charge: &impl Fn(Growth) -> Result<(), &'static str>,
    ) -> Result<(), &'static str> {
        self.collect_observed::<true>(store, root, charge)
    }
    fn collect_observed<const CHARGED: bool>(
        &mut self,
        store: &Store,
        root: TypeId,
        charge: &impl Fn(Growth) -> Result<(), &'static str>,
    ) -> Result<(), &'static str> {
        self.push::<CHARGED>(root, charge)?;
        self.drain::<CHARGED>(store, charge)
    }
    /// Include parameters appearing only in trait arguments, with shared graph memoization.
    /// # Errors
    /// Rejects invalid subjects/applications and exhausted traversal resources.
    pub fn predicate(
        &mut self,
        store: &Store,
        predicate: super::predicates::Predicate,
    ) -> Result<(), &'static str> {
        self.predicate_observed::<false>(store, predicate, &|_| Ok(()))
    }
    fn predicate_observed<const CHARGED: bool>(
        &mut self,
        store: &Store,
        predicate: super::predicates::Predicate,
        charge: &impl Fn(Growth) -> Result<(), &'static str>,
    ) -> Result<(), &'static str> {
        self.collect_observed::<CHARGED>(store, predicate.ty, charge)?;
        if let super::predicates::Bound::ProjectionEqual(result) = predicate.bound {
            self.collect_observed::<CHARGED>(store, result, charge)?;
        }
        if let Some((id, _)) = predicate.bound.trait_parts() {
            let application = store
                .trait_application(id)
                .ok_or("invalid parameter use trait application")?;
            if insert::<CHARGED, _>(&mut self.applications, id, charge)? {
                self.arguments::<CHARGED>(store, application.arguments, charge)?;
                self.drain::<CHARGED>(store, charge)?;
            }
        }
        Ok(())
    }
    fn drain<const CHARGED: bool>(
        &mut self,
        store: &Store,
        charge: &impl Fn(Growth) -> Result<(), &'static str>,
    ) -> Result<(), &'static str> {
        while let Some(ty) = self.work.pop() {
            if CHARGED {
                charge(Growth { work: 1, bytes: 0 })?;
            }
            self.steps += 1;
            if self.steps + self.work.len() > 1_000_000 {
                return Err("parameter use work budget exceeded");
            }
            if !store
                .contains_parameters(ty)
                .ok_or("invalid parameter use type")?
                || !insert::<CHARGED, _>(&mut self.seen, ty, charge)?
            {
                continue;
            }
            let list = match store.kind(ty).ok_or("missing parameter use type")? {
                Kind::Parameter(id) => {
                    insert::<CHARGED, _>(&mut self.parameters, id, charge)?;
                    None
                }
                Kind::ArrayTerm { element, length } => {
                    self.constant::<CHARGED>(store, length, charge)?;
                    self.push::<CHARGED>(element, charge)?;
                    None
                }
                Kind::Array { element, .. }
                | Kind::Pattern { base: element, .. }
                | Kind::Slice(element)
                | Kind::Reference {
                    referent: element, ..
                }
                | Kind::Pointer {
                    pointee: element, ..
                } => {
                    self.push::<CHARGED>(element, charge)?;
                    None
                }
                Kind::Closure {
                    arguments,
                    signature,
                    environment,
                    ..
                } => {
                    self.push::<CHARGED>(signature, charge)?;
                    self.push::<CHARGED>(environment, charge)?;
                    self.arguments::<CHARGED>(store, arguments, charge)?;
                    None
                }
                Kind::Tuple(list) => Some(list),
                Kind::Function {
                    parameters, result, ..
                } => {
                    self.push::<CHARGED>(result, charge)?;
                    Some(parameters)
                }
                Kind::Adt(_, arguments)
                | Kind::Opaque(_, arguments)
                | Kind::Dynamic { arguments, .. }
                | Kind::FunctionItem { arguments, .. } => {
                    if let Some(Kind::FunctionItem { signature, .. }) = store.kind(ty) {
                        self.push::<CHARGED>(signature, charge)?;
                    }
                    self.arguments::<CHARGED>(store, arguments, charge)?;
                    None
                }
                Kind::Projection(application) => {
                    if self.opaque_projections {
                        continue;
                    }
                    self.push::<CHARGED>(application.subject, charge)?;
                    let trait_ = store
                        .trait_application(application.trait_application)
                        .ok_or("invalid parameter-use projection trait")?;
                    self.arguments::<CHARGED>(store, trait_.arguments, charge)?;
                    self.arguments::<CHARGED>(store, application.arguments, charge)?;
                    None
                }
                Kind::Never | Kind::Foreign(_) | Kind::Primitive(_) => None,
            };
            if let Some(list) = list {
                let children = store.list(list).ok_or("invalid parameter use list")?;
                self.capacity(children.len())?;
                if CHARGED {
                    charge(Growth::vector::<TypeId>(
                        self.work.len(),
                        self.work.capacity(),
                        children.len(),
                    ))?;
                }
                self.work.extend_from_slice(children);
            }
        }
        Ok(())
    }
    fn arguments<const CHARGED: bool>(
        &mut self,
        store: &Store,
        arguments: crate::types::GenericArgs,
        charge: &impl Fn(Growth) -> Result<(), &'static str>,
    ) -> Result<(), &'static str> {
        let arguments = store
            .arguments(arguments)
            .ok_or("invalid parameter use arguments")?;
        self.capacity(arguments.len())?;
        if CHARGED {
            charge(Growth {
                work: arguments.len(),
                bytes: 0,
            })?;
        }
        self.steps += arguments.len();
        for argument in arguments {
            match argument {
                Argument::Type(ty) => self.push::<CHARGED>(ty, charge)?,
                Argument::Const(id) => self.constant::<CHARGED>(store, id, charge)?,
                Argument::Lifetime(_) => {}
            }
        }
        Ok(())
    }
    fn constant<const CHARGED: bool>(
        &mut self,
        store: &Store,
        id: crate::types::arguments::ConstId,
        charge: &impl Fn(Growth) -> Result<(), &'static str>,
    ) -> Result<(), &'static str> {
        if let crate::types::arguments::ConstKind::Parameter(id) = store
            .const_kind(id)
            .ok_or("invalid parameter use constant")?
        {
            insert::<CHARGED, _>(&mut self.constants, id, charge)?;
        }
        Ok(())
    }
    fn push<const CHARGED: bool>(
        &mut self,
        ty: TypeId,
        charge: &impl Fn(Growth) -> Result<(), &'static str>,
    ) -> Result<(), &'static str> {
        if CHARGED {
            charge(Growth::vector::<TypeId>(
                self.work.len(),
                self.work.capacity(),
                1,
            ))?;
        }
        self.work.push(ty);
        Ok(())
    }
    fn capacity(&self, count: usize) -> Result<(), &'static str> {
        if self
            .steps
            .saturating_add(self.work.len())
            .saturating_add(count)
            > 1_000_000
        {
            return Err("parameter use work budget exceeded");
        }
        Ok(())
    }
}

fn insert<const CHARGED: bool, T: std::hash::Hash + Eq>(
    set: &mut HashSet<T>,
    value: T,
    charge: &impl Fn(Growth) -> Result<(), &'static str>,
) -> Result<bool, &'static str> {
    if CHARGED {
        charge(Growth { work: 1, bytes: 0 })?;
        if set.contains(&value) {
            return Ok(false);
        }
        charge(Growth::map::<T, ()>(set.len(), set.capacity(), 1))?;
    }
    Ok(set.insert(value))
}
