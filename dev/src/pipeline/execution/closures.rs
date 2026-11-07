//! A reached const closure consumes its actual source template and normal instance proof.
use super::{BodyKey, Declarations, Engine, Error, ProofPurpose, Report, body};
use crate::types::{Kind, instance::Instance};

impl Engine {
    pub(super) fn closure_body(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        instance: Instance,
    ) -> Result<bool, Error> {
        let Some(Kind::Closure { identity, .. }) = facts.types.kind(instance.ty()) else {
            return Err(Error::unlocated(
                "const closure demand lost its complete Self",
            ));
        };
        if facts.closures.identity(identity).is_none() {
            return Ok(false);
        }
        let source =
            body::constants::const_closure_source(facts, instance).map_err(Error::unlocated)?;
        source
            .check_const_source(report)
            .map_err(|e| report.body_error(source.source, e))?;
        if let Some(&index) = self.catalog.functions.get(&instance) {
            self.catalog
                .get(index)
                .ok_or_else(|| report.body_error(source.source, "const closure Code is absent"))?
                .validate_closure_family(facts, instance)
                .map_err(|e| report.body_error(source.source, e))?;
            return Ok(true);
        }
        let key = BodyKey::Closure(identity);
        if self.component_original(key).is_none() {
            self.retain_closure_original(report, facts, source)?;
        }
        let original = self.component_original(key).ok_or_else(|| {
            report.body_error(
                source.source,
                "const closure lost its checked component original",
            )
        })?;
        original
            .validate(facts)
            .map_err(|e| report.body_error(source.source, e))?;
        if original.template.retained_bytes() > 64 * 1024 * 1024 {
            return Err(report.body_error(
                source.source,
                "constant instance copy exceeds 64 MiB budget",
            ));
        }
        body::assembly::prepay_instance(report, &original.template.body)?;
        let materialized = body::instantiate::with_contract(
            facts,
            &original.template,
            instance,
            &original.contract,
        )
        .map_err(|e| report.body_error(source.source, e))?;
        let environment = super::Arc::clone(&original.template.environment);
        self.finish_specialization(report, facts, instance, materialized, &environment, None)?;
        let index = *self.catalog.functions.get(&instance).ok_or_else(|| {
            report.body_error(source.source, "const closure Code was not published")
        })?;
        self.catalog
            .get(index)
            .ok_or_else(|| report.body_error(source.source, "const closure Code disappeared"))?
            .validate_closure_family(facts, instance)
            .map_err(|e| report.body_error(source.source, e))?;
        Ok(true)
    }
    fn retain_closure_original(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        source: crate::sema::closures::Body,
    ) -> Result<(), Error> {
        let owner = source.owner.ok_or_else(|| {
            report.body_error(
                source.source,
                "const closure without a declaration requires its actual anonymous environment",
            )
        })?;
        let input = self
            .contracts
            .closure(facts, &source)
            .map_err(|e| report.body_error(source.source, e))?;
        let template = self.construction.closure(report, facts, &source, owner)?;
        // The constructor and compact qualifier are not proof. Check the real
        // original, then specialize/reprove it through the existing owning path.
        self.prove(report, facts, &template, &input, ProofPurpose::Original)?;
        self.retain_original_component(
            report,
            facts,
            crate::pipeline::original_contexts::Original {
                template,
                contract: input,
            },
        )
    }
}
