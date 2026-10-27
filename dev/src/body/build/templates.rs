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
    imported: Option<std::rc::Rc<crate::body::transported_originals::ImportedOriginals>>,
}
impl Templates {
    #[must_use]
    pub const fn source(bodies: Vec<Template>) -> Self {
        Self {
            bodies,
            imported: None,
        }
    }
    pub(crate) fn attach(
        &mut self,
        imported: Option<std::rc::Rc<crate::body::transported_originals::ImportedOriginals>>,
    ) -> Result<(), &'static str> {
        if self.imported.is_some() {
            return Err("imported originals are already attached");
        }
        self.imported = imported;
        Ok(())
    }
    pub(crate) fn append_original_bound(&self) -> crate::support::retained::Growth {
        if let Some(owner) = &self.imported {
            owner.append_bound()
        } else {
            let mut growth =
                crate::body::transported_originals::ImportedOriginals::default().append_bound();
            growth.bytes = growth.bytes.saturating_add(
                size_of::<crate::body::transported_originals::ImportedOriginals>()
                    + 2 * size_of::<usize>(),
            );
            growth
        }
    }
    pub(crate) fn require_unique_originals(&mut self) -> Result<(), &'static str> {
        if let Some(owner) = &mut self.imported
            && std::rc::Rc::get_mut(owner).is_none()
        {
            return Err("selected original append requires its uniquely owned immutable originals");
        }
        Ok(())
    }
    pub(crate) fn append_original(
        &mut self,
        row: crate::body::transported_originals::Original,
    ) -> Result<(), &'static str> {
        self.require_unique_originals()?;
        let owner = self.imported.get_or_insert_with(|| {
            std::rc::Rc::new(crate::body::transported_originals::ImportedOriginals::default())
        });
        std::rc::Rc::get_mut(owner)
            .ok_or("selected original append lost its uniquely owned immutable originals")?
            .append_selected(row)
    }
    pub(crate) fn iter(&self) -> impl Iterator<Item = &Template> {
        self.bodies.iter().chain(self.imported.iter().flat_map(|i| {
            i.iter()
                .map(crate::body::transported_originals::Original::template)
        }))
    }
    pub(crate) fn len(&self) -> usize {
        self.bodies.len() + self.imported.as_ref().map_or(0, |i| i.len())
    }
    pub(crate) fn input_count(&self, sources: usize) -> usize {
        sources.saturating_add(self.imported.as_ref().map_or(0, |i| i.len()))
    }
    pub(crate) fn imported_originals(
        &self,
    ) -> Option<&std::rc::Rc<crate::body::transported_originals::ImportedOriginals>> {
        self.imported.as_ref()
    }
    pub(crate) fn get(&self, index: usize) -> Option<&Template> {
        if index < self.bodies.len() {
            self.bodies.get(index)
        } else {
            self.imported
                .as_ref()?
                .at(index - self.bodies.len())
                .map(crate::body::transported_originals::Original::template)
        }
    }
    pub(crate) fn inputs<'a>(
        &'a self,
        sources: &'a [crate::body::regions::Contract],
    ) -> impl Iterator<Item = &'a crate::body::regions::Contract> {
        sources.iter().chain(self.imported.iter().flat_map(|i| {
            i.iter()
                .map(crate::body::transported_originals::Original::contract)
        }))
    }
}

