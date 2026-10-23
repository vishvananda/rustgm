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
        let source = instance.source();
        let selected = InlineInstance::select(facts, source, instance.enclosing())
            .map_err(|e| report.body_error(source, e))?;
        if selected != instance {
            return Err(report.body_error(source, "anonymous instance identity changed"));
        }
        if self
            .original_components
            .as_ref()
            .and_then(|rows| rows.active.as_ref())
            .is_some_and(|active| active.owner.pending().has_source(source.into()))
        {
            return self.paid_anonymous_root(report, facts, instance, result);
        }
        if let crate::sema::inline_constants::Context::SourceInitializer(witness) =
            instance.enclosing()
        {
            let parent_result = witness
                .result(facts)
                .map_err(|e| report.body_error(source, e))?;
            witness
                .validate(facts, parent_result)
                .map_err(|e| report.body_error(source, e))?;
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
        let (template, contract) = if let Some(retained) = self.anonymous.remove(&source) {
            retained
        } else {
            self.anonymous_template(report, facts, source)?
        };
        let built =
            body::instantiate::anonymous_result(facts, &template, instance, result, &contract)
                .and_then(|materialized| materialized.into_requested(result))
                .map_err(|e| report.body_error(source, e));
        self.anonymous.insert(source, (template, contract));
        let mut concrete = built?;
        body::constants::eligibility(report, facts, &concrete.body)?;
        body::dispatch::resolve(facts, &mut concrete.body, &mut concrete.contract)
            .map_err(|e| report.body_error(source, e))?;
        body::promote::instance_reusing(
            report,
            facts,
            &mut concrete.body,
            &concrete.destruction,
            &mut self.promotions,
        )?;
        let environment = std::sync::Arc::clone(&self.anonymous[&source].0.environment);
        self.prove_body(
            report,
            facts,
            &concrete.body,
            &concrete.destruction,
            &environment,
            &concrete.contract,
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
            .map_err(|e| report.body_error(source, e))?
        } else {
            Vec::new()
        };
        let index = self
            .catalog
            .anonymous_result(&concrete.body, result, &results)
            .map_err(|e| report.body_error(source, e))?;
        facts.constant_work.anonymous_instances += 1;
        self.roots
            .insert(Target::Anonymous { instance, result }, index);
        Ok(index)
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
            self.prove(report, facts, &template, &contract)?;
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
        let Some(regions) = region else {
            return Ok(true);
        };
        self.prove_loans(
            report,
            facts,
            &template.body,
            &template.destruction,
            &template.environment,
            &regions,
        )?;
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
