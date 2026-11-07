//! Checked constant execution shares demanded CFGs with ordinary compilation.
mod always_defs;
mod anonymous;
mod closures;
mod code_inputs;
mod component_index;
mod components;
mod ctbodies;
mod defining_heads;
mod expressions;
#[cfg(test)]
#[path = "../../tests/invariants/transport_debts.rs"]
mod imported_tests;
mod initializers;
mod inputs;
mod layout_kernel;
#[cfg(test)]
#[path = "../../tests/helpers/materialized_bodies.rs"]
pub(crate) mod materialized_bodies;
mod needs_drop_kernel;
mod parent_children;
mod parent_scope;
mod reached_payments;
mod selected;
#[cfg(test)]
#[path = "../../tests/invariants/execution_storage.rs"]
mod storage_tests;
use super::Prepared;
use crate::support::hash::HashMap;
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
use std::{cell::RefCell, rc::Rc, sync::Arc};

/// Runs one execution step; only a pending Copy decision's own failure
/// becomes the selection wait it names, and any other failure keeps its cause.
fn awaiting<T>(
    facts: &mut Declarations,
    step: impl FnOnce(&mut Declarations) -> Result<T, Error>,
) -> Result<Result<T, Requirement>, Error> {
    facts.pending_copy_selection = None;
    match step(facts) {
        Ok(value) => Ok(Ok(value)),
        Err(error) => crate::sema::compiler_items::awaited_copy_selection(facts, &error.message)
            .map(|dependency| {
                Err(Requirement::Selection(values::execution::SelectionWait(
                    dependency,
                )))
            })
            .ok_or(error),
    }
}

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
            .check_root(report, facts, target)
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
struct CheckedInput {
    retained: Option<const_eval::use_inputs::Source>,
    code: Option<const_eval::code::Prepared>,
}
impl CheckedInput {
    fn take_code(&mut self) -> Result<const_eval::code::Prepared, &'static str> {
        self.code
            .take()
            .ok_or("original proof has no owned executable snapshot")
    }
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum ProofPurpose {
    Original,
    Code,
}
pub(super) struct LoanInput<'a, 'input> {
    pub(super) parameters: &'a Arc<Environment>,
    pub(super) contract: &'a mut std::borrow::Cow<'input, Contract>,
    pub(super) regions: &'a mut borrow::regions::Body,
    pub(super) purpose: ProofPurpose,
}
#[derive(Clone, Copy)]
struct ProofInput<'a> {
    parameters: &'a Arc<Environment>,
    contract: &'a Contract,
    purpose: ProofPurpose,
    materialized: Option<&'a body::instantiate::Materialized>,
}
impl<'a> ProofInput<'a> {
    fn code(parameters: &'a Arc<Environment>, contract: &'a Contract) -> Self {
        Self {
            parameters,
            contract,
            purpose: ProofPurpose::Code,
            materialized: None,
        }
    }
    fn materialized(
        parameters: &'a Arc<Environment>,
        materialized: &'a body::instantiate::Materialized,
    ) -> Self {
        Self {
            parameters,
            contract: &materialized.contract,
            purpose: ProofPurpose::Code,
            materialized: Some(materialized),
        }
    }
}
#[derive(Default)]
struct Session {
    state: Option<const_eval::State>,
    waiting: Option<Requirement>,
}
type InitializerTemplates = HashMap<DefId, (Template, Contract)>;
#[derive(Default)]
pub(super) struct Engine {
    imported: Option<Rc<body::transported_originals::ImportedOriginals>>,
    catalog: Catalog,
    const_calls: Option<Box<const_eval::use_inputs::Inputs>>,
    values: values::Scratch,
    limits: Limits,
    suspended_slots: usize,
    roots: HashMap<Target, usize>,
    sessions: HashMap<Request, Session>,
    completed: HashMap<Request, Constant>,
    construction: body::templates::Prepared,
    contracts: Collector,
    promotions: body::promote::Scratch,
    associated: Option<Box<InitializerTemplates>>,
    originals: HashMap<DefId, Template>,
    anonymous: HashMap<crate::resolve::unit::NodeKey, (Template, Contract)>,
    original_components: Option<Box<components::Retained>>,
    concrete: HashMap<BodyKey, Body>,
    loans: Option<(usize, borrow::loans::templates::Verifier)>,
    inputs: Rc<RefCell<HashMap<BodyKey, Body>>>,
    runtime_templates: Option<Rc<Templates>>,
    runtime_template_indices: HashMap<DefId, usize>,
    retained_expressions: usize,
    charged_templates: crate::support::hash::HashSet<DefId>,
    pending_fields: Option<crate::types::ItemId>,
}
impl Engine {
    pub(super) fn transfer(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        prepared: &mut Prepared,
    ) -> Result<(), Error> {
        self.ensure_imported_originals(report, facts)?;
        prepared.imported = self.imported.as_ref().map(Rc::clone);
        self.transfer_components(report, facts, prepared)?;
        for (key, body) in std::mem::take(&mut self.concrete) {
            if prepared.concrete.contains_key(&key) {
                return Err(report.body_error(body.source, "duplicate retained constant CFG owner"));
            }
            prepared.concrete.insert(key, body);
        }
        for (template, _) in self.associated.iter().flat_map(|map| map.values()) {
            if self.component_contains(template.body.key()) {
                continue;
            }
            prepared
                .templates
                .retain(template.clone())
                .map_err(|e| report.body_error(template.body.source, e))?;
        }
        for (template, _) in self.anonymous.values() {
            if self.component_contains(template.body.key()) {
                continue;
            }
            prepared
                .templates
                .retain(template.clone())
                .map_err(|e| report.body_error(template.body.source, e))?;
        }
        self.transfer_defining_bodies(report, facts, prepared)
    }
    fn transfer_defining_bodies(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        prepared: &mut Prepared,
    ) -> Result<(), Error> {
        let mut deferred = Vec::new();
        for (&definition, template) in &self.originals {
            if matches!(template.body.owner, Owner::AsyncBody { .. }) {
                let growth = crate::support::retained::Growth::vector::<DefId>(
                    deferred.len(),
                    deferred.capacity(),
                    1,
                );
                let bytes = facts
                    .constant_work
                    .template_bytes
                    .saturating_add(growth.bytes);
                if bytes > 64 * 1024 * 1024 {
                    return Err(report.body_error(
                        template.body.source,
                        "deferred original handoff exceeds storage budget",
                    ));
                }
                facts.constant_work.template_bytes = bytes;
                facts.body_work.canonical_steps =
                    facts.body_work.canonical_steps.saturating_add(growth.work);
                deferred.push(definition);
                continue;
            }
            if self.component_contains(template.body.key()) {
                continue;
            }
            if matches!(template.body.owner, Owner::Function { .. }) {
                if body::templates::owns(facts, definition) {
                    prepared
                        .templates
                        .retain(template.clone())
                        .map_err(|e| report.body_error(template.body.source, e))?;
                } else {
                    let function = facts
                        .function_indices
                        .get(&definition)
                        .and_then(|&index| facts.functions.get(index))
                        .copied()
                        .ok_or_else(|| {
                            Error::unlocated("retained defining CFG lost its function")
                        })?;
                    always_defs::validate(report, facts, &function, &template.body)?;
                    let key = BodyKey::Declaration(definition);
                    if prepared.concrete.contains_key(&key) {
                        return Err(report.body_error(
                            template.body.source,
                            "duplicate retained defining CFG owner",
                        ));
                    }
                    prepared.concrete.insert(key, template.body.clone());
                }
            } else {
                return Err(Error::at(
                    FileId(0),
                    0,
                    format!("constant template {} is not a function", definition.0),
                ));
            }
        }
        for definition in deferred {
            let template = self
                .originals
                .remove(&definition)
                .ok_or_else(|| Error::unlocated("deferred handoff lost its owned original"))?;
            let source = template.body.source;
            if self.component_contains(template.body.key()) {
                return Err(report.body_error(
                    source,
                    "deferred original has duplicate component ownership",
                ));
            }
            prepared
                .templates
                .retain_original(facts, template)
                .map_err(|error| report.body_error(source, error))?;
        }
        Ok(())
    }
    fn advance(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        request: &Request,
    ) -> Result<Progress, Error> {
        self.validate_parent_scope(facts, request.target)
            .map_err(|e| Error::at(FileId(0), 0, e))?;
        self.reserve_parent_request(request)
            .map_err(|e| Error::at(FileId(0), 0, e))?;
        if let Target::SourceAssociated { source, result } = request.target {
            source
                .validate(facts, result)
                .map_err(|e| Error::at(crate::resolve::unit::FileId(0), 0, e))?;
        }
        if let Target::Anonymous { instance, .. } = request.target
            && let crate::sema::inline_constants::Context::SourceInitializer(witness) =
                instance.enclosing()
        {
            let checked = (|| {
                witness.validate(facts, witness.result(facts)?)?;
                let selected = crate::sema::inline_constants::InlineInstance::select(
                    facts,
                    instance.source(),
                    instance.enclosing(),
                )?;
                if selected != instance {
                    return Err("anonymous request changed its source context");
                }
                Ok(())
            })();
            checked.map_err(|e| Error::at(FileId(0), 0, e))?;
        }
        if let Some(&value) = self.completed.get(request) {
            self.check_completed_expression(facts, request.target)
                .map_err(|e| Error::at(FileId(0), 0, e))?;
            return Ok(Progress::Complete(value));
        }
        let mut session = self.take_session(request)?;
        loop {
            if let Some(requirement) = session.waiting {
                let state = session.state.as_mut();
                if let Some(surfaced) = self.prepare_session(report, facts, requirement, state)? {
                    return self.retain_session(facts, request, session, surfaced);
                }
                session.waiting = None;
            }
            let target = request.target;
            let root = match awaiting(facts, |facts| self.root(report, facts, target))? {
                Ok(root) => root,
                Err(waiting) => {
                    session.waiting = Some(waiting);
                    return self.retain_session(facts, request, session, waiting);
                }
            };
            let preparation = self.values.preparation_steps();
            let evaluator =
                Evaluator::with_scratch(report, facts, std::mem::take(&mut self.values));
            let limits = self.available_limits()?;
            let mut machine = Machine::source_catalog(
                evaluator,
                &self.catalog,
                self.const_calls.as_deref(),
                limits,
            );
            if let Some(state) = session.state {
                machine.restore(state);
            } else {
                machine
                    .begin(root, &request.arguments)
                    .map_err(|e| report.body_optional_error(e.source, e.message))?;
            }
            let progress = machine
                .advance()
                .map_err(|e| report.body_optional_error(e.source, e.message))?;
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
                    facts.constant_work.code_bytes = self.catalog.bytes.saturating_add(
                        self.const_calls
                            .as_deref()
                            .map_or(0, const_eval::use_inputs::Inputs::retained_bytes),
                    );

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
    /// A root whose Copy decision awaits a queued declaration fact is checked
    /// again, and waits, when its execution request repeats this same root.
    fn check_root(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        target: Target,
    ) -> Result<(), Error> {
        awaiting(facts, |facts| self.root(report, facts, target)).map(|_| ())
    }
    /// Prepares a suspended session's requirement. `Some` names the fact still
    /// awaited; the session itself keeps its own requirement.
    fn prepare_session(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        requirement: Requirement,
        state: Option<&mut const_eval::State>,
    ) -> Result<Option<Requirement>, Error> {
        self.pending_fields = None;
        Ok(
            match awaiting(facts, |facts| {
                self.prepare(report, facts, requirement, state)
            })? {
                Ok(true) => None,
                // The session repeats its own requirement once the
                // declaration worklist publishes the awaited fact.
                Err(waiting) => Some(waiting),
                // Only the missing declaration fact is surfaced to the
                // declaration pipeline.
                Ok(false) => Some(
                    self.pending_fields
                        .take()
                        .map_or(requirement, Requirement::NominalFields),
                ),
            },
        )
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
        state: Option<&mut const_eval::State>,
    ) -> Result<bool, Error> {
        match requirement {
            Requirement::DiscriminantValue(variant) => values::Reader::new(facts)
                .discriminant(variant)
                .map(|value| value.is_some())
                .map_err(|e| Error::at(FileId(0), 0, e)),
            Requirement::NominalFields(item) => Ok(facts.aggregates.get(item).is_some()),
            Requirement::Selection(wait) => Ok(wait.ready(facts)),
            Requirement::FunctionBody(instance) => {
                self.prepare_reached_function(report, facts, instance, state)
            }
            Requirement::CleanupBody(instance) => {
                self.prepare_cleanup_body(report, facts, instance, state)
            }
            Requirement::AnonymousBody { instance, result } => {
                if facts.patterns.pending_bodies.contains(&instance.source()) {
                    return Ok(false);
                }
                self.root(report, facts, Target::Anonymous { instance, result })?;
                Ok(true)
            }
            Requirement::InlineBody { source, result } => {
                facts
                    .inline_constants
                    .as_mut()
                    .ok_or_else(|| report.body_error(source, "missing required inline constant"))?
                    .require(source, result)
                    .map_err(|e| report.body_error(source, e))?;
                if facts.patterns.pending_bodies.contains(&source) {
                    return Ok(false);
                }
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
                if !initializer.type_checked()
                    || facts
                        .patterns
                        .pending_bodies
                        .contains(&initializer.semantic_source())
                {
                    return Ok(false);
                }
                // A named demand must construct and prove its declaration
                // owner before any Code/Row identity is retained. Actual
                // anonymous expression requests keep their distinct owner.
                let index = self.root(report, facts, Target::Initializer(definition))?;
                self.catalog
                    .initializer(index, definition, initializer.kind)
                    .map_err(|e| report.body_error(initializer.value, e))?;
                Ok(true)
            }
            Requirement::SourceAssociatedBody(source) => {
                let result = source
                    .result(facts)
                    .map_err(|e| Error::at(FileId(0), 0, e))?;
                self.source_associated_root(report, facts, source, result)?;
                Ok(true)
            }
            Requirement::AssociatedBody { instance, result } => {
                self.associated_ready(report, facts, instance, result)
            }
        }
    }
    fn function(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        instance: crate::types::instance::Instance,
    ) -> Result<bool, Error> {
        if matches!(facts.types.kind(instance.ty()), Some(Kind::Closure { .. })) {
            return self.closure_body(report, facts, instance);
        }
        crate::sema::compiler_items::check_const_instance(facts, instance)
            .map_err(|e| Error::at(FileId(0), 0, e))?;
        crate::sema::compiler_items::check_const_drop_glue(facts, instance)
            .map_err(|e| Error::at(FileId(0), 0, e))?;
        self.function_body(report, facts, instance, None)
    }
    fn function_body(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        instance: crate::types::instance::Instance,
        default: Option<&crate::sema::function_context::DefaultExecution>,
    ) -> Result<bool, Error> {
        if self.cached_function(report, facts, instance)? {
            return Ok(true);
        }
        let Some(Kind::FunctionItem { item, .. }) = facts.types.kind(instance.ty()) else {
            return Err(Error::at(FileId(0), 0, "missing selected const function"));
        };
        let definition = DefId(item.0);
        if self.imported_function(report, facts, instance, definition)? {
            return Ok(true);
        }
        if !facts.typed_function_bodies.contains(&definition) {
            return Ok(false);
        }
        let function = *facts
            .function_indices
            .get(&definition)
            .and_then(|&index| facts.functions.get(index))
            .ok_or_else(|| Error::at(FileId(0), 0, "missing checked const function"))?;
        let source = function.body.unwrap_or(function.node);
        if facts.patterns.pending_bodies.contains(&source) {
            return Ok(false);
        }
        let input = self
            .contracts
            .function(facts, &function)
            .map_err(|e| report.body_error(source, e))?
            .clone();
        if default.is_none() && self.lent_function(report, facts, function, instance, &input)? {
            return Ok(true);
        }
        if let Some(templates) = self.runtime_templates.as_ref().map(Rc::clone)
            && let Some(&index) = self.runtime_template_indices.get(&definition)
        {
            let template = templates
                .get(index)
                .ok_or_else(|| report.body_error(source, "runtime template index is stale"))?;
            self.specialize(report, facts, instance, template, &input, default)?;
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
        let retained = body::templates::owns(facts, definition);
        let specialize = default.is_some()
            || retained
            || Self::needs_method_dispatch(facts, &template.body)
                .map_err(|e| report.body_error(source, e))?;
        let purpose = if specialize {
            ProofPurpose::Original
        } else {
            ProofPurpose::Code
        };
        let checked = self.prove_defining_original(report, facts, &template, &input, purpose)?;
        if specialize {
            if retained {
                self.reserve_template(report, facts, definition, &template)?;
            }
            self.specialize(report, facts, instance, &template, &input, default)?;
            if retained {
                self.originals.insert(definition, template);
            } else {
                self.concrete
                    .insert(BodyKey::Declaration(definition), template.body);
            }
        } else {
            let index = self
                .publish_code(facts, checked, Some(instance))
                .map_err(|e| report.body_error(source, e))?;
            self.catalog.functions.insert(instance, index);
            self.concrete
                .insert(BodyKey::Declaration(definition), template.body);
        }
        Ok(true)
    }
    fn lent_function(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        function: crate::sema::model::Function,
        instance: crate::types::instance::Instance,
        input: &Contract,
    ) -> Result<bool, Error> {
        let definition = function.definition;
        let source = function.body.unwrap_or(function.node);
        let inputs = Rc::clone(&self.inputs);
        let stored = inputs.try_borrow().map_err(|_| {
            report.body_error(source, "constant CFG lender is preparing another body")
        })?;
        if let Some(body) = stored.get(&BodyKey::Declaration(definition)) {
            // A lent original still carries abstract source member operands.
            // Resolve them through the existing materialized-instance producer
            // before a Code-purpose proof can authorize actual const entry.
            if Self::needs_method_dispatch(facts, body).map_err(|e| report.body_error(source, e))? {
                return Ok(false);
            }
            body::constants::eligibility_charged(report, facts, body)?;
            let mut needed = crate::types::destruction::NeedsDrop::new(
                facts.destructors.implementations().iter().map(|i| i.item),
            );
            let destruction = body::drops::requirements(facts, body, &mut needed)
                .map_err(|e| report.body_error(source, e))?;
            let parameters = Arc::new(
                facts
                    .source_parameter_environment(report, source, definition)
                    .map_err(|e| report.body_error(source, e))?,
            );
            let checked = self.prove_body(
                report,
                facts,
                body,
                &destruction,
                ProofInput::code(&parameters, input),
            )?;
            let index = self
                .publish_code(facts, checked, Some(instance))
                .map_err(|e| report.body_error(source, e))?;
            self.catalog.functions.insert(instance, index);
            return Ok(true);
        }
        Ok(false)
    }
    fn needs_method_dispatch(facts: &mut Declarations, body: &Body) -> Result<bool, &'static str> {
        if facts.source_bindings.is_none() {
            return Ok(false);
        }
        let mut visits = 0;
        let needed = (|| {
            for (index, value) in body.values.iter().enumerate() {
                visits += 1;
                if body::dispatch::trait_function(facts, value) {
                    return Ok(true);
                }
                if matches!(value.kind, body::ir::Rvalue::Invoke { .. }) {
                    let id = body::ir::ValueId(
                        u32::try_from(index)
                            .map_err(|_| "constant invocation value identity overflow")?,
                    );
                    if let Some((ty, _)) = const_eval::code::closure_subject(
                        &facts.types,
                        &body.values,
                        &body.arguments,
                        id,
                    )? && matches!(facts.types.kind(ty), Some(Kind::Closure { environment, .. }) if environment != crate::types::TypeId::UNIT)
                    {
                        return Ok(true);
                    }
                }
            }
            Ok(false)
        })();
        facts.body_work.canonical_steps = facts.body_work.canonical_steps.saturating_add(visits);
        needed
    }
    fn cached_function(
        &self,
        report: &Report,
        facts: &Declarations,
        instance: crate::types::instance::Instance,
    ) -> Result<bool, Error> {
        if !self.catalog.functions.contains_key(&instance) {
            return Ok(false);
        }
        if let Some(originals) = &self.imported
            && let Some(Kind::FunctionItem { item, .. }) = facts.types.kind(instance.ty())
            && let Some(row) = originals.get(DefId(item.0))
        {
            let header = facts
                .function_header(DefId(item.0))
                .map_err(|e| report.body_optional_error(None, e))?
                .ok_or_else(|| {
                    report.body_optional_error(
                        None,
                        "cached imported Code lost its exact declaration",
                    )
                })?;
            if header.body != Some(row.template().body.source)
                || row.contract().signature != Some(header.signature)
                || header
                    .environment
                    .is_none_or(|env| !Arc::ptr_eq(env, &row.template().environment))
            {
                return Err(report.body_error(
                    row.template().body.source,
                    "cached imported Code changed its exact original lineage",
                ));
            }
        }
        Ok(true)
    }
    fn ensure_imported_originals(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
    ) -> Result<(), Error> {
        if self.imported.is_some() {
            return Ok(());
        }
        let Some(originals) =
            body::transported_originals::build(report, facts, &mut self.promotions)?
        else {
            return Ok(());
        };
        // All imported originals, including unused rows, prove before this private
        // provider retains the set or any selected imported Code can be published.
        for row in originals.iter() {
            crate::borrow::storage::validate(&row.template().body)
                .map_err(|e| report.body_error(e.node, e.message))?;
            if !facts.uses_source_copy() {
                super::original_contexts::Original::validate_parts_charged(
                    facts,
                    &row.template().body,
                    row.contract(),
                )
                .map_err(|e| report.body_error(row.template().body.source, e))?;
            }
            self.prove(
                report,
                facts,
                row.template(),
                row.contract(),
                ProofPurpose::Original,
            )?;
        }
        let retained = facts
            .constant_work
            .template_bytes
            .saturating_add(originals.retained_bytes());
        if retained > 64 * 1024 * 1024 {
            return Err(report
                .body_optional_error(None, "imported templates exceed combined retained budget"));
        }
        facts.constant_work.template_bytes = retained;
        self.imported = Some(originals);
        Ok(())
    }
    fn imported_function(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        instance: crate::types::instance::Instance,
        definition: DefId,
    ) -> Result<bool, Error> {
        let Some(header) = facts
            .declaration_header(definition)
            .map_err(|e| report.body_optional_error(None, e))?
        else {
            return Ok(false);
        };
        if header.declaration.source().is_some() {
            return Ok(false);
        }
        if !facts.is_const_function(definition) {
            return Err(report.body_optional_error(None, "imported function is not declared const"));
        }
        // Identifying an imported declaration needs no defining environment.
        // Its actual body producer constructs and proves that environment here.
        self.ensure_imported_originals(report, facts)?;
        let originals = self.imported.as_ref().map(Rc::clone).ok_or_else(|| {
            report.body_optional_error(None, "imported function lacks its proved original set")
        })?;
        let row = originals.get(definition).ok_or_else(|| {
            report.body_optional_error(None, "imported function has no exact original row")
        })?;
        super::original_contexts::Original::validate_parts_charged(
            facts,
            &row.template().body,
            row.contract(),
        )
        .map_err(|e| report.body_error(row.template().body.source, e))?;
        self.specialize(
            report,
            facts,
            instance,
            row.template(),
            row.contract(),
            None,
        )?;
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
            return Err(report.body_error(
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
        default: Option<&crate::sema::function_context::DefaultExecution>,
    ) -> Result<(), Error> {
        let source = template.body.source;
        if template.retained_bytes() > 64 * 1024 * 1024 {
            return Err(report.body_error(source, "constant instance copy exceeds 64 MiB budget"));
        }
        body::assembly::prepay_instance(report, &template.body)?;
        let materialized = body::instantiate::with_contract(facts, template, instance, input)
            .map_err(|e| report.body_error(source, e))?;
        self.finish_specialization(
            report,
            facts,
            instance,
            materialized,
            &template.environment,
            default,
        )
    }
    fn finish_specialization(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        instance: crate::types::instance::Instance,
        mut materialized: body::instantiate::Materialized,
        environment: &Arc<Environment>,
        default: Option<&crate::sema::function_context::DefaultExecution>,
    ) -> Result<(), Error> {
        let source = materialized.body.source;
        if let Some(default) = default {
            body::constants::prepare_default_execution(
                report,
                facts,
                default,
                &mut materialized.body,
            )
            .map_err(|e| report.body_error(source, e))?;
        }
        materialized
            .resolve(facts)
            .map_err(|e| report.body_error(source, e))?;
        if default.is_some() {
            body::constants::eligibility_charged(report, facts, &materialized.body)?;
        }
        body::promote::instance_reusing(
            report,
            facts,
            &mut materialized.body,
            &materialized.destruction,
            &mut self.promotions,
        )?;
        let checked = self.prove_body(
            report,
            facts,
            &materialized.body,
            &materialized.destruction,
            ProofInput::materialized(environment, &materialized),
        )?;
        let index = self
            .publish_code(facts, checked, Some(instance))
            .map_err(|e| report.body_error(source, e))?;
        self.catalog.functions.insert(instance, index);
        Ok(())
    }
    fn prove(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        template: &Template,
        contract: &Contract,
        purpose: ProofPurpose,
    ) -> Result<CheckedInput, Error> {
        self.prove_body(
            report,
            facts,
            &template.body,
            &template.destruction,
            ProofInput {
                parameters: &template.environment,
                contract,
                purpose,
                materialized: None,
            },
        )
    }
    fn prove_body(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        body: &Body,
        destruction: &body::drops::Requirements,
        input: ProofInput<'_>,
    ) -> Result<CheckedInput, Error> {
        let imported_original =
            facts.uses_source_copy() && body.instance.is_none() && body.source.source().is_none();
        let mut prepared = if imported_original {
            Some(
                super::original_contexts::Original::prepare_imported(facts, body, input.contract)
                    .map_err(|e| report.body_error(body.source, e))?,
            )
        } else if facts.source_bindings.is_some()
            || input.contract.requires_defining_cleanup_preparation()
            || body::defining_cleanups::needs_preparation(facts, body)
        {
            Some(input.contract.clone())
        } else {
            None
        };
        if !imported_original && let Some(contract) = prepared.as_mut() {
            let result = if let Some(materialized) = input.materialized {
                materialized.prepare_source_copy(facts, body, contract)
            } else {
                body::copy_requirements::prepare(facts, body, contract)
            };
            result.map_err(|e| report.body_error(body.source, e))?;
        }
        let mut contract = prepared.take().map_or(
            std::borrow::Cow::Borrowed(input.contract),
            std::borrow::Cow::Owned,
        );
        let mut regions = Self::source_regions(report, facts, body, &contract)?;
        let checked = self.prove_loans(
            report,
            facts,
            body,
            destruction,
            &mut LoanInput {
                parameters: input.parameters,
                contract: &mut contract,
                regions: &mut regions,
                purpose: input.purpose,
            },
        )?;
        let contract = contract.as_ref();
        #[cfg(test)]
        if input.purpose == ProofPurpose::Code {
            materialized_bodies::record(
                body,
                input
                    .materialized
                    .is_some_and(body::instantiate::Materialized::has_instance_copy_scope_for_test),
            )
            .map_err(|e| report.body_error(body.source, e))?;
        }
        let (code, uses) = self
            .finish_code(facts, body, contract, &regions, input.purpose, checked)
            .map_err(|e| report.body_error(body.source, e))?;
        let retained = if uses.needed {
            Some(
                self.retained_source(code.as_ref(), Arc::new(contract.clone()), uses)
                    .map_err(|e| report.body_error(body.source, e))?,
            )
        } else {
            None
        };
        Ok(CheckedInput { retained, code })
    }

    fn source_regions(
        report: &Report,
        facts: &mut Declarations,
        body: &Body,
        contract: &Contract,
    ) -> Result<borrow::regions::Body, Error> {
        let source = body.source;
        contract
            .prepare_outlives(&mut facts.types, &mut facts.lifetimes.nominal_requirements)
            .map_err(|e| report.body_error(source, e))?;
        let mut queries = crate::types::outlives::Queries::new(
            &facts.types,
            &facts.lifetimes.nominal_requirements,
            &facts.lifetimes.callables,
        );
        borrow::regions::check_one(
            &facts.types,
            &facts.lifetimes.variances,
            &mut queries,
            contract,
            body,
        )
        .map_err(|e| report.body_error(e.source.unwrap_or(source.into()), e.message))
    }
    fn prove_loans<'body>(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        body: &'body Body,
        destruction: &body::drops::Requirements,
        input: &mut LoanInput<'_, '_>,
    ) -> Result<super::cdrop::Checked<'body>, Error> {
        let source = body.source;
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
                    .map_err(|e| report.body_error(source, e))?,
            ));
        }
        let (_, loans) = self
            .loans
            .as_mut()
            .ok_or_else(|| report.body_error(source, "missing constant loan verifier"))?;
        super::cdrop::one(report, facts, body, destruction, input, loans)
            .map_err(|e| report.body_error(source, e))
    }
}
