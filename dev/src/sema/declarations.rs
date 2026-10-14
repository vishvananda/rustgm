//! Demand aliases and function signatures through an iterative dependency graph.
mod aggregates;
mod annotations;
mod contracts;
mod destructors;
mod generics;
mod lifetimes;
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
    Begin(Option<BinderId>),
    Output(BinderId),
    Finish(BinderId, u32),
}
#[derive(Clone, Copy)]
enum Work {
    Bounds(DefId),
    FunctionArguments(NodeKey, DefId),
    Generic(generics::Declaration),
    PathArguments(NodeKey, DefId, regions::Context),
    Definition(DefId, bool),
    Node(NodeKey, bool, regions::Context),
    Reference(NodeKey, RegionId),
    Parameter(BinderId, bool),
    Signature(NodeKey, Option<DefId>, SignaturePhase),
    DropReceiver(NodeKey, DefId, BinderId),
}
struct Signature {
    parameters: List,
    result: Option<NodeId>,
    binders: List,
    unsafe_: bool,
}
struct Checker<'a> {
    bound_syntax: std::collections::HashMap<DefId, Vec<generics::bounds::Syntax>>,
    relaxed: std::collections::HashSet<DefId>,
    constant_modules:
        Option<std::collections::HashMap<NodeKey, Option<crate::resolve::package::ModuleId>>>,
    constant_scratch: super::constants::Scratch,
    constant_dependencies: std::collections::HashSet<NodeKey>,
    parameter_uses: crate::types::parameters::uses::Uses,
    report: &'a Report,
    facts: Declarations,
    states: Vec<State>,
    work: Vec<Work>,
    children: Vec<TypeId>,
    regions: Regions,
    discriminants: Vec<super::constants::Discriminant>,
    nominal_regions: std::collections::HashMap<ItemId, regions::Context>,
    function_binders: std::collections::HashMap<NodeKey, BinderId>,
    drop_binders: std::collections::HashMap<DefId, BinderId>,
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
        bound_syntax: std::collections::HashMap::new(),
        relaxed: std::collections::HashSet::new(),
        constant_modules: None,
        constant_scratch: super::constants::Scratch::default(),
        constant_dependencies: std::collections::HashSet::new(),
        parameter_uses: crate::types::parameters::uses::Uses::default(),
        report,
        facts: Declarations::new(report)?,
        states: vec![State::Unseen; report.resolution.definitions.len()],
        work: Vec::new(),
        children: Vec::new(),
        regions: Regions::default(),
        discriminants: Vec::new(),
        nominal_regions: std::collections::HashMap::new(),
        function_binders: std::collections::HashMap::new(),
        drop_binders: std::collections::HashMap::new(),
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
        self.prepare_destructors(&mut steps, work_limit)?;
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
        self.complete_nominals(&mut steps, work_limit)?;
        self.validate_destructors()?;
        self.validate_signatures()?;
        self.local_annotations(&mut steps, work_limit)?;
        super::constants::discriminants(self.report, &mut self.facts, &self.discriminants)?;
        self.complete_layouts()
    }
    fn drain(&mut self, steps: &mut usize, limit: usize) -> Result<(), Error> {
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
                Work::Bounds(owner) => self.complete_bounds(owner)?,
                Work::FunctionArguments(key, owner) => {
                    self.complete_function_arguments(key, owner)?;
                }
                Work::Generic(declaration) => self.generic_declaration(declaration)?,
                Work::PathArguments(key, id, context) => self.path_arguments(key, id, context)?,
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
                Work::DropReceiver(key, owner, binder) => {
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
                if matches!(unit.configured.ast.node(node).kind, Kind::Path { .. })
                    && let Some(Target::Definition(owner)) = self
                        .facts
                        .node(NodeKey { file, node })
                        .and_then(|f| f.reference)
                    && self.report.resolution.definitions[owner.0 as usize].kind
                        == DefKind::Function
                    && self.facts.generics.parameters.contains_key(&owner)
                {
                    self.schedule_function_arguments(
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
    fn validate_signatures(&mut self) -> Result<(), Error> {
        for index in 0..self.facts.functions.len() {
            let function = self.facts.functions[index];
            let mut shapes = self
                .facts
                .generics
                .environments
                .get(&function.definition)
                .filter(|e| e.has_shape_assumptions())
                .map(|e| crate::types::parameters::shape::Shapes::new(std::sync::Arc::clone(e)));
            self.validate_bounds(function.definition, &mut shapes)?;
            let Some(Type::Function {
                parameters, result, ..
            }) = self.facts.types.kind(function.signature)
            else {
                return Err(self.error(function.node, "invalid function signature"));
            };
            for ty in self
                .facts
                .types
                .list(parameters)
                .ok_or_else(|| self.error(function.node, "invalid parameters"))?
                .iter()
                .copied()
                .chain(std::iter::once(result))
            {
                let shape = if let Some(shapes) = &mut shapes {
                    shapes
                        .demand(&self.facts.types, ty)
                        .map_err(|e| self.error(function.node, e))?
                } else {
                    crate::types::parameters::shape::Properties {
                        sized: self.facts.types.is_sized(ty) == Some(true),
                        well_formed: self.facts.types.is_well_formed(ty) == Some(true),
                    }
                };
                if !shape.well_formed {
                    return Err(self.error(
                        function.node,
                        "function signature contains an ill-formed type",
                    ));
                }
                if !shape.sized {
                    return Err(self.error(
                        function.node,
                        "function parameters and result must be sized",
                    ));
                }
            }
        }
        for initializer in &self.facts.initializers {
            if self.facts.types.is_well_formed(initializer.ty) != Some(true)
                || self.facts.types.is_sized(initializer.ty) != Some(true)
            {
                return Err(self.error(
                    initializer.node,
                    "constant and static types must be well-formed and sized",
                ));
            }
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
        if !finish {
            if self.states[id.0 as usize] == State::Active {
                return Err(self.error(key, "recursive type alias"));
            }
            self.states[id.0 as usize] = State::Active;
            if matches!(item.kind, ItemKind::Function { .. }) {
                self.work
                    .push(Work::Signature(key, Some(id), SignaturePhase::Begin(None)));
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
                ItemKind::Const { ty, .. } | ItemKind::Static { ty, .. } => {
                    self.node_work(key.file, ty, regions::Context::STATIC);
                }
                _ => return Err(self.error(key, "unsupported type declaration")),
            }
            return Ok(());
        }
        let ty = match item.kind {
            ItemKind::TypeAlias { ty: Some(ty), .. }
            | ItemKind::Const { ty, .. }
            | ItemKind::Static { ty, .. } => self.ty(NodeKey {
                file: key.file,
                node: ty,
            })?,
            _ => return Err(self.error(key, "invalid completed declaration")),
        };
        self.complete(id, key, ty)?;
        if matches!(item.kind, ItemKind::TypeAlias { .. }) {
            self.validate_alias(id, key, ty)?;
        }
        let (value, kind) = match item.kind {
            ItemKind::Const { value, .. } => (value, InitializerKind::Const),
            ItemKind::Static { value, mutable, .. } => (value, InitializerKind::Static { mutable }),
            _ => return Ok(()),
        };
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
            if let Kind::FunctionType(_) = kind {
                self.work.push(Work::Signature(
                    key,
                    None,
                    SignaturePhase::Begin(context.binder),
                ));
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
                self.path_type(key, id, context)?
            }
            Kind::Type { flavor, elements } => self.compound(key, flavor, elements)?,
            _ => return Err(self.error(key, "invalid completed annotation")),
        };
        self.store(key, ty)
    }
    fn schedule_type_path(&mut self, key: NodeKey, context: regions::Context) -> Result<(), Error> {
        let Some(Target::Definition(id)) = self.facts.node(key).and_then(|facts| facts.reference)
        else {
            return Err(self.error(key, "type path needs a resolved declaration"));
        };
        let kind = self.report.resolution.definitions[id.0 as usize].kind;
        if kind == DefKind::Generic
            && !self.facts.definitions[id.0 as usize]
                .is_some_and(|ty| matches!(self.facts.types.kind(ty), Some(Type::Parameter(_))))
        {
            return Err(self.error(key, "const parameter cannot be used as a type"));
        }
        if !matches!(
            kind,
            DefKind::Primitive
                | DefKind::TypeAlias
                | DefKind::Struct
                | DefKind::Enum
                | DefKind::Generic
        ) && self.facts.destructors.self_type(id).is_none()
        {
            return Err(self.error(
                key,
                "nominal or associated types require later semantic assignments",
            ));
        }
        if kind == DefKind::TypeAlias
            && crate::sema::lifetimes::arguments(self.report, key, false)?.is_some()
        {
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
                    ItemKind::TypeAlias { .. } | ItemKind::Function { .. }
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
                            | ItemKind::Impl { .. })
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
            if let Some(receiver) = self.facts.destructors.receivers.get(&key) {
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
                let binder = self.signature_binder(owner, parent, declared);
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
                    self.schedule_receiver(key, owner, binder, signature.parameters)?;
                    return Ok(());
                }
                let unit = self.report.package.unit(key.file);
                for node in unit
                    .configured
                    .ast
                    .list(signature.parameters)
                    .iter()
                    .rev()
                    .copied()
                {
                    if !unit.configured.active[node.0 as usize] {
                        continue;
                    }
                    let ty = parameter_type(
                        self.report,
                        NodeKey {
                            file: key.file,
                            node,
                        },
                    )?;
                    self.work.push(Work::Parameter(binder, true));
                    self.node_work(
                        key.file,
                        ty,
                        regions::Context {
                            binder: Some(binder),
                            position: Position::Input,
                        },
                    );
                    self.work.push(Work::Parameter(binder, false));
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
                    SignaturePhase::Finish(binder, self.regions.count(binder)),
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
            SignaturePhase::Finish(binder, input_count) => {
                let bounds = if owner.is_some() {
                    self.function_bounds(key, binder)?
                } else {
                    Vec::new()
                };
                let bound_regions = self.regions.count(binder);
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
                if owner.is_none() && bound_regions != 0 {
                    crate::types::bound_uses::check_pointer(&self.facts.types, ty)
                        .map_err(|e| self.error(key, e))?;
                }
                if let Some(id) = owner {
                    self.facts
                        .generics
                        .complete_environment(&self.facts.types, id)
                        .map_err(|e| self.error(key, e))?;
                    self.function_regions(key, id, binder, signature.binders, ty, bounds)?;
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
                    self.complete(id, key, item)?;
                } else {
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
        if let Position::Local(body) = context.position {
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
