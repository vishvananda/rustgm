//! Demand aliases and function signatures through an iterative dependency graph.
use super::{
    model::{Declarations, Function},
    regions::{self, BinderId, Position, Regions},
};
use crate::{
    literal::Value,
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
    Definition(DefId, bool),
    Node(NodeKey, bool, regions::Context),
    Reference(NodeKey, RegionId),
    Parameter(BinderId, bool),
    Signature(NodeKey, Option<DefId>, SignaturePhase),
}
struct Signature {
    parameters: List,
    result: Option<NodeId>,
    binders: List,
    unsafe_: bool,
}
struct Checker<'a> {
    report: &'a Report,
    facts: Declarations,
    states: Vec<State>,
    work: Vec<Work>,
    children: Vec<TypeId>,
    regions: Regions,
}
/// Resolve declaration types without checking bodies or reparsing source.
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
        report,
        facts: Declarations::new(report)?,
        states: vec![State::Unseen; report.resolution.definitions.len()],
        work: Vec::new(),
        children: Vec::new(),
        regions: Regions::default(),
    };
    checker.run().map_err(|error| report.diagnostic(error))?;
    Ok(checker.facts)
}
impl Checker<'_> {
    fn run(&mut self) -> Result<(), Error> {
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
            while let Some(work) = self.work.pop() {
                steps += 1;
                if steps > work_limit {
                    return Err(Error::at(
                        FileId(0),
                        0,
                        "declaration type work budget exceeded",
                    ));
                }
                match work {
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
                }
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
            if item.generics.start != item.generics.end
                || item.where_clause.start != item.where_clause.end
            {
                return Err(self.error(key, "generic aliases require later semantic assignments"));
            }
            self.work.push(Work::Definition(id, true));
            match item.kind {
                ItemKind::TypeAlias { ty: Some(ty), .. }
                | ItemKind::Const { ty, .. }
                | ItemKind::Static { ty, .. } => {
                    self.node_work(key.file, ty, regions::Context::ALIAS);
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
        self.complete(id, key, ty)
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
                return Ok(());
            }
            self.work.push(Work::Node(key, true, context));
            match kind {
                Kind::NeverType => {}
                Kind::Group(node) | Kind::Attributed { value: node, .. } => {
                    self.node_work(key.file, node, context);
                }
                Kind::Path { .. } | Kind::Name(_) => {
                    let Some(Target::Definition(id)) =
                        self.facts.node(key).and_then(|facts| facts.reference)
                    else {
                        return Err(self.error(key, "type path needs a resolved declaration"));
                    };
                    if !matches!(
                        self.report.resolution.definitions[id.0 as usize].kind,
                        DefKind::Primitive | DefKind::TypeAlias
                    ) {
                        return Err(self.error(
                            key,
                            "nominal or associated types require later semantic assignments",
                        ));
                    }
                    self.work.push(Work::Definition(id, false));
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
                self.facts.definitions[id.0 as usize]
                    .ok_or_else(|| self.error(key, "missing resolved declaration type"))?
            }
            Kind::Type { flavor, elements } => self.compound(key, flavor, elements)?,
            _ => return Err(self.error(key, "invalid completed annotation")),
        };
        self.store(key, ty)
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
            if function.abi.is_some() {
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
        if abi.is_some() || flags.asynchronous || item.where_clause.start != item.where_clause.end {
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
                || parameter.bounds.start != parameter.bounds.end
            {
                return Err(self.error(
                    key,
                    "type/const parameters and lifetime bounds require later semantic assignments",
                ));
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
            let ty = parameter_type(
                self.report,
                NodeKey {
                    file: key.file,
                    node,
                },
            )?;
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
                let binder = self
                    .regions
                    .binder(parent, self.declared_regions(key, signature.binders)?);
                self.work
                    .push(Work::Signature(key, owner, SignaturePhase::Output(binder)));
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
            }
            SignaturePhase::Output(binder) => {
                self.work.push(Work::Signature(
                    key,
                    owner,
                    SignaturePhase::Finish(binder, self.regions.count(binder)),
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
            SignaturePhase::Finish(binder, input_count) => {
                let bound_regions = self.regions.count(binder);
                if bound_regions != input_count {
                    return Err(self.error(
                        key,
                        if owner.is_some() {
                            "return-only lifetime parameters require lifetime contracts"
                        } else {
                            "function pointer output lifetime is unconstrained by its inputs"
                        },
                    ));
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
                if let Some(id) = owner {
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
        let referent = self.ty(NodeKey {
            file: key.file,
            node,
        })?;
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
                let Kind::Literal(index) = unit.configured.ast.node(length).kind else {
                    return Err(
                        self.error(key, "nonliteral array lengths require constant evaluation")
                    );
                };
                let Value::Integer(value) = unit
                    .literal(index)
                    .map_err(|e| self.error(key, e.message))?
                else {
                    return Err(self.error(key, "array length must be usize"));
                };
                if value
                    .suffix
                    .is_some_and(|suffix| suffix != crate::literal::value::IntegerType::Usize)
                {
                    return Err(self.error(key, "array length must be usize"));
                }
                Type::Array {
                    element,
                    length: u64::try_from(value.magnitude)
                        .map_err(|_| self.error(key, "array length overflows usize"))?,
                }
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
