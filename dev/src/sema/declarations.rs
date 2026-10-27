//! Demand aliases and function signatures through an iterative dependency graph.
mod abstract_types;
mod aggregates;
mod annotations;
mod associated;
mod call_traits;
mod calling_conventions;
use calling_conventions::rust_abi;
pub(super) use calling_conventions::{callable_convention, fixed_parameters, with_variadic};
mod capture_bounds;
pub(in crate::sema) mod cloning;
mod comparison_methods;
mod comparisons;
pub(in crate::sema) use comparisons::prepare as prepare_comparison;
mod aliases;
mod constants;
mod contracts;
#[path = "declarations/overridable_methods.rs"]
mod default_methods;
pub(in crate::sema) use default_methods::is_final_trait_method;
mod deferred_arguments;
mod destructors;
mod evaluation;
mod execution;
mod foreign;
mod generics;
pub(in crate::sema) use generics::prepare_callable_template;
mod heads;
mod impl_headers;
mod implicit_methods;
mod inherent_headers;
mod intrinsics;
pub(in crate::sema) mod iteration;
mod language_hooks;
mod library_types;
mod lifetimes;
mod method_overlap;
mod method_paths;
mod method_prefix;
mod methods;
mod objects;
mod offset_paths;
mod primitive_constants;
mod projections;
mod receivers;
mod self_constructors;
mod self_schema;
mod slice_methods;
mod slice_paths;
mod structural_owners;
mod trait_paths;
mod trait_regions;
mod traits;
pub(in crate::sema) use traits::validate_impl_requirements_prepared;
mod unreduced;
mod validation;
use super::{
    model::{Declarations, Function, Initializer, InitializerKind},
    regions::{self, BinderId, Position, Regions},
};
use crate::{
    resolve::{
        expansion::Report,
        model::{DefId, DefKind, Target},
        package::Error,
        unit::{FileId, NodeKey},
    },
    syntax::{
        ast::{Kind, List, NodeId, TypeFlavor},
        item::{GenericFlavor, Item, ItemKind, Safety},
    },
    types::{ItemId, Kind as Type, Primitive, RegionId, TypeId},
};
#[derive(Clone, Copy, PartialEq, Eq)]
enum State {
    Unseen,
    Active,
    Complete,
}
#[derive(Clone, Copy)]
enum SignaturePhase {
    Begin(regions::Context),
    Output(BinderId),
    Bounds(BinderId, u32),
    Finish(BinderId, u32),
}
#[derive(Clone, Copy)]
enum Work {
    Execution(execution::Request, execution::Phase),
    ClosureAnnotation(NodeKey, regions::Context, Option<usize>),
    CallTraitInputs(BinderId),
    CallTraitOutput(NodeKey, regions::Context),
    ImplHeader(DefId, impl_headers::Phase),
    ConstantMember(crate::sema::traits::implementations::ConstantRequest, bool),
    ValidateImpl(DefId, traits::ImplementationPhase),
    MethodPrefix(NodeKey, regions::Context),
    InherentMethod(NodeKey, associated::Path, regions::Context),
    InherentHeaders(ItemId, crate::resolve::names::NameId),
    EvaluateConstant(evaluation::Request, bool),
    InitializerReady(DefId),
    TraitCallable(NodeKey, trait_paths::Callable),
    ImplicitMethod(NodeKey, regions::Context),
    MethodArguments(NodeKey, NodeKey),
    ProjectionPath(NodeKey, regions::Context),
    ImplicitProjection(NodeKey, regions::Context),
    ProjectionArguments(
        NodeKey,
        regions::Context,
        crate::types::projection::Application,
    ),
    HeaderProjection(NodeKey, regions::Context),
    ProjectionBound(projections::lookup::Seed, bool),
    ProjectionSelection(NodeKey, crate::types::substitute::Request),
    ProjectionResult(NodeKey),
    ProjectionEnvironment(DefId, regions::Context, bool),
    NormalizeBound(NodeKey, regions::Context),
    NominalContract(ItemId, bool),
    Fields(ItemId, bool),
    OpaqueHidden(crate::types::opaque::OpaqueId),
    PointeeContracts(DefId, traits::MetadataPhase),
    Constructor(DefId),
    SelfConstructor(NodeKey, DefId, regions::Context),
    PrepareConstructor(DefId),
    ValueType(DefId),
    ValueContract(TypeId, bool),
    ValueShape(TypeId),
    ConstantUse(NodeKey, regions::Context),
    ConstantArguments(NodeKey),
    Bounds(DefId, regions::Context),
    ItemArguments(NodeKey, DefId),
    AliasVariant(NodeKey, DefId, regions::Context),
    AssociatedPrefix(NodeKey, DefId, regions::Context),
    Generic(generics::Declaration),
    Default(generics::DefaultId, bool),
    PathArguments(NodeKey, DefId, regions::Context),
    Definition(DefId, bool),
    Node(NodeKey, bool, regions::Context),
    Reference(NodeKey, RegionId),
    Parameter(BinderId, bool),
    Signature(NodeKey, Option<DefId>, SignaturePhase),
    Receiver(NodeKey, DefId, BinderId),
}
impl Work {
    fn body_dependency(requirement: crate::sema::bodies::Requirement) -> Self {
        use crate::sema::bodies::Requirement;
        match requirement {
            Requirement::Nominal(item) => Self::ValueType(DefId(item.0)),
            Requirement::Inherent(item, name) => Self::InherentHeaders(item, name),
            Requirement::Definition(definition) => Self::Definition(definition, false),
            Requirement::Constructor(definition) => Self::PrepareConstructor(definition),
            Requirement::ImplHeader(owner) => Self::ImplHeader(owner, impl_headers::Phase::Begin),
            Requirement::ImplementationValidated(owner) => {
                Self::ValidateImpl(owner, traits::ImplementationPhase::Begin)
            }
            Requirement::Fields(item) => Self::Fields(item, false),
            Requirement::Opaque(id) => Self::OpaqueHidden(id),
            Requirement::PointeeContracts(id) => {
                Self::PointeeContracts(id, traits::MetadataPhase::Begin)
            }
        }
    }
    fn selection_dependency(dependency: super::traits::selection::Dependency) -> Self {
        match dependency {
            super::traits::selection::Dependency::Header(owner) => {
                Self::ImplHeader(owner, impl_headers::Phase::Begin)
            }
            super::traits::selection::Dependency::ImplementationValidated(owner) => {
                Self::ValidateImpl(owner, traits::ImplementationPhase::Begin)
            }
            super::traits::selection::Dependency::Fields(item) => Self::Fields(item, false),
            super::traits::selection::Dependency::Opaque(id) => Self::OpaqueHidden(id),
            super::traits::selection::Dependency::PointeeContracts(id) => {
                Self::PointeeContracts(id, traits::MetadataPhase::Begin)
            }
        }
    }
}
struct Signature {
    parameters: List,
    result: Option<NodeId>,
    binders: List,
    unsafe_: bool,
    convention: crate::types::callable::Convention,
}
struct Checker<'a> {
    selected: Option<Box<super::crates::SelectedConsumer<'a>>>,
    trait_aliases: Option<Box<aliases::Pending>>,
    annotation_inputs: annotations::Inputs,
    call_paths: crate::support::hash::HashSet<NodeKey>,
    linkage_sources: crate::support::hash::HashMap<NodeKey, foreign::Attributes>,
    target_feature_sources: crate::support::hash::HashSet<NodeKey>,
    method_queries:
        crate::support::hash::HashMap<NodeKey, Box<super::methods::selection::PathQuery>>,
    inherent_headers: inherent_headers::Pending,
    impl_headers: impl_headers::Pending,
    constant_members: traits::ConstantReadiness,
    early_implementations: Option<Box<traits::EarlyImplementations>>,
    associated_type_bounds: crate::support::hash::HashMap<DefId, traits::AssociatedBounds>,
    opaque_syntax: crate::support::hash::HashMap<NodeKey, abstract_types::Pending>,
    object_syntax: objects::Syntax,
    associated_bindings: generics::bindings::Pending,
    projections: projections::Pending,
    trait_proofs: trait_regions::Proofs,
    method_trait_environments: Option<
        crate::support::hash::HashMap<
            NodeKey,
            std::sync::Arc<super::traits::assumptions::Assumptions>,
        >,
    >,
    nominals: aggregates::Pending,
    defaults: generics::Defaults,
    deferred_bounds: Vec<(DefId, regions::Context)>,
    bound_syntax: crate::support::hash::HashMap<DefId, Vec<generics::bounds::Syntax>>,
    bound_captures: Option<Box<crate::types::substitute::Bound>>,
    relaxed: crate::support::hash::HashSet<DefId>,
    constant_modules:
        Option<crate::support::hash::HashMap<NodeKey, Option<crate::resolve::package::ModuleId>>>,
    constant_scratch: super::constants::Scratch,
    constant_evaluations: evaluation::Pending,
    execution_bodies: execution::Pending,
    constant_dependencies: crate::support::hash::HashSet<NodeKey>,
    constant_captures: generics::Captures,
    parameter_uses: crate::types::parameters::uses::Uses,
    report: &'a Report,
    facts: Declarations,
    states: Vec<State>,
    work: Vec<Work>,
    children: Vec<TypeId>,
    regions: Regions,
    discriminants: super::constants::Discriminants,
    nominal_regions: crate::support::hash::HashMap<ItemId, regions::Context>,
    function_binders: crate::support::hash::HashMap<NodeKey, BinderId>,
    constant_binders: crate::support::hash::HashMap<NodeKey, BinderId>,
    owner_binders: crate::support::hash::HashMap<DefId, BinderId>,
}
/// Resolve declaration types and shape-affecting constants without reparsing source.
///
/// # Errors
/// Rejects recursive aliases, invalid signatures and unsupported annotation semantics.
pub fn analyze(report: &Report) -> Result<Declarations, Error> {
    let mut checker = Checker::new(report)?;
    checker.run().map_err(|error| {
        checker
            .facts
            .retain_error_warnings(report.diagnostic(error))
    })?;
    Ok(checker.facts)
}
/// Resolve declarations with the executable pipeline's checked constant service.
/// # Errors
/// Reports the same declaration errors, including demanded execution failures.
pub fn analyze_with_executor(
    report: &Report,
    executor: Box<dyn super::values::execution::Executor>,
) -> Result<Declarations, Error> {
    let mut checker = Checker::new(report)?;
    checker.facts.set_constant_executor(executor);
    checker.run().map_err(|error| {
        checker
            .facts
            .retain_error_warnings(report.diagnostic(error))
    })?;
    Ok(checker.facts)
}
/// Resume the genuine staged consumer without reconstructing its semantic arena.
pub(in crate::sema) fn analyze_reserved<'a>(
    report: &'a Report,
    mut facts: Declarations,
    mut selected: Box<super::crates::SelectedConsumer<'a>>,
) -> Result<(Declarations, Box<super::crates::SelectedConsumer<'a>>), Error> {
    selected.begin(
        report,
        &mut facts,
        report
            .resolution
            .definitions
            .len()
            .saturating_mul(size_of::<State>()),
    )?;
    #[cfg(test)]
    eprintln!(
        "selected-continuation checker={} service={} states={}",
        size_of::<Checker<'_>>(),
        size_of::<super::crates::SelectedConsumer<'_>>(),
        report.resolution.definitions.len()
    );
    let mut checker = Checker::from_facts(report, facts)?;
    checker.selected = Some(selected);
    checker.import_states()?;
    checker.run().map_err(|error| {
        checker
            .facts
            .retain_error_warnings(report.diagnostic(error))
    })?;
    checker.selected_bodies()?;
    let selected = checker.selected.take().ok_or_else(|| {
        Error::unlocated("staged declaration checking lost its actual selected producer")
    })?;
    let facts = super::bodies::analyze(report, checker.facts)?;
    Ok((facts, selected))
}
impl<'a> Checker<'a> {
    fn new(report: &'a Report) -> Result<Self, Error> {
        if !report.names_resolved {
            return Err(Error::at(
                FileId(0),
                0,
                "type checking requires completed name resolution",
            ));
        }
        let facts = Declarations::new(report)?;
        let mut checker = Self::from_facts(report, facts)?;
        checker
            .import_states()
            .map_err(|error| checker.facts.retain_error_warnings(error))?;
        Ok(checker)
    }
    fn from_facts(report: &'a Report, mut facts: Declarations) -> Result<Self, Error> {
        let linkage_sources = foreign::collect(report, &mut facts.foreign)
            .map_err(|error| facts.retain_error_warnings(error))?;
        let target_feature_sources = call_traits::target_feature_sources(report)
            .map_err(|error| facts.retain_error_warnings(error))?;
        Ok(Self {
            selected: None,
            trait_aliases: aliases::collect(report)?,
            annotation_inputs: annotations::Inputs::default(),
            linkage_sources,
            call_paths: crate::support::hash::HashSet::default(),
            target_feature_sources,
            method_queries: crate::support::hash::HashMap::default(),
            inherent_headers: inherent_headers::Pending::default(),
            impl_headers: impl_headers::Pending::default(),
            constant_members: traits::ConstantReadiness::default(),
            early_implementations: None,
            associated_type_bounds: crate::support::hash::HashMap::default(),
            opaque_syntax: crate::support::hash::HashMap::default(),
            object_syntax: objects::Syntax::default(),
            associated_bindings: generics::bindings::Pending::default(),
            projections: projections::Pending::default(),
            trait_proofs: trait_regions::Proofs::default(),
            method_trait_environments: None,
            nominals: aggregates::Pending::default(),
            defaults: generics::Defaults::default(),
            deferred_bounds: Vec::new(),
            bound_syntax: crate::support::hash::HashMap::default(),
            bound_captures: None,
            relaxed: crate::support::hash::HashSet::default(),
            constant_modules: None,
            constant_scratch: super::constants::Scratch::default(),
            constant_evaluations: evaluation::Pending::default(),
            execution_bodies: execution::Pending::default(),
            constant_dependencies: crate::support::hash::HashSet::default(),
            constant_captures: generics::Captures::default(),
            parameter_uses: crate::types::parameters::uses::Uses::default(),
            report,
            facts,
            states: vec![State::Unseen; report.resolution.definitions.len()],
            work: Vec::new(),
            children: Vec::new(),
            regions: Regions::default(),
            discriminants: super::constants::Discriminants::default(),
            nominal_regions: crate::support::hash::HashMap::default(),
            function_binders: crate::support::hash::HashMap::default(),
            constant_binders: crate::support::hash::HashMap::default(),
            owner_binders: crate::support::hash::HashMap::default(),
        })
    }
}
impl Checker<'_> {
    fn import_states(&mut self) -> Result<(), Error> {
        if let Some(imported) = &self.report.resolution.crates {
            for definition in imported.generic_definitions() {
                let definition = definition.map_err(Error::unlocated)?;
                if self.selected.is_some() {
                    continue;
                }
                match super::crates::source_readiness(&mut self.facts, definition)? {
                    Some(super::crates::ImportedReadiness::Declaration) => {
                        self.states[definition.0 as usize] = State::Complete;
                        continue;
                    }
                    Some(super::crates::ImportedReadiness::ValueBinding) => continue,
                    None => {}
                }
                if self
                    .facts
                    .definitions
                    .get(definition.0 as usize)
                    .is_none_or(Option::is_none)
                {
                    return Err(Error::unlocated(
                        "imported generic declaration lacks its completed canonical type",
                    ));
                }
                self.states[definition.0 as usize] = State::Complete;
            }
            for (definition, _) in imported.functions() {
                self.states[definition.0 as usize] = State::Complete;
            }
            for (definition, _) in imported.constants() {
                self.states[definition.0 as usize] = State::Complete;
            }
            for (definition, _) in imported.records() {
                self.states[definition.0 as usize] = State::Complete;
            }
            for (definition, _) in imported.statics() {
                self.states[definition.0 as usize] = State::Complete;
            }
        }
        Ok(())
    }
    fn work_limit(&self) -> usize {
        self.report
            .package
            .units
            .iter()
            .fold(self.states.len(), |count, unit| {
                count
                    .saturating_add(unit.configured.ast.nodes.len())
                    .saturating_add(unit.configured.ast.edges.len())
            })
            .saturating_mul(16)
    }
    fn run(&mut self) -> Result<(), Error> {
        self.validate_foreign_blocks()?;
        self.reserve_member_parents()?;
        self.prepare_library_types()?;
        self.prepare_intrinsics()?;
        self.reserve_nominals()?;
        self.reserve_implementation_candidates()?;
        self.reserve_inherent_headers()?;
        let mut steps = 0_usize;
        // Linear declaration work is bounded by the input graph, not by a fixed
        // whole-crate size. Alias states diagnose cycles independently.
        let work_limit = self.work_limit();
        self.prepare_call_contracts()?;
        self.prepare_language_contracts()?;
        self.prepare_comparison_contracts()?;
        self.prepare_clone_contract()?;
        self.prepare_iteration_contracts()?;
        let destructors = self.reserve_destructors()?;
        self.complete_nominal_headers(&mut steps, work_limit)?;
        self.prepare_trait_owners(&mut steps, work_limit)?;
        self.prepare_source_call_contracts(&mut steps, work_limit)?;
        self.prepare_destructors(destructors, &mut steps, work_limit)?;
        self.prepare_trait_implementations(&mut steps, work_limit)?;
        self.prepare_early_implementations(&mut steps, work_limit)?;
        self.prepare_inherent_owners(&mut steps, work_limit)?;
        self.index_inherent_methods()?;
        self.prepare_associated_types(&mut steps, work_limit)?;
        if let Some(contracts) = &mut self.facts.generics.traits {
            contracts.complete_results();
        }
        super::traits::results::prepare_outlives(&mut self.facts)
            .map_err(|e| Error::at(FileId(0), 0, e))?;
        for index in 0..self.report.resolution.definitions.len() {
            let id = DefId(
                u32::try_from(index)
                    .map_err(|_| Error::at(FileId(0), 0, "too many type declarations"))?,
            );
            if self.latent_library_definition(id) {
                continue;
            }
            if let Some(selected) = self.selected.as_deref()
                && selected.is_imported(id)?
            {
                continue;
            }
            if !matches!(
                self.report.resolution.definitions[index].kind,
                DefKind::Primitive
                    | DefKind::Function
                    | DefKind::TypeAlias
                    | DefKind::ForeignType
                    | DefKind::Const
                    | DefKind::Static
            ) {
                continue;
            }
            self.work.push(Work::Definition(id, false));
            self.drain(&mut steps, work_limit)?;
        }
        self.complete_deferred_declarations(&mut steps, work_limit)?;
        self.complete_nominals(&mut steps, work_limit)?;
        super::compiler_items::phantom::complete(self.report, &mut self.facts)?;
        super::compiler_items::pointee::prepare(self.report, &mut self.facts)?;
        super::compiler_items::site_records::complete(self.report, &mut self.facts)?;
        super::traits::results::prepare_opaque_outlives(&mut self.facts)
            .map_err(|e| Error::at(FileId(0), 0, e))?;
        self.trait_defaults()?;
        self.validate_destructors()?;
        self.validate_owner_bounds()?;
        self.validate_associated_type_bounds()?;
        self.validate_required_family_bounds()?;
        self.validate_signatures()?;
        self.validate_trait_implementations()?;
        self.validate_trait_impl_owners()?;
        self.validate_trait_regions()?;
        self.validate_inherent_overlaps()?;
        self.local_annotations(&mut steps, work_limit)?;
        self.complete_alias_normalizations(&mut steps, work_limit)?;
        self.validate_projection_regions()?;
        self.validate_objects()?;
        self.complete_discriminants(&mut steps, work_limit)?;
        if !self.method_queries.is_empty() {
            return Err(Error::at(
                FileId(0),
                0,
                "unfinished method applicability queries",
            ));
        }
        self.complete_layouts()?;
        super::api_exposure::check(self.report, &self.facts)?;
        self.reject_alias_execution()?;
        super::specialization::require_execution(&self.facts)
    }
    fn latent_library_definition(&self, id: DefId) -> bool {
        self.report
            .resolution
            .builtin_traits
            .latent_comparison_owner(id)
            .is_some()
            || self.lazy_library_method(id)
            || iteration::collections::owns(self.report, id)
            || iteration::range_contracts::owns(self.report, id)
            || cloning::owns(self.report, id)
            || self
                .report
                .resolution
                .builtin_traits
                .collections()
                .and_then(|c| c.mutable.as_deref())
                .is_some_and(|m| m.read_latent && m.read.definition == id)
    }

    #[expect(
        clippy::too_many_lines,
        reason = "the exhaustive worklist dispatch keeps every declaration continuation in one match"
    )]
    fn drain(&mut self, steps: &mut usize, limit: usize) -> Result<(), Error> {
        let mut scan = Vec::new();
        while let Some(work) = self.work.pop() {
            *steps += 1;
            if *steps > limit {
                return Err(Error::at(
                    FileId(0),
                    0,
                    "declaration type work budget exceeded",
                ));
            }
            match work {
                Work::Execution(requirement, phase) => self.execution_body(requirement, phase)?,
                Work::ClosureAnnotation(key, context, first) => {
                    self.closure_annotation(key, context, first, steps, limit, &mut scan)?;
                }
                Work::CallTraitInputs(binder) => self.regions.begin_callable(binder),
                Work::CallTraitOutput(key, context) => self.call_trait_output(key, context)?,
                Work::ImplHeader(owner, phase) => self.advance_impl_header(owner, phase)?,
                Work::ValidateImpl(owner, phase) => self.advance_impl_validation(owner, phase)?,
                Work::ConstantMember(request, finish) => {
                    self.constant_member(request, finish)?;
                }
                Work::MethodPrefix(key, context) => {
                    self.complete_method_prefix(key, context, steps, limit, &mut scan)?;
                }
                Work::InherentHeaders(item, name) => {
                    self.prepare_inherent_bucket(item, name);
                }
                Work::InherentMethod(key, path, context) => {
                    self.complete_method_path(key, &path, context, steps, limit, &mut scan)?;
                }
                Work::EvaluateConstant(request, resume) => {
                    self.evaluate_constant(request, resume)?;
                }
                Work::InitializerReady(definition) => {
                    self.complete_initializer_readiness(definition);
                }
                Work::TraitCallable(key, callable) => {
                    self.complete_trait_callable(key, callable)?;
                }
                Work::ImplicitMethod(key, context) => {
                    self.complete_implicit_method(key, context, steps, limit, &mut scan)?;
                }
                Work::MethodArguments(key, suffix) => {
                    self.complete_method_arguments(key, suffix)?;
                }
                Work::ProjectionPath(key, context) => {
                    self.complete_projection_path(key, context)?;
                }
                Work::ImplicitProjection(key, context) => {
                    self.complete_implicit_projection(key, context)?;
                }
                Work::ProjectionArguments(key, context, application) => {
                    self.complete_projection_arguments(key, context, application)?;
                }
                Work::HeaderProjection(key, context) => {
                    self.complete_header_projection(key, context)?;
                }
                Work::ProjectionBound(seed, finish) => {
                    self.complete_projection_bound(seed, finish)?;
                }
                Work::ProjectionResult(key) => {
                    self.complete_projection_result(key)?;
                }
                Work::ProjectionSelection(key, request) => {
                    self.complete_projection_selection(key, request)?;
                }
                Work::ProjectionEnvironment(owner, context, finish) => {
                    self.complete_projection_environment(owner, context, finish)?;
                }
                Work::NormalizeBound(key, context) => {
                    self.normalize_annotation(key, self.ty(key)?, context)?;
                }
                Work::NominalContract(item, finish) => self.nominal_contract(item, finish)?,
                Work::Fields(item, finish) => self.nominal_fields(item, finish)?,
                Work::PointeeContracts(id, phase) => self.pointee_member_work(id, phase)?,
                Work::OpaqueHidden(id) => {
                    let declaration = self
                        .facts
                        .opaques
                        .declarations
                        .get(&id)
                        .ok_or_else(|| Error::at(FileId(0), 0, "missing opaque declaration"))?;
                    return Err(self.error(
                        declaration.source,
                        "opaque hidden type requires its completed defining body",
                    ));
                }
                Work::Constructor(id) => self.constructor(id)?,
                Work::SelfConstructor(key, owner, context) => {
                    self.complete_self_constructor(key, owner, context)?;
                }
                Work::PrepareConstructor(id) => self.constant_constructor(id)?,
                Work::ValueType(id) => self.constant_value_type(id),
                Work::ValueContract(ty, finish) => self.constant_type_contract(ty, finish)?,
                Work::ValueShape(ty) => self.constant_value_shape(ty)?,
                Work::ConstantUse(key, context) => {
                    self.constant_use(key, context, steps, limit, &mut scan)?;
                }
                Work::ConstantArguments(key) => self.constant_arguments(key)?,
                Work::Bounds(owner, context) => self.complete_bounds(owner, context)?,
                Work::AliasVariant(key, owner, context) => {
                    self.complete_alias_variant(key, owner, context)?;
                }
                Work::ItemArguments(key, owner) => {
                    self.complete_item_arguments(key, owner)?;
                }
                Work::Generic(declaration) => self.generic_declaration(declaration)?,
                Work::Default(id, finish) => self.generic_default(id, finish)?,
                Work::PathArguments(key, id, context) => self.path_arguments(key, id, context)?,
                Work::AssociatedPrefix(key, owner, context) => {
                    self.complete_associated_prefix(key, owner, context)?;
                }
                Work::Definition(id, finish) => self.definition(id, finish)?,
                Work::Node(key, finish, context) => self.annotation(key, finish, context)?,
                Work::Reference(key, region) => self.reference_type(key, region)?,
                Work::Parameter(id, finish) => {
                    if finish {
                        self.regions.end_parameter(id);
                    } else {
                        self.regions.begin_parameter(id);
                    }
                }
                Work::Signature(key, owner, phase) => {
                    self.signature(key, owner, phase, steps, limit)?;
                }
                Work::Receiver(key, owner, binder) => {
                    self.complete_receiver(key, owner, binder)?;
                }
            }
        }
        Ok(())
    }
    fn local_annotations(&mut self, steps: &mut usize, limit: usize) -> Result<(), Error> {
        let mut scan = Vec::new();
        let mut stack = Vec::new();
        let mut children = Vec::new();
        for (index, unit) in self.report.package.units.iter().enumerate() {
            if unit.include_context.is_some() {
                continue;
            }
            let file = FileId(
                u32::try_from(index)
                    .map_err(|_| Error::at(FileId(0), 0, "too many source files"))?,
            );
            let ast = &unit.configured.ast;
            let mut seen = vec![false; ast.nodes.len()];
            stack.push((ast.root, regions::Context::ALIAS));
            while let Some((node, inherited)) = stack.pop() {
                let kind = ast.node(node).kind;
                // Only bodies, closures and inline constants open a context.
                let context = if matches!(
                    kind,
                    Kind::Item(_) | Kind::Closure { .. } | Kind::Block { .. }
                ) {
                    self.local_context_of(NodeKey { file, node }, kind, inherited)
                } else {
                    inherited
                };
                if !unit.configured.active[node.get() as usize]
                    || std::mem::replace(&mut seen[node.get() as usize], true)
                {
                    continue;
                }
                *steps += 1;
                if *steps > limit {
                    return Err(self.error(
                        NodeKey { file, node },
                        "annotation traversal work budget exceeded",
                    ));
                }
                if matches!(kind, Kind::Closure { .. } | Kind::Block { .. }) {
                    self.nest_local_body(NodeKey { file, node }, inherited, context)?;
                }
                children.clear();
                ast.push_children(node, &mut children);
                stack.extend(children.iter().rev().map(|&child| (child, context)));
                self.local_annotation(
                    NodeKey { file, node },
                    kind,
                    context,
                    steps,
                    limit,
                    &mut scan,
                )?;
            }
        }
        Ok(())
    }
    /// Closures and inline constants share their enclosing body's region identities.
    fn nest_local_body(
        &mut self,
        key: NodeKey,
        inherited: regions::Context,
        context: regions::Context,
    ) -> Result<(), Error> {
        if let (
            regions::Position::Local(own),
            regions::Position::Local(outer) | regions::Position::Constant(outer),
        ) = (context.position, inherited.position)
            && own == key.node
            && outer != key.node
        {
            self.facts
                .body_regions
                .nest(key, NodeKey { node: outer, ..key })
                .map_err(|e| self.error(key, e))?;
        }
        Ok(())
    }
    fn local_annotation(
        &mut self,
        key: NodeKey,
        kind: Kind,
        context: regions::Context,
        steps: &mut usize,
        limit: usize,
        scan: &mut Vec<NodeId>,
    ) -> Result<(), Error> {
        // Method and alias paths are keyed by `Path`/`Qualified` nodes or need
        // a resolved reference; every other check below names its own kind.
        if !matches!(
            kind,
            Kind::GenericMethodCall { .. }
                | Kind::Path { .. }
                | Kind::Qualified { .. }
                | Kind::Closure { .. }
                | Kind::Let { .. }
                | Kind::Cast { .. }
        ) && self.facts.node(key).is_none_or(|f| f.reference.is_none())
        {
            return Ok(());
        }
        if let Kind::GenericMethodCall { method, .. } = kind {
            self.prepare_method_arguments(key, method, context, steps, limit, scan)?;
        }
        if self.schedule_self_constructor(key, context)? {
            self.drain(steps, limit)?;
            return Ok(());
        }
        if self.schedule_method_path(key, context, steps, limit, scan)? {
            self.drain(steps, limit)?;
            return Ok(());
        }
        if self.schedule_alias_variant(key, context, steps, limit, scan)? {
            self.drain(steps, limit)?;
        }
        if matches!(kind, Kind::Name(..) | Kind::Path { .. })
            && let Some(owner) = self.generic_use_owner(key)
        {
            self.schedule_item_arguments(key, owner, context, steps, limit, scan)?;
            self.drain(steps, limit)?;
        }
        if matches!(kind, Kind::Closure { .. }) {
            self.work.push(Work::ClosureAnnotation(key, context, None));
            self.drain(steps, limit)?;
            return Ok(());
        }
        let (Kind::Let { ty: Some(ty), .. } | Kind::Cast { ty, .. }) = kind else {
            return Ok(());
        };
        // Holes stay in the owning body, while the ordinary elaborator
        // resolves every written lifetime and nested callable binder.
        self.annotation_holes(NodeKey { node: ty, ..key }, steps, limit, scan)?;
        self.node_work(key.file, ty, context);
        self.drain(steps, limit)?;
        Ok(())
    }
    fn annotation_holes(
        &mut self,
        key: NodeKey,
        steps: &mut usize,
        limit: usize,
        scan: &mut Vec<NodeId>,
    ) -> Result<(), Error> {
        scan.clear();
        scan.push(key.node);
        let ast = &self.report.package.unit(key.file).configured.ast;
        while let Some(node) = scan.pop() {
            *steps += 1;
            if *steps > limit {
                return Err(self.error(key, "annotation work budget exceeded"));
            }
            if matches!(ast.node(node).kind, Kind::InferType) {
                self.facts.partial_annotations.insert(
                    NodeKey { node, ..key },
                    super::model::PartialAnnotation::Hole,
                );
            }
            ast.push_children(node, scan);
        }
        Ok(())
    }
    fn error(&self, key: NodeKey, message: impl Into<String>) -> Error {
        Declarations::error(self.report, key, message)
    }
    fn kind(&self, key: NodeKey) -> Kind {
        Declarations::kind(self.report, key)
    }
    fn item(&self, key: NodeKey) -> Result<Item, Error> {
        let Kind::Item(index) = self.kind(key) else {
            return Err(self.error(key, "declaration is not an item"));
        };
        Ok(self.report.package.unit(key.file).configured.ast.items[index as usize])
    }
    fn ty(&self, key: NodeKey) -> Result<TypeId, Error> {
        self.facts
            .node(key)
            .and_then(|facts| facts.ty)
            .ok_or_else(|| self.error(key, "missing annotation type fact"))
    }
    fn store(&mut self, key: NodeKey, ty: TypeId) -> Result<(), Error> {
        self.retain_unreduced_annotation(key, ty)?;
        let Some(facts) = self.facts.node_mut(key) else {
            return Err(self.error(key, "invalid annotation node"));
        };
        facts.ty = Some(ty);
        Ok(())
    }
    fn intern(&mut self, key: NodeKey, kind: Type) -> Result<TypeId, Error> {
        self.facts
            .types
            .intern(kind)
            .map_err(|e| self.error(key, e))
    }
    fn node_work(&mut self, file: FileId, node: NodeId, context: regions::Context) {
        self.work
            .push(Work::Node(NodeKey { file, node }, false, context));
    }
    fn activate_iteration_definition(&mut self, id: DefId) -> Result<bool, Error> {
        if cloning::owns(self.report, id) {
            self.activate_iterator_clone(id)?;
            return Ok(true);
        }
        if iteration::range_contracts::owns(self.report, id) {
            self.activate_range(id)?;
            return Ok(true);
        }
        if iteration::collections::owns(self.report, id) {
            self.activate_collection(id)?;
            return Ok(true);
        }
        Ok(false)
    }
    fn activate_comparison_owner(&mut self, id: DefId) -> Result<bool, Error> {
        if let Some(declaration) = self
            .report
            .resolution
            .builtin_traits
            .latent_comparison_owner(id)
        {
            if self.states[declaration.definition.0 as usize] == State::Active {
                return Err(self.error(
                    declaration.origin,
                    "recursive canonical comparison preparation",
                ));
            }
            self.states[declaration.definition.0 as usize] = State::Active;
            self.comparison_family(declaration)?;
            if let Some(contracts) = &mut self.facts.generics.traits {
                contracts.complete_results();
            }
            return Ok(true);
        }
        Ok(false)
    }
    fn definition(&mut self, id: DefId, finish: bool) -> Result<(), Error> {
        if self.activate_iteration_definition(id)? {
            return Ok(());
        }
        if self.states[id.0 as usize] == State::Complete {
            return Ok(());
        }
        if self.activate_comparison_owner(id)? {
            return Ok(());
        }
        if let Some(selected) = self.selected.as_deref_mut()
            && selected.definition(self.report, &mut self.facts, id)?
        {
            self.states[id.0 as usize] = State::Complete;
            return Ok(());
        }
        let definition = &self.report.resolution.definitions[id.0 as usize];
        if definition.kind == DefKind::Primitive {
            return self.primitive_definition(id);
        }
        let key = definition
            .node
            .ok_or_else(|| Error::at(FileId(0), 0, "type declaration has no syntax"))?;
        let item = self.item(key)?;
        if item.kind == ItemKind::ForeignType {
            return self.foreign_type(id, key, item);
        }
        if matches!(
            item.kind,
            ItemKind::Struct { .. } | ItemKind::Enum { .. } | ItemKind::Union { .. }
        ) {
            return self.nominal_header(id, key, finish);
        }
        if !finish {
            if self.states[id.0 as usize] == State::Active {
                return Err(self.error(key, "recursive type alias"));
            }
            self.states[id.0 as usize] = State::Active;
            if matches!(item.kind, ItemKind::Function { .. }) {
                self.work.push(Work::Signature(
                    key,
                    Some(id),
                    SignaturePhase::Begin(regions::Context::ALIAS),
                ));
                return Ok(());
            }
            if item.where_clause.start != item.where_clause.end
                && !matches!(item.kind, ItemKind::TypeAlias { .. })
            {
                return Err(self.error(key, "generic aliases require later semantic assignments"));
            }
            self.work.push(Work::Definition(id, true));
            match item.kind {
                ItemKind::TypeAlias { ty: Some(ty), .. } => {
                    let context = self.template_context(id, key)?;
                    self.node_work(key.file, ty, context);
                    self.declaration_parameters(id, key, context)?;
                }
                ItemKind::Const { ty, .. } => {
                    let context = self.constant_context(id, key)?;
                    self.node_work(key.file, ty, context);
                }
                ItemKind::Static { ty, .. } => {
                    self.node_work(key.file, ty, regions::Context::STATIC);
                }
                _ => return Err(self.error(key, "unsupported type declaration")),
            }
            return Ok(());
        }
        let annotation = match item.kind {
            ItemKind::TypeAlias { ty: Some(ty), .. }
            | ItemKind::Const { ty, .. }
            | ItemKind::Static { ty, .. } => NodeKey {
                file: key.file,
                node: ty,
            },
            _ => return Err(self.error(key, "invalid completed declaration")),
        };
        let ty = self.ty(annotation)?;
        self.complete(id, key, ty)?;
        if matches!(item.kind, ItemKind::TypeAlias { .. }) {
            if let Some(original) = self.facts.unreduced.node(annotation) {
                self.facts.unreduced.retain_definition(id, original);
            }
            self.validate_alias(id, key, ty)?;
            self.facts
                .alias_requirements
                .seal(id)
                .map_err(|e| self.error(key, e))?;
        }
        self.value_initializer(id, key, ty, item)
    }
    fn value_initializer(
        &mut self,
        id: DefId,
        key: NodeKey,
        ty: TypeId,
        item: Item,
    ) -> Result<(), Error> {
        if let ItemKind::Static {
            mutable, safety, ..
        } = item.kind
            && self.register_foreign_global(id, key, ty, mutable, safety)?
        {
            return Ok(());
        }
        let (value, kind) = match item.kind {
            ItemKind::Const { value, .. } => (value, InitializerKind::Const),
            ItemKind::Static { value, mutable, .. } => (value, InitializerKind::Static { mutable }),
            _ => return Ok(()),
        };
        if value.is_none() && self.facts.associated.trait_owner(id).is_some() {
            return Ok(());
        }
        let value =
            value.ok_or_else(|| self.error(key, "constant or static requires an initializer"))?;
        let body = self.initializer_body_source(
            key,
            NodeKey {
                file: key.file,
                node: value,
            },
        )?;
        self.facts
            .push_initializer(Initializer {
                typing: super::model::InitializerTyping::Unseen,
                body,
                definition: id,
                node: key,
                value: NodeKey {
                    file: key.file,
                    node: value,
                },
                ty,
                kind,
            })
            .map_err(|e| self.error(key, e))?;
        Ok(())
    }
    fn initializer_body_source(
        &self,
        declaration: NodeKey,
        root: NodeKey,
    ) -> Result<NodeKey, Error> {
        let mut node = root;
        // Parsed wrapper chains are bounded by this one syntax arena. Blocks
        // remain real enclosing bodies; only transparent wrappers are opened.
        for _ in 0..self
            .report
            .package
            .unit(root.file)
            .configured
            .ast
            .nodes
            .len()
        {
            match self.kind(node) {
                Kind::Group(value) | Kind::Attributed { value, .. } => node.node = value,
                Kind::Closure { .. }
                | Kind::Block {
                    flavor: crate::syntax::ast::BlockFlavor::Const,
                    ..
                } => return Ok(declaration),
                _ => return Ok(root),
            }
        }
        Err(self.error(root, "initializer owner wrapper limit exceeded"))
    }
    fn primitive_definition(&mut self, id: DefId) -> Result<(), Error> {
        let definition = &self.report.resolution.definitions[id.0 as usize];
        let name = self.report.package.names.get(definition.name);
        let primitive = Primitive::ALL
            .into_iter()
            .find(|p| p.name() == name)
            .ok_or_else(|| Error::at(FileId(0), 0, "unknown primitive definition"))?;
        self.facts.definitions[id.0 as usize] = Some(TypeId::primitive(primitive));
        self.states[id.0 as usize] = State::Complete;
        Ok(())
    }
    fn complete(&mut self, id: DefId, key: NodeKey, ty: TypeId) -> Result<(), Error> {
        self.facts.definitions[id.0 as usize] = Some(ty);
        crate::sema::compiler_items::validate_layout_member(self.report, &self.facts, id)?;
        self.states[id.0 as usize] = State::Complete;
        self.store(key, ty)
    }
    fn annotation(
        &mut self,
        key: NodeKey,
        finish: bool,
        context: regions::Context,
    ) -> Result<(), Error> {
        if self.facts.node(key).is_some_and(|facts| facts.ty.is_some()) {
            return Ok(());
        }
        if matches!(
            self.facts.partial_annotations.get(&key),
            Some(super::model::PartialAnnotation::Hole)
        ) {
            return Ok(());
        }
        let kind = self.kind(key);
        if let Kind::TraitType { opaque, bounds } = kind {
            let ty = if opaque {
                self.opaque_result(key, bounds, context, finish)?
            } else {
                self.object_type(key, bounds, context, finish)?
            };
            if let Some(ty) = ty {
                self.store(key, ty)?;
            }
            return Ok(());
        }
        if !finish {
            return self.schedule_annotation(key, kind, context);
        }
        if self.partial_compound(key) {
            self.facts
                .partial_annotations
                .insert(key, super::model::PartialAnnotation::Shape);
            return Ok(());
        }
        let ty = match kind {
            Kind::NeverType => TypeId::NEVER,
            Kind::Group(node) | Kind::Attributed { value: node, .. } => self.ty(NodeKey {
                file: key.file,
                node,
            })?,
            Kind::Path { .. } | Kind::Applied { .. } | Kind::Name(..) => {
                let Some(Target::Definition(id)) =
                    self.facts.node(key).and_then(|facts| facts.reference)
                else {
                    return Err(self.error(key, "missing type path resolution"));
                };
                if self.partial_generic_path(key, id, context)? {
                    return Ok(());
                }
                let ty = self.path_type(key, id, context)?;
                if !matches!(
                    self.report.resolution.definitions[id.0 as usize].kind,
                    DefKind::Primitive | DefKind::Generic
                ) {
                    return self.normalize_annotation(key, ty, context);
                }
                ty
            }
            Kind::TypePattern {
                base,
                constraint: crate::syntax::ast::TypeConstraint::NonNull,
            } => {
                let base = self.ty(NodeKey { node: base, ..key })?;
                if !matches!(self.facts.types.kind(base), Some(Type::Pointer { .. })) {
                    return Err(
                        self.error(key, "non-pointer pattern types require their wider profile")
                    );
                }
                self.facts
                    .types
                    .intern(Type::Pattern {
                        base,
                        constraint: crate::types::Constraint::NonNull,
                    })
                    .map_err(|message| self.error(key, message))?
            }
            Kind::Type { flavor, elements } => self.compound(key, flavor, elements)?,
            _ => return Err(self.error(key, "invalid completed annotation")),
        };
        // Structural annotations only combine children already normalized in this scope.
        // Type applications above can introduce projections through substituted defaults.
        self.store(key, ty)
    }
    fn schedule_type_path(&mut self, key: NodeKey, context: regions::Context) -> Result<(), Error> {
        let Some(Target::Definition(id)) = self.facts.node(key).and_then(|facts| facts.reference)
        else {
            return Err(self.error(key, "type path needs a resolved declaration"));
        };
        let kind = self.report.resolution.definitions[id.0 as usize].kind;
        if kind == DefKind::TypeAlias && self.facts.associated.owner(id).is_some() {
            return Err(self.error(
                key,
                "associated type requires an explicit Self qualification",
            ));
        }
        if kind == DefKind::Generic
            && !self.facts.definitions[id.0 as usize]
                .is_some_and(|ty| matches!(self.facts.types.kind(ty), Some(Type::Parameter(_))))
        {
            return Err(self.error(key, "const parameter cannot be used as a type"));
        }
        if !(matches!(
            kind,
            DefKind::Primitive
                | DefKind::TypeAlias
                | DefKind::ForeignType
                | DefKind::Struct
                | DefKind::Enum
                | DefKind::Generic
        ) || (kind == DefKind::SelfType
            && (self.facts.associated.schema(id).is_some()
                || self.alias_self_schema(id)?.is_some())))
        {
            return Err(self.error(
                key,
                "nominal or associated types require later semantic assignments",
            ));
        }
        if matches!(kind, DefKind::TypeAlias | DefKind::Struct | DefKind::Enum) {
            self.work.push(Work::PathArguments(key, id, context));
        }
        self.work.push(Work::Definition(id, false));
        Ok(())
    }
    fn schedule_reference(
        &mut self,
        key: NodeKey,
        elements: List,
        context: regions::Context,
    ) -> Result<(), Error> {
        let unit = self.report.package.unit(key.file);
        let elements = unit.configured.ast.list(elements);
        let referent = *elements
            .last()
            .ok_or_else(|| self.error(key, "missing reference type"))?;
        let lifetime = if elements.len() == 2 {
            elements.first().copied()
        } else {
            None
        };
        let region = self.region(key, lifetime, context)?;
        self.regions.record(context, region);
        self.object_reference_default(
            NodeKey {
                node: referent,
                ..key
            },
            region,
            context,
        );
        self.work.push(Work::Reference(key, region));
        self.node_work(key.file, referent, context);
        Ok(())
    }
    fn signature_syntax(&self, key: NodeKey, owner: Option<DefId>) -> Result<Signature, Error> {
        if let Kind::FunctionType(index) = self.kind(key) {
            let function = self
                .report
                .package
                .unit(key.file)
                .configured
                .ast
                .function_types[index as usize];
            let (parameters, variadic) = fixed_parameters(self.report, key, function.parameters);
            return Ok(Signature {
                parameters,
                result: function.result,
                binders: function.binders,
                unsafe_: function.unsafe_,
                convention: with_variadic(
                    self.report,
                    key,
                    callable_convention(self.report, key, function.abi)?,
                    variadic,
                )?,
            });
        }
        let item = self.item(key)?;
        let ItemKind::Function {
            flags,
            parameters,
            result,
            body,
            ..
        } = item.kind
        else {
            return Err(self.error(key, "expected function signature"));
        };
        self.validate_default_function(key, owner, body)?;
        if flags.asynchronous {
            return Err(self.error(
                key,
                "async or bounded signatures require later semantic assignments",
            ));
        }
        let (unsafe_, convention) = self.foreign_signature(key, owner, item)?;
        let (parameters, variadic) = fixed_parameters(self.report, key, parameters);
        if variadic
            && owner.is_none_or(|id| !self.report.resolution.foreign_owners.contains_key(&id))
        {
            return Err(self.error(key, "variadic definitions require unstable Rust support"));
        }
        Ok(Signature {
            parameters,
            result,
            binders: item.generics,
            unsafe_,
            convention: with_variadic(self.report, key, convention, variadic)?,
        })
    }
    fn declared_regions(&self, key: NodeKey, list: List) -> Result<Vec<DefId>, Error> {
        let unit = self.report.package.unit(key.file);
        let mut declared = Vec::new();
        for mut node in unit.configured.ast.list(list).iter().copied() {
            if !unit.configured.active[node.get() as usize] {
                continue;
            }
            while let Kind::Attributed { value, .. } = unit.configured.ast.node(node).kind {
                node = value;
            }
            let Kind::Generic(index) = unit.configured.ast.node(node).kind else {
                return Err(self.error(key, "expected lifetime parameter"));
            };
            let parameter = unit.configured.ast.generics[index as usize];
            if parameter.flavor != GenericFlavor::Lifetime
                && matches!(
                    self.item(key)?.kind,
                    ItemKind::TypeAlias { .. }
                        | ItemKind::Function { .. }
                        | ItemKind::Struct { .. }
                        | ItemKind::Enum { .. }
                        | ItemKind::Union { .. }
                        | ItemKind::Impl { .. }
                        | ItemKind::Trait { .. }
                        | ItemKind::TraitAlias { .. }
                )
            {
                continue;
            }
            if parameter.flavor != GenericFlavor::Lifetime
                || (parameter.bounds.start != parameter.bounds.end
                    && !matches!(
                        self.item(key).map(|item| item.kind),
                        Ok(ItemKind::Function { .. }
                            | ItemKind::TypeAlias { .. }
                            | ItemKind::Struct { .. }
                            | ItemKind::Enum { .. }
                            | ItemKind::Union { .. }
                            | ItemKind::Impl { .. }
                            | ItemKind::Trait { .. }
                            | ItemKind::TraitAlias { .. })
                    ))
            {
                let message = match self.item(key).map(|item| item.kind) {
                    Ok(ItemKind::TypeAlias { .. }) => {
                        "generic aliases require later semantic assignments"
                    }
                    Ok(
                        ItemKind::Struct { .. } | ItemKind::Enum { .. } | ItemKind::Union { .. },
                    ) => "parameterized aggregates require lifetime or generic assignments",
                    _ => {
                        "type/const parameters and lifetime bounds require later semantic assignments"
                    }
                };
                return Err(self.error(key, message));
            }
            declared.push(
                self.facts
                    .node(NodeKey {
                        file: key.file,
                        node,
                    })
                    .and_then(|facts| facts.binding)
                    .ok_or_else(|| self.error(key, "missing lifetime parameter definition"))?,
            );
        }
        Ok(declared)
    }
    fn parameters(&mut self, key: NodeKey, list: List) -> Result<(), Error> {
        self.children.clear();
        let unit = self.report.package.unit(key.file);
        for node in unit.configured.ast.list(list).iter().copied() {
            if !unit.configured.active[node.get() as usize] {
                continue;
            }
            let key = self.unattributed(NodeKey {
                file: key.file,
                node,
            });
            if let Some(receiver) = self.facts.receivers.get(&key) {
                self.children.push(receiver.ty);
                continue;
            }
            let ty = parameter_type(self.report, key)?;
            self.children.push(self.ty(NodeKey {
                file: key.file,
                node: ty,
            })?);
        }
        Ok(())
    }
    #[expect(
        clippy::too_many_lines,
        reason = "signature dependency phases keep binder ownership, elision and canonical publication together"
    )]
    fn signature(
        &mut self,
        key: NodeKey,
        owner: Option<DefId>,
        phase: SignaturePhase,
        steps: &mut usize,
        limit: usize,
    ) -> Result<(), Error> {
        let signature = self.signature_syntax(key, owner)?;
        match phase {
            SignaturePhase::Begin(parent) => {
                let declared = self.declared_regions(key, signature.binders)?;
                let binder = self.signature_binder(owner, parent.binder, declared);
                if owner.is_none() {
                    self.inherit_alias_scope(key, parent, binder)?;
                }
                self.work
                    .push(Work::Signature(key, owner, SignaturePhase::Output(binder)));
                if let Some(owner) = owner
                    && self.facts.destructors.method(owner).is_some()
                {
                    self.declaration_parameters(
                        owner,
                        key,
                        regions::Context {
                            binder: Some(binder),
                            position: Position::Input,
                        },
                    )?;
                    self.schedule_drop_receiver(key, owner, binder, signature.parameters)?;
                    return Ok(());
                }
                let unit = self.report.package.unit(key.file);
                let first = unit
                    .configured
                    .ast
                    .list(signature.parameters)
                    .iter()
                    .copied()
                    .find(|node| unit.configured.active[node.get() as usize]);
                for node in unit
                    .configured
                    .ast
                    .list(signature.parameters)
                    .iter()
                    .rev()
                    .copied()
                {
                    if unit.configured.active[node.get() as usize] {
                        self.schedule_parameter(
                            NodeKey { node, ..key },
                            owner,
                            binder,
                            Some(node) == first,
                        )?;
                    }
                }
                if let Some(owner) = owner {
                    self.declaration_parameters(
                        owner,
                        key,
                        regions::Context {
                            binder: Some(binder),
                            position: Position::Input,
                        },
                    )?;
                }
            }
            SignaturePhase::Output(binder) => {
                self.work.push(Work::Signature(
                    key,
                    owner,
                    SignaturePhase::Bounds(binder, self.regions.count(binder)),
                ));
                if let Some(result) = signature.result {
                    self.node_work(
                        key.file,
                        result,
                        regions::Context {
                            binder: Some(binder),
                            position: Position::Output,
                        },
                    );
                }
            }
            SignaturePhase::Bounds(binder, input_count) => {
                self.work.push(Work::Signature(
                    key,
                    owner,
                    SignaturePhase::Finish(binder, input_count),
                ));
                if let Some(owner) = owner {
                    self.schedule_bounds(
                        owner,
                        regions::Context {
                            binder: Some(binder),
                            position: Position::Output,
                        },
                    )?;
                }
            }
            SignaturePhase::Finish(binder, input_count) => {
                if let Some(owner) = owner
                    && self.bound_environment_pending(owner)
                {
                    self.work.push(Work::Signature(key, Some(owner), phase));
                    self.work.push(Work::ProjectionEnvironment(
                        owner,
                        regions::Context {
                            binder: Some(binder),
                            position: Position::Output,
                        },
                        false,
                    ));
                    return Ok(());
                }
                let bounds = if owner.is_some() {
                    self.function_bounds(key, binder)?
                } else {
                    Vec::new()
                };
                let bound_regions = self.regions.count(binder);
                self.complete_alias_scope(binder, bound_regions);
                if owner.is_none() && bound_regions != input_count {
                    return Err(self.error(
                        key,
                        "function pointer output lifetime is unconstrained by its inputs",
                    ));
                }
                if owner.is_none()
                    && self.partial_signature(key, signature.parameters, signature.result)?
                {
                    self.facts.partial_annotations.insert(
                        key,
                        super::model::PartialAnnotation::Function { bound_regions },
                    );
                    return Ok(());
                }
                self.parameters(key, signature.parameters)?;
                let parameters = self
                    .facts
                    .types
                    .intern_list(&self.children)
                    .map_err(|e| self.error(key, e))?;
                let result = signature.result.map_or(Ok(TypeId::UNIT), |node| {
                    self.ty(NodeKey {
                        file: key.file,
                        node,
                    })
                })?;
                let ty = self.intern(
                    key,
                    Type::Function {
                        bound_regions,
                        parameters,
                        result,
                        unsafe_: signature.unsafe_,
                        convention: signature.convention,
                    },
                )?;
                let original = self.unreduced_signature(key, ty, &signature)?;
                if owner.is_none() && bound_regions != 0 {
                    crate::types::bound_uses::check_pointer(&self.facts.types, original)
                        .map_err(|e| self.error(key, e))?;
                }
                if let Some(id) = owner {
                    self.finish_function_environment(key, id)?;
                    let empty = self.function_regions(
                        key,
                        id,
                        binder,
                        signature.binders,
                        contracts::SignatureTypes {
                            normal: ty,
                            original,
                        },
                        bounds,
                    )?;
                    if let Some(empty) = empty {
                        self.retain_empty_scheme(empty, steps, limit)?;
                    }
                    self.facts.lifetimes.named.extend(
                        self.regions
                            .bindings(binder)
                            .map_err(|e| self.error(key, e))?,
                    );
                    let ItemKind::Function { body, flags, .. } = self.item(key)?.kind else {
                        return Err(self.error(key, "invalid function owner"));
                    };
                    if flags.constant || self.const_impl_member(id) {
                        self.facts.constant_functions.insert(id);
                    }
                    self.register_foreign_function(id, key, ty, flags.safety)?;
                    self.register_function_call_traits(key, id);
                    self.facts
                        .function_indices
                        .insert(id, self.facts.functions.len());
                    self.facts.functions.push(Function {
                        definition: id,
                        node: key,
                        signature: ty,
                        parameters: signature.parameters,
                        body: body.map(|node| NodeKey {
                            file: key.file,
                            node,
                        }),
                    });
                    self.validate_published_intrinsic(key)?;
                    let item = self.intern(
                        key,
                        Type::FunctionItem {
                            arguments: crate::types::GenericArgs::EMPTY,
                            item: ItemId(id.0),
                            signature: ty,
                        },
                    )?;
                    if original != ty {
                        let original = self.intern(
                            key,
                            Type::FunctionItem {
                                arguments: crate::types::GenericArgs::EMPTY,
                                item: ItemId(id.0),
                                signature: original,
                            },
                        )?;
                        self.facts.unreduced.retain_node(key, original);
                    }
                    self.complete(id, key, item)?;
                    crate::sema::compiler_items::record_call_effects(&mut self.facts, id)
                        .map_err(|e| self.error(key, e))?;
                } else {
                    if original != ty {
                        self.facts.unreduced.retain_node(key, original);
                    }
                    self.store(key, ty)?;
                }
            }
        }
        Ok(())
    }
    fn region(
        &mut self,
        key: NodeKey,
        lifetime: Option<NodeId>,
        context: regions::Context,
    ) -> Result<RegionId, Error> {
        if let Some(node) = lifetime {
            let lifetime = NodeKey {
                file: key.file,
                node,
            };
            let Kind::Name(span, _) = self.kind(lifetime) else {
                return Err(self.error(lifetime, "invalid lifetime syntax"));
            };
            let name = self
                .report
                .package
                .unit(key.file)
                .spelling(span)
                .map_err(|e| self.error(lifetime, e.message))?;
            // A one-name type path is also a `Name`; only a quoted spelling is a lifetime.
            if !name.starts_with('\'') {
                return Err(self.error(lifetime, "invalid lifetime syntax"));
            }
            if name == "'static" {
                return Ok(RegionId::STATIC);
            }
            if name != "'_" {
                let Some(Target::Definition(id)) =
                    self.facts.node(lifetime).and_then(|facts| facts.reference)
                else {
                    return Err(self.error(lifetime, "missing lifetime resolution"));
                };
                return self
                    .regions
                    .named(context.binder, id)
                    .map_err(|e| self.error(lifetime, e));
            }
        }
        if let Position::Local(body) | Position::Constant(body) = context.position {
            if matches!(context.position, Position::Constant(_)) {
                self.facts
                    .constant_annotation_bodies
                    .insert(NodeKey { node: body, ..key });
            }
            let owner = NodeKey {
                file: key.file,
                node: body,
            };
            let region = self
                .facts
                .body_regions
                .allocate(super::model::BodyRegion {
                    body: owner,
                    source: key,
                    origin: super::model::BodyRegionOrigin::Annotation,
                })
                .map_err(|e| self.error(key, e))?;
            // The annotation node's ordinary work charge also covers this
            // constant-time producer index; completion charges each retained ID.
            self.annotation_inputs.record(owner, region);
            return Ok(region);
        }
        self.regions.elided(context).map_err(|e| self.error(key, e))
    }
    fn reference_type(&mut self, key: NodeKey, region: RegionId) -> Result<(), Error> {
        let Kind::Type { flavor, elements } = self.kind(key) else {
            return Err(self.error(key, "invalid reference annotation"));
        };
        let node = *self
            .report
            .package
            .unit(key.file)
            .configured
            .ast
            .list(elements)
            .last()
            .ok_or_else(|| self.error(key, "missing reference child"))?;
        let child = NodeKey {
            file: key.file,
            node,
        };
        if self.facts.partial_annotations.contains_key(&child) {
            self.facts
                .partial_annotations
                .insert(key, super::model::PartialAnnotation::Reference(region));
            return Ok(());
        }
        let referent = self.ty(child)?;
        let ty = self.intern(
            key,
            Type::Reference {
                mutable: flavor == TypeFlavor::RefMut,
                region,
                referent,
            },
        )?;
        self.store(key, ty)
    }
    fn compound(
        &mut self,
        key: NodeKey,
        flavor: TypeFlavor,
        elements: List,
    ) -> Result<TypeId, Error> {
        let unit = self.report.package.unit(key.file);
        let elements = unit.configured.ast.list(elements);
        if flavor == TypeFlavor::Tuple {
            self.children.clear();
            for node in elements {
                self.children.push(self.ty(NodeKey {
                    file: key.file,
                    node: *node,
                })?);
            }
            let list = self
                .facts
                .types
                .intern_list(&self.children)
                .map_err(|e| self.error(key, e))?;
            return self.intern(key, Type::Tuple(list));
        }
        let node = elements
            .first()
            .copied()
            .ok_or_else(|| self.error(key, "missing type child"))?;
        let element = self.ty(NodeKey {
            file: key.file,
            node,
        })?;
        let kind = match flavor {
            TypeFlavor::Slice => Type::Slice(element),
            TypeFlavor::PointerConst | TypeFlavor::PointerMut => Type::Pointer {
                mutable: flavor == TypeFlavor::PointerMut,
                pointee: element,
            },
            TypeFlavor::Array => {
                let length = *elements
                    .get(1)
                    .ok_or_else(|| self.error(key, "missing array length"))?;
                self.array_type(key, element, length)?
            }
            _ => return Err(self.error(key, "unsupported compound annotation")),
        };
        self.intern(key, kind)
    }
}
pub(super) fn parameter_type(report: &Report, mut key: NodeKey) -> Result<NodeId, Error> {
    loop {
        match Declarations::kind(report, key) {
            Kind::Attributed { value, .. } => key.node = value,
            Kind::Parameter { ty: Some(ty), .. } | Kind::FieldDefinition { ty, .. } => {
                return Ok(ty);
            }
            _ => {
                return Err(Declarations::error(
                    report,
                    key,
                    "function parameter requires an explicit type",
                ));
            }
        }
    }
}

pub(super) fn array_length(report: &Report, key: NodeKey) -> Result<u64, Error> {
    let error = |message| Declarations::error(report, key, message);
    let unit = report.package.unit(key.file);
    let Kind::Literal(index) = unit.configured.ast.node(key.node).kind else {
        return Err(error(
            "nonliteral array lengths require constant evaluation",
        ));
    };
    let crate::literal::Value::Integer(value) =
        unit.literal(index).map_err(|e| error(e.message))?
    else {
        return Err(error("array length must be usize"));
    };
    if value
        .suffix
        .is_some_and(|s| s != crate::literal::value::IntegerType::Usize)
    {
        return Err(error("array length must be usize"));
    }
    u64::try_from(value.magnitude).map_err(|_| error("array length overflows usize"))
}

#[cfg(test)]
#[path = "../../tests/invariants/verifier_states.rs"]
mod specialization_tests;

#[cfg(test)]
#[path = "../../tests/invariants/ancestor_dispatch.rs"]
mod member_execution_tests;
