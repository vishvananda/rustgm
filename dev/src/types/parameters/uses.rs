//! Shared type graphs are visited once while accumulating rigid parameter uses.
use super::ParameterId;
use crate::types::{Kind, Store, TypeId, arguments::Argument};
use std::collections::HashSet;
#[derive(Default)]
pub struct Uses {
    pub parameters: HashSet<ParameterId>,
    seen: HashSet<TypeId>,
    work: Vec<TypeId>,
    pub steps: usize,
}
impl Uses {
    /// Accumulate type parameter occurrences from completed types in one store.
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
                Kind::Array { element, .. }
                | Kind::ArrayTerm { element, .. }
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
                    for argument in arguments {
                        if let Argument::Type(ty) = argument {
                            self.work.push(ty);
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
