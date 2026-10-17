//! Demand aliases and function signatures through an iterative dependency graph.
mod aggregates;
mod annotations;
mod associated;
mod constants;
mod contracts;
mod deferred_arguments;
mod destructors;
mod evaluation;
mod generics;
mod impl_headers;
mod implicit_methods;
mod lifetimes;
mod method_overlap;
mod method_paths;
mod method_prefix;
mod methods;
mod projections;
mod receivers;
mod trait_paths;
mod trait_regions;
mod traits;
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
    MethodPrefix(NodeKey, regions::Context),
    InherentMethod(NodeKey, associated::Path, regions::Context),
    EvaluateConstant(evaluation::Request, bool),
    InitializerReady(DefId),
    TraitCallable(NodeKey, trait_paths::Callable),
    ImplicitMethod(NodeKey, regions::Context),
    MethodArguments(NodeKey, NodeKey),
    ProjectionPath(NodeKey, regions::Context),
    ImplicitProjection(NodeKey, regions::Context),
    HeaderProjection(NodeKey, regions::Context),
    ProjectionBound(projections::lookup::Seed, bool),
    ProjectionSelection(NodeKey, crate::types::substitute::Request),
    ProjectionResult(NodeKey),
    ProjectionEnvironment(DefId, regions::Context, bool),
    NormalizeBound(NodeKey, regions::Context),
    NominalContract(ItemId, bool),
    Fields(ItemId, bool),
    Constructor(DefId),
    ValueType(DefId),
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
struct Signature {
    parameters: List,
    result: Option<NodeId>,
    binders: List,
    unsafe_: bool,
}
struct Checker<'a> {
    associated_type_bounds: std::collections::HashMap<DefId, traits::AssociatedBounds>,
    associated_bindings: generics::bindings::Pending,
    projections: projections::Pending,
    trait_proofs: trait_regions::Proofs,
    method_trait_environments: Option<
        std::collections::HashMap<NodeKey, std::sync::Arc<super::traits::assumptions::Assumptions>>,
    >,
    method_environments: Option<
        std::collections::HashMap<
            NodeKey,
            std::sync::Arc<crate::types::parameters::environment::Environment>,
        >,
    >,
    nominals: aggregates::Pending,
    defaults: generics::Defaults,
    deferred_bounds: Vec<(DefId, regions::Context)>,
    bound_syntax: std::collections::HashMap<DefId, Vec<generics::bounds::Syntax>>,
    relaxed: std::collections::HashSet<DefId>,
    constant_modules:
        Option<std::collections::HashMap<NodeKey, Option<crate::resolve::package::ModuleId>>>,
    constant_scratch: super::constants::Scratch,
    constant_evaluations: evaluation::Pending,
    constant_dependencies: std::collections::HashSet<NodeKey>,
    constant_captures: generics::Captures,
    parameter_uses: crate::types::parameters::uses::Uses,
    report: &'a Report,
    facts: Declarations,
    states: Vec<State>,
    work: Vec<Work>,
    children: Vec<TypeId>,
    regions: Regions,
    discriminants: super::constants::Discriminants,
    nominal_regions: std::collections::HashMap<ItemId, regions::Context>,
    function_binders: std::collections::HashMap<NodeKey, BinderId>,
    constant_binders: std::collections::HashMap<NodeKey, BinderId>,
    owner_binders: std::collections::HashMap<DefId, BinderId>,
}
/// Resolve declaration types and shape-affecting constants without reparsing source.
///
/// # Errors
/// Rejects recursive aliases, invalid signatures and unsupported annotation semantics.
pub fn analyze(report: &Report) -> Result<Declarations, Error> {
    if !report.names_resolved {
        return Err(Error::at(
            FileId(0),
            0,
            "type checking requires completed name resolution",
        ));
    }
    let mut checker = Checker {
        associated_type_bounds: std::collections::HashMap::new(),
        associated_bindings: generics::bindings::Pending::default(),
        projections: projections::Pending::default(),
        trait_proofs: trait_regions::Proofs::default(),
        method_environments: None,
        method_trait_environments: None,
        nominals: aggregates::Pending::default(),
        defaults: generics::Defaults::default(),
        deferred_bounds: Vec::new(),
        bound_syntax: std::collections::HashMap::new(),
        relaxed: std::collections::HashSet::new(),
        constant_modules: None,
        constant_scratch: super::constants::Scratch::default(),
        constant_evaluations: evaluation::Pending::default(),
        constant_dependencies: std::collections::HashSet::new(),
        constant_captures: generics::Captures::default(),
        parameter_uses: crate::types::parameters::uses::Uses::default(),
        report,
        facts: Declarations::new(report)?,
        states: vec![State::Unseen; report.resolution.definitions.len()],
        work: Vec::new(),
        children: Vec::new(),
        regions: Regions::default(),
        discriminants: super::constants::Discriminants::default(),
        nominal_regions: std::collections::HashMap::new(),
        function_binders: std::collections::HashMap::new(),
        constant_binders: std::collections::HashMap::new(),
        owner_binders: std::collections::HashMap::new(),
    };
    checker.run().map_err(|error| report.diagnostic(error))?;
    Ok(checker.facts)
}
impl Checker<'_> {
    fn run(&mut self) -> Result<(), Error> {
        self.reserve_nominals()?;
        let mut steps = 0_usize;
        // Linear declaration work is bounded by the input graph, not by a fixed
        // whole-crate size. Alias states diagnose cycles independently.
        let work_limit = self
            .report
            .package
            .units
            .iter()
            .fold(self.states.len(), |count, unit| {
                count
                    .saturating_add(unit.configured.ast.nodes.len())
                    .saturating_add(unit.configured.ast.edges.len())
            })
            .saturating_mul(16);
        let destructors = self.reserve_destructors()?;
        self.complete_nominal_headers(&mut steps, work_limit)?;
        self.prepare_trait_owners(&mut steps, work_limit)?;
        self.prepare_destructors(destructors, &mut steps, work_limit)?;
        self.prepare_trait_implementations(&mut steps, work_limit)?;
        self.prepare_inherent_owners(&mut steps, work_limit)?;
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
            if !matches!(
                self.report.resolution.definitions[index].kind,
                DefKind::Primitive
                    | DefKind::Function
                    | DefKind::TypeAlias
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
        self.trait_defaults()?;
        self.validate_destructors()?;
        self.validate_owner_bounds()?;
        self.validate_associated_type_bounds()?;
        self.validate_signatures()?;
        self.validate_trait_implementations()?;
        self.validate_trait_impl_owners()?;
        self.validate_trait_regions()?;
        self.index_inherent_methods()?;
        self.validate_inherent_overlaps()?;
        self.local_annotations(&mut steps, work_limit)?;
        self.complete_alias_normalizations(&mut steps, work_limit)?;
        self.validate_projection_regions()?;
        super::constants::finish_discriminants(
            self.report,
            &mut self.facts,
            &mut self.discriminants,
            &mut self.constant_scratch,
        )?;
        self.complete_layouts()
    }
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
                Work::MethodPrefix(key, context) => {
                    self.complete_method_prefix(key, context, steps, limit, &mut scan)?;
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
                Work::Constructor(id) => self.constructor(id)?,
                Work::ValueType(id) => self.constant_value_type(id),
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
                Work::Signature(key, owner, phase) => self.signature(key, owner, phase)?,
                Work::Receiver(key, owner, binder) => {
                    self.complete_receiver(key, owner, binder)?;
                }
            }
        }
        Ok(())
    }
    fn local_annotations(&mut self, steps: &mut usize, limit: usize) -> Result<(), Error> {
        let mut scan = Vec::new();
        let mut nodes = Vec::new();
        let mut contexts = Vec::new();
        for (index, unit) in self.report.package.units.iter().enumerate() {
            if unit.include_context.is_some() {
                continue;
            }
            let file = FileId(
                u32::try_from(index)
                    .map_err(|_| Error::at(FileId(0), 0, "too many source files"))?,
            );
            let mut seen = vec![false; unit.configured.ast.nodes.len()];
            nodes.push(unit.configured.ast.root);
            contexts.push(regions::Context::ALIAS);
            while let Some(node) = nodes.pop() {
                let inherited = contexts.pop().ok_or_else(|| {
                    self.error(NodeKey { file, node }, "missing annotation context")
                })?;
                let context = self.local_context(NodeKey { file, node }, inherited);
                if !unit.configured.active[node.0 as usize]
                    || std::mem::replace(&mut seen[node.0 as usize], true)
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
                let mark = nodes.len();
                unit.configured.ast.push_children(node, &mut nodes);
                nodes[mark..].reverse();
                contexts.resize(nodes.len(), context);
                if let Kind::GenericMethodCall { method, .. } = unit.configured.ast.node(node).kind
                {
                    self.prepare_method_arguments(
                        NodeKey { file, node },
                        method,
                        context,
                        steps,
                        limit,
                        &mut scan,
                    )?;
                }
                if self.schedule_method_path(
                    NodeKey { file, node },
                    context,
                    steps,
                    limit,
                    &mut scan,
                )? {
                    self.drain(steps, limit)?;
                    continue;
                }
                if self.schedule_alias_variant(
                    NodeKey { file, node },
                    context,
                    steps,
                    limit,
                    &mut scan,
                )? {
                    self.drain(steps, limit)?;
                }
                if matches!(unit.configured.ast.node(node).kind, Kind::Path { .. })
                    && let Some(owner) = self.generic_use_owner(NodeKey { file, node })
                {
                    self.schedule_item_arguments(
                        NodeKey { file, node },
                        owner,
                        context,
                        steps,
                        limit,
                        &mut scan,
                    )?;
                    self.drain(steps, limit)?;
                }
                let (Kind::Let { ty: Some(ty), .. } | Kind::Cast { ty, .. }) =
                    unit.configured.ast.node(node).kind
                else {
                    continue;
                };
                // Holes stay in the owning body, while the ordinary elaborator
                // resolves every written lifetime and nested callable binder.
                self.annotation_holes(NodeKey { file, node: ty }, steps, limit, &mut scan)?;
                self.node_work(file, ty, context);
                self.drain(steps, limit)?;
            }
        }
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
    fn definition(&mut self, id: DefId, finish: bool) -> Result<(), Error> {
        if self.states[id.0 as usize] == State::Complete {
            return Ok(());
        }
        let definition = &self.report.resolution.definitions[id.0 as usize];
        if definition.kind == DefKind::Primitive {
            let name = self.report.package.names.get(definition.name);
            let primitive = Primitive::ALL
                .into_iter()
                .find(|p| p.name() == name)
                .ok_or_else(|| Error::at(FileId(0), 0, "unknown primitive definition"))?;
            self.facts.definitions[id.0 as usize] = Some(TypeId::primitive(primitive));
            self.states[id.0 as usize] = State::Complete;
            return Ok(());
        }
        let key = definition
            .node
            .ok_or_else(|| Error::at(FileId(0), 0, "type declaration has no syntax"))?;
        let item = self.item(key)?;
        if matches!(item.kind, ItemKind::Struct { .. } | ItemKind::Enum { .. }) {
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
        self.facts.initializers.push(Initializer {
            type_checked: false,
            definition: id,
            node: key,
            value: NodeKey {
                file: key.file,
                node: value,
            },
            ty,
            kind,
        });
        Ok(())
    }
    fn complete(&mut self, id: DefId, key: NodeKey, ty: TypeId) -> Result<(), Error> {
        self.facts.definitions[id.0 as usize] = Some(ty);
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
        if !finish {
            if matches!(kind, Kind::Path { .. } | Kind::Qualified { .. })
                && self.schedule_implicit_projection(key, context)?
            {
                return Ok(());
            }
            if matches!(kind, Kind::Qualified { .. }) {
                return self.schedule_projection_path(key, context);
            }
            if let Kind::FunctionType(_) = kind {
                self.work
                    .push(Work::Signature(key, None, SignaturePhase::Begin(context)));
                return Ok(());
            }
            if let Kind::Type {
                flavor: TypeFlavor::Ref | TypeFlavor::RefMut,
                elements,
            } = kind
            {
                self.schedule_reference(key, elements, context)?;
                return Ok(());
            }
            self.work.push(Work::Node(key, true, context));
            match kind {
                Kind::NeverType => {}
                Kind::Group(node) | Kind::Attributed { value: node, .. } => {
                    self.node_work(key.file, node, context);
                }
                Kind::Path { .. } | Kind::Name(_) => {
                    self.schedule_type_path(key, context)?;
                }
                Kind::Type { flavor, elements } => {
                    let unit = self.report.package.unit(key.file);
                    let elements = unit.configured.ast.list(elements);
                    let count = elements
                        .len()
                        .saturating_sub(usize::from(flavor == TypeFlavor::Array));
                    for child in elements[..count].iter().rev().copied() {
                        self.node_work(key.file, child, context);
                    }
                }
                Kind::InferType => {
                    return Err(self.error(
                        key,
                        "inference placeholders are not allowed in item signatures",
                    ));
                }
                _ => return Err(self.error(key, "unsupported type annotation")),
            }
            return Ok(());
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
            Kind::Path { .. } | Kind::Name(_) => {
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
                | DefKind::Struct
                | DefKind::Enum
                | DefKind::Generic
        ) || (kind == DefKind::SelfType && self.facts.associated.schema(id).is_some()))
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
        self.work.push(Work::Reference(key, region));
        self.node_work(key.file, referent, context);
        Ok(())
    }
    fn signature_syntax(&self, key: NodeKey) -> Result<Signature, Error> {
        if let Kind::FunctionType(index) = self.kind(key) {
            let function = self
                .report
                .package
                .unit(key.file)
                .configured
                .ast
                .function_types[index as usize];
            if !rust_abi(self.report, key, function.abi)? {
                return Err(
                    self.error(key, "foreign signatures require later semantic assignments")
                );
            }
            return Ok(Signature {
                parameters: function.parameters,
                result: function.result,
                binders: function.binders,
                unsafe_: function.unsafe_,
            });
        }
        let item = self.item(key)?;
        let ItemKind::Function {
            flags,
            abi,
            parameters,
            result,
            ..
        } = item.kind
        else {
            return Err(self.error(key, "expected function signature"));
        };
        if !rust_abi(self.report, key, abi)? || flags.asynchronous {
            return Err(self.error(
                key,
                "foreign, async or bounded signatures require later semantic assignments",
            ));
        }
        Ok(Signature {
            parameters,
            result,
            binders: item.generics,
            unsafe_: flags.safety == Safety::Unsafe,
        })
    }
    fn declared_regions(&self, key: NodeKey, list: List) -> Result<Vec<DefId>, Error> {
        let unit = self.report.package.unit(key.file);
        let mut declared = Vec::new();
        for mut node in unit.configured.ast.list(list).iter().copied() {
            if !unit.configured.active[node.0 as usize] {
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
                        | ItemKind::Impl { .. }
                        | ItemKind::Trait { .. }
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
                            | ItemKind::Impl { .. }
                            | ItemKind::Trait { .. })
                    ))
            {
                let message = match self.item(key).map(|item| item.kind) {
                    Ok(ItemKind::TypeAlias { .. }) => {
                        "generic aliases require later semantic assignments"
                    }
                    Ok(ItemKind::Struct { .. } | ItemKind::Enum { .. }) => {
                        "parameterized aggregates require lifetime or generic assignments"
                    }
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
            if !unit.configured.active[node.0 as usize] {
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
    ) -> Result<(), Error> {
        let signature = self.signature_syntax(key)?;
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
                    .find(|node| unit.configured.active[node.0 as usize]);
                for node in unit
                    .configured
                    .ast
                    .list(signature.parameters)
                    .iter()
                    .rev()
                    .copied()
                {
                    if unit.configured.active[node.0 as usize] {
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
                    },
                )?;
                let original = self.unreduced_signature(key, ty, &signature)?;
                if owner.is_none() && bound_regions != 0 {
                    crate::types::bound_uses::check_pointer(&self.facts.types, original)
                        .map_err(|e| self.error(key, e))?;
                }
                if let Some(id) = owner {
                    self.finish_function_environment(key, id)?;
                    self.function_regions(
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
                    self.facts.lifetimes.named.extend(
                        self.regions
                            .bindings(binder)
                            .map_err(|e| self.error(key, e))?,
                    );
                    let ItemKind::Function { body, .. } = self.item(key)?.kind else {
                        return Err(self.error(key, "invalid function owner"));
                    };
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
            let Kind::Name(span) = self.kind(lifetime) else {
                return Err(self.error(lifetime, "invalid lifetime syntax"));
            };
            let name = self
                .report
                .package
                .unit(key.file)
                .spelling(span)
                .map_err(|e| self.error(lifetime, e.message))?;
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
            let ordinal = u32::try_from(self.facts.body_regions.len())
                .ok()
                .and_then(|n| n.checked_add(1))
                .filter(|n| *n < 0x8000_0000)
                .ok_or_else(|| self.error(key, "body region identity space exhausted"))?;
            self.facts.body_regions.push(super::model::BodyRegion {
                body: NodeKey {
                    file: key.file,
                    node: body,
                },
                source: key,
                origin: super::model::BodyRegionOrigin::Annotation,
            });
            return Ok(RegionId(ordinal));
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
            TypeFlavor::Path => return Ok(element),
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

/// Explicit Rust ABI spellings use the same calling convention as omission.
fn rust_abi(report: &Report, key: NodeKey, abi: Option<NodeId>) -> Result<bool, Error> {
    let Some(node) = abi else { return Ok(true) };
    let key = NodeKey { node, ..key };
    let unit = report.package.unit(key.file);
    let Kind::Literal(index) = unit.configured.ast.node(node).kind else {
        return Ok(false);
    };
    let value = unit
        .literal(index)
        .map_err(|e| Declarations::error(report, key, e.message))?;
    Ok(matches!(value, crate::literal::Value::String(bytes) if bytes.as_ref() == b"Rust"))
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
