//! Selected associated roots share retained source templates and iterative execution.
use super::{
    BodyKey, Contract, Declarations, DefId, Engine, Error, Report, Target, Template, body,
};
use crate::{sema::values::Instance, types::TypeId};
impl Engine {
    pub(super) fn associated_ready(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        instance: Instance,
        result: TypeId,
    ) -> Result<bool, Error> {
        let ready = facts
            .initializer_index(instance.definition)
            .and_then(|i| facts.initializers.get(i))
            .is_some_and(|initializer| {
                initializer.type_checked()
                    && !facts
                        .patterns
                        .pending_bodies
                        .contains(&initializer.semantic_source())
            });
        if !ready {
            return Ok(false);
        }
        self.root(report, facts, Target::Associated { instance, result })?;
        Ok(true)
    }
    pub(super) fn associated_root(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        instance: Instance,
        result: TypeId,
    ) -> Result<usize, Error> {
        let canonical = instance
            .result_type(facts)
            .map_err(|e| Error::at(super::FileId(0), 0, e))?;
        if canonical != result {
            return Err(Error::at(
                super::FileId(0),
                0,
                "selected initializer request result differs from canonical result",
            ));
        }
        if let Some(&index) = self.roots.get(&Target::Associated { instance, result }) {
            return Ok(index);
        }
        self.selected_root(report, facts, instance, result, None)
    }
    pub(super) fn source_associated_root(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        source: crate::sema::values::execution::SourceComputation,
        result: TypeId,
    ) -> Result<usize, Error> {
        let selected = source
            .resolve(facts, result)
            .map_err(|e| Error::at(super::FileId(0), 0, e))?;
        if let Some(&index) = self.roots.get(&Target::SourceAssociated { source, result }) {
            return Ok(index);
        }
        self.selected_root(report, facts, selected.selected, result, Some(source))
    }
    fn selected_root(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        instance: Instance,
        result: TypeId,
        source_value: Option<crate::sema::values::execution::SourceComputation>,
    ) -> Result<usize, Error> {
        let initializer = facts
            .initializer_index(instance.definition)
            .and_then(|i| facts.initializers.get(i))
            .copied()
            .ok_or_else(|| {
                Error::at(
                    super::FileId(0),
                    0,
                    "selected associated constant has no initializer",
                )
            })?;
        let source = initializer.semantic_source();
        if !initializer.type_checked()
            || !matches!(initializer.kind, crate::sema::model::InitializerKind::Const)
        {
            return Err(report.body_error(source, "selected associated initializer is not ready"));
        }
        let (template, contract) = if let Some(value) = self
            .associated
            .as_deref_mut()
            .and_then(|map| map.remove(&instance.definition))
        {
            value
        } else {
            self.associated_template(report, facts, &initializer)?
        };
        let built = if let Some(witness) = source_value {
            body::instantiate::source_initializer(facts, &template, witness, result, &contract)
                .and_then(|source| source.into_requested(result))
        } else {
            body::instantiate::selected_initializer(facts, &template, instance, &contract)
        }
        .map_err(|e| report.body_error(source, e));
        self.associated
            .get_or_insert_with(Default::default)
            .insert(instance.definition, (template, contract));
        let mut concrete = built?;
        if source_value.is_none() && concrete.body.result != result {
            return Err(
                report.body_error(source, "selected initializer materialized result changed")
            );
        }
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
        let environment = std::sync::Arc::clone(
            &self
                .associated
                .as_deref()
                .and_then(|map| map.get(&instance.definition))
                .ok_or_else(|| report.body_error(source, "lost selected initializer template"))?
                .0
                .environment,
        );
        let checked = self.prove_body(
            report,
            facts,
            &concrete.body,
            &concrete.destruction,
            super::ProofInput::code(&environment, &concrete.contract),
        )?;
        let index = self
            .publish_selected(facts, &concrete, source_value, result)
            .map_err(|e| report.body_error(source, e))?;
        self.retain_const_inputs(facts, index, &concrete.body, checked, None)
            .map_err(|e| report.body_error(source, e))?;
        facts.constant_work.associated_instances += 1;
        let target = source_value.map_or(Target::Associated { instance, result }, |source| {
            Target::SourceAssociated { source, result }
        });
        self.roots.insert(target, index);
        Ok(index)
    }
    fn publish_selected(
        &mut self,
        facts: &mut Declarations,
        concrete: &body::instantiate::Materialized,
        witness: Option<crate::sema::values::execution::SourceComputation>,
        result: TypeId,
    ) -> Result<usize, &'static str> {
        let Some(witness) = witness else {
            return self.catalog.insert(&concrete.body);
        };
        let crate::body::ir::Owner::InitializerInstance { instance, .. } = concrete.body.owner
        else {
            return Err("selected publication has another body owner");
        };
        let results = body::instantiate::source_inline_results(
            facts,
            &concrete.body,
            &concrete.contract,
            witness,
        )?;
        self.catalog.source_initializer(
            facts,
            &concrete.body,
            (
                instance,
                concrete
                    .body
                    .source
                    .source()
                    .ok_or("source initializer witness has an imported body")?,
                result,
            ),
            witness,
            &results,
        )
    }
    pub(super) fn associated_template(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        initializer: &crate::sema::model::Initializer,
    ) -> Result<(Template, Contract), Error> {
        let source = initializer.semantic_source();
        if let Some(row) = self.component_original(BodyKey::Declaration(initializer.definition)) {
            row.validate(facts)
                .map_err(|e| report.body_error(source, e))?;
            Self::associated_storage(
                report,
                facts,
                source,
                row.template
                    .retained_bytes()
                    .saturating_add(row.contract.retained_bytes())
                    .saturating_add(crate::support::retained::new_map::<
                        DefId,
                        (Template, Contract),
                    >(1)),
            )?;
            facts.constant_work.associated_templates += 1;
            return Ok((row.template.clone(), row.contract.clone()));
        }
        let existing = self
            .runtime_template_indices
            .get(&initializer.definition)
            .copied()
            .and_then(|index| self.runtime_templates.as_ref()?.bodies.get(index));
        // The ordinary verifier and provider each own one immutable typed body. Charge before cloning.
        let mut template = if let Some(existing) = existing {
            let bytes = existing.retained_bytes();
            Self::associated_storage(report, facts, source, bytes)?;
            existing.clone()
        } else {
            let template = self.construction.initializer(report, facts, initializer)?;
            Self::associated_storage(
                report,
                facts,
                source,
                template.retained_bytes().saturating_mul(2),
            )?;
            template
        };
        let mut contract = self
            .contracts
            .initializer(facts, initializer)
            .map_err(|e| report.body_error(source, e))?;
        body::dispatch::original_initializer(facts, &mut template.body, &mut contract, initializer)
            .map_err(|e| report.body_error(source, e))?;
        body::promote::instance_reusing(
            report,
            facts,
            &mut template.body,
            &template.destruction,
            &mut self.promotions,
        )?;
        self.prove(
            report,
            facts,
            &template,
            &contract,
            super::ProofPurpose::Original,
        )?;
        Self::associated_storage(
            report,
            facts,
            source,
            contract
                .retained_bytes()
                .saturating_add(crate::support::retained::new_map::<
                    DefId,
                    (Template, Contract),
                >(1)),
        )?;
        facts.constant_work.associated_templates += 1;
        Ok((template, contract))
    }
    fn associated_storage(
        report: &Report,
        facts: &mut Declarations,
        source: crate::resolve::unit::NodeKey,
        bytes: usize,
    ) -> Result<(), Error> {
        facts.constant_work.template_bytes = facts
            .constant_work
            .template_bytes
            .checked_add(bytes)
            .filter(|&n| n <= 64 * 1024 * 1024)
            .ok_or_else(|| {
                report.body_error(
                    source,
                    "selected initializer template storage exceeds 64 MiB budget",
                )
            })?;
        Ok(())
    }
}