#[derive(Default)]
struct ClosureTemplates {
    bodies: crate::support::hash::HashMap<crate::types::closure::ClosureId, Template>,
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
    bodies: crate::support::hash::HashMap<crate::resolve::model::DefId, Template>,
    anonymous: crate::support::hash::HashMap<crate::resolve::unit::NodeKey, Template>,
    closures: Option<Box<ClosureTemplates>>,
    built: usize,
    reused: usize,
}
impl Prepared {
    /// Build one already typed anonymous constant using retained construction scratch.
    /// The same fixed source environment must also verify the resulting template;
    /// possible opaque cleanup is retained until executable head selection.
    /// # Errors
    /// Reports missing facts and the same const eligibility failures as functions.
    pub fn expression(
        &mut self,
        report: &Report,
        facts: &Declarations,
        source: crate::resolve::unit::NodeKey,
        result: crate::types::TypeId,
        environment: Arc<Environment>,
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
        let mut scoped =
            (!facts.opaques.declarations.is_empty()).then(|| query.scoped(environment));
        let query = scoped.as_mut().unwrap_or(query);
        let body = super::Builder::new(
            report,
            facts,
            scratch,
            Body::new(crate::body::ir::Owner::AnonymousConstant, source, result),
            scope_plan,
            Some(query),
        )?
        .run(None)
        .map_err(|e| report.diagnostic(e))?;
        crate::body::constants::eligibility(report, facts, &body)?;
        let requirements = crate::body::drops::requirements(facts, &body, query)
            .map_err(|e| report.body_error(source, e))?;
        Ok((body, requirements))
    }
    pub(crate) fn initializer(
        &mut self,
        report: &Report,
        facts: &Declarations,
        initializer: &crate::sema::model::Initializer,
    ) -> Result<Template, Error> {
        if let Some(template) = self.bodies.remove(&initializer.definition) {
            if template.body.source != initializer.semantic_source().into()
                || template.body.result != initializer.ty
                || !matches!(template.body.owner,crate::body::ir::Owner::Initializer {definition,..} if definition==initializer.definition)
            {
                return Err(report.body_error(
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
            .map_err(|e| report.body_error(body.source, e))?;
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
            .ok_or_else(|| report.body_error(source, "missing anonymous template"))?;
        let owner = inline
            .declaration
            .ok_or_else(|| report.body_error(source, "anonymous template has no declaration"))?;
        let result = inline
            .result
            .ok_or_else(|| report.body_error(source, "anonymous template result is not ready"))?;
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
        )?
        .run(None)
        .map_err(|e| report.diagnostic(e))?;
        crate::body::constants::eligibility(report, facts, &body)?;
        let destruction = crate::body::drops::requirements(facts, &body, &mut scoped)
            .map_err(|e| report.body_error(source, e))?;
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
            .map_err(|message| report.body_error(body.source, message))?;
        self.built += 1;
        Ok(Template {
            body,
            destruction,
            environment,
        })
    }

    pub(crate) fn retain_component(
        &mut self,
        templates: Vec<Template>,
    ) -> Result<(), &'static str> {
        let mut keys = crate::support::hash::HashSet::with_capacity_and_hasher(
            templates.len(),
            crate::support::hash::BuildFxHasher::default(),
        );
        for template in &templates {
            let body = &template.body;
            let present = match body.owner {
                crate::body::ir::Owner::Function { definition, .. }
                | crate::body::ir::Owner::Initializer { definition, .. } => {
                    self.bodies.contains_key(&definition)
                }
                crate::body::ir::Owner::Closure { identity, .. } => self
                    .closures
                    .as_ref()
                    .is_some_and(|c| c.bodies.contains_key(&identity)),
                crate::body::ir::Owner::AnonymousConstant => body
                    .source
                    .source()
                    .is_some_and(|source| self.anonymous.contains_key(&source)),
                _ => return Err("original component contains a selected instance"),
            };
            if body.instance.is_some() || present || !keys.insert(body.key()) {
                return Err("original component conflicts with a retained template owner");
            }
        }
        for template in templates {
            self.retain(template)?;
        }
        Ok(())
    }
    /// Retain a completed original function for later template qualification and lowering.
    /// # Errors
    /// Rejects duplicate retained owners and non-function bodies.
    pub fn retain(&mut self, template: Template) -> Result<(), &'static str> {
        if matches!(
            template.body.owner,
            crate::body::ir::Owner::AnonymousConstant
        ) {
            let source = template
                .body
                .source
                .source()
                .ok_or("imported anonymous template requires its authenticated body catalog")?;
            if self.anonymous.insert(source, template).is_some() {
                return Err("anonymous template was retained twice");
            }
            return Ok(());
        }
        if let crate::body::ir::Owner::Closure { identity, .. } = template.body.owner {
            if self
                .closures
                .get_or_insert_with(Default::default)
                .bodies
                .insert(identity, template)
                .is_some()
            {
                return Err("closure template was retained twice");
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
            return Err(report.body_error(
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
        if self.closures.as_ref().is_some_and(|c| !c.bodies.is_empty()) {
            return Err(Error::at(
                crate::resolve::unit::FileId(0),
                0,
                "retained closure template has no program owner",
            ));
        }
        if let Some(template) = self.bodies.values().next() {
            return Err(report.body_error(
                template.body.source,
                "retained template has no program owner",
            ));
        }
        crate::body::constants::alias_cycles(report, facts, &bodies)?;
        Ok(Templates::source(bodies))
    }

    pub(crate) fn closure(
        &mut self,
        report: &Report,
        facts: &Declarations,
        closure: &crate::sema::closures::Body,
        owner: crate::resolve::model::DefId,
    ) -> Result<Template, Error> {
        if let Some(template) = self
            .closures
            .as_mut()
            .and_then(|c| c.bodies.remove(&closure.identity))
        {
            if template.body.source != closure.semantic_source().into()
                || template.body.key() != crate::body::ir::BodyKey::Closure(closure.identity)
            {
                return Err(
                    report.body_error(closure.source, "retained closure original changed owner")
                );
            }
            self.reused += 1;
            return Ok(template);
        }
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
            .map_err(|message| report.body_error(body.source, message))?;
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
            .map_err(|e| report.body_error(source, e))
    }
}

#[cfg(test)]
#[path = "../../../tests/invariants/conditional_parents.rs"]
mod inline_component_tests;
