//! Freeze declaration size conditions before final shape and field validation.
use super::{Checker, Error};
use crate::sema::nominals::sizes;
use std::sync::Arc;

impl Checker<'_> {
    pub(super) fn complete_nominal_sizes(&mut self) -> Result<(), Error> {
        if self.nominals.entries.is_empty()
            && self
                .report
                .resolution
                .crates
                .as_ref()
                .is_none_or(|imported| imported.records().next().is_none())
        {
            return Ok(());
        }
        for index in 0..self.nominals.entries.len() {
            let entry = &self.nominals.entries[index];
            let missing = sizes::prepare(&mut self.facts, entry.item)
                .map_err(|error| Error::at(entry.key.file, 0, error))?;
            if missing.is_some() {
                return Err(self.error(entry.key, "incomplete nominal sizedness dependency"));
            }
        }
        let pending = self.facts.generics.size_rules.take().ok_or_else(|| {
            Error::at(
                crate::resolve::unit::FileId(0),
                0,
                "missing size declarations",
            )
        })?;
        if !pending.active.is_empty() {
            return Err(Error::at(
                crate::resolve::unit::FileId(0),
                0,
                "unfinished nominal fields",
            ));
        }
        self.facts
            .types
            .seal_nominal_sizes(Arc::new(pending.rules))
            .map_err(|error| Error::at(crate::resolve::unit::FileId(0), 0, error))?;
        for requirement in pending.requirements {
            let first = u32::try_from(self.facts.body_regions.len())
                .ok()
                .and_then(|count| count.checked_add(1))
                .ok_or_else(|| self.error(requirement.source, "too many body regions"))?;
            let mut proof =
                crate::sema::nominals::Proof::for_declaration(requirement.environment, 0, first);
            if !proof
                .prove(
                    &mut self.facts,
                    crate::types::parameters::predicates::Predicate {
                        ty: requirement.ty,
                        bound: crate::types::parameters::predicates::Bound::Sized,
                    },
                )
                .map_err(|e| self.error(requirement.source, e))?
            {
                return Err(self.error(requirement.source, "early generic argument must be sized"));
            }
            if let Some(proof) = proof.take_trait_proof() {
                proof
                    .retain_body(&mut self.facts, requirement.body, requirement.source)
                    .map_err(|e| self.error(requirement.source, e))?;
            }
        }
        for dispatch in pending.dispatches {
            let source = self.report.resolution.definitions[dispatch.owner.0 as usize].node;
            crate::sema::dispatch::Dispatcher::recheck_size(&mut self.facts, dispatch).map_err(
                |error| match source {
                    Some(source) => self.error(source, error),
                    None => Error::at(crate::resolve::unit::FileId(0), 0, error),
                },
            )?;
        }
        Ok(())
    }
}
