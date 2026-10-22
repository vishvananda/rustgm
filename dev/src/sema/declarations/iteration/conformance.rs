//! Canonical headers pay the same associated-result and override conformance duties.
use super::{Checker, DefId, Error};
use crate::{
    sema::traits::conformance,
    support::iteration::{Blanket, Trait},
};
impl Checker<'_> {
    pub(in super::super) fn validate_iteration_blanket(
        &mut self,
        owner: DefId,
    ) -> Result<bool, Error> {
        let Some(registry) = self.facts.iteration.as_deref() else {
            return Ok(false);
        };
        if !registry.owns_impl(owner) {
            return Ok(false);
        }
        let Some(original) = registry
            .originals
            .iter()
            .flatten()
            .find(|o| o.declaration.definition == owner)
            .copied()
        else {
            return Ok(false);
        };
        let canonical = registry.canonical;
        let source = canonical.origin;
        let declaration = original.declaration;
        let expected = canonical.get(if declaration.kind == Blanket::Identity {
            Trait::IntoIterator
        } else {
            Trait::Iterator
        });
        self.validate_trait_impl_contract(owner, source, original.predicate)?;
        let mut types = vec![(expected.item, declaration.item)];
        if let (Some(expected), Some(actual)) = (expected.into_iter, declaration.into_iter) {
            types.push((expected, actual));
        }
        self.iteration_associated_definitions(source, original, &types)?;
        let definitions = conformance::Definitions::new(&self.facts, original.predicate, &types)
            .map_err(|e| self.error(source, e))?;
        let function = |definition| {
            self.facts
                .associated
                .language_functions()
                .find(|f| f.definition == definition)
                .copied()
        };
        let actual = function(declaration.method)
            .ok_or_else(|| self.error(source, "missing canonical iteration impl method"))?;
        let expected = function(expected.method)
            .ok_or_else(|| self.error(source, "missing canonical iteration trait method"))?;
        conformance::check(
            &mut self.facts,
            owner,
            original.predicate,
            expected,
            actual,
            &definitions,
        )
        .map_err(|e| self.error(source, e))?;
        self.facts
            .generics
            .implementations
            .as_mut()
            .ok_or_else(|| {
                Error::at(
                    source.file,
                    0,
                    "missing canonical iteration implementations",
                )
            })?
            .complete(
                owner,
                vec![(expected.definition, Some(actual.definition))],
                types,
                Vec::new(),
            )
            .map_err(|e| self.error(source, e))?;
        Ok(true)
    }
    fn iteration_associated_definitions(
        &mut self,
        source: crate::resolve::unit::NodeKey,
        original: crate::sema::iteration::Original,
        types: &[(DefId, DefId)],
    ) -> Result<(), Error> {
        let owner = original.declaration.definition;
        let environment = self.facts.generics.environments.get(&owner).cloned();
        let mut proof = self.declaration_proof(owner, environment);
        let mut substitution = None;
        for &(expected, actual) in types {
            let ty = self.facts.definitions[actual.0 as usize].ok_or_else(|| {
                self.error(source, "missing canonical iteration associated definition")
            })?;
            if !proof
                .check(&mut self.facts, ty)
                .map_err(|e| self.error(source, e))?
                .well_formed
            {
                return Err(self.error(
                    source,
                    "canonical iteration associated definition is not well-formed",
                ));
            }
            for requirement in self.associated_result_bounds(
                source,
                original.predicate,
                expected,
                ty,
                &mut substitution,
            )? {
                if !proof
                    .prove_clause(&mut self.facts, requirement)
                    .map_err(|e| self.error(source, e))?
                {
                    return Err(self.error(
                        source,
                        "canonical iteration associated definition does not satisfy its contract",
                    ));
                }
            }
        }
        self.retain_trait_proof(owner, source, &mut proof);
        Ok(())
    }
}
