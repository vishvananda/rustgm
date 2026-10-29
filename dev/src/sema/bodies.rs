//! Scalar body constraints consume resolved identities; fallback occurs once per body.
mod aggregates;
mod alignment;
mod annotations;
mod api_stability;
mod atomic_ordering;
mod bounds;
mod callee_inputs;
mod calls;
mod closures;
mod comparisons;
mod conditions;
mod declared_callees;
mod defining_bodies;
mod dependency_roots;
mod discarded_results;
mod divergence;
mod dot_calls;
mod effect_facts;
mod target_permissions;
mod tracked_readiness;
pub(super) use effect_facts::CallContexts;
pub use effect_facts::{OffsetPlanId, SourceCallUse, TryPlanId};
mod assembly_values;
mod environment_values;
mod field_storage;
mod finalization;
mod formatting_values;
mod generated_bounds;
mod generics;
mod hidden_results;
mod inline;
mod invocations;
mod item_values;
mod iteration;
mod language_hooks;
mod lifetimes;
mod matches;
mod methods;
mod nested;
mod nominals;
mod notifications;
mod owner_arguments;
mod pattern_constants;
mod variadics;
pub(super) use pattern_constants::PreparedScope;
mod patterns;
mod projections;
mod readiness;
pub(super) use readiness::{Dependency, Requirement};
mod addresses;
mod displacement_inputs;
mod intrinsic_items;
mod pointer_casts;
mod pointers;
mod privacy;
mod range_values;
mod references;
mod safety;
mod selected_arguments;
mod slots;
mod text;
mod trait_goals;
mod try_control;
mod unsizing;
use super::{
    declarations,
    model::{AdjustmentKind, Declarations, Function, Operation, RegionEquality},
    scalar,
};
use crate::support::hash::HashSet;
use crate::{
    lex::token::Punctuation as Op,
    literal::{Value, value::FloatType},
    resolve::{
        expansion::Report,
        model::{DefId, DefKind, Target},
        package::{Error, ModuleId},
        unit::{FileId, NodeKey},
    },
    syntax::ast::{BlockFlavor, Kind, List, NodeId, Unary},
    types::{
        Kind as Type, Primitive, TypeId,
        infer::{Constructor, Context, Domain, TermId},
    },
};

/// Check function bodies and remaining initializers using completed declaration facts.
///
/// # Errors
/// Reports type mismatches, unresolved variables and explicitly unsupported body semantics.
pub fn analyze(report: &Report, facts: Declarations) -> Result<Declarations, Error> {
    analyze_with_hidden(report, facts, |report, facts, id| {
        let declaration = facts
            .opaques
            .declarations
            .get(&id)
            .ok_or_else(|| Error::at(FileId(0), 0, "missing opaque declaration"))?;
        Err(Declarations::error(
            report,
            declaration.source,
            "opaque hidden lifetimes require the defining body's region proof",
        ))
    })
}

