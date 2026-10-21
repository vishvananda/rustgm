//! Checked constant execution shares demanded CFGs with ordinary compilation.
mod anonymous;
mod expressions;
mod initializers;
mod inputs;
#[cfg(test)]
#[path = "../../tests/invariants/execution_storage.rs"]
mod storage_tests;
use super::Prepared;
use crate::{
    body::{
        self,
        ir::{Body, BodyKey, Owner},
        regions::{Collector, Contract},
        templates::{Template, Templates},
    },
    borrow,
    const_eval::{self, Limits, Machine, code::Catalog},
    resolve::{expansion::Report, model::DefId, package::Error, unit::FileId},
    sema::{
        model::Declarations,
        values::{
            self, Constant, Evaluator,
            execution::{Executor, Progress, Request, Requirement, Target},
        },
    },
    types::{ItemId, Kind, parameters::environment::Environment},
};
pub(crate) use inputs::Context;
use std::{cell::RefCell, collections::HashMap, rc::Rc, sync::Arc};

pub(super) struct Proxy(pub Rc<RefCell<Engine>>);
impl Executor for Proxy {
    fn check(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        target: Target,
    ) -> Result<(), Error> {
        self.0
            .try_borrow_mut()
            .map_err(|_| Error::at(FileId(0), 0, "reentrant constant eligibility request"))?
            .root(report, facts, target)
            .map(|_| ())
    }
    fn advance(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        request: &Request,
    ) -> Result<Progress, Error> {
        self.0
            .try_borrow_mut()
            .map_err(|_| Error::at(FileId(0), 0, "reentrant constant execution request"))?
            .advance(report, facts, request)
    }
}
#[derive(Default)]
struct Session {
    state: Option<const_eval::State>,
    waiting: Option<Requirement>,
}
#[derive(Default)]
pub(super) struct Engine {
    catalog: Catalog,
    values: values::Scratch,
    limits: Limits,
    suspended_slots: usize,
    roots: HashMap<Target, usize>,
    sessions: HashMap<Request, Session>,
    completed: HashMap<Request, Constant>,
    construction: body::templates::Prepared,
    contracts: Collector,
    promotions: body::promote::Scratch,
    originals: HashMap<DefId, Template>,
    anonymous: HashMap<crate::resolve::unit::NodeKey, (Template, Contract)>,
    concrete: HashMap<BodyKey, Body>,
    loans: Option<(usize, borrow::loans::templates::Verifier)>,
    inputs: Rc<RefCell<HashMap<BodyKey, Body>>>,
    runtime_templates: Option<Rc<Templates>>,
    runtime_template_indices: HashMap<DefId, usize>,
    retained_expressions: usize,
    charged_templates: std::collections::HashSet<DefId>,
}
impl Engine {
    pub(super) fn transfer(
        &mut self,
        report: &Report,
        prepared: &mut Prepared,
    ) -> Result<(), Error> {
        for (key, body) in std::mem::take(&mut self.concrete) {
            if prepared.concrete.contains_key(&key) {
                return Err(Declarations::error(
                    report,
                    body.source,
                    "duplicate retained constant CFG owner",
                ));
            }
            prepared.concrete.insert(key, body);
        }
        for (template, _) in self.anonymous.values() {
            prepared
                .templates
                .retain(template.clone())
                .map_err(|e| Declarations::error(report, template.body.source, e))?;
        }
        for (&definition, template) in &self.originals {
            if matches!(template.body.owner, Owner::Function { .. }) {
                prepared
                    .templates
                    .retain(template.clone())
                    .map_err(|e| Declarations::error(report, template.body.source, e))?;
            } else {
                return Err(Error::at(
                    FileId(0),
                    0,
                    format!("constant template {} is not a function", definition.0),
                ));
            }
        }
        Ok(())
    }
    fn advance(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        request: &Request,
    ) -> Result<Progress, Error> {
        if let Some(&value) = self.completed.get(request) {
            return Ok(Progress::Complete(value));
        }
        let mut session = self.take_session(request)?;
        loop {
            if let Some(requirement) = session.waiting {
                if !self.prepare(report, facts, requirement)? {
                    return self.retain_session(facts, request, session, requirement);
                }
                session.waiting = None;
            }
            let root = self.root(report, facts, request.target)?;
            let preparation = self.values.preparation_steps();
            let evaluator =
                Evaluator::with_scratch(report, facts, std::mem::take(&mut self.values));
            let limits = self.available_limits()?;
            let mut machine = Machine::catalog(evaluator, &self.catalog, limits);
            if let Some(state) = session.state {
                machine.restore(state);
            } else {
                machine
                    .begin(root, &request.arguments)
                    .map_err(|e| Declarations::error(report, e.source, e.message))?;
            }
            let progress = machine
                .advance()
                .map_err(|e| Declarations::error(report, e.source, e.message))?;
            let work = machine.work();
            match progress {
                Progress::Complete(value) => {
                    let (_, scratch) = machine.suspend();
                    self.retain_values(facts, scratch, preparation);
                    facts.constant_work.expressions += 1;
                    facts.constant_work.frames_started += work.calls;
                    facts.constant_work.steps += work.steps;
                    facts.constant_work.peak_frames =
                        facts.constant_work.peak_frames.max(work.peak_frames);
                    facts.constant_work.peak_slots =
                        facts.constant_work.peak_slots.max(work.peak_slots);
                    facts.constant_work.code_bytes = self.catalog.bytes;

                    self.completed.insert(request.clone(), value);
                    return Ok(Progress::Complete(value));
                }
                Progress::Prepare(requirement) => {
                    let (state, scratch) = machine.suspend();
                    self.retain_values(facts, scratch, preparation);
                    facts.constant_work.resumes += 1;
                    session = Session {
                        state: Some(state),
                        waiting: Some(requirement),
                    };
                }
            }
        }
    }
    fn take_session(&mut self, request: &Request) -> Result<Session, Error> {
        let session = self.sessions.remove(request).unwrap_or_default();
        let retained = session
            .state
            .as_ref()
            .map_or(0, const_eval::State::retained_slots);
        self.suspended_slots = self.suspended_slots.checked_sub(retained).ok_or_else(|| {
            Error::at(
                FileId(0),
                0,
                "constant continuation storage accounting is inconsistent",
            )
        })?;
        Ok(session)
    }
    fn retain_session(
        &mut self,
        facts: &mut Declarations,
        request: &Request,
        session: Session,
        requirement: Requirement,
    ) -> Result<Progress, Error> {
        let retained = session
            .state
            .as_ref()
            .map_or(0, const_eval::State::retained_slots);
        self.suspended_slots = self
            .suspended_slots
            .checked_add(retained)
            .filter(|slots| *slots <= self.limits.slots)
            .ok_or_else(|| Error::at(FileId(0), 0, "constant suspended storage limit exceeded"))?;
        facts.constant_work.peak_suspended_slots = facts
            .constant_work
            .peak_suspended_slots
            .max(self.suspended_slots);
        self.sessions.insert(request.clone(), session);
        Ok(Progress::Prepare(requirement))
    }
    fn available_limits(&self) -> Result<Limits, Error> {
        Ok(Limits {
            slots: self
                .limits
                .slots
                .checked_sub(self.suspended_slots)
                .ok_or_else(|| {
                    Error::at(FileId(0), 0, "constant suspended storage exceeds limit")
                })?,
            ..self.limits
        })
    }
    fn retain_values(
        &mut self,
        facts: &mut Declarations,
        scratch: values::Scratch,
        preparation: usize,
    ) {
        facts.constant_work.value_preparation += scratch.preparation_steps() - preparation;
        self.values = scratch;
    }
    fn prepare(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        requirement: Requirement,
    ) -> Result<bool, Error> {
        match requirement {
            Requirement::FunctionBody(instance) => self.function(report, facts, instance),
            Requirement::AnonymousBody { instance, result } => {
                self.root(report, facts, Target::Anonymous { instance, result })?;
                Ok(true)
            }
            Requirement::InlineBody { source, result } => {
                facts
                    .inline_constants
                    .as_mut()
                    .ok_or_else(|| {
                        Declarations::error(report, source, "missing required inline constant")
                    })?
                    .require(source, result)
                    .map_err(|e| Declarations::error(report, source, e))?;
                self.root(
                    report,
                    facts,
                    Target::Expression {
                        source,
                        result,
                        context: None,
                    },
                )?;
                Ok(true)
            }
            Requirement::InitializerBody(definition) => {
                let index = facts
                    .initializer_index(definition)
                    .ok_or_else(|| Error::at(FileId(0), 0, "missing required initializer"))?;
                let initializer = facts.initializers[index];
                if !initializer.type_checked() {
                    return Ok(false);
                }
                let target = if initializer.semantic_source() == initializer.value {
                    Target::Expression {
                        source: initializer.value,
                        result: initializer.ty,
                        context: None,
                    }
                } else {
                    Target::Initializer(definition)
                };
                let index = self.root(report, facts, target)?;
                self.catalog
                    .initializer(index, definition, initializer.kind)
                    .map_err(|e| Declarations::error(report, initializer.value, e))?;
                Ok(true)
            }
            Requirement::AssociatedBody(instance) => {
                let mut evaluator = Evaluator::new(report, facts);
                evaluator.associated(instance)?;
                Ok(true)
            }
        }
    }
    fn function(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        instance: crate::types::instance::Instance,
    ) -> Result<bool, Error> {
        if self.catalog.functions.contains_key(&instance) {
            return Ok(true);
        }
        let Some(Kind::FunctionItem { item, .. }) = facts.types.kind(instance.ty()) else {
            return Err(Error::at(FileId(0), 0, "missing selected const function"));
        };
        let definition = DefId(item.0);
        if !facts.typed_function_bodies.contains(&definition) {
            return Ok(false);
        }
        let function = *facts
            .function_indices
            .get(&definition)
            .and_then(|&index| facts.functions.get(index))
            .ok_or_else(|| Error::at(FileId(0), 0, "missing checked const function"))?;
        let source = function.body.unwrap_or(function.node);
        let input = self
            .contracts
            .function(facts, &function)
            .map_err(|e| Declarations::error(report, source, e))?
            .clone();
        let inputs = Rc::clone(&self.inputs);
        let stored = inputs.try_borrow().map_err(|_| {
            Declarations::error(
                report,
                source,
                "constant CFG lender is preparing another body",
            )
        })?;
        if let Some(body) = stored.get(&BodyKey::Declaration(definition)) {
            body::constants::eligibility(report, facts, body)?;
            let mut needed = crate::types::destruction::NeedsDrop::new(
                facts.destructors.implementations().iter().map(|i| i.item),
            );
            let destruction = body::drops::requirements(facts, body, &mut needed)
                .map_err(|e| Declarations::error(report, source, e))?;
            let parameters = Arc::new(
                facts
                    .generics
                    .parameter_environment(&facts.types, definition)
                    .map_err(|e| Declarations::error(report, source, e))?,
            );
            self.prove_body(report, facts, body, &destruction, &parameters, &input)?;
            let index = self
                .catalog
                .insert(body)
                .map_err(|e| Declarations::error(report, source, e))?;
            self.catalog.functions.insert(instance, index);
            return Ok(true);
        }
        drop(stored);
        if let Some(templates) = self.runtime_templates.as_ref().map(Rc::clone)
            && let Some(&index) = self.runtime_template_indices.get(&definition)
        {
            self.specialize(report, facts, instance, &templates.bodies[index], &input)?;
            return Ok(true);
        }
        let template = if let Some(template) = self.originals.remove(&definition) {
            template
        } else {
            let mut template = self.construction.function(report, facts, &function)?;
            body::promote::instance_reusing(
                report,
                facts,
                &mut template.body,
                &template.destruction,
                &mut self.promotions,
            )?;
            template
        };
        self.prove(report, facts, &template, &input)?;
        if facts.generics.has_parameters(definition)
            || facts.opaques.owners.contains_key(&definition)
        {
            self.reserve_template(report, facts, definition, &template)?;
            self.specialize(report, facts, instance, &template, &input)?;
            self.originals.insert(definition, template);
        } else {
            let index = self
                .catalog
                .insert(&template.body)
                .map_err(|e| Declarations::error(report, source, e))?;
            self.catalog.functions.insert(instance, index);
            self.concrete
                .insert(BodyKey::Declaration(definition), template.body);
        }
        Ok(true)
    }
    fn reserve_template(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        definition: DefId,
        template: &Template,
    ) -> Result<(), Error> {
        let bytes = template.retained_bytes();
        let retained = if self.charged_templates.contains(&definition) {
            facts.constant_work.template_bytes
        } else {
            facts
                .constant_work
                .template_bytes
                .saturating_add(bytes.saturating_mul(2))
        };
        if retained > 64 * 1024 * 1024 {
            return Err(Declarations::error(
                report,
                template.body.source,
                "constant template copies exceed 64 MiB budget",
            ));
        }
        facts.constant_work.template_bytes = retained;
        self.charged_templates.insert(definition);
        Ok(())
    }
    fn specialize(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        instance: crate::types::instance::Instance,
        template: &Template,
        input: &Contract,
    ) -> Result<(), Error> {
        let source = template.body.source;
        if template.retained_bytes() > 64 * 1024 * 1024 {
            return Err(Declarations::error(
                report,
                source,
                "constant instance copy exceeds 64 MiB budget",
            ));
        }
        let mut materialized = body::instantiate::with_contract(facts, template, instance, input)
            .map_err(|e| Declarations::error(report, source, e))?;
        body::dispatch::resolve(facts, &mut materialized.body, &mut materialized.contract)
            .map_err(|e| Declarations::error(report, source, e))?;
        body::promote::instance_reusing(
            report,
            facts,
            &mut materialized.body,
            &materialized.destruction,
            &mut self.promotions,
        )?;
        self.prove_body(
            report,
            facts,
            &materialized.body,
            &materialized.destruction,
            &template.environment,
            &materialized.contract,
        )?;
        let index = self
            .catalog
            .insert(&materialized.body)
            .map_err(|e| Declarations::error(report, source, e))?;
        self.catalog.functions.insert(instance, index);
        Ok(())
    }
    fn prove(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        template: &Template,
        contract: &Contract,
    ) -> Result<(), Error> {
        self.prove_body(
            report,
            facts,
            &template.body,
            &template.destruction,
            &template.environment,
            contract,
        )
    }
    fn prove_body(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        body: &Body,
        destruction: &body::drops::Requirements,
        parameters: &Arc<Environment>,
        contract: &Contract,
    ) -> Result<(), Error> {
        let source = body.source;
        contract
            .prepare_outlives(&mut facts.types, &mut facts.lifetimes.nominal_requirements)
            .map_err(|e| Declarations::error(report, source, e))?;
        let mut queries = crate::types::outlives::Queries::new(
            &facts.types,
            &facts.lifetimes.nominal_requirements,
            &facts.lifetimes.callables,
        );
        let regions = borrow::regions::check_one(
            &facts.types,
            &facts.lifetimes.variances,
            &mut queries,
            contract,
            body,
        )
        .map_err(|e| Declarations::error(report, e.source.unwrap_or(source), e.message))?;
        let custom = facts
            .destructors
            .implementations()
            .iter()
            .map(|i| i.item)
            .collect::<Vec<ItemId>>();
        if self
            .loans
            .as_ref()
            .is_none_or(|(count, _)| *count != custom.len())
        {
            self.loans = Some((
                custom.len(),
                borrow::loans::templates::Verifier::new(&custom)
                    .map_err(|e| Declarations::error(report, source, e))?,
            ));
        }
        let (_, loans) = self
            .loans
            .as_mut()
            .ok_or_else(|| Declarations::error(report, source, "missing constant loan verifier"))?;
        let mut copies = crate::types::copy::Copies::default();
        loans
            .body(
                &mut borrow::Environment {
                    types: &facts.types,
                    definitions: &facts.aggregates,
                    derives: &facts.derives,
                    copies: &mut copies,
                },
                body,
                destruction,
                parameters,
                &regions,
                &|ty, field| body::drops::nominal_position(facts, ty, field),
            )
            .map_err(|e| Declarations::error(report, e.node.unwrap_or(source), e.message))?;
        Ok(())
    }
}
