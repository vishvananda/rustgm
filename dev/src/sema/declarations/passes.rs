//! Crate-wide declaration passes consume every declaration's completed facts.
//!
//! Ordinary checking stops at the first failed pass. An analysis-only census
//! records that failure as its terminal record, then still runs each later pass
//! as ordinary checking would, on whatever facts exist. Those lookahead records
//! follow the failure and may be derived from it; they never make a census, or
//! any facts, usable.
use super::{Checker, Error, FileId};
use crate::sema::census;

type Pass = for<'a, 'b> fn(&'b mut Checker<'a>, &mut usize, usize) -> Result<(), Error>;

/// Each later pass's first error after the failed crate-wide pass.
pub(super) struct Lookahead {
    /// The failed pass, which the terminal record names.
    pub(super) pass: &'static str,
    /// Each later pass that failed and its first error, in pass order.
    pub(super) following: Vec<(&'static str, Error)>,
}

/// Each pass in its fixed order, named for census records.
const PASSES: [(&str, Pass); 29] = [
    ("complete_deferred_declarations", |checker, steps, limit| {
        checker.complete_deferred_declarations(steps, limit)
    }),
    ("complete_nominals", |checker, steps, limit| {
        checker.complete_nominals(steps, limit)
    }),
    ("complete_compiler_value_headers", |checker, _, _| {
        checker.complete_compiler_value_headers()
    }),
    ("prepare_opaque_outlives", |checker, _, _| {
        crate::sema::traits::results::prepare_opaque_outlives(&mut checker.facts)
            .map_err(|error| Error::at(FileId(0), 0, error))
    }),
    ("trait_defaults", |checker, _, _| checker.trait_defaults()),
    ("validate_destructors", |checker, _, _| {
        checker.validate_destructors()
    }),
    ("validate_async_destructors", |checker, _, _| {
        checker.validate_async_destructors()
    }),
    ("validate_owner_bounds", |checker, _, _| {
        checker.validate_owner_bounds()
    }),
    ("validate_associated_type_bounds", |checker, _, _| {
        checker.validate_associated_type_bounds()
    }),
    ("validate_required_family_bounds", |checker, _, _| {
        checker.validate_required_family_bounds().map(drop)
    }),
    ("validate_signatures", |checker, _, _| {
        checker.validate_signatures()
    }),
    ("complete_pinning", |checker, _, _| {
        crate::sema::compiler_items::pinning::complete(checker.report, &mut checker.facts)
    }),
    ("complete_interval_constructor", |checker, _, _| {
        crate::sema::compiler_items::interval_values::complete_constructor(
            checker.report,
            &mut checker.facts,
        )
    }),
    ("complete_stream_helpers", |checker, _, _| {
        crate::sema::compiler_items::streams::complete_helpers(checker.report, &mut checker.facts)
    }),
    ("validate_trait_implementations", |checker, _, _| {
        checker.validate_trait_implementations()
    }),
    ("validate_const_parameter_types", |checker, _, _| {
        checker.validate_const_parameter_types()
    }),
    ("validate_trait_impl_owners", |checker, _, _| {
        checker.validate_trait_impl_owners()
    }),
    ("validate_trait_regions", |checker, _, _| {
        checker.validate_trait_regions()
    }),
    ("validate_inherent_overlaps", |checker, _, _| {
        checker.validate_inherent_overlaps()
    }),
    ("local_annotations", |checker, steps, limit| {
        checker.local_annotations(steps, limit)
    }),
    ("complete_alias_normalizations", |checker, steps, limit| {
        checker.complete_alias_normalizations(steps, limit)
    }),
    ("validate_projection_regions", |checker, _, _| {
        checker.validate_projection_regions()
    }),
    ("validate_objects", |checker, _, _| {
        checker.validate_objects()
    }),
    ("complete_discriminants", |checker, steps, limit| {
        checker.complete_discriminants(steps, limit)
    }),
    ("finish_method_queries", |checker, _, _| {
        if checker.method_queries.is_empty() {
            Ok(())
        } else {
            Err(Error::at(
                FileId(0),
                0,
                "unfinished method applicability queries",
            ))
        }
    }),
    ("complete_layouts", |checker, _, _| {
        checker.complete_layouts()
    }),
    ("check_api_exposure", |checker, _, _| {
        crate::sema::api_exposure::check(checker.report, &checker.facts).map(drop)
    }),
    ("reject_alias_execution", |checker, _, _| {
        checker.reject_alias_execution()
    }),
    ("require_specialization_execution", |checker, _, _| {
        crate::sema::specialization::require_execution(&checker.facts)
    }),
];

impl Checker<'_> {
    /// Run every crate-wide pass in order and return the first error. A census
    /// whose first failure is not sticky budget exhaustion first parks its
    /// ledger, so each later pass runs exactly as in ordinary checking and
    /// records nothing, and keeps each later pass's first error as lookahead.
    pub(super) fn crate_passes(&mut self, steps: &mut usize, limit: usize) -> Result<(), Error> {
        for (index, &(pass, run)) in PASSES.iter().enumerate() {
            let Err(error) = run(self, steps, limit) else {
                continue;
            };
            if self.census.is_some() && !Self::sticky(&error, *steps, limit) {
                let parked = self.census.take();
                let later = PASSES.get(index + 1..).unwrap_or_default();
                let following = self.look_ahead(later, steps, limit);
                self.census = parked;
                if let Some(ledger) = self.census.as_deref_mut() {
                    ledger.lookahead = Some(Box::new(Lookahead { pass, following }));
                }
            }
            return Err(error);
        }
        Ok(())
    }
    /// Budget exhaustion is sticky crate state, so it also ends a lookahead.
    fn look_ahead(
        &mut self,
        passes: &[(&'static str, Pass)],
        steps: &mut usize,
        limit: usize,
    ) -> Vec<(&'static str, Error)> {
        let mut following = Vec::new();
        for &(pass, run) in passes {
            if let Err(error) = run(self, steps, limit) {
                let sticky = Self::sticky(&error, *steps, limit);
                following.push((pass, error));
                if sticky {
                    break;
                }
            }
        }
        following
    }
    fn sticky(error: &Error, steps: usize, limit: usize) -> bool {
        steps > limit || census::exhausts_budget(&error.message)
    }
}
