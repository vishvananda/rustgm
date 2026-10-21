//! Closed expression execution retains checked inline CFG ownership across early and late demand.
use super::{
    Arc, Body, BodyKey, Declarations, Engine, Environment, Error, Report, Target, Template,
    Templates, body,
};
use crate::{resolve::unit::NodeKey, types::TypeId};
impl Engine {
    pub(super) fn root(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        target: Target,
    ) -> Result<usize, Error> {
        if let Target::Associated { instance, result } = target {
            let canonical = instance
                .result_type(facts)
                .map_err(|e| Error::at(crate::resolve::unit::FileId(0), 0, e))?;
            if canonical != result {
                return Err(Error::at(
                    crate::resolve::unit::FileId(0),
                    0,
                    "selected initializer request result differs from canonical result",
                ));
            }
            if let Some(&index) = self.roots.get(&target) {
                return Ok(index);
            }
            return self.associated_root(report, facts, instance, result);
        }
        if let Some(&index) = self.roots.get(&target) {
            return Ok(index);
        }
        if let Target::Initializer(definition) = target {
            return self.initializer_root(report, facts, definition);
        }
        if let Target::Anonymous { instance, result } = target {
            return self.anonymous_root(report, facts, instance, result);
        }
        let Target::Expression {
            source,
            result,
            context,
        } = target
        else {
            return Err(Error::at(
                crate::resolve::unit::FileId(0),
                0,
                "constant root is not a typed expression",
            ));
        };
        if context.is_some() {
            return Err(Declarations::error(
                report,
                source,
                "associated constant execution needs a specialized body",
            ));
        }
        if let Some(index) = self.lent_expression(report, facts, source, result)? {
            self.roots.insert(target, index);
            return Ok(index);
        }
        let templates = self.checked_expression(report, facts, source, result)?;
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
                Declarations::error(
                    report,
                    source,
                    "retained constant bodies exceed 64 MiB budget",
                )
            })?;
        let index = self
            .catalog
            .insert(&templates.bodies[0].body)
            .map_err(|e| Declarations::error(report, source, e))?;
        if retain {
            let template =
                templates.bodies.into_iter().next().ok_or_else(|| {
                    Declarations::error(report, source, "lost retained inline body")
                })?;
            if self
                .concrete
                .insert(BodyKey::Anonymous(source), template.body)
                .is_some()
            {
                return Err(Declarations::error(
                    report,
                    source,
                    "duplicate retained inline CFG owner",
                ));
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
    ) -> Result<Templates, Error> {
        let (mut body, destruction) = self
            .construction
            .expression(report, facts, source, result)?;
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
            .map_err(|e| Declarations::error(report, source, e))?;
        body::dispatch::resolve(facts, &mut body, &mut contract)
            .map_err(|e| Declarations::error(report, source, e))?;
        let template = Template {
            body,
            destruction,
            environment: Arc::new(
                Environment::new(&facts.types, &[])
                    .map_err(|e| Declarations::error(report, source, e))?,
            ),
        };
        let templates = Templates {
            bodies: vec![template],
        };
        self.prove(report, facts, &templates.bodies[0], &contract)?;
        Ok(templates)
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
            Declarations::error(
                report,
                source,
                "constant CFG lender is preparing another expression",
            )
        })?;
        let Some(body) = stored.get(&BodyKey::Anonymous(source)) else {
            return Ok(None);
        };
        if body.source != source
            || body.result != result
            || !matches!(body.owner, crate::body::ir::Owner::AnonymousConstant)
        {
            return Err(Declarations::error(
                report,
                source,
                "lent inline CFG identity or result changed",
            ));
        }
        body::constants::eligibility(report, facts, body)?;
        let mut needed = crate::types::destruction::NeedsDrop::new(
            facts.destructors.implementations().iter().map(|i| i.item),
        );
        let destruction = body::drops::requirements(facts, body, &mut needed)
            .map_err(|e| Declarations::error(report, source, e))?;
        let contract = self
            .contracts
            .expression(facts, source)
            .map_err(|e| Declarations::error(report, source, e))?;
        let environment = Arc::new(
            Environment::new(&facts.types, &[])
                .map_err(|e| Declarations::error(report, source, e))?,
        );
        self.prove_body(report, facts, body, &destruction, &environment, &contract)?;
        let index = self
            .catalog
            .insert(body)
            .map_err(|e| Declarations::error(report, source, e))?;
        Ok(Some(index))
    }
}
