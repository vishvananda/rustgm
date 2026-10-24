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
use crate::expand::marks::ItemContext;
use crate::syntax::{
    ast::{Kind, List, NodeId},
    item::{GenericFlavor, Item, ItemKind, Shape, Visibility},
};
/// Kinds whose `push_children` arm appends nothing. Declaration walks have no
/// effect at these nodes, so they are not queued at all.
pub(crate) const fn is_leaf(kind: &Kind) -> bool {
    matches!(
        kind,
        Kind::CrateRoot(_)
            | Kind::Receiver { .. }
            | Kind::DeferredLiteral(_)
            | Kind::Literal(_)
            | Kind::Name(..)
            | Kind::EmptyStatement
            | Kind::Continue(_)
            | Kind::Wildcard
            | Kind::RestPattern
            | Kind::InferType
            | Kind::NeverType
            | Kind::Variadic
    )
}
/// Borrow a node's kind without copying the node record.
fn kind_of<L>(ast: &crate::syntax::ast::Syntax<L>, node: NodeId) -> &Kind {
    &ast.nodes[node.get() as usize].kind
}
/// Small expression, pattern and type subtrees built only from leaves, such as
/// `a::b`, `x + f(y)`, `x: u64` or `u64`, contain no declaration, scope or
/// expansion site; declaration walks skip them whole. Each arm inspects exactly
/// the children `push_children` appends, to a fixed depth so repeated checks by
/// ancestors stay constant per node.
pub(crate) fn is_inert<L>(ast: &crate::syntax::ast::Syntax<L>, node: NodeId) -> bool {
    match kind_of(ast, node) {
        Kind::Type { elements: list, .. } | Kind::Tuple(list) | Kind::Array(list) => {
            all_plain(ast, *list)
        }
        Kind::Pattern { path, elements, .. } => {
            path.is_none_or(|path| plain(ast, path)) && all_plain(ast, *elements)
        }
        Kind::Binding {
            subpattern: value, ..
        }
        | Kind::Return(value) => value.is_none_or(|value| plain(ast, value)),
        Kind::Parameter { pattern, ty } => {
            plain(ast, *pattern) && ty.is_none_or(|ty| plain_type(ast, ty))
        }
        Kind::Unary { operand: value, .. } | Kind::Field { value, .. } | Kind::Group(value) => {
            plain(ast, *value)
        }
        Kind::Binary { left, right, .. }
        | Kind::Cast {
            value: left,
            ty: right,
        }
        | Kind::Index {
            value: left,
            index: right,
        } => plain(ast, *left) && plain_type(ast, *right),
        Kind::Call {
            callee: value,
            arguments,
        }
        | Kind::MethodCall {
            receiver: value,
            arguments,
            ..
        } => plain(ast, *value) && all_plain(ast, *arguments),
        kind => plain_kind(ast, kind),
    }
}
/// A leaf, a path of leaf segments or a binding without a subpattern.
fn plain<L>(ast: &crate::syntax::ast::Syntax<L>, node: NodeId) -> bool {
    plain_kind(ast, kind_of(ast, node))
}
fn plain_kind<L>(ast: &crate::syntax::ast::Syntax<L>, kind: &Kind) -> bool {
    match kind {
        Kind::Path { segments, .. } => ast
            .list(*segments)
            .iter()
            .all(|segment| is_leaf(kind_of(ast, *segment))),
        Kind::Binding {
            subpattern: None, ..
        } => true,
        kind => is_leaf(kind),
    }
}
/// A plain node or a type whose elements are all plain, such as `u64`.
fn plain_type<L>(ast: &crate::syntax::ast::Syntax<L>, node: NodeId) -> bool {
    match kind_of(ast, node) {
        Kind::Type { elements, .. } => all_plain(ast, *elements),
        kind => plain_kind(ast, kind),
    }
}
fn all_plain<L>(ast: &crate::syntax::ast::Syntax<L>, list: List) -> bool {
    ast.list(list).iter().all(|child| plain(ast, *child))
}
#[cfg(test)]
#[path = "../../tests/invariants/declaration_walks.rs"]
mod tests;
#[derive(Clone, Copy)]
pub(crate) struct Context {
    pub module: ModuleId,
    pub file: FileId,
    pub scope: ScopeId,
    pub foreign: Option<NodeKey>,
}
impl Context {
    pub fn foreign_children(self, node: NodeId, kind: ItemKind) -> Option<NodeKey> {
        match kind {
            ItemKind::ExternBlock { .. } => Some(NodeKey {
                file: self.file,
                node,
            }),
            ItemKind::MacroInvocation(_) => self.foreign,
            _ => None,
        }
    }
}
/// Spellings of the language primitive types, in definition order.
pub(crate) const PRIMITIVES: [&str; 17] = [
    "bool", "char", "str", "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64",
    "u128", "usize", "f32", "f64",
];
/// Declarations the expansion scan leaves to resolution, in scan order: they
/// keep the identities the second declaration walk used to give them.
#[derive(Clone, Copy)]
pub(crate) enum Pending {
    /// Unnameable `impl Trait` inputs of a function, in its body context.
    Anonymous(Context, NodeId, Item),
    Generic(Context, NodeId, u32),
    /// A restricted visibility no expansion step validated: textual macros.
    Visibility(Context, NodeId, Visibility),
}
#[derive(Default)]
pub(crate) struct State {
    pub facts: Resolution,
    pub hygiene: Option<super::hygiene::Hygiene>,
    scratch: Vec<NodeId>,
    pending: Vec<(Context, NodeId)>,
    visits: Vec<(Context, NodeId, Namespace)>,
    paths: Vec<Vec<NameId>>,
    patterns: Vec<super::patterns::PatternFrame>,
    builtin_names: Vec<u8>,
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
            paths: &mut self.paths,
            patterns: &mut self.patterns,
            builtin_names: &mut self.builtin_names,
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
    /// Reusable path-name buffers; nested references borrow distinct entries.
    pub paths: &'a mut Vec<Vec<NameId>>,
    /// Reusable pattern-binding frames, likewise borrowed per nesting level.
    pub patterns: &'a mut Vec<super::patterns::PatternFrame>,
    /// Per-name cache: 0 unknown, 1 ordinary, 2 may select language metadata.
    pub builtin_names: &'a mut Vec<u8>,
}
impl Resolver<'_> {
    /// Declare scopes and forward-visible items of unexpanded modules. The
    /// expansion scan does this work itself while it expands a crate.
    pub fn collect(&mut self) -> Result<(), Error> {
        self.facts
            .module_scopes
            .resize(self.package.modules.len(), ScopeId(0));
        if self.facts.scopes.is_empty() {
            self.new_scope(None, ModuleId(0), true)?;
        }
        self.primitives()?;
        self.module(ModuleId(0), ScopeId(0));
        while let Some((c, node)) = self.pending.pop() {
            self.walk_inner(c, node)?;
        }
        Ok(())
    }
    /// Language primitive types, found after every lexical scope. Defined once:
    /// resolution after expansion defines them unless an import needed them first.
    pub fn primitives(&mut self) -> Result<(), Error> {
        if !self.facts.primitives.is_empty() {
            return Ok(());
        }
        for spelling in PRIMITIVES {
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
        Ok(())
    }
    pub fn context(&self, module: ModuleId, scope: ScopeId) -> Context {
        Context {
            module,
            scope,
            file: self.package.modules[module.0 as usize].file,
            foreign: None,
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
        });
        Ok(id)
    }
    /// A lexical scope introduced by an expression, binder or type node; only
    /// closures stop label lookup. An `if` scope covers its condition and
    /// consequence but not its alternative.
    pub fn lexical_scope(&mut self, c: Context, node: NodeId) -> Result<ScopeId, Error> {
        let scope = self.new_scope(Some(c.scope), c.module, false)?;
        self.facts.scopes[scope.0 as usize].label_barrier =
            matches!(self.kind(c, node), Kind::Closure { .. });
        self.facts.node_scopes.insert(c.module, c.file, node, scope);
        Ok(scope)
    }
    /// Define what the expansion scan recorded, publishing generic parameters
    /// to the imports that observed their keys.
    pub fn declare_pending(
        &mut self,
        pending: &[Pending],
        work: &mut super::imports::Work,
    ) -> Result<(), Error> {
        work.extend(self.facts);
        for declaration in pending {
            let (Pending::Anonymous(c, node, _)
            | Pending::Generic(c, node, _)
            | Pending::Visibility(c, node, _)) = *declaration;
            // Eager macro arguments the scan visited are retired after use.
            if !self.active(c, node) {
                continue;
            }
            match *declaration {
                Pending::Anonymous(c, node, item) => {
                    self.reserve_anonymous_parameters(c, node, item)?;
                }
                Pending::Visibility(c, _, visibility) => {
                    self.visibility(c, visibility)?;
                }
                Pending::Generic(c, node, index) => {
                    let id = self.generic(c, node, index)?;
                    let definition = &self.facts.definitions[id.0 as usize];
                    work.declared(
                        &self.facts.bindings,
                        (
                            definition.scope,
                            definition.name,
                            definition.namespace,
                            ItemContext::ROOT,
                        ),
                    );
                }
            }
        }
        Ok(())
    }
    /// Define a generic parameter in its owner's scope.
    pub fn generic(&mut self, c: Context, node: NodeId, index: u32) -> Result<DefId, Error> {
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
        )
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
            let context = self.package.local_context(key.file, offset)?;
            self.facts
                .bindings
                .define_local(
                    (definition.scope, definition.name, definition.namespace),
                    context,
                    Binding {
                        target: id,
                        access: definition.access,
                        origin: Origin::Direct,
                        ambiguous: false,
                    },
                    shadow,
                )
                .map_err(|e| Error::at(key.file, offset, e))?;
            self.facts.definitions.push(definition);
            return Ok(id);
        }
        let binding = Binding {
            target: id,
            access: definition.access,
            origin: Origin::Direct,
            ambiguous: false,
        };
        if self
            .facts
            .bindings
            .declare(
                (definition.scope, definition.name, definition.namespace),
                binding,
                shadow,
            )
            .map_err(|e| Error::at(FileId(0), 0, e))?
        {
            self.facts.definitions.push(definition);
            return Ok(id);
        }
        let node = definition.node;
        Err(Error::at(
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
        ))
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
        self.package.unit(c.file).configured.active[node.get() as usize]
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
            let scope = self.lexical_scope(c, node)?;
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
            | Kind::While { .. }
            | Kind::Label { .. }
            | Kind::Bound { .. }
            | Kind::Where { .. }
            | Kind::FunctionType(_) => c.scope = self.lexical_scope(c, node)?,
            Kind::Generic(index) => {
                self.generic(c, node, index)?;
            }
            Kind::Macro { .. } => {
                self.facts
                    .expansions
                    .push((c.scope, NodeKey { file: c.file, node }));
                return Ok(());
            }
            _ => {}
        }
        self.push_pending(c, node);
        Ok(())
    }
    /// Queue a node's non-leaf children in source order for the LIFO worklist.
    fn push_pending(&mut self, c: Context, node: NodeId) {
        let ast = &self.package.unit(c.file).configured.ast;
        let mark = self.scratch.len();
        ast.push_children(node, self.scratch);
        for index in (mark..self.scratch.len()).rev() {
            let child = self.scratch[index];
            if !is_inert(ast, child) {
                self.pending.push((c, child));
            }
        }
        self.scratch.truncate(mark);
    }
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
        let body_context = self.declare_item(c, node, item, access)?;
        self.reserve_anonymous_parameters(body_context, node, item)?;
        self.push_pending(body_context, node);
        Ok(())
    }
    /// Register an ordinary item and its member namespaces without walking its
    /// children; generic parameters are defined when the walk reaches them.
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
        if matches!(item.kind, ItemKind::Function { body: None, .. })
            && c.foreign.is_none()
            && self.facts.scopes[c.scope.0 as usize].trait_access.is_none()
            && self.package.source_core.is_none()
        {
            return Err(self.error(c, node, "function requires a definition"));
        }
        if matches!(item.kind, ItemKind::ExternBlock { .. }) {
            self.facts
                .foreign_blocks
                .insert(NodeKey { file: c.file, node });
        }
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
            self.new_scope(Some(c.scope), c.module, true)?
        } else {
            c.scope
        };
        if owned {
            self.facts.scopes[body.0 as usize].associated =
                matches!(item.kind, ItemKind::Trait { .. } | ItemKind::Impl { .. });
            self.facts.scopes[body.0 as usize].trait_access =
                matches!(item.kind, ItemKind::Trait { .. }).then_some(access);
        }
        self.facts.node_scopes.insert(c.module, c.file, node, body);
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
            ItemKind::DeclarativeMacro { .. } => {
                return Err(self.error(
                    c,
                    node,
                    "scoped declarative macro hygiene is not implemented",
                ));
            }
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
            if let Some(owner) = c.foreign {
                self.facts.foreign_owners.insert(definition, owner);
            }
            if external.is_some() {
                self.expose_external(c, definition)
                    .map_err(|e| self.error(c, node, e))?;
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
        let body_context = Context {
            scope: body,
            foreign: c.foreign_children(node, item.kind),
            ..c
        };
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
                        Access::ExternalPrivate => Access::ExternalPrivate,
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
    /// Read a path into a pooled buffer; return it with `recycle_path`.
    pub fn pooled_path(&mut self, c: Context, node: NodeId) -> Result<(bool, Vec<NameId>), Error> {
        let mut names = self.paths.pop().unwrap_or_default();
        names.clear();
        let absolute = self.package.path_names_into(c.file, node, &mut names)?;
        Ok((absolute, names))
    }
    pub fn recycle_path(&mut self, names: Vec<NameId>) {
        self.paths.push(names);
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
        self.prepare_builtin_path(c, node, absolute || local_absolute, Namespace::Type, &path)?;
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
