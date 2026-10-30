//! Reservation declarations preserve checking without acquiring positive impl authority.
use super::{Checker, DefId, Error, NodeKey};
use crate::sema::traits::implementations::Polarity;
impl Checker<'_> {
    pub(super) fn reservation_impl(
        &self,
        source: Option<NodeKey>,
        owner: DefId,
    ) -> Result<bool, Error> {
        let registry = self
            .facts
            .generics
            .implementations
            .as_ref()
            .ok_or_else(|| self.implementation_error(source, "missing impl polarity registry"))?;
        registry
            .polarity(&self.facts.types, &self.facts.associated, owner)
            .map(|polarity| polarity == Polarity::Reservation)
            .map_err(|error| self.implementation_error(source, error))
    }
    pub(in crate::sema::declarations::traits) fn reserved_type_parent(
        &mut self,
        owner: DefId,
        key: NodeKey,
        parent: crate::types::parameters::predicates::Predicate,
        proof: &mut crate::sema::nominals::Proof,
    ) -> Result<(), Error> {
        if !self.reservation_impl(Some(key), owner)? {
            return Ok(());
        }
        let (application, _) = parent
            .bound
            .trait_parts()
            .ok_or_else(|| self.error(key, "associated type lost its actual parent trait"))?;
        let requirement = crate::types::parameters::predicates::Predicate {
            ty: parent.ty,
            bound: crate::types::parameters::predicates::Bound::Trait(application),
        };
        crate::sema::header_attributes::charge_source_contract_query(&self.facts, 1)?;
        let before = proof.trait_selection_steps();
        let result = proof.prove_clause(
            &mut self.facts,
            crate::types::parameters::clauses::Clause::Predicate(requirement),
        );
        crate::sema::header_attributes::charge_source_contract_query(
            &self.facts,
            proof.trait_selection_steps().saturating_sub(before),
        )?;
        if !result.map_err(|error| self.error(key, error))? {
            return Err(self.error(
                key,
                "associated type definition requires actual parent trait proof",
            ));
        }
        Ok(())
    }
    pub(super) fn reservation_pair(
        &self,
        source: Option<NodeKey>,
        owners: [DefId; 2],
        steps: &mut usize,
    ) -> Result<bool, Error> {
        *steps = steps.saturating_add(2);
        if *steps > 1_000_000 {
            return Err(self.implementation_error(
                source,
                "trait implementation overlap work budget exceeded",
            ));
        }
        for owner in owners {
            if self.reservation_impl(source, owner)? {
                return Ok(true);
            }
        }
        Ok(false)
    }
}
