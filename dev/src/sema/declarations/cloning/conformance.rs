//! The canonical Clone impl pays normal Self, method and lifetime requirements.
use super::Error;
use crate::{
    resolve::model::BuiltinCloning,
    sema::{cloning::Original, nominals::Proof, traits::conformance},
};
impl super::super::comparisons::LibraryContracts<'_> {
    pub(super) fn check_iterator_clone(
        &mut self,
        canonical: BuiltinCloning,
        original: Original,
    ) -> Result<(), Error> {
        let source = canonical.origin;
        let owner = original.declaration.definition;
        let environment = self.facts.generics.environments.get(&owner).cloned();
        let mut proof = Proof::for_declaration(environment, 1, 1);
        let requirements = super::super::traits::impl_contract_obligations(
            self.facts,
            original.predicate,
            &mut proof,
        )
        .map_err(|e| self.error(source, e))?;
        self.library_impl_lifetimes(
            owner,
            1,
            source,
            original.predicate,
            &requirements,
            &mut proof,
        )?;
        let definitions = conformance::Definitions::new(self.facts, original.predicate, &[])
            .map_err(|e| self.error(source, e))?;
        let function = |id| {
            self.facts
                .associated
                .language_functions()
                .find(|f| f.definition == id)
                .copied()
        };
        let actual = function(original.declaration.method)
            .ok_or_else(|| self.error(source, "missing iterator Clone method"))?;
        let expected = function(canonical.method)
            .ok_or_else(|| self.error(source, "missing canonical Clone method"))?;
        conformance::check(
            self.facts,
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
            .ok_or_else(|| Error::at(source.file, 0, "missing Clone implementation inventory"))?
            .complete(
                owner,
                vec![(canonical.method, Some(original.declaration.method))],
                Vec::new(),
                Vec::new(),
            )
            .map_err(|e| self.error(source, e))?;
        Ok(())
    }
}
