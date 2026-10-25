//! Closed expression execution retains checked inline CFG ownership across early and late demand.
use super::{
    Arc, Body, BodyKey, Declarations, Engine, Environment, Error, Report, Target, Template,
    Templates, body,
};
use crate::{resolve::unit::NodeKey, types::TypeId};
#[cfg(test)]
#[path = "../../../tests/invariants/expression_owners.rs"]
mod tests;
impl Engine {
    pub(super) fn check_completed_expression(
        &self,
        facts: &Declarations,
        target: Target,
    ) -> Result<(), &'static str> {
        let Target::Expression {
            source,
            result,
            context,
        } = target
        else {
            return Ok(());
        };
        if context.is_some() {
            return Err("associated constant execution needs a specialized body");
        }
        let index = *self
            .roots
            .get(&target)
            .ok_or("completed expression lost its canonical Code root")?;
        self.catalog.expression(facts, index, source, result)?;
        Ok(())
    }
    pub(super) fn root(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        target: Target,
    ) -> Result<usize, Error> {
        if let Target::SourceAssociated { source, result } = target {
            return self.source_associated_root(report, facts, source, result);
        }
        if let Target::Associated { instance, result } = target {
            return self.associated_root(report, facts, instance, result);
        }
        if let Target::Anonymous { instance, result } = target
            && matches!(
                instance.enclosing(),
                crate::sema::inline_constants::Context::SourceInitializer(_)
            )
        {
            // Warm source-only entries still validate their exact live caller.
            return self.anonymous_root(report, facts, instance, result);
        }
        if let Target::Initializer(definition) = target {
            // Named warm entries still authenticate their exact declaration
            // source, type, kind and canonical Code before reusing proof.
            return self.initializer_root(report, facts, definition);
        }
        if let Target::Expression {
            source,
            context: Some(_),
            ..
        } = target
        {
            return Err(report.body_error(
                source,
                "associated constant execution needs a specialized body",
            ));
        }
        if let Some(&index) = self.roots.get(&target) {
            if let Target::Expression { source, result, .. } = target {
                self.catalog
                    .expression(facts, index, source, result)
                    .map_err(|e| report.body_error(source, e))?;
            }
            return Ok(index);
        }
        if let Target::Anonymous { instance, result } = target {
            return self.anonymous_root(report, facts, instance, result);
        }
        let Target::Expression { source, result, .. } = target else {
            return Err(Error::at(
                crate::resolve::unit::FileId(0),
                0,
                "constant root is not a typed expression",
            ));
        };
        self.expression_root(report, facts, source, result)
    }
    fn expression_root(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        source: NodeKey,
        result: TypeId,
    ) -> Result<usize, Error> {
        let target = Target::Expression {
            source,
            result,
            context: None,
        };
        if let Some(index) = self.lent_expression(report, facts, source, result)? {
            self.roots.insert(target, index);
            return Ok(index);
        }
        let (templates, checked) = self.checked_expression(report, facts, source, result)?;
        let retain = facts
            .inline_constants
            .as_deref()
            .is_some_and(|registry| registry.get(source).is_some());
        let additional = if retain {
            templates.bodies[0]
                .body
                .retained_bytes()
                .saturating_add(crate::support::retained::new_map::<BodyKey, Body>(1))
        } else {
            0
        };
        let retained = facts
            .constant_work
            .template_bytes
            .checked_add(additional)
            .filter(|&bytes| bytes <= 64 * 1024 * 1024)
            .ok_or_else(|| {
                report.body_error(source, "retained constant bodies exceed 64 MiB budget")
            })?;
        let index = self
            .publish_code(facts, checked, None)
            .map_err(|e| report.body_error(source, e))?;
        if retain {
            let template = templates
                .bodies
                .into_iter()
                .next()
                .ok_or_else(|| report.body_error(source, "lost retained inline body"))?;
            if self
                .concrete
                .insert(BodyKey::Anonymous(source), template.body)
                .is_some()
            {
                return Err(report.body_error(source, "duplicate retained inline CFG owner"));
            }
            self.retained_expressions = self.retained_expressions.saturating_add(additional);
            facts.constant_work.expression_bytes = self.retained_expressions;
            facts.constant_work.template_bytes = retained;
        }
        self.roots.insert(target, index);
        Ok(index)
    }

    fn checked_expression(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        source: NodeKey,
        result: TypeId,
    ) -> Result<(Templates, super::CheckedInput), Error> {
        let environment = Arc::new(
            Environment::new(&facts.types, &[]).map_err(|e| report.body_error(source, e))?,
        );
        let (mut body, destruction) = self.construction.expression(
            report,
            facts,
            source,
            result,
            Arc::clone(&environment),
        )?;
        body::promote::instance_reusing(
            report,
            facts,
            &mut body,
            &destruction,
            &mut self.promotions,
        )?;
        let mut contract = self
            .contracts
            .expression(facts, source)
            .map_err(|e| report.body_error(source, e))?;
        body::dispatch::resolve(facts, &mut body, &mut contract)
            .map_err(|e| report.body_error(source, e))?;
        let template = Template {
            body,
            destruction,
            environment,
        };
        let templates = Templates::source(vec![template]);
        let checked = self.prove(
            report,
            facts,
            &templates.bodies[0],
            &contract,
            super::ProofPurpose::Code,
        )?;
        Ok((templates, checked))
    }

    fn lent_expression(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        source: NodeKey,
        result: TypeId,
    ) -> Result<Option<usize>, Error> {
        let inputs = std::rc::Rc::clone(&self.inputs);
        let stored = inputs.try_borrow().map_err(|_| {
            report.body_error(
                source,
                "constant CFG lender is preparing another expression",
            )
        })?;
        let Some(body) = stored.get(&BodyKey::Anonymous(source)) else {
            return Ok(None);
        };
        if body.source != source.into()
            || body.result != result
            || !matches!(body.owner, crate::body::ir::Owner::AnonymousConstant)
        {
            return Err(report.body_error(source, "lent inline CFG identity or result changed"));
        }
        body::constants::eligibility_charged(report, facts, body)?;
        let mut needed = crate::types::destruction::NeedsDrop::new(
            facts.destructors.implementations().iter().map(|i| i.item),
        );
        let destruction = body::drops::requirements(facts, body, &mut needed)
            .map_err(|e| report.body_error(source, e))?;
        let contract = self
            .contracts
            .expression(facts, source)
            .map_err(|e| report.body_error(source, e))?;
        let environment = Arc::new(
            Environment::new(&facts.types, &[]).map_err(|e| report.body_error(source, e))?,
        );
        let checked = self.prove_body(
            report,
            facts,
            body,
            &destruction,
            super::ProofInput::code(&environment, &contract),
        )?;
        let index = self
            .publish_code(facts, checked, None)
            .map_err(|e| report.body_error(source, e))?;
        Ok(Some(index))
    }
}
