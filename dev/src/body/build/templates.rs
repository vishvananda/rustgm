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
/// Every original child belongs to the same defining source family as its parent.
pub(crate) fn owns(facts: &Declarations, owner: crate::resolve::model::DefId) -> bool {
    facts.generics.has_parameters(owner) || facts.opaques.owners.contains_key(&owner)
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
    anonymous: std::collections::HashMap<crate::resolve::unit::NodeKey, Template>,
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
    pub(crate) fn initializer(
        &mut self,
        report: &Report,
        facts: &Declarations,
        initializer: &crate::sema::model::Initializer,
    ) -> Result<Template, Error> {
        if let Some(template) = self.bodies.remove(&initializer.definition) {
            if template.body.source != initializer.semantic_source()
                || template.body.result != initializer.ty
                || !matches!(template.body.owner,crate::body::ir::Owner::Initializer {definition,..} if definition==initializer.definition)
            {
                return Err(Declarations::error(
                    report,
                    initializer.semantic_source(),
                    "retained initializer template changed",
                ));
            }
            self.reused += 1;
            return Ok(template);
        }
        let environment = environment(
            report,
            facts,
            initializer.definition,
            initializer.semantic_source(),
        )?;
        let query = self.query.get_or_insert_with(|| {
            NeedsDrop::new(facts.destructors.implementations().iter().map(|i| i.item))
        });
        let mut scoped = query.scoped(Arc::clone(&environment));
        let scratch = self
            .scratch
            .get_or_insert_with(|| Box::new(Scratch::new(facts)));
        let body = super::initializer_body(report, facts, scratch, initializer, Some(&mut scoped))?;
        crate::body::constants::eligibility(report, facts, &body)?;
        let destruction = crate::body::drops::requirements(facts, &body, &mut scoped)
            .map_err(|e| Declarations::error(report, body.source, e))?;
        Ok(Template {
            body,
            destruction,
            environment,
        })
    }
    pub(crate) fn anonymous(
        &mut self,
        report: &Report,
        facts: &Declarations,
        source: crate::resolve::unit::NodeKey,
    ) -> Result<Template, Error> {
        if let Some(template) = self.anonymous.remove(&source) {
            self.reused += 1;
            return Ok(template);
        }
        let inline = facts
            .inline_constants
            .as_deref()
            .and_then(|r| r.get(source))
            .ok_or_else(|| Declarations::error(report, source, "missing anonymous template"))?;
        let owner = inline.declaration.ok_or_else(|| {
            Declarations::error(report, source, "anonymous template has no declaration")
        })?;
        let result = inline.result.ok_or_else(|| {
            Declarations::error(report, source, "anonymous template result is not ready")
        })?;
        let environment = environment(report, facts, owner, source)?;
        let query = self.query.get_or_insert_with(|| {
            NeedsDrop::new(facts.destructors.implementations().iter().map(|i| i.item))
        });
        let mut scoped = query.scoped(Arc::clone(&environment));
        let scratch = self
            .scratch
            .get_or_insert_with(|| Box::new(Scratch::new(facts)));
        let scope_plan = crate::body::scopes::constant_with_scratch(
            report,
            facts,
            source,
            std::mem::take(&mut scratch.scopes),
        )?;
        let body = super::Builder::new(
            report,
            facts,
            scratch,
            Body::new(crate::body::ir::Owner::AnonymousConstant, source, result),
            scope_plan,
            Some(&mut scoped),
        )
        .run(None)
        .map_err(|e| report.diagnostic(e))?;
        crate::body::constants::eligibility(report, facts, &body)?;
        let destruction = crate::body::drops::requirements(facts, &body, &mut scoped)
            .map_err(|e| Declarations::error(report, source, e))?;
        self.built += 1;
        Ok(Template {
            body,
            destruction,
            environment,
        })
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
        if matches!(
            template.body.owner,
            crate::body::ir::Owner::AnonymousConstant
        ) {
            if self
                .anonymous
                .insert(template.body.source, template)
                .is_some()
            {
                return Err("anonymous template was retained twice");
            }
            return Ok(());
        }
        let (crate::body::ir::Owner::Function { definition, .. }
        | crate::body::ir::Owner::Initializer { definition, .. }) = template.body.owner
        else {
            return Err("retained defining template requires a declaration owner");
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
            if (function.body.is_some()
                || facts
                    .methods
                    .demanded_length()
                    .is_some_and(|l| l.definition == function.definition))
                && owns(facts, function.definition)
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
            bodies.push(self.initializer(report, facts, initializer)?);
        }
        for closure in facts.closures.all() {
            if let Some(owner) = closure.owner
                && owns(facts, owner)
            {
                bodies.push(self.closure(report, facts, closure, owner)?);
            }
        }
        for (source, _) in facts.constant_bodies() {
            if facts
                .inline_constants
                .as_deref()
                .and_then(|r| r.get(source))
                .is_some_and(|b| {
                    b.evaluation == crate::sema::inline_constants::Evaluation::DeferredTemplate
                })
            {
                bodies.push(self.anonymous(report, facts, source)?);
            }
        }
        if !self.anonymous.is_empty() {
            return Err(Declarations::error(
                report,
                *self.anonymous.keys().next().ok_or_else(|| {
                    Error::at(
                        crate::resolve::unit::FileId(0),
                        0,
                        "lost anonymous template",
                    )
                })?,
                "anonymous template has no program owner",
            ));
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

    fn closure(
        &mut self,
        report: &Report,
        facts: &Declarations,
        closure: &crate::sema::closures::Body,
        owner: crate::resolve::model::DefId,
    ) -> Result<Template, Error> {
        let environment = environment(report, facts, owner, closure.source)?;
        let query = self.query.get_or_insert_with(|| {
            NeedsDrop::new(facts.destructors.implementations().iter().map(|i| i.item))
        });
        let mut scoped = query.scoped(Arc::clone(&environment));
        let scratch = self
            .scratch
            .get_or_insert_with(|| Box::new(Scratch::new(facts)));
        let body = super::closures::body(report, facts, scratch, closure, Some(&mut scoped))?;
        let destruction = crate::body::drops::requirements(facts, &body, &mut scoped)
            .map_err(|message| Declarations::error(report, body.source, message))?;
        self.built += 1;
        Ok(Template {
            body,
            destruction,
            environment,
        })
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

#[cfg(test)]
#[path = "../../../tests/invariants/conditional_parents.rs"]
mod inline_component_tests;
