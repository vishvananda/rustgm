//! Even unused alias declarations consume ordinary formation and scoped-region proofs.
use super::{Checker, Edge, Error};
use crate::{sema::nominals::Proof, types::arguments::Argument};

impl Checker<'_> {
    pub(in crate::sema::declarations) fn validate_alias_declarations(
        &mut self,
    ) -> Result<(), Error> {
        let count = self
            .trait_aliases
            .as_ref()
            .map_or(0, |table| table.headers.len());
        for index in 0..count {
            let pending = self
                .trait_aliases
                .as_ref()
                .ok_or_else(|| Error::unlocated("missing alias declaration owner"))?;
            let header = &pending.headers[index];
            let owner = header.owner;
            if !header.completed || !pending.closed {
                return Err(self.error(owner.source, "alias declaration is not complete"));
            }
            let edges = header.edges.len();
            pending
                .budget
                .charge(edges.saturating_add(1), 0)
                .map_err(|error| self.error(owner.source, error))?;
            let environment = self
                .facts
                .generics
                .environments
                .get(&owner.definition)
                .cloned();
            let mut proof = self.declaration_proof(owner.definition, environment);
            self.validate_bounds(owner.definition, &mut proof)?;
            for edge in 0..edges {
                let edge = self
                    .trait_aliases
                    .as_ref()
                    .ok_or_else(|| Error::unlocated("missing alias argument owner"))?
                    .headers[index]
                    .edges[edge];
                self.validate_alias_arguments(edge, &mut proof)?;
            }
            self.retain_trait_proof(owner.definition, owner.source, &mut proof);
        }
        Ok(())
    }

    fn validate_alias_arguments(&mut self, edge: Edge, proof: &mut Proof) -> Result<(), Error> {
        if edge.quantified.is_some() {
            return Err(self.error(
                edge.source,
                "quantified alias argument formation requires a scoped consumer",
            ));
        }
        let count = self
            .facts
            .types
            .arguments(edge.arguments)
            .ok_or_else(|| self.error(edge.source, "invalid alias formation arguments"))?
            .len();
        self.trait_aliases
            .as_ref()
            .ok_or_else(|| self.error(edge.source, "missing alias formation owner"))?
            .budget
            .charge(count.saturating_add(1), 0)
            .map_err(|error| self.error(edge.source, error))?;
        for index in 0..=count {
            let ty = if index == count {
                edge.original_subject
            } else {
                let argument = self
                    .facts
                    .types
                    .arguments(edge.arguments)
                    .and_then(|mut values| values.nth(index))
                    .ok_or_else(|| self.error(edge.source, "missing alias formation argument"))?;
                match argument {
                    Argument::Type(ty) => ty,
                    Argument::Const(value) => {
                        self.facts.types.const_type(value).ok_or_else(|| {
                            self.error(edge.source, "missing alias const argument type")
                        })?
                    }
                    Argument::Lifetime(_) => continue,
                }
            };
            if !proof
                .check(&mut self.facts, ty)
                .map_err(|error| self.error(edge.source, error))?
                .well_formed
            {
                return Err(self.error(edge.source, "alias argument must be well-formed"));
            }
        }
        Ok(())
    }
}
