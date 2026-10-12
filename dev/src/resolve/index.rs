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
pub(crate) struct Resolver<'a> {
    pub package: &'a mut Package,
    pub facts: Resolution,
    pub scratch: Vec<NodeId>,
    pub pending: Vec<(Context, NodeId)>,
    pub visits: Vec<(Context, NodeId, Namespace)>,
}
impl Resolver<'_> {
    pub fn collect(&mut self) -> Result<(), Error> {
        self.facts
            .module_scopes
            .resize(self.package.modules.len(), ScopeId(0));
        self.new_scope(None, ModuleId(0), true)?;
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
            label_barrier: barrier,
            bindings: HashMap::new(),
            order: Vec::new(),
        });
        Ok(id)
    }
    pub fn define(&mut self, definition: Definition, shadow: bool) -> Result<DefId, Error> {
        let id = DefId(
            u32::try_from(self.facts.definitions.len())
                .map_err(|_| Error::at(FileId(0), 0, "too many definitions"))?,
        );
        let scope = &mut self.facts.scopes[definition.scope.0 as usize];
        let key = (definition.name, definition.namespace);
        if scope.bindings.contains_key(&key) && !shadow {
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
            let scope = self.new_scope(Some(c.scope), c.module, false)?;
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
                c.scope = self.new_scope(Some(c.scope), c.module, false)?;
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
            .push_children(node, &mut self.scratch);
        let end = self.scratch.len();
        for index in (mark..end).rev() {
            self.pending.push((c, self.scratch[index]));
        }
        self.scratch.truncate(mark);
        Ok(())
    }
    #[expect(
        clippy::too_many_lines,
        reason = "one declaration-kind dispatch creates namespaces and their owned scopes"
    )]
    fn item(&mut self, c: Context, node: NodeId, index: u32) -> Result<(), Error> {
        let item = self.package.unit(c.file).configured.ast.items[index as usize];
        let access = self.visibility(c, item.visibility)?;
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
            let scope = self.new_scope(None, module, true)?;
            self.item_definition(
                c,
                node,
                item,
                (Namespace::Type, DefKind::Module),
                access,
                Some(scope),
            )?;
            self.module(module, scope);
            return Ok(());
        }
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
            self.new_scope(Some(c.scope), c.module, true)?
        } else {
            c.scope
        };
        self.facts.scopes[body.0 as usize].associated =
            matches!(item.kind, ItemKind::Trait { .. } | ItemKind::Impl { .. });
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
            self.item_definition(
                c,
                node,
                item,
                (namespace, kind),
                access,
                member.then_some(body),
            )?;
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
        let mark = self.scratch.len();
        self.package
            .unit(c.file)
            .configured
            .ast
            .push_children(node, &mut self.scratch);
        let end = self.scratch.len();
        for index in (mark..end).rev() {
            self.pending.push((body_context, self.scratch[index]));
        }
        self.scratch.truncate(mark);
        Ok(())
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
        match self.kind(c, node) {
            Kind::Name(span) => Ok((false, vec![self.package.name(c.file, span)?])),
            Kind::Applied { base, .. } => self.path_names(c, base),
            Kind::Path { absolute, segments } => {
                let mut names = Vec::with_capacity((segments.end - segments.start) as usize);
                for offset in segments.start..segments.end {
                    let id = self.package.unit(c.file).configured.ast.edges[offset as usize];
                    names.extend(self.path_names(c, id)?.1);
                }
                Ok((absolute, names))
            }
            _ => Err(self.error(c, node, "expected a name path")),
        }
    }
    fn import(
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