/// Complete a demanded defining body before resuming its dependent auto proof.
/// The callback closes hidden lifetimes from that body's verified typed CFG.
/// # Errors
/// Reports ordinary body errors, dependency cycles and failed hidden-type proofs.
pub fn analyze_with_hidden(
    report: &Report,
    mut facts: Declarations,
    mut close: impl FnMut(
        &Report,
        &mut Declarations,
        crate::types::opaque::OpaqueId,
    ) -> Result<(), Error>,
) -> Result<Declarations, Error> {
    facts.require_lint_context(report)?;
    super::compiler_items::pointee::authenticate(report, &facts)
        .map_err(|error| facts.retain_error_warnings(report.diagnostic(error)))?;
    super::compiler_items::discriminant::authenticate(report, &facts)
        .map_err(|error| facts.retain_error_warnings(report.diagnostic(error)))?;
    let mut checker = Checker::new(report, &mut facts);
    checker.functions(&mut close).map_err(|error| {
        checker
            .facts
            .retain_error_warnings(report.diagnostic(error))
    })?;
    for index in 0..checker.facts.initializers.len() {
        if !checker.facts.initializers[index].type_checked() {
            checker.initializer(index).map_err(|error| {
                checker
                    .facts
                    .retain_error_warnings(report.diagnostic(error))
            })?;
        }
    }
    checker.complete_constant_patterns().map_err(|error| {
        checker
            .facts
            .retain_error_warnings(report.diagnostic(error))
    })?;
    super::compiler_items::pointee::authenticate(report, &facts)
        .map_err(|error| facts.retain_error_warnings(report.diagnostic(error)))?;
    super::compiler_items::discriminant::authenticate(report, &facts)
        .map_err(|error| facts.retain_error_warnings(report.diagnostic(error)))?;
    Ok(facts)
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum FrameKind {
    Try,
    Loop,
    While,
    For,
    Block,
}
#[derive(Clone, Copy)]
struct Frame {
    node: NodeId,
    body: NodeId,
    kind: FrameKind,
    label: Option<DefId>,
    result: TermId,
    breaks: Option<usize>,
    expected: Option<TermId>,
    in_condition: bool,
}
struct BreakSite {
    node: NodeId,
    value: Option<NodeId>,
    next: Option<usize>,
}
struct PendingAdjustment {
    node: NodeId,
    actual: TermId,
    expected: TermId,
    kind: AdjustmentKind,
    reset: bool,
}
enum Work {
    BodyInputs(Box<nested::InputRefresh>),
    Conditional {
        node: NodeId,
        pattern: NodeId,
        value: NodeId,
        expected: Option<TermId>,
    },
    MatchArms {
        node: NodeId,
        expected: Option<TermId>,
    },
    Arm {
        index: u32,
        term: TermId,
        expected: Option<TermId>,
    },
    BindPattern {
        pattern: NodeId,
        term: TermId,
        value: Option<NodeId>,
    },
    Enter(NodeId, Option<TermId>),
    Exit(NodeId),
    CompleteExpression(NodeId, TermId),
    Field(NodeId, aggregates::Field),
    CoerceElement(NodeId, TermId),
    PrepareBorrowOperand {
        node: NodeId,
        operand: NodeId,
        expected: TermId,
    },
    CallArguments(NodeId, Option<TermId>),
    PrepareCallArgument {
        call: NodeId,
        argument: NodeId,
        parameter: u32,
    },
    CoerceCallArgument {
        call: NodeId,
        argument: NodeId,
        parameter: u32,
    },
    MethodArguments(NodeId, Option<Box<std::vec::IntoIter<DefId>>>),
    AssignmentValue(NodeId),
    ForIterator(NodeId),
    ForPattern(NodeId, TermId),
    CollectClosure,
    CollectInline,
}
/// Cleared storage can be reused between short declaration-time constants.
/// Successful checks clear touched slots; caller evidence is cleared before reuse.
#[derive(Default)]
pub(super) struct Scratch {
    terms: slots::Terms,
    diverges: slots::Slots<bool>,
    locals: slots::Slots<Option<TermId>>,
    touched: Vec<NodeId>,
    work: Vec<Work>,
    checks: Vec<NodeId>,
    call_types: Option<Box<calls::CallTypes>>,
}
pub(super) struct Checker<'a> {
    pattern_queries: Option<Box<super::constants::Scratch>>,
    pub(super) availability: super::traits::implementations::Availability,
    report: &'a Report,
    pub(super) facts: &'a mut Declarations,
    state: State,
}
pub(super) struct State {
    closures: Option<Box<closures::Scheduler>>,
    closure_children: Vec<usize>,
    closure_cursor: usize,
    steps: usize,
    completion: readiness::Completion,
    alignment: alignment::Pending,
    file: FileId,
    body: NodeKey,
    module: ModuleId,
    field_seen: Vec<bool>,
    result: TypeId,
    result_term: Option<TermId>,
    opaque_terms: Vec<(crate::types::opaque::OpaqueId, TermId)>,
    opaque_callables: Option<Box<crate::support::hash::HashMap<TermId, TypeId>>>,
    in_function: bool,
    safety: safety::Context,
    infer: Context,
    terms: slots::Terms,
    diverges: slots::Slots<bool>,
    locals: slots::Slots<Option<TermId>>,
    touched: Vec<NodeId>,
    bindings: Vec<DefId>,
    pattern_work: Vec<patterns::Work>,
    pattern_types: Vec<(super::patterns::PatternId, TermId)>,
    pattern_refutable: bool,
    pending_patterns: Option<Box<pattern_constants::Pending>>,
    pattern_constants: bool,
    coverage_cache: super::patterns::coverage::Cache,
    range_work: Vec<matches::RangeWork>,
    pattern_requirements: Vec<(super::patterns::PatternId, bool)>,
    matches: Vec<u32>,
    work: Vec<Work>,
    children: Vec<NodeId>,
    checks: Vec<NodeId>,
    negated: HashSet<NodeId>,
    adjustments: Vec<PendingAdjustment>,
    reference_adjustments:
        crate::support::hash::HashMap<usize, (super::model::ReferenceConversion, TermId)>,
    unsizing_adjustments: crate::support::hash::HashMap<usize, unsizing::Pending>,
    pointer_adjustments: crate::support::hash::HashMap<usize, pointers::Pending>,
    callable_adjustments: crate::support::hash::HashMap<usize, TermId>,
    pointer_casts: Vec<NodeId>,
    reference_expected: HashSet<NodeId>,
    address_dereferences: HashSet<NodeId>,
    raw_address_places: HashSet<NodeId>,
    union_write_places: HashSet<NodeId>,
    language_places: Option<Box<language_hooks::Places>>,
    unsizing_contexts: crate::support::hash::HashMap<NodeId, TermId>,
    reference_work: Vec<references::CoercionWork>,
    reference_steps: usize,
    break_roots: crate::support::hash::HashMap<NodeId, usize>,
    discarded_adjustments: crate::support::hash::HashMap<usize, (TermId, TermId)>,
    calls: Vec<(NodeId, TermId)>,
    pending_indexes: crate::support::hash::HashMap<NodeId, TermId>,
    call_types: Box<calls::CallTypes>,
    parameter_copies: Option<crate::types::copy::Copies>,
    parameter_shapes: Option<crate::types::parameters::shape::Shapes>,
    nominal_shapes: Option<Box<super::nominals::Proof>>,
    body_binders: u32,
    environment_owner: Option<DefId>,
    method_constants: Option<Box<super::arguments::Cache>>,
    method_environment: Option<std::sync::Arc<crate::types::parameters::environment::Environment>>,
    trait_environment: Option<std::sync::Arc<super::traits::assumptions::Assumptions>>,
    canonical_trait_inference: Option<Box<Context>>,
    canonical_trait_proofs: Option<Box<super::traits::selection::Reuse>>,
    canonical_requirements: Option<Box<trait_goals::CanonicalRequirements>>,
    annotations: super::annotations::Scratch,
    projections: projections::Pending,
    notifications: notifications::Notifications,
    storage: Option<Box<super::stored_fields::Preparation>>,
    finalization: Option<Box<finalization::Finalization>>,
    trait_goals: Option<Box<trait_goals::Pending>>,
    type_children: Vec<TermId>,
    annotation_checks: Vec<NodeId>,
    annotation_children: HashSet<NodeId>,
    type_annotations: HashSet<NodeId>,
    frames: Vec<Frame>,
    pending_label: Option<(NodeId, DefId)>,
    assignments: Vec<(NodeId, Option<DefId>, Option<Op>)>,
    breaks: Vec<BreakSite>,
}
impl Scratch {
    /// Start from cleared reusable slots; suspended bodies keep their own sparse state.
    pub(super) fn resumable(&mut self) {
        if !matches!(self.terms, slots::Terms::Sparse(_)) {
            self.terms = slots::Terms::sparse();
            self.diverges = slots::Slots::sparse();
            self.locals = slots::Slots::sparse();
        }
    }
}
impl<'a> Checker<'a> {
    pub(super) fn new(report: &'a Report, facts: &'a mut Declarations) -> Self {
        Self::with_scratch(report, facts, Scratch::default())
    }
    pub(super) fn with_scratch(
        report: &'a Report,
        facts: &'a mut Declarations,
        mut scratch: Scratch,
    ) -> Self {
        scratch.terms.files(facts.nodes.len());
        Self {
            availability: super::traits::implementations::Availability::Validated,
            report,
            facts,
            pattern_queries: None,
            state: State {
                closures: None,
                closure_children: Vec::new(),
                closure_cursor: 0,
                steps: 0,
                completion: readiness::Completion::Inactive,
                alignment: alignment::Pending::default(),
                terms: scratch.terms,
                diverges: scratch.diverges,
                locals: scratch.locals,
                file: FileId(0),
                body: NodeKey {
                    file: FileId(0),
                    node: NodeId::new(0),
                },
                module: ModuleId(0),
                field_seen: Vec::new(),
                result: TypeId::UNIT,
                result_term: None,
                opaque_terms: Vec::new(),
                opaque_callables: None,
                in_function: true,
                safety: safety::Context::default(),
                infer: Context::default(),
                touched: scratch.touched,
                bindings: Vec::new(),
                pattern_work: Vec::new(),
                pattern_types: Vec::new(),
                pattern_refutable: false,
                pending_patterns: None,
                pattern_constants: false,
                coverage_cache: super::patterns::coverage::Cache::default(),
                range_work: Vec::new(),
                pattern_requirements: Vec::new(),
                matches: Vec::new(),
                work: scratch.work,
                children: Vec::new(),
                checks: scratch.checks,
                negated: HashSet::default(),
                adjustments: Vec::new(),
                reference_adjustments: crate::support::hash::HashMap::default(),
                unsizing_adjustments: crate::support::hash::HashMap::default(),
                pointer_adjustments: crate::support::hash::HashMap::default(),
                callable_adjustments: crate::support::hash::HashMap::default(),
                pointer_casts: Vec::new(),
                reference_expected: HashSet::default(),
                address_dereferences: HashSet::default(),
                raw_address_places: HashSet::default(),
                union_write_places: HashSet::default(),
                language_places: None,
                unsizing_contexts: crate::support::hash::HashMap::default(),
                reference_work: Vec::new(),
                reference_steps: 0,
                break_roots: crate::support::hash::HashMap::default(),
                discarded_adjustments: crate::support::hash::HashMap::default(),
                calls: Vec::new(),
                pending_indexes: crate::support::hash::HashMap::default(),
                call_types: scratch.call_types.unwrap_or_default(),
                parameter_copies: None,
                parameter_shapes: None,
                nominal_shapes: None,
                body_binders: 0,
                method_constants: None,
                method_environment: None,
                environment_owner: None,
                trait_environment: None,
                canonical_trait_inference: None,
                canonical_trait_proofs: None,
                canonical_requirements: None,
                annotations: super::annotations::Scratch::default(),
                projections: projections::Pending::default(),
                notifications: notifications::Notifications::default(),
                storage: None,
                finalization: None,
                trait_goals: None,
                type_children: Vec::new(),
                annotation_checks: Vec::new(),
                annotation_children: HashSet::default(),
                type_annotations: HashSet::default(),
                frames: Vec::new(),
                pending_label: None,
                assignments: Vec::new(),
                breaks: Vec::new(),
            },
        }
    }
    pub(super) fn has_pending_patterns(&self) -> bool {
        self.state.pending_patterns.is_some()
    }
    /// Consume only after successful checking; failed checks discard their slots.
    pub(super) fn into_scratch(mut self) -> Scratch {
        self.state.call_types.clear();
        Scratch {
            terms: self.state.terms,
            diverges: self.state.diverges,
            locals: self.state.locals,
            touched: self.state.touched,
            work: self.state.work,
            checks: self.state.checks,
            call_types: Some(self.state.call_types),
        }
    }
    fn prepare_constant(&mut self, initializer: super::model::Initializer) -> Result<(), Error> {
        let key = initializer.semantic_source();
        // Annotation regions were numbered for the syntax root; the semantic
        // owner continues in that unit.
        self.facts
            .body_regions
            .nest(key, initializer.value)
            .map_err(|e| Declarations::error(self.report, key, e))?;
        let ty = initializer.ty;
        let owner = initializer.definition;
        let scope = self.report.resolution.definitions[owner.0 as usize].scope;
        self.begin_constant(
            key,
            ty,
            self.report.resolution.scopes[scope.0 as usize].module,
        );
        self.state.body_binders = self
            .facts
            .associated
            .schema(owner)
            .map_or(0, |s| s.captures);
        self.prepare_bound_environment(key.node, owner, None)
    }
    fn begin_constant(&mut self, key: NodeKey, ty: TypeId, module: ModuleId) {
        self.state.call_types.clear();
        self.state.projections = projections::Pending::default();
        self.state.notifications = notifications::Notifications::default();
        self.state.storage = None;
        self.state.finalization = None;
        self.state.trait_goals = None;
        self.state.body_binders = 0;
        self.state.method_environment = None;
        self.state.canonical_trait_proofs = None;
        self.state.canonical_requirements = None;
        self.state.environment_owner = None;
        self.state.trait_environment = None;
        self.state.canonical_trait_inference = None;
        self.state.parameter_copies = None;
        self.state.parameter_shapes = None;
        self.state.body = key;
        self.facts.body_regions.activate(key);
        self.prepare_file(key.file);
        self.state.module = module;
        self.state.result = ty;
        self.state.result_term = None;
        self.state.opaque_terms.clear();
        self.state.opaque_callables = None;
        self.state.in_function = false;
        self.state.safety = safety::Context::default();
    }
    pub(super) fn anonymous_constant(
        &mut self,
        key: NodeKey,
        ty: TypeId,
        module: ModuleId,
    ) -> Result<(), Error> {
        let progress = self.start_anonymous_constant(key, ty, module)?;
        self.require_complete_body(progress)
    }
    pub(super) fn initializer(&mut self, index: usize) -> Result<(), Error> {
        let progress = self.start_initializer(index)?;
        self.require_complete_body(progress)
    }
}
impl Checker<'_> {
    fn prepare_file(&mut self, file: FileId) {
        self.state.locals.resize(self.facts.definitions.len(), None);
        self.state.file = file;
        self.state
            .terms
            .prepare(file, self.facts.nodes[file.0 as usize].len());
        self.state
            .diverges
            .resize(self.facts.nodes[file.0 as usize].len(), false);
    }
    fn key(&self, node: NodeId) -> NodeKey {
        NodeKey {
            file: self.state.file,
            node,
        }
    }
    fn kind(&self, node: NodeId) -> Kind {
        Declarations::kind(self.report, self.key(node))
    }
    fn error(&self, node: NodeId, message: impl std::fmt::Display) -> Error {
        Declarations::error(self.report, self.key(node), message.to_string())
    }
    fn known(&mut self, node: NodeId, ty: TypeId) -> Result<TermId, Error> {
        if self.facts.types.contains_projections(ty) == Some(true) {
            return self.known_projection(node, ty);
        }
        self.state
            .infer
            .known(&self.facts.types, ty)
            .map_err(|e| self.error(node, e))
    }
    fn variable(&mut self, node: NodeId, domain: Domain) -> Result<TermId, Error> {
        self.state
            .infer
            .variable(domain)
            .map_err(|e| self.error(node, e))
    }
    fn term(&self, node: NodeId) -> Result<TermId, Error> {
        self.state
            .terms
            .get(self.key(node))
            .ok_or_else(|| self.error(node, "missing body type constraint"))
    }
    fn save(&mut self, node: NodeId, term: TermId) {
        if self.state.terms.insert(self.key(node), term).is_none() {
            self.state.touched.push(node);
        }
    }
    fn equate(&mut self, node: NodeId, left: TermId, right: TermId) -> Result<(), Error> {
        if self.state.projections.active() {
            return self.deferred_equation(
                node,
                left,
                right,
                crate::types::infer::deferred::Relation::Equality,
            );
        }
        self.state
            .infer
            .equate(&self.facts.types, left, right)
            .map_err(|e| self.error(node, e))
    }
    fn shape_equation(&mut self, node: NodeId, left: TermId, right: TermId) -> Result<(), Error> {
        // Generic holes also occur beneath callable binders. Once an argument
        // solves such a hole, caller universals must remain free there rather
        // than becoming lifetimes owned by the nested callable.
        // Opening a fully determined projected callable also introduces aliases.
        // Its projected arguments must use the same identities even when no
        // generic call or unresolved projection triggered their preparation.
        let (left, right) = if self.state.body_binders != 0
            && (self.state.call_types.opened_projection_call
                || !self.state.call_types.generic.is_empty()
                || !self.state.call_types.constants.is_empty()
                || self.state.call_types.generic_constructors
                || self.state.projections.has_context())
        {
            (
                self.alias_argument(node, left)?,
                self.alias_argument(node, right)?,
            )
        } else {
            (left, right)
        };
        if self.state.projections.active() {
            return self.deferred_equation(
                node,
                left,
                right,
                crate::types::infer::deferred::Relation::Shape,
            );
        }
        self.state
            .infer
            .equate_shape(&self.facts.types, left, right)
            .map_err(|e| self.error(node, e))
    }
    fn head(&mut self, node: NodeId, term: TermId) -> Result<Option<Constructor>, Error> {
        self.advance_projections(node)?;
        self.state
            .infer
            .head(&self.facts.types, term)
            .map_err(|e| self.error(node, e))
    }
    fn contextual_head(
        &mut self,
        node: NodeId,
        term: TermId,
    ) -> Result<Option<Constructor>, Error> {
        self.advance_projections(node)?;
        match self.state.infer.head(&self.facts.types, term) {
            Err(crate::types::infer::Error::Normalization) => Ok(None),
            result => result.map_err(|e| self.error(node, e)),
        }
    }
    fn child(&mut self, node: NodeId, term: TermId, index: usize) -> Result<TermId, Error> {
        self.state
            .infer
            .child(&self.facts.types, term, index)
            .map_err(|e| self.error(node, e))
    }
    fn active(&self, node: NodeId) -> bool {
        self.report.package.unit(self.state.file).configured.active[node.get() as usize]
    }
    fn function(&mut self, function: Function) -> Result<(), Error> {
        if function.body.is_none()
            && self
                .facts
                .foreign
                .as_deref()
                .and_then(|r| r.function(function.definition))
                .is_some_and(|f| f.linkage == crate::support::linkage::Linkage::Import)
        {
            return Ok(());
        }
        if self
            .facts
            .typed_function_bodies
            .contains(&function.definition)
        {
            return Ok(());
        }
        let progress = self.start_function(function)?;
        self.require_complete_body(progress)?;
        self.facts.typed_function_bodies.insert(function.definition);
        Ok(())
    }
    fn start_function(&mut self, function: Function) -> Result<Option<Dependency>, Error> {
        self.state.call_types.clear();
        self.state.projections = projections::Pending::default();
        self.state.notifications = notifications::Notifications::default();
        self.state.storage = None;
        self.state.finalization = None;
        self.state.trait_goals = None;
        self.state.body_binders = self
            .facts
            .lifetimes
            .functions
            .get(&function.definition)
            .map_or(0, |f| f.total);
        self.state.in_function = true;
        let Some(body) = function.body else {
            if self.source_intrinsic(function.node.node, function.definition)? {
                return Ok(None);
            }
            if self
                .facts
                .associated
                .trait_owner(function.definition)
                .is_some()
                || self
                    .facts
                    .intrinsics
                    .identify(function.definition)
                    .is_some()
            {
                return Ok(None);
            }
            return Err(Declarations::error(
                self.report,
                function.node,
                "function body is required",
            ));
        };
        self.prepare_file(body.file);
        self.state.body = body;
        self.facts.body_regions.activate(body);
        self.prepare_bound_environment(body.node, function.definition, Some(function.signature))?;
        let scope = self.report.resolution.definitions[function.definition.0 as usize].scope;
        self.state.module = self.report.resolution.scopes[scope.0 as usize].module;
        let Some(Type::Function {
            result, unsafe_, ..
        }) = self.facts.types.kind(function.signature)
        else {
            return Err(self.error(body.node, "missing function signature"));
        };
        self.state.safety = safety::Context::default();
        self.state.safety.function = super::target_features::body_unsafe(
            self.report,
            self.facts,
            function.definition,
            unsafe_,
        )?;
        self.state.result = result;
        self.prepare_opaque_results(body.node)?;
        let unit = self.report.package.unit(self.state.file);
        for mut node in unit
            .configured
            .ast
            .list(function.parameters)
            .iter()
            .copied()
        {
            if !self.active(node) {
                continue;
            }
            while let Kind::Attributed { value, .. } = self.kind(node) {
                node = value;
            }
            if let Some(receiver) = self.facts.receivers.get(&self.key(node)).copied() {
                let term = self.known(node, receiver.ty)?;
                self.state.locals[receiver.definition.0 as usize] = Some(term);
                self.state.bindings.push(receiver.definition);
                self.save(receiver.binding.node, term);
                self.save(node, term);
                continue;
            }
            let Kind::Parameter {
                pattern,
                ty: Some(ty),
            } = self.kind(node)
            else {
                return Err(self.error(node, "unsupported parameter pattern"));
            };
            let ty = self
                .facts
                .node(self.key(ty))
                .and_then(|f| f.ty)
                .ok_or_else(|| self.error(node, "missing parameter annotation"))?;
            let term = self.known(node, ty)?;
            self.binding(pattern, term, None)?;
            self.save(node, term);
        }
        self.state.completion = readiness::Completion::Ordinary;
        self.start_body(body.node)?;
        self.advance_body()
    }
    pub(super) fn advance_body(&mut self) -> Result<Option<Dependency>, Error> {
        loop {
            if let Some(dependency) = self.advance_current_body()? {
                let owner = match dependency.requirement {
                    Requirement::ImplHeader(owner) | Requirement::Definition(owner) => Some(owner),
                    _ => None,
                };
                if matches!(
                    self.availability,
                    super::traits::implementations::Availability::Validated
                ) && let Some(owner) = owner
                    && super::declarations::iteration::range_contracts::owns(self.report, owner)
                {
                    super::declarations::iteration::range_contracts::prepare(
                        self.report,
                        self.facts,
                        owner,
                        true,
                    )?;
                    continue;
                }
                if matches!(
                    self.availability,
                    super::traits::implementations::Availability::Validated
                ) && let Some(owner) = owner
                    && super::declarations::cloning::owns(self.report, owner)
                {
                    super::declarations::cloning::prepare(self.report, self.facts, owner, true)?;
                    continue;
                }
                if matches!(
                    self.availability,
                    super::traits::implementations::Availability::Validated
                ) && let Some(owner) = owner
                    && super::declarations::iteration::collections::owns(self.report, owner)
                {
                    super::declarations::iteration::collections::prepare(
                        self.report,
                        self.facts,
                        owner,
                        true,
                    )?;
                    continue;
                }
                return Ok(Some(dependency));
            }
            if !self.continue_nested_bodies()? {
                return Ok(None);
            }
        }
    }
    fn advance_current_body(&mut self) -> Result<Option<Dependency>, Error> {
        let body = self.state.body.node;
        if let Some(dependency) = self.resume_body_inputs()? {
            return Ok(Some(dependency));
        }
        if let Some(finalization) = self.state.finalization.take() {
            return self.resume_finalization(*finalization);
        }
        if self.state.trait_goals.is_some() {
            return self.finish(body);
        }
        loop {
            let body = self.state.body.node;
            if matches!(
                self.availability,
                super::traits::implementations::Availability::Declarations
            ) && let projections::Advance::Definition(dependency) =
                self.poll_projections(body)?
            {
                return Ok(Some(dependency));
            }
            let Some(work) = self.state.work.pop() else {
                break;
            };
            self.state.steps += 1;
            if self.state.steps > 1_000_000 {
                return Err(self.error(body, "body type work budget exceeded"));
            }
            match work {
                Work::BodyInputs(inputs) => {
                    if let Some(dependency) = self.finish_body_inputs(*inputs)? {
                        return Ok(Some(dependency));
                    }
                }
                Work::CollectClosure => self.collect_closure()?,
                Work::CollectInline => self.collect_inline()?,
                Work::Conditional {
                    node,
                    pattern,
                    value,
                    expected,
                } => self.conditional_pattern(node, pattern, value, expected)?,
                Work::MatchArms { node, expected } => self.match_arms(node, expected)?,
                Work::Arm {
                    index,
                    term,
                    expected,
                } => self.match_arm(index, term, expected)?,
                Work::BindPattern {
                    pattern,
                    term,
                    value,
                } => {
                    if let Some(value) = value {
                        self.coerce(value, term)?;
                    }
                    self.binding(pattern, term, value)?;
                }
                Work::Enter(node, expected) if self.active(node) => self.enter(node, expected)?,
                Work::Enter(..) => {}
                Work::CoerceElement(node, expected) => self.coerce(node, expected)?,
                work @ (Work::PrepareBorrowOperand { .. }
                | Work::PrepareCallArgument { .. }
                | Work::CoerceCallArgument { .. }) => {
                    if let Some(dependency) = self.resume_operand_work(work)? {
                        return Ok(Some(dependency));
                    }
                }
                Work::CallArguments(node, expected) => self.call_arguments(node, expected)?,
                Work::MethodArguments(node, headers) => {
                    if let Some(dependency) = self.method_arguments(node, headers)? {
                        return Ok(Some(dependency));
                    }
                }
                Work::ForIterator(node) => {
                    if let Some(dependency) = self.for_iterator_work(node)? {
                        return Ok(Some(dependency));
                    }
                }
                Work::ForPattern(node, item) => self.pattern(node, item, true)?,
                Work::AssignmentValue(node) => {
                    let Kind::Binary { left, right, .. } = self.kind(node) else {
                        return Err(self.error(node, "invalid assignment work"));
                    };
                    let expected = self.term(left)?;
                    self.state.work.push(Work::Enter(right, Some(expected)));
                }
                Work::Exit(node) => {
                    if let Some(dependency) = self.complete_body_expression(node)? {
                        return Ok(Some(dependency));
                    }
                }
                Work::Field(node, field) => {
                    let term = self.complete_field(node, field)?;
                    self.schedule_expression_completion(node, term)?;
                }
                Work::CompleteExpression(node, term) => self.exit_expression(node, term)?,
            }
        }
        self.finish(self.state.body.node)
    }
    /// Declaration waits retain the exact operand continuation, before evaluation.
    fn resume_operand_work(&mut self, work: Work) -> Result<Option<Dependency>, Error> {
        let dependency = match &work {
            Work::PrepareBorrowOperand {
                node,
                operand,
                expected,
            } => self.prepare_borrow_operand(*node, *operand, *expected)?,
            Work::PrepareCallArgument {
                call,
                argument,
                parameter,
            } => self.prepare_call_argument(*call, *argument, *parameter)?,
            Work::CoerceCallArgument {
                call,
                argument,
                parameter,
            } => self.coerce_call_argument(*call, *argument, *parameter)?,
            _ => return Err(self.error(self.state.body.node, "invalid operand continuation")),
        };
        if dependency.is_some() {
            self.state.work.push(work);
        }
        Ok(dependency)
    }
    fn complete_body_expression(&mut self, node: NodeId) -> Result<Option<Dependency>, Error> {
        if let Some(requirement) = self
            .formatting_readiness(node)?
            .or(self.offset_readiness(node)?)
            .or(self.comparison_readiness(node)?)
            .or(self.assignment_readiness(node)?)
            .or(self.binary_default_readiness(node)?)
            .or_else(|| self.value_readiness(node))
        {
            self.state.work.push(Work::Exit(node));
            return Ok(Some(Dependency {
                requirement,
                source: self.key(node),
            }));
        }
        let term = if let Kind::Field { value, name } = self.kind(node) {
            let field = self.prepare_field(node, value, name)?;
            if let Some(item) = self.field_dependency(&field) {
                self.state.work.push(Work::Field(node, field));
                return Ok(Some(Dependency {
                    requirement: Requirement::Nominal(item),
                    source: self.key(node),
                }));
            }
            self.complete_field(node, field)?
        } else {
            self.expression(node)?
        };
        self.schedule_expression_completion(node, term)?;
        Ok(None)
    }
    fn schedule_expression_completion(&mut self, node: NodeId, term: TermId) -> Result<(), Error> {
        if matches!(
            self.availability,
            super::traits::implementations::Availability::Declarations
        ) && self.state.projections.active()
        {
            self.state.work.push(Work::CompleteExpression(node, term));
            Ok(())
        } else {
            self.exit_expression(node, term)
        }
    }
    fn exit_expression(&mut self, node: NodeId, term: TermId) -> Result<(), Error> {
        self.leave_safety(node)?;
        if !self.state.type_annotations.is_empty() {
            self.state.type_annotations.remove(&node);
        }
        self.save(node, term);
        let mut local_exit = false;
        if self
            .state
            .frames
            .last()
            .is_some_and(|f| f.node == node && f.kind == FrameKind::Block)
        {
            let frame = self
                .state
                .frames
                .pop()
                .ok_or_else(|| self.error(node, "missing block control frame"))?;
            local_exit = frame.breaks.is_some();
            let result = self.finish_frame(frame, Some(term))?;
            self.save(node, result);
        }
        self.finish_divergence(node, local_exit)?;
        Ok(())
    }

    fn publish_body_types(
        &mut self,
        body: NodeId,
        solved: &crate::types::infer::Solved,
    ) -> Result<(), Error> {
        let mut well_formed = HashSet::default();
        let mut nominal_types = bounds::Requirements::default();
        // A pointer conversion's immediate reborrow introduces a dereferenced
        // storage place even when the source expression is only a reference.
        for ty in self.pointer_storage(solved) {
            self.prepare_storage(body, ty)?;
        }
        // A generic result retains declaration lifetimes even when its expression
        // uses free aliases. Ownership queries need both canonical field contexts.
        if self.facts.types.contains_parameters(self.state.result) == Some(true) {
            self.prepare_storage(body, self.state.result)?;
        }
        // An implicit dereference need not have an AST node among `touched`.
        // Its selected field receiver still supplies a concrete type query key.
        // Retain discovery order so appended canonical types are deterministic.
        for index in 0..self.state.call_types.field_receivers.len() {
            let receiver = self.state.call_types.field_receivers[index];
            let ty = solved.types[receiver.0 as usize];
            self.prepare_storage(body, ty)?;
        }
        self.publish_touched_types(solved, &mut well_formed, &mut nominal_types)?;
        for index in 0..self.state.call_types.methods.len() {
            let (node, pending) = &self.state.call_types.methods[index];
            let (node, types) = (*node, pending.value_types(solved));
            // Receiver reads and lifetime conversions are runtime values even
            // when no expression node has their precise instantiated type.
            for ty in types {
                self.value_type(node, ty, &mut well_formed, &mut nominal_types)?;
            }
        }
        for index in 0..self.state.call_types.invocations.len() {
            let (node, pending) = &self.state.call_types.invocations[index];
            let (node, ty) = (*node, solved.types[pending.receiver.0 as usize]);
            self.value_type(node, ty, &mut well_formed, &mut nominal_types)?;
        }
        // Call inputs need their instantiated contracts even when an argument
        // diverges and therefore has no value node with the parameter's type.
        // Results already occur in `touched`; share input lifetime formation
        // with value types. Nominal proofs belong to the complete signature,
        // and diverging arguments need no additional concrete storage preparation.
        let mut call_inputs = crate::types::substitute::Bound::lower(1);
        for index in 0..self.state.calls.len() {
            let (node, term) = self.state.calls[index];
            let signature = solved.types[term.0 as usize];
            self.nominal_requirements(node, signature, &mut nominal_types)?;
            let Some(Type::Function {
                parameters,
                bound_regions: 0,
                convention,
                ..
            }) = self.facts.types.kind(signature)
            else {
                return Err(self.error(node, "call requires an opened signature"));
            };
            let count = self
                .facts
                .types
                .list(parameters)
                .ok_or_else(|| self.error(node, "missing call inputs"))?
                .len();
            if convention.variadic {
                self.validate_variadic_arguments(node, count, solved)?;
            }
            for parameter in 0..count {
                let ty = self
                    .facts
                    .types
                    .list(parameters)
                    .ok_or_else(|| self.error(node, "missing call inputs"))?[parameter];
                let ty = call_inputs
                    .apply(&mut self.facts.types, ty)
                    .map_err(|e| self.error(node, e))?;
                self.require_value_formation(node, ty, &mut well_formed);
            }
        }
        for index in 0..self.state.bindings.len() {
            let id = self.state.bindings[index];
            if let Some(term) = self.state.locals.remove(id.0 as usize) {
                let ty = solved.types[term.0 as usize];
                if !self.body_is_sized(body, ty)? {
                    return Err(self.error(body, "local bindings must be sized"));
                }
                self.facts.definitions[id.0 as usize] = Some(ty);
            }
        }
        nominal_types.finish(self)
    }
    /// Publish each touched node's solved type. A value type repeated under the
    /// same node conditions adds no fact: its requirements, storage and
    /// formation were already published. Bodies repeat a few types, so a
    /// short list finds them.
    fn publish_touched_types(
        &mut self,
        solved: &crate::types::infer::Solved,
        well_formed: &mut HashSet<TypeId>,
        nominal_types: &mut bounds::Requirements,
    ) -> Result<(), Error> {
        const PUBLISHED: usize = 32;
        let mut published = std::mem::take(&mut self.state.call_types.published);
        published.clear();
        for index in 0..self.state.touched.len() {
            let node = self.state.touched[index];
            let term = self.term(node)?;
            let key = self.key(node);
            let ty = solved.types[term.0 as usize];
            let conditions = (
                ty,
                self.state.type_annotations.contains(&node),
                self.state.annotation_children.contains(&node),
            );
            if !published.contains(&conditions) {
                if published.len() < PUBLISHED {
                    published.push(conditions);
                }
                self.value_type(node, ty, well_formed, nominal_types)?;
            }
            if key != self.state.body
                && self
                    .facts
                    .inline_constants
                    .as_deref()
                    .and_then(|registry| registry.get(key))
                    .is_some_and(|body| body.root.is_some())
            {
                self.facts
                    .node_mut(key)
                    .ok_or_else(|| {
                        Declarations::error(self.report, key, "missing inline expression facts")
                    })?
                    .adjustment = None;
                self.facts.reference_adjustments.remove(&key);
                self.facts.pointer_adjustments.remove(&key);
                self.facts.unsizing_adjustments.remove(&key);
            }
            self.facts
                .node_mut(key)
                .ok_or_else(|| Declarations::error(self.report, key, "invalid body node"))?
                .ty = Some(ty);
        }
        self.state.call_types.published = published;
        Ok(())
    }
    fn value_type(
        &mut self,
        node: NodeId,
        ty: TypeId,
        well_formed: &mut HashSet<TypeId>,
        nominal_types: &mut bounds::Requirements,
    ) -> Result<(), Error> {
        self.nominal_requirements(node, ty, nominal_types)?;
        if self.facts.generics.has_nominal_requirements
            && !self.state.type_annotations.contains(&node)
        {
            self.prepare_storage(node, ty)?;
        }
        if !self.body_shape(node, ty)?.well_formed {
            return Err(self.error(node, "body uses an ill-formed type"));
        }
        self.require_value_formation(node, ty, well_formed);
        Ok(())
    }
    fn require_value_formation(
        &mut self,
        node: NodeId,
        ty: TypeId,
        well_formed: &mut HashSet<TypeId>,
    ) {
        if (self.facts.types.contains_regions(ty) == Some(true)
            || self.facts.types.contains_parameters(ty) == Some(true))
            && !self.state.annotation_children.contains(&node)
            && well_formed.insert(ty)
        {
            self.facts.well_formed.push(super::model::WellFormed {
                body: self.state.body,
                source: self.key(node),
                ty,
            });
        }
    }
    fn binding(
        &mut self,
        mut node: NodeId,
        term: TermId,
        input: Option<NodeId>,
    ) -> Result<(), Error> {
        if !self.simple_pattern(node) {
            let valid = input
                .map(|input| self.pattern_input_validity(input))
                .transpose()?
                .unwrap_or(true);
            self.pattern(node, term, valid)?;
            let root = self.facts.patterns.roots[&self.key(node)];
            self.check_static_binding(node, root, input)?;
            self.state.alignment.patterns.push((root, input));
            return Ok(());
        }
        while let Kind::Attributed { value, .. } | Kind::Group(value) = self.kind(node) {
            self.save(node, term);
            node = value;
        }
        match self.kind(node) {
            Kind::Binding {
                by_ref: false,
                subpattern: None,
                ..
            } => {
                let id = self
                    .facts
                    .node(self.key(node))
                    .and_then(|f| f.binding)
                    .ok_or_else(|| self.error(node, "missing resolved binding"))?;
                self.state.locals[id.0 as usize] = Some(term);
                self.state.bindings.push(id);
            }
            Kind::Wildcard => {}
            _ => return Err(self.error(node, "compound patterns require pattern semantics")),
        }
        self.save(node, term);
        Ok(())
    }
    #[expect(
        clippy::too_many_lines,
        reason = "syntax dispatch schedules context-sensitive child constraints without recursive traversal"
    )]
    fn enter(&mut self, node: NodeId, expected: Option<TermId>) -> Result<(), Error> {
        if let Some(expected) = expected
            && matches!(
                self.contextual_head(node, expected)?,
                Some(Constructor::Reference { .. } | Constructor::Pointer { .. })
            )
        {
            self.state.reference_expected.insert(node);
            if matches!(self.kind(node), Kind::If { .. } | Kind::Match { .. }) {
                self.state.unsizing_contexts.insert(node, expected);
            }
        }
        if let Some(frame) = self.state.frames.last_mut()
            && frame.kind == FrameKind::While
            && frame.body == node
        {
            frame.in_condition = false;
        }
        if matches!(
            self.kind(node),
            Kind::Block {
                flavor: BlockFlavor::Const,
                ..
            }
        ) && self.key(node) != self.state.body
        {
            return self.enter_inline(node, expected);
        }
        if matches!(self.kind(node), Kind::Closure { .. }) {
            return self.enter_closure(node, expected);
        }
        self.enter_safety(node)?;
        self.enter_control(node, expected)?;
        self.state.work.push(Work::Exit(node));
        self.state.children.clear();
        match self.kind(node) {
            Kind::InlineAssembly(args) => self.enter_assembly(node, args)?,
            Kind::TryBlock { body, .. } => return self.enter_try_block(node, body, expected),
            Kind::For { iterator, .. } => {
                self.state.work.push(Work::ForIterator(node));
                self.state.work.push(Work::Enter(iterator, None));
                return Ok(());
            }
            Kind::Match { value, .. } => {
                self.state.work.push(Work::MatchArms { node, expected });
                self.state.work.push(Work::Enter(value, None));
                return Ok(());
            }
            Kind::Unary {
                op: Unary::Borrow | Unary::BorrowMut,
                operand,
            } => {
                if matches!(
                    self.kind(node),
                    Kind::Unary {
                        op: Unary::BorrowMut,
                        ..
                    }
                ) {
                    self.mark_mutable_language_place(operand)?;
                }
                return self.enter_borrow(node, operand, expected);
            }
            Kind::Unary {
                op: Unary::RawConst | Unary::RawMut,
                operand,
            } => {
                if matches!(
                    self.kind(node),
                    Kind::Unary {
                        op: Unary::RawMut,
                        ..
                    }
                ) {
                    self.mark_mutable_language_place(operand)?;
                }
                return self.enter_address(node, operand, expected);
            }
            Kind::Range {
                inclusive: false,
                start: Some(start),
                end: Some(end),
            } => {
                return self.enter_range(node, start, end, expected);
            }
            Kind::Tuple(elements) | Kind::Array(elements) => {
                return self.enter_elements(node, elements, expected);
            }
            Kind::Repeat { value, count } => {
                return self.enter_repeat(node, value, count, expected);
            }
            Kind::Struct { path, fields, base } => {
                return self.enter_struct(node, path, fields, base, expected);
            }
            Kind::OffsetOf { .. }
            | Kind::PatternEndpoint { .. }
            | Kind::Path { .. }
            | Kind::Name(..)
            | Kind::Qualified { .. }
            | Kind::Item(_) => {}
            Kind::Let {
                pattern,
                ty,
                value,
                otherwise: None,
            } => {
                let term = if let Some(ty) = ty {
                    self.annotation(ty)?
                } else {
                    self.variable(node, Domain::Any)?
                };
                if self.simple_pattern(pattern) {
                    self.binding(pattern, term, value)?;
                } else {
                    self.state.work.push(Work::BindPattern {
                        pattern,
                        term,
                        value,
                    });
                }
                self.state
                    .work
                    .extend(value.map(|value| Work::Enter(value, Some(term))));
                return Ok(());
            }
            Kind::Let {
                pattern,
                ty,
                value: Some(value),
                otherwise: Some(otherwise),
            } => {
                return self.enter_let_else(node, pattern, ty, value, otherwise);
            }
            Kind::LetCondition { pattern, value } => {
                self.state.work.push(Work::Conditional {
                    node,
                    pattern,
                    value,
                    expected: None,
                });
                self.state.work.push(Work::Enter(value, None));
                return Ok(());
            }
            Kind::Cast { value, ty } => {
                self.annotation(ty)?;
                self.hint_pointer_cast(node, ty, expected)?;
                self.state.children.push(value);
            }
            Kind::FieldInit { value, .. }
            | Kind::Group(value)
            | Kind::Attributed { value, .. }
            | Kind::Label { value, .. } => {
                self.state.work.push(Work::Enter(value, expected));
                return Ok(());
            }
            Kind::Block {
                statements, tail, ..
            }
            | Kind::Expansion {
                children: statements,
                tail,
            } => {
                self.state
                    .work
                    .extend(tail.map(|value| Work::Enter(value, expected)));
                let unit = self.report.package.unit(self.state.file);
                self.state.work.extend(
                    unit.configured
                        .ast
                        .list(statements)
                        .iter()
                        .rev()
                        .map(|n| Work::Enter(*n, None)),
                );
                return Ok(());
            }
            Kind::If {
                condition,
                then,
                otherwise,
            } => {
                self.state
                    .work
                    .extend(otherwise.map(|n| Work::Enter(n, expected)));
                self.state.work.push(Work::Enter(then, expected));
                self.state.work.push(Work::Enter(condition, None));
                return Ok(());
            }
            Kind::Return(value) => {
                let result = self.result_term(node)?;
                self.state
                    .work
                    .extend(value.map(|n| Work::Enter(n, Some(result))));
                return Ok(());
            }
            Kind::Binary {
                op: Op::Equal,
                left,
                ..
            } => {
                self.mark_union_write(left)?;
                self.mark_mutable_language_place(left)?;
                self.state.work.push(Work::AssignmentValue(node));
                self.state.work.push(Work::Enter(left, None));
                return Ok(());
            }
            Kind::Binary { op, left, right } if scalar::compound_operator(op).is_some() => {
                self.mark_mutable_language_place(left)?;
                self.state.children.extend([left, right]);
            }
            Kind::MethodCall { receiver, .. } | Kind::GenericMethodCall { receiver, .. } => {
                self.prepare_language_receiver(receiver)?;
                self.state.work.push(Work::MethodArguments(node, None));
                self.state.work.push(Work::Enter(receiver, None));
                return Ok(());
            }
            Kind::Call { callee, .. } => {
                self.state.work.push(Work::CallArguments(node, expected));
                self.state.work.push(Work::Enter(callee, None));
                return Ok(());
            }
            _ => self
                .report
                .package
                .unit(self.state.file)
                .configured
                .ast
                .push_children(node, &mut self.state.children),
        }
        self.state.work.extend(
            self.state
                .children
                .iter()
                .rev()
                .map(|n| Work::Enter(*n, None)),
        );
        Ok(())
    }
    #[expect(
        clippy::too_many_lines,
        reason = "typed expression constraints keep operand and result relationships in one dispatch"
    )]
    fn expression(&mut self, node: NodeId) -> Result<TermId, Error> {
        if let Some(value) = super::primitive_constants::selected(self.facts, self.key(node))
            .map_err(|e| self.error(node, e))?
        {
            let ty = self
                .facts
                .types
                .constant(value)
                .ok_or_else(|| self.error(node, "missing primitive constant value"))?
                .ty;
            return self.known(node, ty);
        }
        match self.kind(node) {
            Kind::Match { value, arms } => self.match_result(node, value, arms),
            Kind::Arm { guard, value, .. } => {
                if let Some(guard) = guard {
                    let boolean = self.known(guard, TypeId::primitive(Primitive::Bool))?;
                    self.coerce(guard, boolean)?;
                }
                self.term(value)
            }
            Kind::Closure { .. }
            | Kind::Range {
                inclusive: false,
                start: Some(_),
                end: Some(_),
            } => self.term(node),
            Kind::InlineAssembly(args) => self.assembly_result(node, args),
            Kind::LibraryValue { kind, value } => self.library_value(node, kind, value),
            Kind::Literal(index) => self.literal(node, index),
            Kind::PatternEndpoint { .. } => self.pattern_endpoint(node),
            Kind::Path { .. } | Kind::Name(..) | Kind::Qualified { .. } => {
                let Some(Target::Definition(id)) =
                    self.facts.node(self.key(node)).and_then(|f| f.reference)
                else {
                    return Err(self.error(node, "missing value resolution"));
                };
                if let Some(term) = self.state.locals[id.0 as usize] {
                    return Ok(term);
                }
                super::deprecation::check(self.report, self.facts, self.key(node), id)?;
                crate::resolve::stability::check_imported_api(
                    self.report,
                    self.key(node),
                    id,
                    false,
                )?;
                let definition = &self.report.resolution.definitions[id.0 as usize];
                if !matches!(
                    definition.kind,
                    DefKind::Function
                        | DefKind::Const
                        | DefKind::Static
                        | DefKind::Constructor
                        | DefKind::Variant
                ) && !self.facts.generics.constants.contains_key(&id)
                {
                    return Err(self.error(node, "unsupported value declaration"));
                }
                let ty = self.facts.definitions[id.0 as usize]
                    .ok_or_else(|| self.error(node, "missing value type"))?;
                self.select_unit_constructor(node, id)?;
                if let Some(variant) = self.facts.members.constructor(id)
                    && self.facts.generics.parameters.contains_key(&DefId(
                        self.facts.members.variants[variant.0 as usize].owner.0,
                    ))
                {
                    return self.generic_constructor(node, ty, variant);
                }
                if definition.kind == DefKind::Static {
                    self.check_static_access(node, id)?;
                }
                if definition.kind == DefKind::Const && self.facts.associated.owner(id).is_some() {
                    return self.associated_constant(node, id, ty);
                }
                if self.facts.generics.has_parameters(id)
                    || self.facts.methods.qualified.contains_key(&self.key(node))
                {
                    return self.generic_function(node, id, ty);
                }
                if definition.kind == DefKind::Const
                    && !self.state.in_function
                    && let Some(owner) = self.state.environment_owner
                {
                    self.facts
                        .constant_dependencies
                        .entry(owner)
                        .or_default()
                        .push(super::values::Dependency::Constant(
                            super::values::Instance {
                                definition: id,
                                arguments: crate::types::GenericArgs::EMPTY,
                            },
                        ));
                }
                let ty = self.function_item(node, id, ty)?;
                self.ordinary_bound_obligations(node, id, ty)?;
                self.known(node, ty)
            }
            Kind::Group(value) | Kind::Attributed { value, .. } | Kind::Label { value, .. } => {
                self.term(value)
            }
            Kind::TryBlock { body, .. } => self.finish_try_block(node, body),
            Kind::OffsetOf {
                container, fields, ..
            } => self.offset_of(node, container, fields),
            Kind::Unary {
                op: Unary::Try,
                operand,
            } => self.question_mark(node, operand),
            Kind::ConstClosure { .. } => {
                Err(self.error(node, "const closure callable semantics are not implemented"))
            }
            Kind::Block {
                flavor: BlockFlavor::Plain | BlockFlavor::Unsafe | BlockFlavor::Const,
                statements,
                tail,
            }
            | Kind::Expansion {
                children: statements,
                tail,
            } => {
                if let Some(tail) = tail {
                    return self.term(tail);
                }
                let unit = self.report.package.unit(self.state.file);
                let mut diverges = false;
                for statement in unit.configured.ast.list(statements) {
                    if self.active(*statement) {
                        diverges |= self.state.diverges[statement.get() as usize];
                    }
                }
                self.known(
                    node,
                    if diverges {
                        TypeId::NEVER
                    } else {
                        TypeId::UNIT
                    },
                )
            }
            Kind::Tuple(elements) | Kind::Array(elements) => self.finish_elements(node, elements),
            Kind::Repeat { value, .. } => self.finish_repeat(node, value),
            Kind::Struct { base, .. } => self.finish_struct(node, base),
            Kind::FieldInit { value, .. } => self.finish_field_init(node, value),
            Kind::Field { value, name } => self.field(node, value, name),
            Kind::Index { value, index } => self.array_index(node, value, index),
            Kind::EmptyStatement | Kind::Item(_) => self.known(node, TypeId::UNIT),
            Kind::Statement(value) => {
                let term = self.term(value)?;
                let never = self.head(node, term)? == Some(Constructor::Never);
                self.known(node, if never { TypeId::NEVER } else { TypeId::UNIT })
            }
            Kind::Let {
                pattern,
                value,
                otherwise,
                ..
            } => {
                if let Some(otherwise) = otherwise {
                    let never = self.known(otherwise, TypeId::NEVER)?;
                    self.coerce(otherwise, never)?;
                }
                let mut diverges = false;
                if let Some(value) = value {
                    diverges = self.state.diverges[value.get() as usize];
                    // Compound patterns were constrained after their initializer
                    // by BindPattern; a literal's node type may now be adjusted.
                    if otherwise.is_none() && self.simple_pattern(pattern) {
                        self.coerce(value, self.term(pattern)?)?;
                    }
                }
                self.known(
                    node,
                    if diverges {
                        TypeId::NEVER
                    } else {
                        TypeId::UNIT
                    },
                )
            }
            Kind::Unary {
                op: op @ (Unary::Negate | Unary::Not),
                operand,
            } => {
                if self.facts.source_bindings.is_some() {
                    if op == Unary::Negate
                        && let Some(literal) = self.literal_operand(operand)
                    {
                        self.state.negated.insert(literal);
                    }
                    return self.source_unary_operator(node, op, operand);
                }
                let kind = language_hooks::unary_hook(op)
                    .ok_or_else(|| self.error(node, "missing unary library hook"))?;
                if self.uses_language_operator(node, kind, operand)? {
                    return self.language_operator(node, kind, operand, None);
                }
                if matches!(
                    self.kind(node),
                    Kind::Unary {
                        op: Unary::Negate,
                        ..
                    }
                ) && let Some(literal) = self.literal_operand(operand)
                {
                    self.state.negated.insert(literal);
                }
                self.state.checks.push(node);
                self.term(operand)
            }
            Kind::Unary {
                op: op @ (Unary::Borrow | Unary::BorrowMut),
                operand,
            } => self.borrow(node, operand, op == Unary::BorrowMut),
            Kind::Unary {
                op: op @ (Unary::RawConst | Unary::RawMut),
                operand,
            } => self.address(node, operand, op == Unary::RawMut),
            Kind::Unary {
                op: Unary::Dereference,
                operand,
            } => self.dereference(node, operand),
            Kind::LetCondition { .. } => self.known(node, TypeId::primitive(Primitive::Bool)),
            Kind::Binary { op, left, right } => {
                let term = self.binary(node, op, left, right)?;
                if op == Op::AndAnd
                    && !self.facts.patterns.chains.is_empty()
                    && (self.facts.patterns.chains.contains(&self.key(left))
                        || self.facts.patterns.chains.contains(&self.key(right)))
                {
                    self.facts.patterns.chains.insert(self.key(node));
                }
                Ok(term)
            }
            Kind::Call { callee, arguments } => self.call(node, callee, arguments),
            Kind::MethodCall {
                receiver,
                arguments,
                ..
            }
            | Kind::GenericMethodCall {
                receiver,
                arguments,
                ..
            } => self.call(node, receiver, arguments),
            Kind::Loop { body } | Kind::For { body, .. } => self.finish_loop(node, body, None),
            Kind::While { condition, body } => self.finish_loop(node, body, Some(condition)),
            Kind::Break { label, value } => self.transfer(node, label.is_some(), value, false),
            Kind::Continue(label) => self.transfer(node, label.is_some(), None, true),
            Kind::Return(value) => {
                if !self.state.in_function {
                    return Err(self.error(node, "return is outside a function"));
                }
                let expected = self.result_term(node)?;
                if let Some(value) = value {
                    self.coerce(value, expected)?;
                } else {
                    let unit = self.known(node, TypeId::UNIT)?;
                    self.equate(node, unit, expected)?;
                }
                self.known(node, TypeId::NEVER)
            }
            Kind::If {
                condition,
                then,
                otherwise,
            } => {
                let boolean = self.known(node, TypeId::primitive(Primitive::Bool))?;
                self.coerce(condition, boolean)?;
                let result = if let Some(otherwise) = otherwise {
                    self.join(node, then, otherwise)?
                } else {
                    let unit = self.known(node, TypeId::UNIT)?;
                    self.coerce(then, unit)?;
                    unit
                };
                Ok(result)
            }
            Kind::Cast { value, ty } => {
                let source = self.term(value)?;
                let target = self.term(ty)?;
                self.prepare_pointer_cast(node, value, source, target)?;
                if matches!(
                    self.head(node, source)?,
                    Some(
                        Constructor::FunctionItem(..)
                            | Constructor::FunctionItemArgs(..)
                            | Constructor::Closure(..)
                    )
                ) && matches!(self.head(node, target)?, Some(Constructor::Function { .. }))
                {
                    self.closure_pointer_environment(node, source)?;
                    let signature = self.child(node, source, 0)?;
                    self.callable_shape(node, signature, target)?;
                }
                self.state.checks.push(node);
                Ok(target)
            }
            _ => Err(self.error(
                node,
                "body construct requires later semantic implementation",
            )),
        }
    }
    fn literal(&mut self, node: NodeId, index: u32) -> Result<TermId, Error> {
        let value = self
            .report
            .package
            .unit(self.state.file)
            .literal(index)
            .map_err(|e| self.error(node, e.message))?;
        let primitive = match value {
            Value::Integer(value) => {
                if let Some(suffix) = value.suffix {
                    Primitive::ALL
                        .into_iter()
                        .find(|p| p.name() == suffix.name())
                        .ok_or_else(|| self.error(node, "invalid integer suffix"))?
                } else {
                    return self.variable(node, Domain::Integer);
                }
            }
            Value::Float(value) => match value.suffix {
                Some(FloatType::F32) => Primitive::F32,
                Some(FloatType::F64) => Primitive::F64,
                Some(FloatType::F16) => Primitive::F16,
                Some(FloatType::F128) => Primitive::F128,
                None => return self.variable(node, Domain::Float),
            },
            Value::Boolean(_) => Primitive::Bool,
            Value::Character(_) => Primitive::Char,
            Value::Byte(_) => Primitive::U8,
            Value::String(bytes) => return self.text_literal_type(node, &bytes, true),
            Value::ByteString(bytes) => return self.text_literal_type(node, &bytes, false),
            Value::CString(_) => {
                return Err(self.error(node, "C string values require the library type assignment"));
            }
        };
        self.known(node, TypeId::primitive(primitive))
    }
    fn coerce(&mut self, source: NodeId, expected: TermId) -> Result<(), Error> {
        self.coerce_one(
            source,
            expected,
            self.state.reference_expected.contains(&source),
        )?;
        self.finish_reference_coercions()
    }
    fn record_adjustment(
        &mut self,
        node: NodeId,
        expected: TermId,
        kind: AdjustmentKind,
    ) -> Result<(), Error> {
        self.state.adjustments.push(PendingAdjustment {
            node,
            actual: self.term(node)?,
            expected,
            kind,
            reset: false,
        });
        Ok(())
    }
    fn coerce_one(
        &mut self,
        source: NodeId,
        expected: TermId,
        identity_reborrow: bool,
    ) -> Result<(), Error> {
        let actual = self.term(source)?;
        self.advance_projections(source)?;
        let actual_head = self.state.infer.head(&self.facts.types, actual);
        let expected_head = self.state.infer.head(&self.facts.types, expected);
        for head in [&actual_head, &expected_head] {
            if let Err(error) = head
                && *error != crate::types::infer::Error::Normalization
            {
                return Err(self.error(source, error));
            }
        }
        if matches!(actual_head, Err(crate::types::infer::Error::Normalization))
            || matches!(
                expected_head,
                Err(crate::types::infer::Error::Normalization)
            )
        {
            // A future normal form may complete an equality, but cannot supply
            // an implicit conversion or reborrow missing at this source point.
            let kind = if actual_head == Ok(Some(Constructor::Never)) {
                AdjustmentKind::Never
            } else {
                self.shape_equation(source, actual, expected)?;
                AdjustmentKind::Equality
            };
            self.record_adjustment(source, expected, kind)?;
            return Ok(());
        }
        let actual_head = actual_head.map_err(|e| self.error(source, e))?;
        let expected_head = expected_head.map_err(|e| self.error(source, e))?;
        if actual_head == Some(Constructor::Never) {
            self.record_adjustment(source, expected, AdjustmentKind::Never)?;
            return Ok(());
        }
        if matches!(
            actual_head,
            Some(
                Constructor::FunctionItem(..)
                    | Constructor::FunctionItemArgs(..)
                    | Constructor::Closure(..)
            )
        ) && matches!(expected_head, Some(Constructor::Function { .. }))
        {
            self.require_reifiable_head(source, actual_head)?;
            self.closure_pointer_environment(source, actual)?;
            let signature = self.child(source, actual, 0)?;
            let contextual = self.target_pointer_signature(source, actual, signature)?;
            let view = self.callable_conversion(source, contextual, expected)?;
            self.callable_shape(source, view, expected)?;
            let index = self.state.adjustments.len();
            self.record_adjustment(source, expected, AdjustmentKind::FunctionPointer)?;
            if view != signature {
                self.state.callable_adjustments.insert(index, view);
            }
            return Ok(());
        }
        if matches!(actual_head, Some(Constructor::Function { .. }))
            && matches!(expected_head, Some(Constructor::Function { .. }))
        {
            let view = self.callable_conversion(source, actual, expected)?;
            self.callable_shape(source, view, expected)?;
            let index = self.state.adjustments.len();
            self.record_adjustment(source, expected, AdjustmentKind::Equality)?;
            if view != actual {
                self.state.callable_adjustments.insert(index, view);
            }
            return Ok(());
        }
        if matches!(actual_head, Some(Constructor::Reference { .. }))
            && matches!(expected_head, Some(Constructor::Reference { .. }))
            && self.reference_coercion(source, actual, expected, identity_reborrow)?
        {
            return Ok(());
        }
        if matches!(expected_head, Some(Constructor::Pointer { .. }))
            && matches!(
                actual_head,
                Some(Constructor::Reference { .. } | Constructor::Pointer { .. })
            )
        {
            return self.pointer_coercion(source, actual, expected);
        }
        let inferred = if expected_head.is_none() {
            self.inferred_regions(source, actual)?
        } else {
            actual
        };
        self.callable_shape(source, inferred, expected)?;
        let index = self.state.adjustments.len();
        self.record_adjustment(source, expected, AdjustmentKind::Equality)?;
        if inferred != actual
            && matches!(
                actual_head,
                Some(Constructor::FunctionItem(..) | Constructor::FunctionItemArgs(..))
            )
        {
            self.state.call_types.input_context.freshening(index);
        }
        Ok(())
    }
    fn join(&mut self, node: NodeId, left: NodeId, right: NodeId) -> Result<TermId, Error> {
        let common = if let Some(expected) = self.contextual_unsizing_result(node)? {
            expected
        } else {
            self.join_types(node, self.term(left)?, self.term(right)?)?
        };
        self.coerce(left, common)?;
        self.coerce(right, common)?;
        Ok(common)
    }
    fn join_types(&mut self, node: NodeId, a: TermId, b: TermId) -> Result<TermId, Error> {
        let ah = self.head(node, a)?;
        let bh = self.head(node, b)?;
        if ah == Some(Constructor::Never) {
            return Ok(b);
        }
        if bh == Some(Constructor::Never) {
            return Ok(a);
        }
        if matches!(ah, Some(Constructor::Reference { .. }))
            && matches!(bh, Some(Constructor::Reference { .. }))
        {
            return self.reference_join(node, a, b);
        }
        if matches!(ah, Some(Constructor::Pointer { .. }))
            && matches!(bh, Some(Constructor::Pointer { .. }))
        {
            return self.pointer_join(node, a, b);
        }
        let a_item = matches!(
            ah,
            Some(
                Constructor::FunctionItem(..)
                    | Constructor::FunctionItemArgs(..)
                    | Constructor::Closure(..)
            )
        );
        let b_item = matches!(
            bh,
            Some(
                Constructor::FunctionItem(..)
                    | Constructor::FunctionItemArgs(..)
                    | Constructor::Closure(..)
            )
        );
        let distinct_items = match (ah, bh) {
            (Some(Constructor::FunctionItem(a, _)), Some(Constructor::FunctionItem(b, _))) => {
                a != b
            }
            _ => {
                a_item
                    && b_item
                    && self
                        .state
                        .infer
                        .incompatible(&self.facts.types, a, b)
                        .map_err(|e| self.error(node, e))?
            }
        };
        if distinct_items
            || (a_item && matches!(bh, Some(Constructor::Function { .. })))
            || (b_item && matches!(ah, Some(Constructor::Function { .. })))
        {
            let left = if a_item { self.child(node, a, 0)? } else { a };
            let right = if b_item { self.child(node, b, 0)? } else { b };
            self.shape_equation(node, left, right)?;
            return self.inferred_regions(node, left);
        }
        self.shape_equation(node, a, b)?;
        self.inferred_regions(node, a)
    }
    fn binary(
        &mut self,
        node: NodeId,
        op: Op,
        left: NodeId,
        right: NodeId,
    ) -> Result<TermId, Error> {
        if op == Op::Equal || scalar::compound_operator(op).is_some() {
            return self.assignment(node, op, left, right);
        }
        if self.facts.source_bindings.is_some()
            && matches!(
                op,
                Op::ShiftLeft | Op::Or | Op::Plus | Op::Minus | Op::Slash
            )
        {
            let hook = language_hooks::binary_hook(op)
                .ok_or_else(|| self.error(node, "invalid source binary hook"))?;
            return self.source_binary_operator(node, hook, left, right);
        }
        if let Some(kind) = language_hooks::binary_hook(op)
            && self.uses_language_operator(node, kind, left)?
        {
            return self.language_operator(node, kind, left, Some(right));
        }
        let comparison = match op {
            Op::EqualEqual => Some(crate::support::comparisons::Method::Eq),
            Op::BangEqual => Some(crate::support::comparisons::Method::Ne),
            Op::Less => Some(crate::support::comparisons::Method::Lt),
            Op::LessEqual => Some(crate::support::comparisons::Method::Le),
            Op::Greater => Some(crate::support::comparisons::Method::Gt),
            Op::GreaterEqual => Some(crate::support::comparisons::Method::Ge),
            _ => None,
        };
        if let Some(method) = comparison
            && self.report.package.uses_source_bindings()
        {
            return self.source_comparison_operator(node, method, left, right);
        }
        if let Some(method) = comparison
            && self.uses_comparison(node, method, left, right)?
        {
            return self.comparison(node, method, left, right);
        }
        let a = self.term(left)?;
        let b = self.term(right)?;
        self.state.checks.push(node);
        match op {
            Op::AndAnd | Op::OrOr => {
                let boolean = self.known(node, TypeId::primitive(Primitive::Bool))?;
                self.coerce(left, boolean)?;
                self.coerce(right, boolean)?;
                Ok(boolean)
            }
            Op::ShiftLeft | Op::ShiftRight => {
                let integer = self.variable(node, Domain::Integer)?;
                self.equate(node, b, integer)?;
                Ok(a)
            }
            Op::EqualEqual
            | Op::BangEqual
            | Op::Less
            | Op::Greater
            | Op::LessEqual
            | Op::GreaterEqual => {
                if matches!(self.head(node, a)?, Some(Constructor::Pointer { .. })) {
                    self.coerce(right, a)?;
                } else {
                    self.equate(node, a, b)?;
                }
                self.known(node, TypeId::primitive(Primitive::Bool))
            }
            Op::Plus
            | Op::Minus
            | Op::Star
            | Op::Slash
            | Op::Percent
            | Op::And
            | Op::Or
            | Op::Caret => {
                self.equate(node, a, b)?;
                Ok(a)
            }
            _ => Err(self.error(
                node,
                "assignment and operator semantics are not implemented yet",
            )),
        }
    }
    fn call(&mut self, node: NodeId, _callee: NodeId, _arguments: List) -> Result<TermId, Error> {
        let active = self
            .state
            .call_types
            .active
            .remove(&node)
            .ok_or_else(|| self.error(node, "missing call instantiation"))?;
        let term = active.signature;
        let Some(Constructor::Function {
            parameters,
            unsafe_,
            bound_regions,
            ..
        }) = self.head(node, term)?
        else {
            return Err(self.error(node, "call requires a known function type"));
        };
        self.target_call_permission(node, active.target_function, unsafe_)?;
        if bound_regions != 0 {
            return Err(self.error(node, "lifetime-polymorphic call retains an unopened binder"));
        }
        self.state.calls.push((node, term));
        self.child(node, term, parameters as usize)
    }
    fn enter_control(&mut self, node: NodeId, expected: Option<TermId>) -> Result<(), Error> {
        let label = if self
            .state
            .pending_label
            .is_some_and(|(target, _)| target == node)
        {
            self.state.pending_label.take().map(|(_, label)| label)
        } else {
            None
        };
        let (kind, body) = match self.kind(node) {
            Kind::Loop { body } => (FrameKind::Loop, body),
            Kind::While { body, .. } => (FrameKind::While, body),
            Kind::For { body, .. } => (FrameKind::For, body),
            Kind::Block { .. } if label.is_some() => (FrameKind::Block, node),
            Kind::Label { value, .. } => {
                let label = self
                    .facts
                    .node(self.key(node))
                    .and_then(|f| f.binding)
                    .ok_or_else(|| self.error(node, "missing label definition"))?;
                self.state.pending_label = Some((value, label));
                return Ok(());
            }
            _ => return Ok(()),
        };
        let result = if matches!(kind, FrameKind::While | FrameKind::For) {
            self.known(node, TypeId::UNIT)?
        } else {
            self.variable(node, Domain::Any)?
        };
        self.state.frames.push(Frame {
            node,
            body,
            kind,
            label,
            result,
            breaks: None,
            expected: if matches!(kind, FrameKind::While | FrameKind::For) {
                None
            } else {
                expected
            },
            in_condition: kind == FrameKind::While,
        });
        Ok(())
    }
    fn finish_loop(
        &mut self,
        node: NodeId,
        body: NodeId,
        condition: Option<NodeId>,
    ) -> Result<TermId, Error> {
        let frame = self
            .state
            .frames
            .pop()
            .ok_or_else(|| self.error(node, "missing loop control frame"))?;
        if frame.node != node {
            return Err(self.error(node, "mismatched loop control frame"));
        }
        let unit = self.known(node, TypeId::UNIT)?;
        self.coerce(body, unit)?;
        if let Some(condition) = condition {
            let boolean = self.known(node, TypeId::primitive(Primitive::Bool))?;
            self.coerce(condition, boolean)?;
            self.state.diverges[node.get() as usize] =
                frame.breaks.is_none() && self.state.diverges[condition.get() as usize];
        }
        self.finish_frame(
            frame,
            if condition.is_some() || frame.kind == FrameKind::For {
                Some(unit)
            } else {
                None
            },
        )
    }
    fn finish_frame(&mut self, frame: Frame, base: Option<TermId>) -> Result<TermId, Error> {
        if let Some(root) = frame.breaks {
            self.state.break_roots.insert(frame.node, root);
        }
        let expected = self.frame_expected(frame)?;
        let mut common = if frame.breaks.is_some() {
            expected.or(base)
        } else {
            base
        };
        let mut site = frame.breaks;
        while let Some(index) = site {
            let value = self.state.breaks[index].value;
            let node = self.state.breaks[index].node;
            let term = if let Some(value) = value {
                self.term(value)?
            } else {
                self.known(node, TypeId::UNIT)?
            };
            common = Some(if let Some(previous) = common {
                self.join_types(node, previous, term)?
            } else {
                term
            });
            site = self.state.breaks[index].next;
        }
        let common = if let Some(common) = common {
            common
        } else {
            self.known(frame.node, TypeId::NEVER)?
        };
        self.equate(frame.node, frame.result, common)?;
        let reference = matches!(
            self.contextual_head(frame.node, common)?,
            Some(Constructor::Reference { .. })
        );
        let identity_reborrow = self.state.reference_expected.contains(&frame.node);
        if frame.kind == FrameKind::Block {
            if reference
                && let Kind::Block {
                    tail: Some(tail), ..
                } = self.kind(frame.node)
            {
                if frame.breaks.is_some() {
                    self.state.reference_expected.insert(tail);
                }
                self.coerce_one(
                    tail,
                    frame.result,
                    identity_reborrow || frame.breaks.is_some(),
                )?;
                self.finish_reference_coercions()?;
            } else {
                self.coerce(frame.node, if reference { frame.result } else { common })?;
            }
        }
        let mut site = frame.breaks;
        while let Some(index) = site {
            if let Some(value) = self.state.breaks[index].value {
                self.coerce_one(
                    value,
                    if reference { frame.result } else { common },
                    identity_reborrow || self.state.reference_expected.contains(&value),
                )?;
                self.finish_reference_coercions()?;
            }
            site = self.state.breaks[index].next;
        }
        Ok(frame.result)
    }
    fn frame_expected(&mut self, frame: Frame) -> Result<Option<TermId>, Error> {
        if let Some(expected) = frame.expected
            && self.contextual_head(frame.node, expected)?.is_some()
        {
            return Ok(Some(expected));
        }
        Ok(None)
    }
    fn call_arguments(&mut self, node: NodeId, expected: Option<TermId>) -> Result<(), Error> {
        let Kind::Call { callee, arguments } = self.kind(node) else {
            return Err(self.error(node, "invalid call work"));
        };
        let receiver = self.term(callee)?;
        let mut term = receiver;
        let constructor = matches!(self.head(node, term)?,
        Some(Constructor::FunctionItem(item, _) | Constructor::FunctionItemArgs(item, _))
            if self.facts.members.constructor(DefId(item.0)).is_some_and(|variant| {
                let owner = self.facts.members.variants[variant.0 as usize].owner;
                self.facts.generics.parameters.contains_key(&DefId(owner.0))
            }));
        if matches!(
            self.head(node, term)?,
            Some(Constructor::FunctionItem(..) | Constructor::FunctionItemArgs(..))
        ) {
            term = self.child(node, term, 0)?;
        }
        term = if matches!(self.head(node, term)?, Some(Constructor::Function { .. })) {
            self.retain_defining_member(node, callee, receiver)?;
            self.call_signature(node, term)?
        } else {
            self.invocation_signature(node, callee, arguments)?
        };
        let target_function = self.selected_target_function(node, receiver)?;
        self.retain_active_call(node, term, target_function)?;
        let Some(Constructor::Function { parameters, .. }) = self.head(node, term)? else {
            return Err(self.error(node, "call requires a known function type"));
        };
        if constructor && let Some(expected) = expected {
            let result = self.child(node, term, parameters as usize)?;
            if let Some(expected) =
                self.unsizing_constructor_context(node, result, Some(expected))?
            {
                self.shape_equation(node, result, expected)?;
            }
        }
        if !constructor && let Some(expected) = expected {
            let result = self.child(node, term, parameters as usize)?;
            if self.contextual_head(node, result)?.is_none()
                && matches!(
                    self.contextual_head(node, expected)?,
                    Some(Constructor::Function { .. })
                )
            {
                self.callable_shape(node, result, expected)?;
            }
        }
        self.schedule_call_arguments(node, term, arguments, 0)
    }
    fn schedule_call_arguments(
        &mut self,
        node: NodeId,
        term: TermId,
        arguments: List,
        offset: usize,
    ) -> Result<(), Error> {
        let Some(Constructor::Function {
            parameters,
            convention,
            ..
        }) = self.head(node, term)?
        else {
            return Err(self.error(node, "call requires a known function type"));
        };
        let unit = self.report.package.unit(self.state.file);
        let count = unit
            .configured
            .ast
            .list(arguments)
            .iter()
            .filter(|n| self.active(**n))
            .count();
        if count + offset < parameters as usize
            || (!convention.variadic && count + offset != parameters as usize)
        {
            return Err(self.error(node, "call argument count mismatch"));
        }
        let mut index = count + offset;
        for argument in unit.configured.ast.list(arguments).iter().rev() {
            if self.active(*argument) {
                index -= 1;
                if index >= parameters as usize {
                    self.state.work.push(Work::Enter(*argument, None));
                    continue;
                }
                let expected = self.child(node, term, index)?;
                let parameter = u32::try_from(index)
                    .map_err(|_| self.error(node, "call parameter index exhausted"))?;
                self.state.work.push(Work::CoerceCallArgument {
                    call: node,
                    argument: *argument,
                    parameter,
                });
                if self.state.call_types.obligations.is_empty() {
                    self.state.work.push(Work::Enter(*argument, Some(expected)));
                } else {
                    self.state.work.push(Work::PrepareCallArgument {
                        call: node,
                        argument: *argument,
                        parameter,
                    });
                }
            }
        }
        Ok(())
    }
    fn transfer(
        &mut self,
        node: NodeId,
        labeled: bool,
        value: Option<NodeId>,
        continuing: bool,
    ) -> Result<TermId, Error> {
        let index = if labeled {
            let Some(Target::Definition(label)) =
                self.facts.node(self.key(node)).and_then(|f| f.reference)
            else {
                return Err(self.error(node, "missing control label resolution"));
            };
            self.state
                .frames
                .iter()
                .rposition(|f| f.label == Some(label))
        } else {
            self.state
                .frames
                .iter()
                .rposition(|frame| frame.kind != FrameKind::Try)
        }
        .ok_or_else(|| self.error(node, "break or continue is outside a loop"))?;
        let frame = self.state.frames[index];
        if !labeled && (frame.kind == FrameKind::Block || frame.in_condition) {
            return Err(self.error(
                node,
                "control transfer here requires an explicit loop label",
            ));
        }
        if continuing {
            if frame.kind == FrameKind::Block {
                return Err(self.error(node, "cannot continue a labeled block"));
            }
        } else {
            if value.is_some() && frame.kind == FrameKind::While {
                return Err(self.error(node, "while loops cannot break with a value"));
            }
            if value.is_some() && frame.kind == FrameKind::For {
                return Err(self.error(node, "for loops cannot break with a value"));
            }
            let expected = self.frame_expected(frame)?.unwrap_or(frame.result);
            if let Some(value) = value {
                if matches!(
                    self.head(value, expected)?,
                    Some(Constructor::Reference { .. })
                ) {
                    self.state.reference_expected.insert(value);
                }
                self.coerce(value, expected)?;
            } else {
                let unit = self.known(node, TypeId::UNIT)?;
                self.equate(node, unit, expected)?;
            }
            let site = self.state.breaks.len();
            self.state.breaks.push(BreakSite {
                node,
                value,
                next: frame.breaks,
            });
            self.state.frames[index].breaks = Some(site);
        }
        let key = self.key(node);
        self.facts
            .node_mut(key)
            .ok_or_else(|| Declarations::error(self.report, key, "invalid control node"))?
            .operation = Some(Operation::Transfer {
            target: frame.node,
            continuing,
        });
        self.known(node, TypeId::NEVER)
    }
    fn assignment(
        &mut self,
        node: NodeId,
        op: Op,
        mut left: NodeId,
        right: NodeId,
    ) -> Result<TermId, Error> {
        let a = self.term(left)?;
        let b = self.term(right)?;
        while let Kind::Group(inner) | Kind::Attributed { value: inner, .. } = self.kind(left) {
            left = inner;
        }
        let target = self.assignment_target(node, left)?;
        if let Some(kind) = language_hooks::assignment_hook(op)
            && self.uses_language_assignment(node, kind, left, right)?
        {
            return self.language_operator(node, kind, left, Some(right));
        }
        let operator = scalar::compound_operator(op);
        if matches!(operator, Some(Op::ShiftLeft | Op::ShiftRight)) {
            let integer = self.variable(node, Domain::Integer)?;
            self.equate(node, b, integer)?;
        } else if operator.is_some() {
            self.equate(node, a, b)?;
        } else {
            self.coerce(right, a)?;
        }
        self.state.assignments.push((node, target, operator));
        let never = self.head(node, b)? == Some(Constructor::Never);
        self.known(node, if never { TypeId::NEVER } else { TypeId::UNIT })
    }
    fn validate_assignments(&mut self) -> Result<(), Error> {
        for (node, target, operator) in &self.state.assignments {
            let Kind::Binary { left, right, .. } = self.kind(*node) else {
                return Err(self.error(*node, "invalid assignment syntax"));
            };
            if let Some(operator) = operator
                && !scalar::binary(*operator, self.primitive(left)?, self.primitive(right)?)
            {
                return Err(self.error(*node, "compound assignment is not defined for these types"));
            }
            let operation = if let Some(target) = target {
                Operation::Assign {
                    target: *target,
                    operator: *operator,
                    left: self.ty(left)?,
                    right: self.ty(right)?,
                }
            } else {
                Operation::AssignPlace {
                    operator: *operator,
                    left: self.ty(left)?,
                    right: self.ty(right)?,
                }
            };
            let key = self.key(*node);
            self.facts
                .node_mut(key)
                .ok_or_else(|| Declarations::error(self.report, key, "invalid assignment node"))?
                .operation = Some(operation);
        }
        Ok(())
    }
    fn ty(&self, node: NodeId) -> Result<TypeId, Error> {
        self.facts
            .node(self.key(node))
            .and_then(|f| f.adjustment.map(|a| a.target).or(f.ty))
            .ok_or_else(|| self.error(node, "missing solved expression type"))
    }
    fn primitive(&self, node: NodeId) -> Result<Primitive, Error> {
        if let Some(Type::Primitive(p)) = self.facts.types.kind(self.ty(node)?) {
            Ok(p)
        } else {
            Err(self.error(node, "operation requires a scalar type"))
        }
    }
    fn literal_operand(&self, mut node: NodeId) -> Option<NodeId> {
        loop {
            match self.kind(node) {
                Kind::Group(value) => node = value,
                Kind::Literal(_) => return Some(node),
                _ => return None,
            }
        }
    }
    fn validate_operations(&mut self) -> Result<(), Error> {
        for index in 0..self.state.checks.len() {
            let node = self.state.checks[index];
            if let Kind::InlineAssembly(args) = self.kind(node) {
                self.validate_assembly(node, args)?;
                continue;
            }
            let operation = match self.kind(node) {
                Kind::Unary { op, operand } => {
                    if !scalar::unary(op, self.primitive(operand)?) {
                        return Err(
                            self.error(node, "unary operator is not defined for this scalar type")
                        );
                    }
                    Operation::Unary {
                        operator: op,
                        operand: self.ty(operand)?,
                        literal: self.literal_operand(operand).is_some(),
                    }
                }
                Kind::Binary { op, left, right } => {
                    let pointer = matches!(
                        self.facts.types.kind(self.ty(left)?),
                        Some(Type::Pointer { .. })
                    );
                    if pointer && !self.state.in_function {
                        return Err(
                            self.error(node, "pointers cannot be compared during const eval")
                        );
                    }
                    let valid = if pointer {
                        matches!(
                            op,
                            Op::EqualEqual
                                | Op::BangEqual
                                | Op::Less
                                | Op::LessEqual
                                | Op::Greater
                                | Op::GreaterEqual
                        )
                    } else {
                        scalar::binary(op, self.primitive(left)?, self.primitive(right)?)
                    };
                    if !valid {
                        return Err(self.error(
                            node,
                            "binary operator is not defined for these scalar types",
                        ));
                    }
                    Operation::Binary {
                        operator: op,
                        left: self.ty(left)?,
                        right: self.ty(right)?,
                    }
                }
                Kind::Cast { value, ty } => self.validate_cast(node, value, ty)?,
                _ => return Err(self.error(node, "invalid scalar operation obligation")),
            };
            let key = self.key(node);
            if let Operation::Cast {
                kind: super::model::Cast::FunctionPointer,
                source,
                target,
            } = operation
            {
                self.facts.retain_coercion(super::model::Coercion {
                    body: self.state.body,
                    source: key,
                    actual: source,
                    expected: target,
                    kind: AdjustmentKind::FunctionPointer,
                    reference: None,
                });
            }
            self.facts
                .node_mut(key)
                .ok_or_else(|| Declarations::error(self.report, key, "invalid operation node"))?
                .operation = Some(operation);
        }
        Ok(())
    }
    fn validate_cast(
        &mut self,
        node: NodeId,
        value: NodeId,
        ty: NodeId,
    ) -> Result<Operation, Error> {
        let kind = if let Some(kind) = self.validate_pointer_cast(node, value, ty)? {
            kind
        } else if matches!(
            self.facts.types.kind(self.ty(value)?),
            Some(
                Type::FunctionItem { .. }
                    | Type::Closure {
                        environment: TypeId::UNIT,
                        ..
                    }
            )
        ) && matches!(
            self.facts.types.kind(self.ty(ty)?),
            Some(Type::Function { .. })
        ) {
            super::model::Cast::FunctionPointer
        } else if let Some(Type::Adt(item, _)) = self.facts.types.kind(self.ty(value)?) {
            if !self.facts.enum_casts.contains(&item)
                || self.facts.destructors.has_declared(item)
                || self.primitive(ty)?.integer().is_none()
            {
                return Err(self.error(
                    node,
                    "only eligible field-less enums can be cast to integers",
                ));
            }
            super::model::Cast::EnumToInteger
        } else {
            scalar::cast(self.primitive(value)?, self.primitive(ty)?)
                .ok_or_else(|| self.error(node, "invalid scalar cast"))?
        };
        if kind == super::model::Cast::FunctionPointer
            && let Some(Type::FunctionItem { item, .. }) = self.facts.types.kind(self.ty(value)?)
        {
            self.require_reifiable_item(node, DefId(item.0))?;
        }
        Ok(Operation::Cast {
            kind,
            source: self.ty(value)?,
            target: self.ty(ty)?,
        })
    }
    fn validate_values(&mut self) -> Result<(), Error> {
        // Keep only negated literal IDs; no per-function copy of the file's AST.
        for index in 0..self.state.touched.len() {
            let node = self.state.touched[index];
            if let Kind::Statement(value) = self.kind(node) {
                self.validate_discarded_result(node, value)?;
                continue;
            }
            if let Kind::Repeat { value, .. } = self.kind(node) {
                self.validate_repeat(node, value)?;
                continue;
            }
            let Kind::Literal(index) = self.kind(node) else {
                continue;
            };
            let unit = self.report.package.unit(self.state.file);
            let value = unit
                .literal(index)
                .map_err(|e| self.error(node, e.message))?;
            if matches!(value, Value::String(_) | Value::ByteString(_)) {
                continue;
            }
            let negated = self.state.negated.contains(&node);
            let scalar = scalar::literal(&value, self.primitive(node)?, negated)
                .map_err(|e| self.error(node, e))?;
            let key = self.key(node);
            self.facts
                .node_mut(key)
                .ok_or_else(|| Declarations::error(self.report, key, "invalid literal node"))?
                .scalar = Some(scalar);
        }
        Ok(())
    }
}

#[cfg(test)]
pub(in crate::sema) use target_permissions::observe_active_calls;
