//! A reached const closure consumes its actual source template and normal instance proof.
use super::{Declarations, Engine, Error, ProofPurpose, Report, body};
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
        self.specialize(report, facts, instance, &template, &input, None)?;
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
}
