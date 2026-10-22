//! Both activation routes use ordinary impl, associated and method proof machinery.
use super::{Checker, Collection, DefId, Error};
use crate::{
    resolve::expansion::Report,
    sema::{
        iteration::collections::{Original, Status},
        model::Declarations,
        nominals::Proof,
        traits::conformance,
    },
    types::{
        Kind,
        outlives::Queries,
        parameters::{
            clauses::Clause,
            predicates::{Bound, Predicate},
        },
        relations::{Constraints, ProofScope},
    },
};
impl Checker<'_> {
    pub(in super::super::super) fn validate_collection_impl(
        &mut self,
        owner: DefId,
    ) -> Result<bool, Error> {
        if !self
            .facts
            .iteration
            .as_deref()
            .and_then(|r| r.collections.as_deref())
            .is_some_and(|r| {
                r.originals
                    .iter()
                    .flatten()
                    .any(|o| o.declaration.definition == owner && o.predicate.is_some())
            })
        {
            return Ok(false);
        }
        complete(self.report, &mut self.facts, false)?;
        Ok(true)
    }
}
pub(super) fn complete(
    report: &Report,
    facts: &mut Declarations,
    overlap: bool,
) -> Result<(), Error> {
    let Some(registry) = facts
        .iteration
        .as_deref()
        .and_then(|r| r.collections.as_deref())
    else {
        return Ok(());
    };
    if registry.status == Status::Complete {
        return Ok(());
    }
    let originals = registry.originals;
    let source = registry.canonical.origin;
    if overlap {
        check_overlap(report, facts, source)?;
    }
    let mut builder = super::super::super::comparisons::LibraryContracts { report, facts };
    for kind in [
        Collection::SliceIterator,
        Collection::SharedSlice,
        Collection::SharedArray,
    ] {
        let original = originals[kind as usize]
            .ok_or_else(|| builder.error(source, "missing prepared collection original"))?;
        if !builder
            .facts
            .generics
            .implementations
            .as_ref()
            .is_some_and(|r| r.is_validated(original.declaration.definition))
        {
            builder.validate_collection(original)?;
        }
    }
    builder
        .facts
        .iteration
        .as_deref_mut()
        .and_then(|r| r.collections.as_deref_mut())
        .ok_or_else(|| Error::at(source.file, 0, "missing collection completion marker"))?
        .status = Status::Complete;
    super::super::super::cloning::prepare_requested(report, facts)
}
pub(in crate::sema::declarations) fn check_overlap(
    report: &Report,
    facts: &mut Declarations,
    source: crate::resolve::unit::NodeKey,
) -> Result<(), Error> {
    let pairs = facts
        .generics
        .implementations
        .as_mut()
        .ok_or_else(|| Error::at(source.file, 0, "missing collection candidate inventory"))?
        .take_overlap_pairs();
    let mut steps = 0usize;
    for (a, owner_a, b, owner_b) in pairs {
        let (possible, work) = crate::sema::traits::implementations::overlap::possible(
            facts,
            (a, owner_a),
            (b, owner_b),
        )
        .map_err(|e| Declarations::error(report, source, e))?;
        steps = steps.saturating_add(work);
        let possible = if possible {
            let (possible, work) = crate::sema::traits::implementations::overlap::iteration(
                facts,
                (a, owner_a),
                (b, owner_b),
            )
            .map_err(|e| Declarations::error(report, source, e))?;
            steps = steps.saturating_add(work);
            possible
        } else {
            false
        };
        if steps > 1_000_000 {
            return Err(Declarations::error(
                report,
                source,
                "trait implementation overlap work budget exceeded",
            ));
        }
        if possible {
            return Err(Declarations::error(
                report,
                source,
                "overlapping trait implementation headers require coherence proof",
            ));
        }
    }
    Ok(())
}
impl super::super::super::comparisons::LibraryContracts<'_> {
    fn validate_collection(&mut self, original: Original) -> Result<(), Error> {
        let declaration = original.declaration;
        let owner = declaration.definition;
        let canonical = self
            .facts
            .iteration
            .as_deref()
            .ok_or_else(|| {
                Error::at(
                    super::super::super::FileId(0),
                    0,
                    "missing iteration registry",
                )
            })?
            .canonical;
        let source = canonical.origin;
        let predicate = original
            .predicate
            .ok_or_else(|| self.error(source, "collection impl has no predicate"))?;
        let expected = canonical.get(
            declaration
                .kind
                .trait_()
                .ok_or_else(|| self.error(source, "collection impl has no trait"))?,
        );
        let environment = self.facts.generics.environments.get(&owner).cloned();
        let mut proof =
            Proof::for_declaration(environment, u32::from(declaration.lifetime.is_some()), 1);
        let requirements = super::super::super::traits::impl_contract_obligations(
            self.facts, predicate, &mut proof,
        )
        .map_err(|e| self.error(source, e))?;
        let mut types = vec![(
            expected.item,
            declaration
                .item
                .ok_or_else(|| self.error(source, "missing collection Item"))?,
        )];
        if let (Some(expected), Some(actual)) = (expected.into_iter, declaration.into_iter) {
            types.push((expected, actual));
        }
        self.collection_associated(source, predicate, &types, &mut proof)?;
        self.library_impl_lifetimes(
            owner,
            u32::from(declaration.lifetime.is_some()),
            source,
            predicate,
            &requirements,
            &mut proof,
        )?;
        let definitions = conformance::Definitions::new(self.facts, predicate, &types)
            .map_err(|e| self.error(source, e))?;
        let function = |id| {
            self.facts
                .associated
                .language_functions()
                .find(|f| f.definition == id)
                .copied()
        };
        let actual = function(declaration.method)
            .ok_or_else(|| self.error(source, "missing collection impl method"))?;
        let expected = function(expected.method)
            .ok_or_else(|| self.error(source, "missing collection trait method"))?;
        conformance::check(self.facts, owner, predicate, expected, actual, &definitions)
            .map_err(|e| self.error(source, e))?;
        self.facts
            .generics
            .implementations
            .as_mut()
            .ok_or_else(|| {
                Error::at(
                    source.file,
                    0,
                    "missing collection implementation inventory",
                )
            })?
            .complete(
                owner,
                vec![(expected.definition, Some(actual.definition))],
                types,
                Vec::new(),
            )
            .map_err(|e| self.error(source, e))?;
        Ok(())
    }
    fn collection_associated(
        &mut self,
        source: crate::resolve::unit::NodeKey,
        predicate: Predicate,
        types: &[(super::DefId, super::DefId)],
        proof: &mut Proof,
    ) -> Result<(), Error> {
        let mut substitution = None;
        for &(expected, actual) in types {
            let ty = self.facts.definitions[actual.0 as usize]
                .ok_or_else(|| self.error(source, "missing collection associated definition"))?;
            if !proof
                .check(self.facts, ty)
                .map_err(|e| self.error(source, e))?
                .well_formed
            {
                return Err(self.error(
                    source,
                    "collection associated definition is not well-formed",
                ));
            }
            for requirement in
                self.associated_result_bounds(source, predicate, expected, ty, &mut substitution)?
            {
                if !proof
                    .prove_clause(self.facts, requirement)
                    .map_err(|e| self.error(source, e))?
                {
                    return Err(self.error(
                        source,
                        "collection associated definition does not satisfy its contract",
                    ));
                }
            }
        }
        Ok(())
    }
    pub(in crate::sema::declarations) fn library_impl_lifetimes(
        &self,
        owner: super::DefId,
        captures: u32,
        source: crate::resolve::unit::NodeKey,
        predicate: Predicate,
        requirements: &crate::sema::traits::Requirements,
        proof: &mut Proof,
    ) -> Result<(), Error> {
        let mut queries = Queries::new(
            &self.facts.types,
            &self.facts.lifetimes.nominal_requirements,
            &self.facts.lifetimes.callables,
        );
        let mut constraints = Constraints::new(
            &self.facts.types,
            &self.facts.lifetimes.variances,
            captures,
            1_000_000,
        )
        .map_err(|e| self.error(source, e.to_string()))?;
        for parameter in self.facts.generics.parameters_in_scope(owner) {
            if let crate::types::arguments::Argument::Type(ty) = parameter.argument
                && let Some(Kind::Parameter(id)) = self.facts.types.kind(ty)
            {
                constraints
                    .parameter(id)
                    .map_err(|e| self.error(source, e.to_string()))?;
            }
        }
        // Precisely the real impl input WF supplies its field outlives facts.
        constraints
            .assume_well_formed(&mut queries, predicate.ty)
            .map_err(|e| self.error(source, e))?;
        constraints
            .assume_outlives_clauses(
                &mut queries,
                self.facts
                    .generics
                    .predicates_in_scope(owner)
                    .map(|p| p.value),
            )
            .map_err(|e| self.error(source, e))?;
        for &(a, b) in &requirements.regions {
            constraints
                .outlives(a, b, 0)
                .map_err(|e| self.error(source, e.to_string()))?;
        }
        for &requirement in requirements.required.iter().chain(&requirements.implied) {
            if let Clause::Predicate(Predicate {
                ty,
                bound: Bound::Outlives(shorter),
            }) = requirement
            {
                constraints
                    .require_type(&mut queries, ty, shorter, ProofScope::ROOT, 0)
                    .map_err(|e| self.error(source, e))?;
            }
        }
        if let Some(proof) = proof.take_trait_proof() {
            proof
                .require(&mut constraints, &mut queries)
                .map_err(|e| self.error(source, e))?;
        }
        constraints.solve().map(|_| ()).map_err(|e| {
            self.error(
                source,
                format!("collection impl lifetime requirement is not satisfied: {e}"),
            )
        })
    }
}
