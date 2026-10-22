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
            .map_err(|e| Declarations::error(report, source, e))?;
        if selected != instance {
            return Err(Declarations::error(
                report,
                source,
                "anonymous instance identity changed",
            ));
        }
        if let crate::sema::inline_constants::Context::SourceInitializer(witness) =
            instance.enclosing()
        {
            let parent_result = witness
                .result(facts)
                .map_err(|e| Declarations::error(report, source, e))?;
            witness
                .validate(facts, parent_result)
                .map_err(|e| Declarations::error(report, source, e))?;
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
                .map_err(|e| Declarations::error(report, source, e));
        self.anonymous.insert(source, (template, contract));
        let mut concrete = built?;
        body::constants::eligibility(report, facts, &concrete.body)?;
        body::dispatch::resolve(facts, &mut concrete.body, &mut concrete.contract)
            .map_err(|e| Declarations::error(report, source, e))?;
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
            .map_err(|e| Declarations::error(report, source, e))?
        } else {
            Vec::new()
        };
        let index = self
            .catalog
            .anonymous_result(&concrete.body, result, &results)
            .map_err(|e| Declarations::error(report, source, e))?;
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
    fn anonymous_template(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        source: crate::resolve::unit::NodeKey,
    ) -> Result<(Template, Contract), Error> {
        let mut template = self.construction.anonymous(report, facts, source)?;
        let mut contract = self
            .contracts
            .expression(facts, source)
            .map_err(|e| Declarations::error(report, source, e))?;
        body::dispatch::original_anonymous(facts, &mut template.body, &mut contract)
            .map_err(|e| Declarations::error(report, source, e))?;
        body::promote::instance_reusing(
            report,
            facts,
            &mut template.body,
            &template.destruction,
            &mut self.promotions,
        )?;
        self.prove(report, facts, &template, &contract)?;
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
                Declarations::error(
                    report,
                    source,
                    "anonymous template storage exceeds 64 MiB budget",
                )
            })?;
        facts.constant_work.anonymous_templates += 1;
        Ok((template, contract))
    }
}
