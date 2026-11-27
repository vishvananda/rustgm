//! Exact enclosing instances specialize one retained anonymous template and contract.
use super::{Contract, Declarations, Engine, Error, Report, Target, Template, body};
use crate::{sema::inline_constants::InlineInstance, types::TypeId};
impl Engine {
    pub(super) fn anonymous_root(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        instance: InlineInstance,
        result: TypeId,
    ) -> Result<usize, Error> {
        let origin = instance.origin(facts).ok_or_else(|| {
            Error::unlocated("anonymous instance lost its retained typed template")
        })?;
        let selected = instance
            .select_in(facts, instance.enclosing())
            .map_err(|e| report.body_error(origin, e))?;
        if selected != instance {
            return Err(report.body_error(origin, "anonymous instance identity changed"));
        }
        if self
            .original_components
            .as_ref()
            .and_then(|rows| rows.active.as_ref())
            .is_some_and(|active| active.owner.pending().has_source(origin))
        {
            return self.paid_anonymous_root(report, facts, instance, result);
        }
        if let crate::sema::inline_constants::Context::SourceInitializer(witness) =
            instance.enclosing()
        {
            let parent_result = witness
                .result(facts)
                .map_err(|e| report.body_error(origin, e))?;
            witness
                .validate(facts, parent_result)
                .map_err(|e| report.body_error(origin, e))?;
            self.root(
                report,
                facts,
                Target::SourceAssociated {
                    source: witness,
                    result: parent_result,
                },
            )?;
        }
        if let Some(&index) = self.roots.get(&Target::Anonymous { instance, result }) {
            return Ok(index);
        }
        let (mut concrete, environment) =
            self.specialize_anonymous(report, facts, instance, result, origin)?;
        body::constants::eligibility_charged(report, facts, &concrete.body)?;
        body::dispatch::resolve(facts, &mut concrete.body, &mut concrete.contract)
            .map_err(|e| report.body_error(origin, e))?;
        body::promote::instance_reusing(
            report,
            facts,
            &mut concrete.body,
            &concrete.destruction,
            &mut self.promotions,
        )?;
        let mut checked = self.prove_body(
            report,
            facts,
            &concrete.body,
            &concrete.destruction,
            super::ProofInput::code(&environment, &concrete.contract),
        )?;
        let results = if let crate::sema::inline_constants::Context::SourceInitializer(witness) =
            instance.enclosing()
        {
            body::instantiate::source_inline_results(
                facts,
                &concrete.body,
                &concrete.contract,
                witness,
            )
            .map_err(|e| report.body_error(origin, e))?
        } else {
            Vec::new()
        };
        let registration = self
            .catalog
            .stage_anonymous(
                checked
                    .take_code()
                    .map_err(|e| report.body_error(origin, e))?,
                result,
                &results,
            )
            .map_err(|e| report.body_error(origin, e))?;
        let index = registration
            .publish_source(facts, &mut self.const_calls, checked.retained, None)
            .map_err(|e| report.body_error(origin, e))?;
        self.refresh_code_bytes(facts);
        facts.constant_work.anonymous_instances += 1;
        self.roots
            .insert(Target::Anonymous { instance, result }, index);
        Ok(index)
    }
    fn specialize_anonymous(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        instance: InlineInstance,
        result: TypeId,
        origin: crate::body::ir::BodyOrigin,
    ) -> Result<
        (
            body::instantiate::Materialized,
            std::sync::Arc<crate::types::parameters::environment::Environment>,
        ),
        Error,
    > {
        Ok(match origin {
            crate::resolve::body_origin::BodyOrigin::Source(source) => {
                let (template, contract) = if let Some(retained) = self.anonymous.remove(&source) {
                    retained
                } else {
                    self.anonymous_template(report, facts, source)?
                };
                let built = body::instantiate::anonymous_result(
                    facts, &template, instance, result, &contract,
                )
                .and_then(|materialized| materialized.into_requested(result))
                .map_err(|e| report.body_error(origin, e));
                let environment = std::sync::Arc::clone(&template.environment);
                self.anonymous.insert(source, (template, contract));
                (built?, environment)
            }
            crate::resolve::body_origin::BodyOrigin::Imported(imported_origin) => {
                let original = self
                    .imported_original(
                        report,
                        facts,
                        body::transported_originals::SourceTarget::Anonymous(imported_origin.id()),
                    )?
                    .ok_or_else(|| {
                        report.body_error(origin, "imported anonymous constant lacks originals")
                    })?;
                let row = original.row().ok_or_else(|| {
                    report.body_error(origin, "imported anonymous constant has no original row")
                })?;
                super::super::original_contexts::Original::validate_parts_charged(
                    facts,
                    &row.template().body,
                    row.contract(),
                )
                .map_err(|e| report.body_error(origin, e))?;
                let built = body::instantiate::anonymous_result(
                    facts,
                    row.template(),
                    instance,
                    result,
                    row.contract(),
                )
                .and_then(|materialized| materialized.into_requested(result))
                .map_err(|e| report.body_error(origin, e))?;
                let environment = std::sync::Arc::clone(&row.template().environment);
                (built, environment)
            }
        })
    }
    pub(super) fn retain_anonymous(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        source: crate::resolve::unit::NodeKey,
    ) -> Result<(), Error> {
        if !self.anonymous.contains_key(&source) {
            let template = self.anonymous_template(report, facts, source)?;
            self.anonymous.insert(source, template);
        }
        Ok(())
    }
    pub(super) fn anonymous_template(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        source: crate::resolve::unit::NodeKey,
    ) -> Result<(Template, Contract), Error> {
        let (template, contract) = self.build_anonymous(report, facts, source)?;
        if contract.output.is_some()
            && self.anonymous_needs_parent(report, facts, &template, &contract)?
        {
            return self.original_component(
                report,
                facts,
                crate::pipeline::original_contexts::Original { template, contract },
            );
        }
        if contract.output.is_none() {
            self.prove(
                report,
                facts,
                &template,
                &contract,
                super::ProofPurpose::Original,
            )?;
        }
        Self::reserve_anonymous(report, facts, &template, &contract)?;
        facts.constant_work.anonymous_templates += 1;
        Ok((template, contract))
    }
    fn anonymous_needs_parent(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        template: &Template,
        contract: &Contract,
    ) -> Result<bool, Error> {
        let source = template.body.source;
        contract
            .prepare_outlives(&mut facts.types, &mut facts.lifetimes.nominal_requirements)
            .map_err(|e| report.body_error(source, e))?;
        let mut queries = crate::types::outlives::Queries::new(
            &facts.types,
            &facts.lifetimes.nominal_requirements,
            &facts.lifetimes.callables,
        );
        let region = crate::borrow::regions::check_original_child(
            &facts.types,
            &facts.lifetimes.variances,
            &mut queries,
            contract,
            &template.body,
        )
        .map_err(|e| report.body_error(e.source.unwrap_or(source.into()), e.message))?;
        let Some(mut regions) = region else {
            return Ok(true);
        };
        let mut contract = std::borrow::Cow::Borrowed(contract);
        let checked = self.prove_loans(
            report,
            facts,
            &template.body,
            &template.destruction,
            &mut super::LoanInput {
                parameters: &template.environment,
                contract: &mut contract,
                regions: &mut regions,
                purpose: super::ProofPurpose::Original,
            },
        )?;
        drop(checked);
        Ok(false)
    }
    pub(super) fn build_anonymous(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        source: crate::resolve::unit::NodeKey,
    ) -> Result<(Template, Contract), Error> {
        let mut template = self.construction.anonymous(report, facts, source)?;
        let mut contract = self
            .contracts
            .expression(facts, source)
            .map_err(|e| report.body_error(source, e))?;
        body::dispatch::original_anonymous(facts, &mut template.body, &mut contract)
            .map_err(|e| report.body_error(source, e))?;
        body::promote::instance_reusing(
            report,
            facts,
            &mut template.body,
            &template.destruction,
            &mut self.promotions,
        )?;
        Ok((template, contract))
    }
    fn reserve_anonymous(
        report: &Report,
        facts: &mut Declarations,
        template: &Template,
        contract: &Contract,
    ) -> Result<(), Error> {
        // The provider and ordinary template verifier each own a typed CFG copy.
        // Contract vectors/proof payload belong to the provider; shared Arc roots are excluded.
        let bytes = template
            .retained_bytes()
            .saturating_mul(2)
            .saturating_add(contract.retained_bytes())
            .saturating_add(crate::support::retained::new_map::<
                crate::resolve::unit::NodeKey,
                (Template, Contract),
            >(1));
        facts.constant_work.template_bytes = facts
            .constant_work
            .template_bytes
            .checked_add(bytes)
            .filter(|&n| n <= 64 * 1024 * 1024)
            .ok_or_else(|| {
                report.body_error(
                    template.body.source,
                    "anonymous template storage exceeds 64 MiB budget",
                )
            })?;
        Ok(())
    }
}
