//! Copy parsed fragments by typed facts, rebasing dense IDs and source provenance.
mod span_preflight;
use super::{
    ast::{Kind, List, NodeId, Syntax},
    attribute,
    item::{AttributeCapture, ItemKind, Visibility},
    token_tree::{Span, error},
};
use crate::lex::token::LexError;
pub(crate) use span_preflight::validate as validate_spans;

/// Original arena bounds require no speculative AST copy. A later module
/// instance copies only these typed facts, excluding prior generated syntax.
#[derive(Clone, Copy)]
pub(crate) struct Extent {
    pub nodes: usize,
    items: usize,
    gate_sites: usize,
    variants: usize,
    use_trees: usize,
    generics: usize,
    function_types: usize,
    attributes: usize,
    edges: usize,
    literals: usize,
    captures: usize,
    root: NodeId,
    lookahead_high_water: usize,
}
impl Extent {
    pub(crate) fn new<L>(ast: &Syntax<L>) -> Self {
        Self {
            nodes: ast.nodes.len(),
            items: ast.items.len(),
            gate_sites: ast.gate_sites.as_ref().map_or(0, |entries| entries.len()),
            variants: ast.variants.len(),
            use_trees: ast.use_trees.len(),
            generics: ast.generics.len(),
            function_types: ast.function_types.len(),
            attributes: ast.attributes.len(),
            edges: ast.edges.len(),
            literals: ast.literals.len(),
            captures: ast.captures.len(),
            root: ast.root,
            lookahead_high_water: ast.lookahead_high_water,
        }
    }
    pub(crate) fn copy<L: Clone>(&self, ast: &Syntax<L>) -> Result<Syntax<L>, LexError> {
        Ok(Syntax {
            nodes: prefix(&ast.nodes, self.nodes)?,
            items: prefix(&ast.items, self.items)?,
            gate_sites: ast.gate_sites_prefix(self.gate_sites)?,
            variants: prefix(&ast.variants, self.variants)?,
            use_trees: prefix(&ast.use_trees, self.use_trees)?,
            generics: prefix(&ast.generics, self.generics)?,
            function_types: prefix(&ast.function_types, self.function_types)?,
            attributes: prefix(&ast.attributes, self.attributes)?,
            edges: prefix(&ast.edges, self.edges)?,
            literals: prefix(&ast.literals, self.literals)?,
            captures: prefix(&ast.captures, self.captures)?,
            root: self.root,
            lookahead_high_water: self.lookahead_high_water,
            pending: super::ast::Pending::default(),
        })
    }
}
fn prefix<T: Clone>(values: &[T], count: usize) -> Result<Vec<T>, LexError> {
    values
        .get(..count)
        .map(<[T]>::to_vec)
        .ok_or_else(|| error(0, "invalid original syntax prefix"))
}

