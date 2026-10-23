//! Shared type graphs are visited once while accumulating rigid parameter uses.
mod sole;
use super::ParameterId;
use crate::support::hash::HashSet;
use crate::types::{Kind, Store, TypeId, arguments::Argument};
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
        self.work.push(root);
        self.drain(store)
    }
    /// Include parameters appearing only in trait arguments, with shared graph memoization.
    /// # Errors
    /// Rejects invalid subjects/applications and exhausted traversal resources.
    pub fn predicate(
        &mut self,
        store: &Store,
        predicate: super::predicates::Predicate,
    ) -> Result<(), &'static str> {
        self.collect(store, predicate.ty)?;
        if let super::predicates::Bound::ProjectionEqual(result) = predicate.bound {
            self.collect(store, result)?;
        }
        if let Some((id, _)) = predicate.bound.trait_parts() {
            let application = store
                .trait_application(id)
                .ok_or("invalid parameter use trait application")?;
            if self.applications.insert(id) {
                self.arguments(store, application.arguments)?;
                self.drain(store)?;
            }
        }
        Ok(())
    }
    fn drain(&mut self, store: &Store) -> Result<(), &'static str> {
        while let Some(ty) = self.work.pop() {
            self.steps += 1;
            if self.steps + self.work.len() > 1_000_000 {
                return Err("parameter use work budget exceeded");
            }
            if !store
                .contains_parameters(ty)
                .ok_or("invalid parameter use type")?
                || !self.seen.insert(ty)
            {
                continue;
            }
            let list = match store.kind(ty).ok_or("missing parameter use type")? {
                Kind::Parameter(id) => {
                    self.parameters.insert(id);
                    None
                }
                Kind::ArrayTerm { element, length } => {
                    self.constant(store, length)?;
                    self.work.push(element);
                    None
                }
                Kind::Array { element, .. }
                | Kind::Slice(element)
                | Kind::Reference {
                    referent: element, ..
                }
                | Kind::Pointer {
                    pointee: element, ..
                } => {
                    self.work.push(element);
                    None
                }
                Kind::Closure {
                    arguments,
                    signature,
                    environment,
                    ..
                } => {
                    self.work.extend([signature, environment]);
                    self.arguments(store, arguments)?;
                    None
                }
                Kind::Tuple(list) => Some(list),
                Kind::Function {
                    parameters, result, ..
                } => {
                    self.work.push(result);
                    Some(parameters)
                }
                Kind::Adt(_, arguments)
                | Kind::Opaque(_, arguments)
                | Kind::Dynamic { arguments, .. }
                | Kind::FunctionItem { arguments, .. } => {
                    if let Some(Kind::FunctionItem { signature, .. }) = store.kind(ty) {
                        self.work.push(signature);
                    }
                    self.arguments(store, arguments)?;
                    None
                }
                Kind::Projection(application) => {
                    if self.opaque_projections {
                        continue;
                    }
                    self.work.push(application.subject);
                    let trait_ = store
                        .trait_application(application.trait_application)
                        .ok_or("invalid parameter-use projection trait")?;
                    self.arguments(store, trait_.arguments)?;
                    self.arguments(store, application.arguments)?;
                    None
                }
                Kind::Never | Kind::Primitive(_) => None,
            };
            if let Some(list) = list {
                let children = store.list(list).ok_or("invalid parameter use list")?;
                self.capacity(children.len())?;
                self.work.extend_from_slice(children);
            }
        }
        Ok(())
    }
    fn arguments(
        &mut self,
        store: &Store,
        arguments: crate::types::GenericArgs,
    ) -> Result<(), &'static str> {
        let arguments = store
            .arguments(arguments)
            .ok_or("invalid parameter use arguments")?;
        self.capacity(arguments.len())?;
        self.steps += arguments.len();
        for argument in arguments {
            match argument {
                Argument::Type(ty) => self.work.push(ty),
                Argument::Const(id) => self.constant(store, id)?,
                Argument::Lifetime(_) => {}
            }
        }
        Ok(())
    }
    fn constant(
        &mut self,
        store: &Store,
        id: crate::types::arguments::ConstId,
    ) -> Result<(), &'static str> {
        if let crate::types::arguments::ConstKind::Parameter(id) = store
            .const_kind(id)
            .ok_or("invalid parameter use constant")?
        {
            self.constants.insert(id);
        }
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
