//! A declaration pass creates scopes and forward-visible items before resolving uses.
use super::{
    model::{
        Access, Binding, DefId, DefKind, Definition, Import, Namespace, Origin, Resolution, Scope,
        ScopeId,
    },
    names::NameId,
    package::{Error, ModuleId, Package},
    unit::{FileId, NodeKey},
};
use crate::syntax::{
    ast::{Kind, List, NodeId},
    item::{GenericFlavor, Item, ItemKind, Shape, Visibility},
};
use std::collections::HashMap;
#[derive(Clone, Copy)]
pub(crate) struct Context {
    pub module: ModuleId,
    pub file: FileId,
    pub scope: ScopeId,
}
#[derive(Default)]
pub(crate) struct State {
    pub facts: Resolution,
    pub hygiene: Option<super::hygiene::Hygiene>,
    scratch: Vec<NodeId>,
    pending: Vec<(Context, NodeId)>,
    visits: Vec<(Context, NodeId, Namespace)>,
}
impl State {
    pub fn bind<'a>(&'a mut self, package: &'a mut Package) -> Resolver<'a> {
        Resolver {
            package,
            facts: &mut self.facts,
            hygiene: &mut self.hygiene,
            scratch: &mut self.scratch,
            pending: &mut self.pending,
            visits: &mut self.visits,
        }
    }
}
pub(crate) struct Resolver<'a> {
    pub package: &'a mut Package,
    pub facts: &'a mut Resolution,
    pub hygiene: &'a mut Option<super::hygiene::Hygiene>,
    pub scratch: &'a mut Vec<NodeId>,
    pub pending: &'a mut Vec<(Context, NodeId)>,
    pub visits: &'a mut Vec<(Context, NodeId, Namespace)>,
}
impl Resolver<'_> {
    pub fn collect(&mut self) -> Result<(), Error> {
        self.facts
            .module_scopes
            .resize(self.package.modules.len(), ScopeId(0));
        if self.facts.scopes.is_empty() {
            self.new_scope(None, ModuleId(0), true)?;
        }
        for spelling in [
            "bool", "char", "str", "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32",
            "u64", "u128", "usize", "f32", "f64",
        ] {
            let name = self
                .package
                .names
                .intern(spelling)
                .map_err(|e| Error::at(FileId(0), 0, e))?;
            let id = DefId(
                u32::try_from(self.facts.definitions.len())
                    .map_err(|_| Error::at(FileId(0), 0, "too many definitions"))?,
            );
            self.facts.definitions.push(Definition {
                name,
                namespace: Namespace::Type,
                kind: DefKind::Primitive,
                scope: ScopeId(0),
                node: None,
                access: Access::Public,
                members: None,
            });
            self.facts.primitives.insert(name, id);
        }
        self.module(ModuleId(0), ScopeId(0));
        while let Some((c, node)) = self.pending.pop() {
            self.walk_inner(c, node)?;
        }
        Ok(())
    }
    pub fn context(&self, module: ModuleId, scope: ScopeId) -> Context {
        Context {
            module,
            scope,
            file: self.package.modules[module.0 as usize].file,
        }
    }
    pub fn error(&self, context: Context, node: NodeId, message: impl Into<String>) -> Error {
        Error::at(
            context.file,
            self.package
                .unit(context.file)
                .configured
                .ast
                .node(node)
                .span
                .start,
            message,
        )
    }
    pub fn new_scope(
        &mut self,
        parent: Option<ScopeId>,
        module: ModuleId,
        barrier: bool,
    ) -> Result<ScopeId, Error> {
        let id = ScopeId(
            u32::try_from(self.facts.scopes.len())
                .map_err(|_| Error::at(FileId(0), 0, "too many scopes"))?,
        );
        self.facts.scopes.push(Scope {
            parent,
            module,
            barrier,
            associated: false,
            trait_access: None,
            nominal_self: None,
            label_barrier: barrier,
            bindings: HashMap::new(),
            order: Vec::new(),
        });
        Ok(id)
    }
    fn node_scope(&mut self, c: Context, node: NodeId, barrier: bool) -> Result<ScopeId, Error> {
        if self.hygiene.is_some()
            && let Some(scope) = self.facts.node_scopes.get(&(c.module, node)).copied()
        {
            let frame = &mut self.facts.scopes[scope.0 as usize];
            frame.parent = Some(c.scope);
            frame.barrier = barrier;
            frame.label_barrier = barrier;
            return Ok(scope);
        }
        self.new_scope(Some(c.scope), c.module, barrier)
    }
    pub fn define(&mut self, definition: Definition, shadow: bool) -> Result<DefId, Error> {
        let id = DefId(
            u32::try_from(self.facts.definitions.len())
                .map_err(|_| Error::at(FileId(0), 0, "too many definitions"))?,
        );
        if self.hygiene.is_some() && super::hygiene::Hygiene::is_local(definition.kind) {
            let key = definition
                .node
                .ok_or_else(|| Error::at(FileId(0), 0, "local binding has no source"))?;
            let offset = super::hygiene::name_offset(self.package, key);
            let context = self.package.expansion_context(key.file, offset)?;
            if let Some(hygiene) = self.hygiene {
                hygiene
                    .define(&definition, id, context, shadow)
                    .map_err(|e| Error::at(key.file, offset, e))?;
            }
            self.facts.definitions.push(definition);
            return Ok(id);
        }
        let scope = &mut self.facts.scopes[definition.scope.0 as usize];
        let key = (definition.name, definition.namespace);
        if scope
            .bindings
            .get(&key)
            .is_some_and(|binding| !matches!(binding.origin, Origin::Glob(_)))
            && !shadow
        {
            let node = definition.node;
            return Err(Error::at(
                node.map_or(FileId(0), |n| n.file),
                node.map_or(0, |n| {
                    self.package
                        .unit(n.file)
                        .configured
                        .ast
                        .node(n.node)
                        .span
                        .start
                }),
                format!(
                    "duplicate {} name `{}`",
                    definition.namespace.name(),
                    self.package.names.get(definition.name)
                ),
            ));
        }
        if !scope.bindings.contains_key(&key) {
            scope.order.push(key);
        }
        scope.bindings.insert(
            key,
            Binding {
                target: id,
                access: definition.access,
                origin: Origin::Direct,
                ambiguous: false,
            },
        );
        self.facts.definitions.push(definition);
        Ok(id)
    }
    fn module(&mut self, module: ModuleId, scope: ScopeId) {
        let context = self.context(module, scope);
        let root = self.package.modules[module.0 as usize].root;
        self.facts.module_scopes[module.0 as usize] = scope;
        if let Kind::Item(index) = self.kind(context, root)
            && let ItemKind::Module { items: Some(items) } =
                self.package.unit(context.file).configured.ast.items[index as usize].kind
        {
            self.list(context, items);
            return;
        }
        self.pending.push((context, root));
    }
    pub fn kind(&self, c: Context, node: NodeId) -> Kind {
        self.package.unit(c.file).configured.ast.node(node).kind
    }
    pub fn active(&self, c: Context, node: NodeId) -> bool {
        self.package.unit(c.file).configured.active[node.0 as usize]
    }
    fn list(&mut self, c: Context, list: List) {
        for offset in (list.start..list.end).rev() {
            let node = self.package.unit(c.file).configured.ast.edges[offset as usize];
            self.pending.push((c, node));
        }
    }
    fn walk_inner(&mut self, mut c: Context, node: NodeId) -> Result<(), Error> {
        if !self.active(c, node) {
            return Ok(());
        }
        if let Kind::If {
            condition,
            then,
            otherwise,
        } = self.kind(c, node)
        {
            let scope = self.node_scope(c, node, false)?;
            self.facts.node_scopes.insert((c.module, node), scope);
            let condition_context = Context { scope, ..c };
            if let Some(otherwise) = otherwise {
                self.pending.push((c, otherwise));
            }
            self.pending.push((condition_context, then));
            self.pending.push((condition_context, condition));
            return Ok(());
        }
        match self.kind(c, node) {
            Kind::Item(index) => return self.item(c, node, index),
            Kind::Block { .. }
            | Kind::Closure { .. }
            | Kind::Arm { .. }
            | Kind::For { .. }
            | Kind::If { .. }
            | Kind::While { .. }
            | Kind::Label { .. }
            | Kind::Bound { .. }
            | Kind::Where { .. }
            | Kind::FunctionType(_) => {
                c.scope = self.node_scope(c, node, false)?;
                self.facts.scopes[c.scope.0 as usize].label_barrier =
                    matches!(self.kind(c, node), Kind::Closure { .. });
                self.facts.node_scopes.insert((c.module, node), c.scope);
            }
            Kind::Generic(index) => {
                let generic = self.package.unit(c.file).configured.ast.generics[index as usize];
                let name = self.package.name(c.file, generic.name)?;
                let namespace = match generic.flavor {
                    GenericFlavor::Lifetime => Namespace::Lifetime,
                    GenericFlavor::Type => Namespace::Type,
                    GenericFlavor::Const => Namespace::Value,
                };
                self.define(
                    Definition {
                        name,
                        namespace,
                        kind: DefKind::Generic,
                        scope: c.scope,
                        node: Some(NodeKey { file: c.file, node }),
                        access: Access::Within(c.module),
                        members: None,
                    },
                    false,
                )?;
            }
            Kind::Macro { .. } => {
                self.facts
                    .expansions
                    .push((c.scope, NodeKey { file: c.file, node }));
                return Ok(());
            }
            _ => {}
        }
        let mark = self.scratch.len();
        self.package
            .unit(c.file)
            .configured
            .ast
            .push_children(node, self.scratch);
        let end = self.scratch.len();
        for index in (mark..end).rev() {
            self.pending.push((c, self.scratch[index]));
        }
        self.scratch.truncate(mark);
        Ok(())
    }
    fn item(&mut self, c: Context, node: NodeId, index: u32) -> Result<(), Error> {
        let item = self.package.unit(c.file).configured.ast.items[index as usize];
        let access = self.visibility(c, item.visibility)?;
        if let Some(hygiene) = self.hygiene {
            if matches!(item.kind, ItemKind::MacroRules { .. }) {
                hygiene.scope(c.module, node, c.scope);
                self.facts.node_scopes.insert((c.module, node), c.scope);
                return Ok(());
            }
            if matches!(item.kind, ItemKind::Use(_)) {
                return Ok(());
            }
        }
        if let ItemKind::Use(tree) = item.kind {
            return self.import(c, tree, &[], false, access);
        }
        if let ItemKind::Module { .. } = item.kind {
            let key = NodeKey { file: c.file, node };
            let module = *self
                .package
                .children
                .get(&(c.module, key))
                .ok_or_else(|| self.error(c, node, "missing loaded module"))?;
            let scope = if self.hygiene.is_some() {
                self.facts.module_scopes[module.0 as usize]
            } else {
                self.new_scope(None, module, true)?
            };
            if self.hygiene.is_none() {
                self.item_definition(
                    c,
                    node,
                    item,
                    (Namespace::Type, DefKind::Module),
                    access,
                    Some(scope),
                )?;
            }
            self.module(module, scope);
            return Ok(());
        }
        let body_context = if self.hygiene.is_some()
            && let Some(scope) = self.facts.node_scopes.get(&(c.module, node)).copied()
        {
            Context { scope, ..c }
        } else {
            self.declare_item(c, node, item, access)?
        };
        self.reserve_anonymous_parameters(body_context, node, item)?;
        let mark = self.scratch.len();
        self.package
            .unit(c.file)
            .configured
            .ast
            .push_children(node, self.scratch);
        let end = self.scratch.len();
        for index in (mark..end).rev() {
            self.pending.push((body_context, self.scratch[index]));
        }
        self.scratch.truncate(mark);
        Ok(())
    }
    /// Register an ordinary item and its member namespaces without walking its
    /// children. Expansion deliberately leaves generic parameters to resolution.
    #[expect(
        clippy::too_many_lines,
        reason = "one item-header dispatch defines its namespaces, member scope and enum variants"
    )]
    pub fn declare_item(
        &mut self,
        c: Context,
        node: NodeId,
        item: Item,
        access: Access,
    ) -> Result<Context, Error> {
        let access = self.facts.scopes[c.scope.0 as usize]
            .trait_access
            .unwrap_or(access);
        let owned = matches!(
            item.kind,
            ItemKind::Function { .. }
                | ItemKind::Struct { .. }
                | ItemKind::Enum { .. }
                | ItemKind::Union { .. }
                | ItemKind::Trait { .. }
                | ItemKind::Impl { .. }
                | ItemKind::TypeAlias { .. }
                | ItemKind::Const { .. }
                | ItemKind::Static { .. }
        );
        let body = if owned {
            self.node_scope(c, node, true)?
        } else {
            c.scope
        };
        if owned {
            self.facts.scopes[body.0 as usize].associated =
                matches!(item.kind, ItemKind::Trait { .. } | ItemKind::Impl { .. });
            self.facts.scopes[body.0 as usize].trait_access =
                matches!(item.kind, ItemKind::Trait { .. }).then_some(access);
        }
        self.facts.node_scopes.insert((c.module, node), body);
        let member = matches!(
            item.kind,
            ItemKind::Struct { .. }
                | ItemKind::Enum { .. }
                | ItemKind::Union { .. }
                | ItemKind::Trait { .. }
        );
        let (namespace, kind) = match item.kind {
            ItemKind::Function { .. } => (Namespace::Value, DefKind::Function),
            ItemKind::Struct { .. } | ItemKind::Union { .. } => (Namespace::Type, DefKind::Struct),
            ItemKind::Enum { .. } => (Namespace::Type, DefKind::Enum),
            ItemKind::Trait { .. } => (Namespace::Type, DefKind::Trait),
            ItemKind::Const { .. } => (Namespace::Value, DefKind::Const),
            ItemKind::Static { .. } => (Namespace::Value, DefKind::Static),
            ItemKind::TypeAlias { .. } => (Namespace::Type, DefKind::TypeAlias),
            ItemKind::MacroRules { .. } => (Namespace::Macro, DefKind::Macro),
            ItemKind::ExternCrate { .. } => (Namespace::Type, DefKind::ExternCrate),
            _ => (Namespace::Type, DefKind::SelfType),
        };
        if item.name.is_some() {
            let external = self.builtin_crate(c, node, item)?;
            let definition = self.item_definition(
                c,
                node,
                item,
                (namespace, kind),
                access,
                external.or_else(|| member.then_some(body)),
            )?;
            if external.is_some() {
                self.expose_external(c, definition);
            }
            if matches!(kind, DefKind::Struct | DefKind::Enum) {
                self.facts.scopes[body.0 as usize].nominal_self = Some(definition);
            }
        }
        if let ItemKind::Struct {
            shape: Shape::Unit | Shape::Tuple,
            ..
        } = item.kind
        {
            let constructor_access = self.constructor_access(c, item, access)?;
            self.item_definition(
                c,
                node,
                item,
                (Namespace::Value, DefKind::Constructor),
                constructor_access,
                None,
            )?;
        }
        let body_context = Context { scope: body, ..c };
        if matches!(item.kind, ItemKind::Trait { .. } | ItemKind::Impl { .. }) {
            let name = self
                .package
                .names
                .intern("Self")
                .map_err(|e| self.error(c, node, e))?;
            self.define(
                Definition {
                    name,
                    namespace: Namespace::Type,
                    kind: DefKind::SelfType,
                    scope: body,
                    node: Some(NodeKey { file: c.file, node }),
                    access: Access::Within(c.module),
                    members: Some(body),
                },
                false,
            )?;
        }
        if let ItemKind::Enum { variants } = item.kind {
            for offset in variants.start..variants.end {
                let mut variant_node =
                    self.package.unit(c.file).configured.ast.edges[offset as usize];
                if !self.active(c, variant_node) {
                    continue;
                }
                while let Kind::Attributed { value, .. } = self.kind(c, variant_node) {
                    variant_node = value;
                }
                let Kind::Variant(index) = self.kind(c, variant_node) else {
                    return Err(self.error(c, variant_node, "invalid enum variant"));
                };
                let variant = self.package.unit(c.file).configured.ast.variants[index as usize];
                let name = self.package.name(c.file, variant.name)?;
                for ns in [Namespace::Type, Namespace::Value] {
                    if ns == Namespace::Value && variant.shape == Shape::Record {
                        continue;
                    }
                    self.define(
                        Definition {
                            name,
                            namespace: ns,
                            kind: DefKind::Variant,
                            scope: body,
                            node: Some(NodeKey {
                                file: c.file,
                                node: variant_node,
                            }),
                            access,
                            members: None,
                        },
                        false,
                    )?;
                }
            }
        }
        Ok(body_context)
    }
    fn constructor_access(
        &mut self,
        c: Context,
        item: Item,
        mut access: Access,
    ) -> Result<Access, Error> {
        let ItemKind::Struct { fields, .. } = item.kind else {
            return Ok(access);
        };
        for offset in fields.start..fields.end {
            let mut node = self.package.unit(c.file).configured.ast.edges[offset as usize];
            if !self.active(c, node) {
                continue;
            }
            while let Kind::Attributed { value, .. } = self.kind(c, node) {
                node = value;
            }
            if let Kind::FieldDefinition { visibility, .. } = self.kind(c, node) {
                let field = self.visibility(c, visibility)?;
                if let Access::Within(restriction) = field {
                    access = match access {
                        Access::Public => field,
                        Access::Within(existing) => {
                            if self.package.is_descendant(restriction, existing) {
                                field
                            } else {
                                access
                            }
                        }
                    };
                }
            }
        }
        Ok(access)
    }
    fn item_definition(
        &mut self,
        c: Context,
        node: NodeId,
        item: Item,
        identity: (Namespace, DefKind),
        access: Access,
        members: Option<ScopeId>,
    ) -> Result<DefId, Error> {
        let (ns, kind) = identity;
        let span = if let ItemKind::ExternCrate {
            rename: Some(rename),
        } = item.kind
        {
            rename
        } else {
            item.name
                .ok_or_else(|| self.error(c, node, "missing item name"))?
        };
        let name = self.package.name(c.file, span)?;
        let id = self.define(
            Definition {
                name,
                namespace: ns,
                kind,
                scope: c.scope,
                node: Some(NodeKey { file: c.file, node }),
                access,
                members,
            },
            false,
        )?;
        self.facts.item_definitions.insert((c.module, node, ns), id);
        Ok(id)
    }
    pub fn visibility(&mut self, c: Context, visibility: Visibility) -> Result<Access, Error> {
        let Visibility::Restricted(node) = visibility else {
            return Ok(if visibility == Visibility::Public {
                Access::Public
            } else {
                Access::Within(c.module)
            });
        };
        let (_, path) = self.path_names(c, node)?;
        let mut module = c.module;
        for (index, name) in path.into_iter().enumerate() {
            match self.package.names.get(name) {
                "crate" if index == 0 => module = ModuleId(0),
                "self" if index == 0 => {}
                "super" => {
                    module = self.package.modules[module.0 as usize]
                        .parent
                        .ok_or_else(|| self.error(c, node, "super escapes crate root"))?;
                }
                _ => {
                    let current = module;
                    module = self
                        .package
                        .modules
                        .iter()
                        .enumerate()
                        .find(|(_, m)| m.parent == Some(current) && m.name == name)
                        .and_then(|(i, _)| u32::try_from(i).ok())
                        .map(ModuleId)
                        .ok_or_else(|| {
                            self.error(c, node, "visibility must name an ancestor module")
                        })?;
                }
            }
        }
        if !self.package.is_descendant(c.module, module) {
            return Err(self.error(c, node, "visibility must name an ancestor module"));
        }
        Ok(Access::Within(module))
    }
    pub fn path_names(&mut self, c: Context, node: NodeId) -> Result<(bool, Vec<NameId>), Error> {
        self.package.path_names(c.file, node)
    }
    pub fn import(
        &mut self,
        c: Context,
        node: NodeId,
        prefix: &[NameId],
        absolute: bool,
        access: Access,
    ) -> Result<(), Error> {
        let Kind::UseTree(index) = self.kind(c, node) else {
            return Err(self.error(c, node, "expected import tree"));
        };
        let tree = self.package.unit(c.file).configured.ast.use_trees[index as usize];
        let (local_absolute, local) = tree
            .prefix
            .map(|n| self.path_names(c, n))
            .transpose()?
            .unwrap_or_default();
        let mut path = prefix.to_vec();
        path.extend(local);
        self.prepare_builtin_path(c, node, absolute || local_absolute, &path)?;
        if tree.children.start == tree.children.end {
            let alias = tree
                .rename
                .map(|n| self.path_names(c, n))
                .transpose()?
                .and_then(|(_, names)| names.first().copied());
            self.facts.imports.push(Import {
                scope: c.scope,
                node: NodeKey { file: c.file, node },
                path,
                absolute: absolute || local_absolute,
                alias,
                glob: tree.glob,
                access,
            });
        } else {
            for offset in tree.children.start..tree.children.end {
                let child = self.package.unit(c.file).configured.ast.edges[offset as usize];
                self.import(c, child, &path, absolute || local_absolute, access)?;
            }
        }
        Ok(())
    }
}
