//! Named roots preserve declaration ownership beside their executable expression.
use super::{Body, BodyKey, Declarations, DefId, Engine, Error, Report, Target, body};
impl Engine {
    pub(super) fn initializer_root(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        definition: DefId,
    ) -> Result<usize, Error> {
        if let Some(&index) = self.catalog.definitions.get(&definition) {
            self.roots.insert(Target::Initializer(definition), index);
            return Ok(index);
        }
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
        let mut template = self.construction.initializer(report, facts, &initializer)?;
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
        self.prove(report, facts, &template, &contract)?;
        let additional = template
            .body
            .retained_bytes()
            .saturating_add(crate::support::retained::new_map::<BodyKey, Body>(1));
        let retained = facts
            .constant_work
            .template_bytes
            .checked_add(additional)
            .filter(|&bytes| bytes <= 64 * 1024 * 1024)
            .ok_or_else(|| {
                report.body_error(source, "retained initializer exceeds 64 MiB budget")
            })?;
        let index = self
            .catalog
            .insert(&template.body)
            .map_err(|e| report.body_error(source, e))?;
        if self
            .concrete
            .insert(BodyKey::Declaration(definition), template.body)
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
}