struct Bases {
    nodes: u32,
    edges: u32,
    literals: u32,
    captures: u32,
    items: u32,
    gate_sites: usize,
    variants: u32,
    use_trees: u32,
    generics: u32,
    function_types: u32,
    attributes: u32,
}
fn base(existing: usize, added: usize) -> Result<u32, LexError> {
    existing
        .checked_add(added)
        .and_then(|n| u32::try_from(n).ok())
        .ok_or_else(|| error(0, "syntax transfer exceeds 32-bit arena identities"))?;
    u32::try_from(existing).map_err(|_| error(0, "invalid syntax arena size"))
}
impl Bases {
    fn new<L, M>(source: &Syntax<L>, destination: &Syntax<M>) -> Result<Self, LexError> {
        Ok(Self {
            nodes: base(destination.nodes.len(), source.nodes.len())?,
            edges: base(destination.edges.len(), source.edges.len())?,
            literals: base(destination.literals.len(), source.literals.len())?,
            captures: base(destination.captures.len(), source.captures.len())?,
            items: base(destination.items.len(), source.items.len())?,
            gate_sites: destination
                .gate_sites
                .as_ref()
                .map_or(0, |entries| entries.len()),
            variants: base(destination.variants.len(), source.variants.len())?,
            use_trees: base(destination.use_trees.len(), source.use_trees.len())?,
            generics: base(destination.generics.len(), source.generics.len())?,
            function_types: base(
                destination.function_types.len(),
                source.function_types.len(),
            )?,
            attributes: base(destination.attributes.len(), source.attributes.len())?,
        })
    }
    fn node(&self, node: &mut NodeId) {
        *node = self.rebased(*node);
    }
    fn rebased(&self, node: NodeId) -> NodeId {
        NodeId::new(node.get() + self.nodes)
    }
    fn optional(&self, node: &mut Option<NodeId>) {
        if let Some(node) = node {
            self.node(node);
        }
    }
    fn list(&self, list: &mut List) {
        list.start += self.edges;
        list.end += self.edges;
    }
    fn visibility(&self, visibility: &mut Visibility) {
        if let Visibility::Restricted(node) = visibility {
            self.node(node);
        }
    }
    fn restore<L>(&self, destination: &mut Syntax<L>) {
        destination.nodes.truncate(self.nodes as usize);
        destination.edges.truncate(self.edges as usize);
        destination.literals.truncate(self.literals as usize);
        destination.captures.truncate(self.captures as usize);
        destination.items.truncate(self.items as usize);
        destination.truncate_gate_sites(self.gate_sites);
        destination.variants.truncate(self.variants as usize);
        destination.use_trees.truncate(self.use_trees as usize);
        destination.generics.truncate(self.generics as usize);
        destination
            .function_types
            .truncate(self.function_types as usize);
        destination.attributes.truncate(self.attributes as usize);
    }
}
/// Append one occurrence of an already parsed fragment. Independent occurrences
/// get independent node IDs for later inference, while parsing is never repeated.
/// The caller maps literal ownership and spelling/provenance spans explicitly.
///
/// # Errors
/// Rejects arena overflow, a missing root, or callback failures. On failure the
/// destination arenas are restored; callback side effects remain caller-owned.
pub fn append<L, M>(
    source: &Syntax<L>,
    destination: &mut Syntax<M>,
    mut span: impl FnMut(Span) -> Result<Span, LexError>,
    mut literal: impl FnMut(&L) -> Result<M, LexError>,
) -> Result<NodeId, LexError> {
    if source.root.get() as usize >= source.nodes.len() {
        return Err(error(0, "syntax fragment has no root"));
    }
    let bases = Bases::new(source, destination)?;
    let result = bases.append(source, destination, &mut span, &mut literal);
    if result.is_err() {
        bases.restore(destination);
    } else {
        destination.lookahead_high_water = destination
            .lookahead_high_water
            .max(source.lookahead_high_water);
        // Appended IDs follow every existing one, so both lists stay ascending.
        let pending = &mut destination.pending;
        pending
            .owners
            .extend(source.pending.owners.iter().map(|n| bases.rebased(*n)));
        pending
            .literals
            .extend(source.pending.literals.iter().map(|n| bases.rebased(*n)));
    }
    result.map(|()| NodeId::new(source.root.get() + bases.nodes))
}
fn optional_span(
    span: &mut Option<Span>,
    map: &mut impl FnMut(Span) -> Result<Span, LexError>,
) -> Result<(), LexError> {
    if let Some(span) = span {
        *span = map(*span)?;
    }
    Ok(())
}
impl Bases {
    fn append<L, M>(
        &self,
        source: &Syntax<L>,
        destination: &mut Syntax<M>,
        span: &mut impl FnMut(Span) -> Result<Span, LexError>,
        literal: &mut impl FnMut(&L) -> Result<M, LexError>,
    ) -> Result<(), LexError> {
        for node in &source.nodes {
            let mut node = *node;
            node.span = span(node.span)?;
            self.kind(&mut node.kind, span)?;
            destination.nodes.push(node);
        }
        destination.edges.extend(
            source
                .edges
                .iter()
                .map(|n| NodeId::new(n.get() + self.nodes)),
        );
        for value in &source.literals {
            destination.literals.push(literal(value)?);
        }
        for capture in &source.captures {
            destination.captures.push(capture.map_spans(span)?);
        }
        for item in &source.items {
            let mut item = *item;
            optional_span(&mut item.name, span)?;
            self.visibility(&mut item.visibility);
            self.list(&mut item.generics);
            self.list(&mut item.where_clause);
            self.item(&mut item.kind, span)?;
            destination.items.push(item);
        }
        if let Some(entries) = &source.gate_sites {
            entries
                .charge(1 + entries.len(), source.gate_sites_bytes())
                .map_err(|message| error(0, message))?;
            for entry in entries.entries() {
                entry.validate(source)?;
                destination.record_typed_gate_site(
                    entry.item + self.items,
                    self.rebased(entry.node),
                    span(entry.span)?,
                    entry.kind,
                )?;
            }
        }
        for variant in &source.variants {
            let mut variant = *variant;
            variant.name = span(variant.name)?;
            self.list(&mut variant.fields);
            self.optional(&mut variant.discriminant);
            destination.variants.push(variant);
        }
        for tree in &source.use_trees {
            let mut tree = *tree;
            self.optional(&mut tree.prefix);
            self.list(&mut tree.children);
            self.optional(&mut tree.rename);
            optional_span(&mut tree.glob, span)?;
            destination.use_trees.push(tree);
        }
        for generic in &source.generics {
            let mut generic = *generic;
            generic.name = span(generic.name)?;
            self.optional(&mut generic.ty);
            self.list(&mut generic.bounds);
            self.optional(&mut generic.default);
            destination.generics.push(generic);
        }
        for function in &source.function_types {
            let mut function = *function;
            self.abi(&mut function.abi);
            self.list(&mut function.binders);
            self.list(&mut function.parameters);
            self.optional(&mut function.result);
            destination.function_types.push(function);
        }
        for attribute in &source.attributes {
            let forest = attribute.forest.map_spans(span)?;
            let mut attribute = attribute.attribute;
            attribute.span = span(attribute.span)?;
            if let attribute::Input::Doc(body) = &mut attribute.input {
                *body = span(*body)?;
            }
            destination
                .attributes
                .push(AttributeCapture { forest, attribute });
        }
        Ok(())
    }
    #[expect(
        clippy::too_many_lines,
        reason = "exhaustive syntax-ID and span transfer dispatch"
    )]
    fn kind(
        &self,
        kind: &mut Kind,
        span: &mut impl FnMut(Span) -> Result<Span, LexError>,
    ) -> Result<(), LexError> {
        match kind {
            Kind::Receiver { name, lifetime, .. } => {
                *name = span(*name)?;
                *lifetime = span(*lifetime)?;
            }
            Kind::LibraryValue { value, .. } => self.optional(value),
            Kind::Name(name, _) => *name = span(*name)?,
            Kind::GenericMethodCall {
                receiver,
                method,
                arguments,
            } => {
                self.node(receiver);
                self.node(method);
                self.list(arguments);
            }
            Kind::DeferredLiteral(token) => {
                let mapped = span(Span {
                    start: token.start,
                    end: token.end,
                })?;
                token.start = mapped.start;
                token.end = mapped.end;
            }
            Kind::CrateRoot(_)
            | Kind::Variadic
            | Kind::EmptyStatement
            | Kind::Wildcard
            | Kind::RestPattern
            | Kind::InferType
            | Kind::NeverType => {}
            Kind::Item(index) => *index += self.items,
            Kind::Generic(index) => *index += self.generics,
            Kind::FunctionType(index) => *index += self.function_types,
            Kind::Variant(index) => *index += self.variants,
            Kind::UseTree(index) => *index += self.use_trees,
            Kind::Literal(index) => *index += self.literals,
            Kind::Crate(list)
            | Kind::PreciseCapture(list)
            | Kind::Tuple(list)
            | Kind::Array(list)
            | Kind::Type { elements: list, .. }
            | Kind::Path { segments: list, .. }
            | Kind::TraitType { bounds: list, .. } => self.list(list),
            Kind::Attributed { attributes, value } => {
                attributes.start += self.attributes;
                attributes.end += self.attributes;
                self.node(value);
            }
            Kind::Applied {
                base,
                arguments,
                output,
                ..
            } => {
                self.node(base);
                self.list(arguments);
                self.optional(output);
            }
            Kind::Qualified {
                ty,
                trait_path,
                path,
            } => {
                self.node(ty);
                self.optional(trait_path);
                self.node(path);
            }
            Kind::Constraint {
                target,
                value,
                bounds,
            } => {
                self.node(target);
                self.optional(value);
                self.list(bounds);
            }
            Kind::Bound { binders, value, .. } => {
                self.list(binders);
                self.node(value);
            }
            Kind::Where {
                binders,
                ty,
                bounds,
                ..
            } => {
                self.list(binders);
                self.node(ty);
                self.list(bounds);
            }
            Kind::FieldDefinition {
                name,
                visibility,
                ty,
            } => {
                optional_span(name, span)?;
                self.visibility(visibility);
                self.node(ty);
            }
            Kind::StructPattern { path, fields, .. } => {
                self.node(path);
                self.list(fields);
            }
            Kind::PatternField { name, pattern } => {
                *name = span(*name)?;
                self.node(pattern);
            }
            Kind::PatternRange { start, end, .. } | Kind::Range { start, end, .. } => {
                self.optional(start);
                self.optional(end);
            }
            Kind::TypePattern { base: node, .. }
            | Kind::Group(node)
            | Kind::Statement(node)
            | Kind::Unary { operand: node, .. }
            | Kind::Loop { body: node }
            | Kind::ReferencePattern { pattern: node, .. } => self.node(node),
            Kind::Repeat {
                value: left,
                count: right,
            }
            | Kind::Binary { left, right, .. }
            | Kind::Cast {
                value: left,
                ty: right,
            }
            | Kind::Index {
                value: left,
                index: right,
            }
            | Kind::LetCondition {
                pattern: left,
                value: right,
            }
            | Kind::While {
                condition: left,
                body: right,
            } => {
                self.node(left);
                self.node(right);
            }
            Kind::Call { callee, arguments } => {
                self.node(callee);
                self.list(arguments);
            }
            Kind::MethodCall {
                receiver,
                name,
                arguments,
            } => {
                self.node(receiver);
                *name = span(*name)?;
                self.list(arguments);
            }
            Kind::Field { value, name }
            | Kind::FieldInit { name, value }
            | Kind::Label { name, value } => {
                self.node(value);
                *name = span(*name)?;
            }
            Kind::Struct { path, fields, base } => {
                self.node(path);
                self.list(fields);
                self.optional(base);
            }
            Kind::Block {
                statements, tail, ..
            }
            | Kind::Expansion {
                children: statements,
                tail,
            } => {
                self.list(statements);
                self.optional(tail);
            }
            Kind::Let {
                pattern,
                ty,
                value,
                otherwise,
            } => {
                self.node(pattern);
                self.optional(ty);
                self.optional(value);
                self.optional(otherwise);
            }
            Kind::If {
                condition,
                then,
                otherwise,
            } => {
                self.node(condition);
                self.node(then);
                self.optional(otherwise);
            }
            Kind::For {
                pattern,
                iterator,
                body,
            } => {
                self.node(pattern);
                self.node(iterator);
                self.node(body);
            }
            Kind::Return(node) => self.optional(node),
            Kind::ImplRestriction { path } => self.node(path),
            Kind::Break { label, value } => {
                optional_span(label, span)?;
                self.optional(value);
            }
            Kind::Continue(label) => optional_span(label, span)?,
            Kind::Match { value, arms } => {
                self.node(value);
                self.list(arms);
            }
            Kind::Arm {
                pattern,
                guard,
                value,
            } => {
                self.node(pattern);
                self.optional(guard);
                self.node(value);
            }
            Kind::Closure {
                parameters,
                result,
                body,
                ..
            } => {
                self.list(parameters);
                self.optional(result);
                self.node(body);
            }
            Kind::Parameter { pattern, ty } => {
                self.node(pattern);
                self.optional(ty);
            }
            Kind::Macro { path, capture, .. } => {
                self.node(path);
                *capture += self.captures;
            }
            Kind::Binding {
                name, subpattern, ..
            } => {
                *name = span(*name)?;
                self.optional(subpattern);
            }
            Kind::Pattern { path, elements, .. } => {
                self.optional(path);
                self.list(elements);
            }
        }
        Ok(())
    }
    fn abi(&self, abi: &mut super::item::Abi) {
        if let super::item::Abi::Extern(value) = abi {
            self.optional(value);
        }
    }
    fn item(
        &self,
        kind: &mut ItemKind,
        span: &mut impl FnMut(Span) -> Result<Span, LexError>,
    ) -> Result<(), LexError> {
        match kind {
            ItemKind::Function {
                abi,
                parameters,
                result,
                body,
                ..
            } => {
                self.abi(abi);
                self.list(parameters);
                self.optional(result);
                self.optional(body);
            }
            ItemKind::Struct { fields, .. } | ItemKind::Union { fields } => self.list(fields),
            ItemKind::Enum { variants } => self.list(variants),
            ItemKind::Trait {
                restriction,
                bounds,
                items,
                ..
            } => {
                self.optional(restriction);
                self.list(bounds);
                self.list(items);
            }
            ItemKind::Impl {
                trait_path,
                ty,
                items,
                ..
            } => {
                self.optional(trait_path);
                self.node(ty);
                self.list(items);
            }
            ItemKind::Const { ty, value } | ItemKind::Static { ty, value, .. } => {
                self.node(ty);
                self.optional(value);
            }
            ItemKind::TraitAlias { bounds, .. } => self.list(bounds),
            ItemKind::ForeignType => {}
            ItemKind::TypeAlias { bounds, ty } => {
                self.list(bounds);
                self.optional(ty);
            }
            ItemKind::Module { items } => {
                if let Some(items) = items {
                    self.list(items);
                }
            }
            ItemKind::Use(node) | ItemKind::MacroInvocation(node) => self.node(node),
            ItemKind::ExternCrate { rename } => optional_span(rename, span)?,
            ItemKind::ExternBlock { abi, items, .. } => {
                self.optional(abi);
                self.list(items);
            }
            ItemKind::MacroRules { capture } => *capture += self.captures,
            ItemKind::DeclarativeMacro {
                parameters,
                capture,
            } => {
                if let Some(parameters) = parameters {
                    *parameters += self.captures;
                }
                *capture += self.captures;
            }
        }
        Ok(())
    }
}
