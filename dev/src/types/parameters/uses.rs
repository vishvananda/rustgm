//! Shared type graphs are visited once while accumulating rigid parameter uses.
use super::ParameterId;
use crate::types::{Kind, Store, TypeId, arguments::Argument};
use std::collections::HashSet;
#[derive(Default)]
pub struct Uses {
    pub parameters: HashSet<ParameterId>,
    pub constants: HashSet<ParameterId>,
    seen: HashSet<TypeId>,
    work: Vec<TypeId>,
    pub steps: usize,
}
impl Uses {
    /// Accumulate type and const parameter occurrences from completed types in one store.
    /// # Errors
    /// Rejects malformed canonical identities and exhausted traversal resources.
    pub fn collect(&mut self, store: &Store, root: TypeId) -> Result<(), &'static str> {
        self.work.push(root);
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
                Kind::Tuple(list) => Some(list),
                Kind::Function {
                    parameters, result, ..
                } => {
                    self.work.push(result);
                    Some(parameters)
                }
                Kind::Adt(_, arguments) | Kind::FunctionItem { arguments, .. } => {
                    if let Some(Kind::FunctionItem { signature, .. }) = store.kind(ty) {
                        self.work.push(signature);
                    }
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
