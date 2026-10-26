//! Even unused alias declarations consume ordinary formation and scoped-region proofs.
use super::{Checker, Edge, Error, Header};
use crate::{
    sema::nominals::Proof,
    types::{
        arguments::Argument,
        parameters::{clauses::Clause, predicates::Bound},
    },
};

impl Checker<'_> {
    /// Direct contracts disappear into ordinary caller predicates. Wider forms
    /// retain their fence until every formation and substitution consumer exists.
    pub(super) fn direct_alias_execution(&self, header: &Header) -> Result<bool, Error> {
        let source = header.owner.source;
        let error = |message| self.error(source, message);
        let schema = header
            .schema
            .as_ref()
            .ok_or_else(|| error("missing alias schema"))?;
        let parameters = self
            .facts
            .generics
            .parameters
            .get(&header.owner.definition)
            .ok_or_else(|| error("missing alias formals"))?;
        let clauses = self
            .facts
            .generics
            .predicates
            .get(&header.owner.definition)
            .ok_or_else(|| error("missing alias clauses"))?;
        let ast = &self.report.package.unit(source.file).configured.ast;
        let crate::syntax::ast::Kind::Item(item) = ast.node(source.node).kind else {
            return Err(error("alias owner lost its actual declaration"));
        };
        let crate::syntax::item::ItemKind::TraitAlias {
            constant: false, ..
        } = ast.items[item as usize].kind
        else {
            return Ok(false);
        };
        if !parameters.is_empty()
            || schema.captures != 0
            || !schema.bounds.is_empty()
            || !header.edges.is_empty()
        {
            return Ok(false);
        }
        self.trait_aliases
            .as_ref()
            .ok_or_else(|| error("missing alias owner"))?
            .budget
            .charge(clauses.len().saturating_add(1), 0)
            .map_err(error)?;
        for clause in clauses {
            let Clause::Predicate(predicate) = clause.value else {
                return Ok(false);
            };
            if matches!(
                predicate.bound,
                Bound::ConstTrait(_) | Bound::MaybeConstTrait(_)
            ) || !clause
                .value
                .checks_parameter_default(&self.facts.types)
                .map_err(error)?
            {
                return Ok(false);
            }
        }
        Ok(true)
    }

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
