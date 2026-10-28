//! Named roots preserve declaration ownership beside their executable expression.
use super::{Body, BodyKey, Declarations, DefId, Engine, Error, Report, Target, body};
impl Engine {
    pub(super) fn initializer_root(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        definition: DefId,
    ) -> Result<usize, Error> {
        let initializer = facts
            .initializer_index(definition)
            .and_then(|index| facts.initializers.get(index))
            .copied()
            .ok_or_else(|| Error::at(super::FileId(0), 0, "missing selected initializer"))?;
        let source = initializer.semantic_source();
        if !initializer.type_checked()
            || facts.associated.owner(definition).is_some()
            || facts.generics.has_parameters(definition)
        {
            return Err(report.body_error(
                source,
                "selected initializer requires a completed closed body",
            ));
        }
        if let Some(&index) = self.catalog.definitions.get(&definition) {
            self.catalog
                .validate_entry(index)
                .and_then(|()| {
                    self.catalog
                        .initializer(index, definition, initializer.kind)
                })
                .map_err(|e| report.body_error(source, e))?;
            let code = self.catalog.code(index).ok_or_else(|| {
                report.body_error(source, "named initializer lost its actual Code")
            })?;
            if code.source.source() != Some(source)
                || code.result != initializer.ty
                || code.instance.is_some()
                || !code.const_context
            {
                return Err(
                    report.body_error(source, "named initializer changed its source or type")
                );
            }
            self.roots.insert(Target::Initializer(definition), index);
            return Ok(index);
        }
        let mut template = self.construction.initializer(report, facts, &initializer)?;
        let original = Self::original_initializer_body(report, facts, &template.body)?;
        let mut contract = self
            .contracts
            .initializer(facts, &initializer)
            .map_err(|e| report.body_error(source, e))?;
        body::dispatch::resolve(facts, &mut template.body, &mut contract)
            .map_err(|e| report.body_error(source, e))?;
        body::promote::instance_reusing(
            report,
            facts,
            &mut template.body,
            &template.destruction,
            &mut self.promotions,
        )?;
        let checked = self.prove(
            report,
            facts,
            &template,
            &contract,
            super::ProofPurpose::Code,
        )?;
        let additional = original
            .as_ref()
            .map_or_else(|| template.body.retained_bytes(), Body::retained_bytes)
            .saturating_add(crate::support::retained::new_map::<BodyKey, Body>(1));
        let prepaid = original.as_ref().map_or(0, Body::retained_bytes);
        let retained = facts
            .constant_work
            .template_bytes
            .checked_add(additional.saturating_sub(prepaid))
            .filter(|&bytes| bytes <= 64 * 1024 * 1024)
            .ok_or_else(|| {
                report.body_error(source, "retained initializer exceeds 64 MiB budget")
            })?;
        let index = self
            .publish_code(facts, checked, None)
            .map_err(|e| report.body_error(source, e))?;
        if self
            .concrete
            .insert(
                BodyKey::Declaration(definition),
                original.unwrap_or(template.body),
            )
            .is_some()
        {
            return Err(report.body_error(source, "duplicate retained initializer CFG"));
        }
        facts.constant_work.template_bytes = retained;
        self.retained_expressions = self.retained_expressions.saturating_add(additional);
        facts.constant_work.expression_bytes = self.retained_expressions;
        self.roots.insert(Target::Initializer(definition), index);
        Ok(index)
    }
    /// The ordinary cache owns semantic originals, never a selected body's
    /// fresh region variables without their matching dispatch contract.
    fn original_initializer_body(
        report: &Report,
        facts: &mut Declarations,
        body: &Body,
    ) -> Result<Option<Body>, Error> {
        if !Self::needs_method_dispatch(facts, body) {
            return Ok(None);
        }
        let retained = facts
            .constant_work
            .template_bytes
            .checked_add(body.retained_bytes())
            .filter(|&bytes| bytes <= 64 * 1024 * 1024)
            .ok_or_else(|| {
                report.body_error(body.source, "retained initializer exceeds 64 MiB budget")
            })?;
        facts.constant_work.template_bytes = retained;
        Ok(Some(body.clone()))
    }
}
