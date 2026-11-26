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
        let origin = facts
            .closures
            .origin(identity)
            .ok_or_else(|| Error::unlocated("const closure lost its actual body origin"))?;
        let source =
            body::constants::const_closure_source(facts, instance).map_err(Error::unlocated)?;
        if !facts
            .closures
            .check_const_source(report, identity)
            .map_err(|e| report.body_error(origin, e))?
        {
            return Err(report.body_error(
                origin,
                "non-const closures cannot be called in a const context",
            ));
        }
        if let Some(&index) = self.catalog.functions.get(&instance) {
            self.catalog
                .get(index)
                .ok_or_else(|| report.body_error(origin, "const closure Code is absent"))?
                .validate_closure_family(facts, instance)
                .map_err(|e| report.body_error(origin, e))?;
            return Ok(true);
        }
        if origin.source().is_none() {
            if body::constants::scoped_const_closure(facts, source.owner)
                .map_err(|e| report.body_error(origin, e))?
            {
                crate::sema::dispatch::check_closed_const_closure(facts, instance)
                    .map_err(|e| report.body_error(origin, e))?;
            }
            self.imported_closure_body(report, facts, instance, identity, origin)?;
            return Ok(true);
        }
        let key = BodyKey::Closure(identity);
        if self.component_original(key).is_none() {
            self.retain_closure_original(report, facts, source)?;
        }
        let original = self.component_original(key).ok_or_else(|| {
            report.body_error(origin, "const closure lost its checked component original")
        })?;
        original
            .validate(facts)
            .map_err(|e| report.body_error(origin, e))?;
        if original.template.retained_bytes() > 64 * 1024 * 1024 {
            return Err(report.body_error(origin, "constant instance copy exceeds 64 MiB budget"));
        }
        body::assembly::prepay_instance(report, &original.template.body)?;
        let materialized = body::instantiate::with_contract(
            facts,
            &original.template,
            instance,
            &original.contract,
        )
        .map_err(|e| report.body_error(origin, e))?;
        let environment = super::Arc::clone(&original.template.environment);
        self.finish_specialization(report, facts, instance, materialized, &environment, None)?;
        let index = *self
            .catalog
            .functions
            .get(&instance)
            .ok_or_else(|| report.body_error(origin, "const closure Code was not published"))?;
        self.catalog
            .get(index)
            .ok_or_else(|| report.body_error(origin, "const closure Code disappeared"))?
            .validate_closure_family(facts, instance)
            .map_err(|e| report.body_error(origin, e))?;
        Ok(true)
    }
    fn imported_closure_body(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        instance: Instance,
        identity: crate::types::closure::ClosureId,
        origin: crate::body::ir::BodyOrigin,
    ) -> Result<(), Error> {
        self.ensure_imported_originals(report, facts)?;
        let originals = self
            .imported
            .as_ref()
            .map(std::rc::Rc::clone)
            .ok_or_else(|| {
                report.body_error(
                    origin,
                    "imported const closure lacks its proved original set",
                )
            })?;
        let row = originals.closure(identity).ok_or_else(|| {
            report.body_error(origin, "imported const closure has no exact original row")
        })?;
        super::super::original_contexts::Original::validate_parts_charged(
            facts,
            &row.template().body,
            row.contract(),
        )
        .map_err(|e| report.body_error(origin, e))?;
        self.specialize(
            report,
            facts,
            instance,
            row.template(),
            row.contract(),
            None,
        )?;
        let index = *self.catalog.functions.get(&instance).ok_or_else(|| {
            report.body_error(origin, "imported const closure Code was not published")
        })?;
        self.catalog
            .get(index)
            .ok_or_else(|| report.body_error(origin, "imported const closure Code disappeared"))?
            .validate_closure_family(facts, instance)
            .map_err(|e| report.body_error(origin, e))
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
