//! Only a live actual-parent proof supplies a conditional child's formal premises.
use super::{Declarations, Engine, Error, Report, Target, body};
use crate::{sema::inline_constants::InlineInstance, types::TypeId};
impl Engine {
    pub(super) fn paid_anonymous_root(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        instance: InlineInstance,
        result: TypeId,
    ) -> Result<usize, Error> {
        let source = instance.source();
        let error = |message: &str| report.body_error(source, message);
        self.validate_parent_scope(facts, Target::Anonymous { instance, result })
            .map_err(error)?;
        if let Some(&index) = self.roots.get(&Target::Anonymous { instance, result }) {
            self.catalog.validate_entry(index).map_err(error)?;
            return Ok(index);
        }
        let (template, contract) = if let Some(original) = self.anonymous.remove(&source) {
            original
        } else {
            self.anonymous_template(report, facts, source)?
        };
        let built = (|| {
            let (_, paid) = self.paid_interface(facts, instance, result)?;
            body::instantiate::conditional_anonymous_result(
                facts,
                &template,
                paid.port(),
                &contract,
            )?
            .into_requested(result)
        })();
        let environment = std::sync::Arc::clone(&template.environment);
        self.anonymous.insert(source, (template, contract));
        let mut concrete = built.map_err(error)?;
        body::constants::eligibility_charged(report, facts, &concrete.body)?;
        body::dispatch::resolve(facts, &mut concrete.body, &mut concrete.contract)
            .map_err(|e| report.body_error(source, e))?;
        body::promote::instance_reusing(
            report,
            facts,
            &mut concrete.body,
            &concrete.destruction,
            &mut self.promotions,
        )?;
        concrete
            .contract
            .prepare_outlives(&mut facts.types, &mut facts.lifetimes.nominal_requirements)
            .map_err(error)?;
        let regions = self
            .paid_child_regions(facts, &concrete.body, &concrete.contract, instance, result)
            .map_err(|e| report.body_error(e.source.unwrap_or(source.into()), e.message))?;
        self.prove_loans(
            report,
            facts,
            &concrete.body,
            &concrete.destruction,
            &environment,
            &regions,
        )?;
        let (code, needed) = self
            .finish_code(
                facts,
                &concrete.body,
                &concrete.contract,
                &regions,
                super::ProofPurpose::Code,
            )
            .map_err(error)?;
        let mut checked = super::CheckedInput {
            retained: needed.then(|| std::sync::Arc::new(concrete.contract.clone())),
            code,
        };
        let (scope, _) = self
            .paid_interface(facts, instance, result)
            .map_err(error)?;
        let index = self
            .catalog
            .paid_anonymous_prepared(scope, checked.take_code().map_err(error)?, result)
            .map_err(error)?;
        self.retain_const_inputs(facts, index, &concrete.body, checked, None)
            .map_err(error)?;
        facts.constant_work.anonymous_instances =
            facts.constant_work.anonymous_instances.saturating_add(1);
        self.roots
            .insert(Target::Anonymous { instance, result }, index);
        Ok(index)
    }
    fn paid_child_regions(
        &self,
        facts: &mut Declarations,
        body: &body::ir::Body,
        input: &body::regions::Contract,
        instance: InlineInstance,
        result: TypeId,
    ) -> Result<crate::borrow::regions::Body, crate::borrow::regions::Error> {
        let failure = |message: &str| crate::borrow::regions::Error {
            source: Some(body.source.into()),
            message: message.into(),
        };
        let (_, paid) = self
            .paid_interface(facts, instance, result)
            .map_err(failure)?;
        let mut queries = crate::types::outlives::Queries::new(
            &facts.types,
            &facts.lifetimes.nominal_requirements,
            &facts.lifetimes.callables,
        );
        crate::borrow::regions::paid_inline(
            &facts.types,
            &facts.lifetimes.variances,
            &mut queries,
            input,
            body,
            &paid,
        )
    }
    /// The only borrower of a paid child also charges failed exact-owner lookups.
    fn paid_interface(
        &self,
        facts: &mut Declarations,
        instance: InlineInstance,
        result: TypeId,
    ) -> Result<
        (
            crate::const_eval::lease_codes::Scope,
            crate::borrow::incremental::PaidInline<'_>,
        ),
        &'static str,
    > {
        let active = self
            .original_components
            .as_ref()
            .and_then(|rows| rows.active.as_ref())
            .ok_or("conditional child has no actual proved parent")?;
        let mut work = 0;
        let port = active.owner.pending().request(
            facts,
            active.owner.body(),
            active.owner.input(),
            instance,
            result,
            &mut work,
        );
        facts.body_work.canonical_steps = facts.body_work.canonical_steps.saturating_add(work);
        Ok((active.code, active.owner.paid(port?)?))
    }
}
