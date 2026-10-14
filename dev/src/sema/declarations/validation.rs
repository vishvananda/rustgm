//! Validate declaration uses under their own immutable assumptions.
use super::{Checker, Error, Type};
use crate::sema::nominals::Proof;

impl Checker<'_> {
    pub(super) fn validate_signatures(&mut self) -> Result<(), Error> {
        for index in 0..self.facts.functions.len() {
            let function = self.facts.functions[index];
            let environment = self
                .facts
                .generics
                .environments
                .get(&function.definition)
                .cloned();
            let mut proof = Proof::new(environment);
            self.validate_bounds(function.definition, &mut proof)?;
            let Some(Type::Function {
                parameters, result, ..
            }) = self.facts.types.kind(function.signature)
            else {
                return Err(self.error(function.node, "invalid function signature"));
            };
            let count = self
                .facts
                .types
                .list(parameters)
                .ok_or_else(|| self.error(function.node, "invalid parameters"))?
                .len();
            for index in 0..=count {
                let ty = if index == count {
                    result
                } else {
                    self.facts
                        .types
                        .list(parameters)
                        .ok_or_else(|| self.error(function.node, "invalid parameters"))?[index]
                };
                let shape = proof
                    .check(&mut self.facts, ty)
                    .map_err(|e| self.error(function.node, e))?;
                if !shape.well_formed {
                    return Err(self.error(
                        function.node,
                        "function signature contains an ill-formed type",
                    ));
                }
                if !shape.sized {
                    return Err(self.error(
                        function.node,
                        "function parameters and result must be sized",
                    ));
                }
            }
        }
        let mut proof = Proof::new(None);
        for index in 0..self.facts.initializers.len() {
            let initializer = self.facts.initializers[index];
            let shape = proof
                .check(&mut self.facts, initializer.ty)
                .map_err(|e| self.error(initializer.node, e))?;
            if !shape.well_formed || !shape.sized {
                return Err(self.error(
                    initializer.node,
                    "constant and static types must be well-formed and sized",
                ));
            }
        }
        Ok(())
    }
}
