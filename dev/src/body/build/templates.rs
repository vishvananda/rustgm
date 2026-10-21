//! Definition CFGs retain possible destruction; they are not executable programs.
pub use super::constructors::templates as constructors;
use super::{Declarations, Error, Report, Scratch, function_body};
use crate::{
    body::{drops::Requirements, ir::Body},
    types::{destruction::NeedsDrop, parameters::environment::Environment},
};
use std::sync::Arc;

#[derive(Clone)]
pub struct Template {
    pub body: Body,
    pub destruction: Requirements,
    pub environment: Arc<Environment>,
}
impl Template {
    /// Both template consumers share the immutable permission environment.
    /// The charge covers this owner's inline records and every owned body and
    /// destruction collection; canonical stores and `Arc` pointees are excluded.
    pub(crate) fn retained_bytes(&self) -> usize {
        size_of::<Self>()
            .saturating_add(self.body.allocation_bytes())
            .saturating_add(self.destruction.allocation_bytes())
    }
}
pub struct Templates {
    pub bodies: Vec<Template>,
}
/// Build each generic or opaque-defining body once using the ordinary scope/CFG builder.
/// The result retains possible cleanup even without a declared custom destructor.
/// It cannot be passed to the concrete program verifier or `LowIR` lowerer.
/// # Errors
/// Reports missing typed facts, unsupported operations and exhausted work.
pub fn build(report: &Report, facts: &Declarations) -> Result<Templates, Error> {
    Prepared::default().finish(report, facts)
}

/// One CFG scratch arena and owned defining bodies survive semantic dependency waits.
#[derive(Default)]
pub struct Prepared {
    scratch: Option<Box<Scratch>>,
    query: Option<NeedsDrop>,
    bodies: std::collections::HashMap<crate::resolve::model::DefId, Template>,
    built: usize,
    reused: usize,
}
impl Prepared {
    /// Build one already typed anonymous constant using retained construction scratch.
    /// # Errors
    /// Reports missing facts and the same const eligibility failures as functions.
    pub fn expression(
        &mut self,
        report: &Report,
        facts: &Declarations,
        source: crate::resolve::unit::NodeKey,
        result: crate::types::TypeId,
    ) -> Result<(Body, Requirements), Error> {
        let scratch = self
            .scratch
            .get_or_insert_with(|| Box::new(Scratch::new(facts)));
        let scope_plan = crate::body::scopes::constant_with_scratch(
            report,
            facts,
            source,
            std::mem::take(&mut scratch.scopes),
        )?;
        let query = self.query.get_or_insert_with(|| {
            NeedsDrop::new(facts.destructors.implementations().iter().map(|i| i.item))
        });
        let body = super::Builder::new(
            report,
            facts,
            scratch,
            Body::new(crate::body::ir::Owner::AnonymousConstant, source, result),
            scope_plan,
            Some(query),
        )
        .run(None)
        .map_err(|e| report.diagnostic(e))?;
        crate::body::constants::eligibility(report, facts, &body)?;
        let requirements = crate::body::drops::requirements(facts, &body, query)
            .map_err(|e| Declarations::error(report, source, e))?;
        Ok((body, requirements))
    }
    /// Actual CFG construction and subsequent reuse counts.
    #[must_use]
    pub const fn work(&self) -> (usize, usize) {
        (self.built, self.reused)
    }

    /// Retrieve one retained body or build it from its completed semantic facts.
    /// # Errors
    /// Reports the same typed CFG, eligibility and destruction errors as `build`.
    pub fn function(
        &mut self,
        report: &Report,
        facts: &Declarations,
        function: &crate::sema::model::Function,
    ) -> Result<Template, Error> {
        if let Some(template) = self.bodies.remove(&function.definition) {
            self.reused += 1;
            return Ok(template);
        }
        let environment = environment(report, facts, function.definition, function.node)?;
        let query = self.query.get_or_insert_with(|| {
            NeedsDrop::new(facts.destructors.implementations().iter().map(|i| i.item))
        });
        let mut scoped = query.scoped(Arc::clone(&environment));
        let scratch = self
            .scratch
            .get_or_insert_with(|| Box::new(Scratch::new(facts)));
        let body = function_body(report, facts, scratch, function, Some(&mut scoped))?;
        crate::body::constants::eligibility(report, facts, &body)?;
        let destruction = crate::body::drops::requirements(facts, &body, &mut scoped)
            .map_err(|message| Declarations::error(report, body.source, message))?;
        self.built += 1;
        Ok(Template {
            body,
            destruction,
            environment,
        })
    }

    /// Retain a completed original function for later template qualification and lowering.
    /// # Errors
    /// Rejects duplicate retained owners and non-function bodies.
    pub fn retain(&mut self, template: Template) -> Result<(), &'static str> {
        let crate::body::ir::Owner::Function { definition, .. } = template.body.owner else {
            return Err("retained defining template requires a function owner");
        };
        match self.bodies.entry(definition) {
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(template);
                Ok(())
            }
            std::collections::hash_map::Entry::Occupied(_) => {
                Err("defining template was retained twice")
            }
        }
    }

    /// Complete the template set in the same declaration order as the ordinary builder.
    /// # Errors
    /// Rejects invalid templates and retained owners outside the requested program.
    pub fn finish(mut self, report: &Report, facts: &Declarations) -> Result<Templates, Error> {
        let mut bodies = Vec::new();
        for function in &facts.functions {
            if function.body.is_some()
                && (facts.generics.has_parameters(function.definition)
                    || facts.opaques.owners.contains_key(&function.definition))
            {
                bodies.push(self.function(report, facts, function)?);
            }
        }
        for initializer in &facts.initializers {
            if facts.associated.owner(initializer.definition).is_none()
                && !facts.generics.has_parameters(initializer.definition)
            {
                continue;
            }
            let environment =
                environment(report, facts, initializer.definition, initializer.value)?;
            let query = self.query.get_or_insert_with(|| {
                NeedsDrop::new(facts.destructors.implementations().iter().map(|i| i.item))
            });
            let mut scoped = query.scoped(Arc::clone(&environment));
            let scratch = self
                .scratch
                .get_or_insert_with(|| Box::new(Scratch::new(facts)));
            let body =
                super::initializer_body(report, facts, scratch, initializer, Some(&mut scoped))?;
            crate::body::constants::eligibility(report, facts, &body)?;
            let destruction = crate::body::drops::requirements(facts, &body, &mut scoped)
                .map_err(|message| Declarations::error(report, body.source, message))?;
            bodies.push(Template {
                body,
                destruction,
                environment,
            });
        }
        if let Some(template) = self.bodies.values().next() {
            return Err(Declarations::error(
                report,
                template.body.source,
                "retained template has no program owner",
            ));
        }
        crate::body::constants::alias_cycles(report, facts, &bodies)?;
        Ok(Templates { bodies })
    }
}
fn environment(
    report: &Report,
    facts: &Declarations,
    owner: crate::resolve::model::DefId,
    source: crate::resolve::unit::NodeKey,
) -> Result<Arc<Environment>, Error> {
    if let Some(environment) = facts.generics.body_environment(owner) {
        Ok(Arc::clone(environment))
    } else {
        facts
            .generics
            .parameter_environment(&facts.types, owner)
            .map(Arc::new)
            .map_err(|e| Declarations::error(report, source, e))
    }
}
