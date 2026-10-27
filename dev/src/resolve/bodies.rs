//! Lexical traversal resolves uses in evaluation order and introduces local bindings.
use super::{
    index::{Context, Resolver},
    lookup::{Lookup, LookupMode},
    model::{Access, DefId, DefKind, Definition, Namespace, Reference, Target},
    package::{Error, ModuleId},
    unit::NodeKey,
};
use crate::support::library_types::LibraryType;
use crate::syntax::{
    ast::{Kind, List, NodeId, TypeFlavor},
    item::{GenericFlavor, ItemKind},
};
#[cfg(test)]
#[path = "../../tests/invariants/fallback_members.rs"]
mod tests;
impl Resolver<'_> {
    pub fn bodies(&mut self) -> Result<(), Error> {
        for index in 0..self.package.modules.len() {
            let module = ModuleId(
                u32::try_from(index)
                    .map_err(|_| Error::at(super::unit::FileId(0), 0, "too many modules"))?,
            );
            let c = self.context(module, self.facts.module_scopes[index]);
            let root = self.package.modules[index].root;
            if let Kind::Item(item) = self.kind(c, root)
                && let ItemKind::Module { items: Some(items) } =
                    self.package.unit(c.file).configured.ast.items[item as usize].kind
            {
                self.visit_list(c, items, Namespace::Value)?;
            } else {
                self.visit(c, root, Namespace::Value)?;
            }
        }
        self.reserve_operator_comparisons()?;
        self.reserve_assignment_operators()?;
        self.reserve_iteration()?;
        self.reserve_cloning()?;
        self.reserve_ranges()
    }
    pub fn visit_list(&mut self, c: Context, list: List, ns: Namespace) -> Result<(), Error> {
        for offset in list.start..list.end {
            let node = self.package.unit(c.file).configured.ast.edges[offset as usize];
            self.visit(c, node, ns)?;
        }
        Ok(())
    }
    fn optional(&mut self, c: Context, node: Option<NodeId>, ns: Namespace) -> Result<(), Error> {
        node.map_or(Ok(()), |n| self.visit(c, n, ns))
    }
    pub fn visit(&mut self, c: Context, node: NodeId, ns: Namespace) -> Result<(), Error> {
        let mark = self.visits.len();
        self.visits.push((c, node, ns));
        while self.visits.len() > mark {
            if let Some((c, node, ns)) = self.visits.pop() {
                self.visit_one(c, node, ns)?;
            }
        }
        Ok(())
    }
    #[expect(
        clippy::too_many_lines,
        reason = "typed syntax dispatch keeps name namespaces and lexical introduction points together"
    )]
    fn visit_one(&mut self, mut c: Context, node: NodeId, ns: Namespace) -> Result<(), Error> {
        if !self.active(c, node) {
            return Ok(());
        }
        if let Some(scope) = self.facts.node_scopes.get(&(c.module, node)) {
            c.scope = *scope;
        }
        if !self.package.uses_source_bindings()
            && let Kind::Binary { op, .. } = self.kind(c, node)
        {
            if let Some(kind) = crate::syntax::compound::hook(op) {
                self.facts.builtin_traits.note_assignment_operator(
                    kind,
                    c.module,
                    NodeKey { file: c.file, node },
                );
            }
            let kind = match op {
                crate::lex::token::Punctuation::EqualEqual
                | crate::lex::token::Punctuation::BangEqual => {
                    Some(crate::support::comparisons::Comparison::PartialEq)
                }
                crate::lex::token::Punctuation::Less
                | crate::lex::token::Punctuation::LessEqual
                | crate::lex::token::Punctuation::Greater
                | crate::lex::token::Punctuation::GreaterEqual => {
                    Some(crate::support::comparisons::Comparison::PartialOrd)
                }
                _ => None,
            };
            if let Some(kind) = kind {
                self.facts.builtin_traits.note_comparison_operator(
                    kind,
                    c.module,
                    NodeKey { file: c.file, node },
                );
            }
        }
        match self.kind(c, node) {
            Kind::OffsetOf { container, .. } => self.visit(c, container, Namespace::Type),
            Kind::InlineAssembly(arguments) => self.visit_list(c, arguments, Namespace::Value),
            Kind::AssemblyOperand { input, output, .. } => {
                self.optional(c, input, Namespace::Value)?;
                if output != input {
                    self.optional(c, output, Namespace::Value)?;
                }
                Ok(())
            }
            Kind::AssemblyTemplate { .. }
            | Kind::AssemblyRegister { .. }
            | Kind::AssemblyOrigin(_)
            | Kind::AssemblyOptions(_)
            | Kind::AssemblyPlaceholder { .. }
            | Kind::Macro { .. }
            | Kind::UseTree(_) => Ok(()),
            Kind::LibraryValue { kind, value } => {
                self.prepare_library_value(c, node, kind)?;
                self.optional(c, value, Namespace::Value)
            }
            Kind::Item(index) => self.item_body(c, node, index),
            Kind::Path { .. } | Kind::Name(..) | Kind::CrateRoot(_) => self.reference(c, node, ns),
            Kind::Let {
                pattern,
                ty,
                value,
                otherwise,
            } => {
                self.optional(c, ty, Namespace::Type)?;
                self.optional(c, value, Namespace::Value)?;
                self.optional(c, otherwise, Namespace::Value)?;
                self.bind_pattern(c, pattern, DefKind::Local)
            }
            Kind::LetCondition { pattern, value } => {
                self.visit(c, value, Namespace::Value)?;
                self.bind_pattern(c, pattern, DefKind::Local)
            }
            Kind::Range {
                inclusive: false,
                start: Some(start),
                end: Some(end),
            } => {
                self.facts
                    .builtin_traits
                    .note_range(c.module, NodeKey { file: c.file, node });
                self.visit(c, start, Namespace::Value)?;
                self.visit(c, end, Namespace::Value)
            }
            Kind::For {
                pattern,
                iterator,
                body,
            } => {
                self.facts
                    .builtin_traits
                    .note_collection(c.module, NodeKey { file: c.file, node });
                self.visit(c, iterator, Namespace::Value)?;
                self.bind_pattern(c, pattern, DefKind::Local)?;
                self.visit(c, body, Namespace::Value)
            }
            Kind::Arm {
                pattern,
                guard,
                value,
            } => {
                self.bind_pattern(c, pattern, DefKind::Local)?;
                self.optional(c, guard, Namespace::Value)?;
                self.visit(c, value, Namespace::Value)
            }
            Kind::Closure {
                parameters,
                result,
                body,
                ..
            } => {
                self.parameters(c, parameters)?;
                self.optional(c, result, Namespace::Type)?;
                self.visit(c, body, Namespace::Value)
            }
            Kind::Parameter { pattern, ty } => {
                self.optional(c, ty, Namespace::Type)?;
                self.bind_pattern(c, pattern, DefKind::Parameter)
            }
            Kind::Receiver { .. } => self.bind_pattern(c, node, DefKind::Parameter),
            Kind::Cast { value, ty } => {
                self.visit(c, value, Namespace::Value)?;
                self.visit(c, ty, Namespace::Type)
            }
            Kind::TypePattern { base, .. } => self.visit(c, base, Namespace::Type),
            Kind::Type { flavor, elements } => {
                for offset in elements.start..elements.end {
                    let child = self.package.unit(c.file).configured.ast.edges[offset as usize];
                    let ns = if flavor == TypeFlavor::Array && offset + 1 == elements.end {
                        Namespace::Value
                    } else {
                        Namespace::Type
                    };
                    self.visit(c, child, ns)?;
                }
                Ok(())
            }
            Kind::Struct { path, fields, base } => {
                self.visit(c, path, Namespace::Type)?;
                self.visit_list(c, fields, Namespace::Value)?;
                self.optional(c, base, Namespace::Value)
            }
            Kind::GenericMethodCall {
                receiver,
                method,
                arguments,
            } => {
                let (_, names) = self.pooled_path(c, method)?;
                let &[name] = names.names.as_slice() else {
                    return Err(self.error(c, node, "method requires one name segment"));
                };
                self.recycle_path(names);
                if let Some(method) =
                    crate::support::comparisons::Method::from_name(self.package.names.get(name))
                {
                    self.prepare_comparison_method(c, node, method)?;
                }
                self.prepare_slice_method(c, node, name)?;
                self.note_clone_name(c, node, name);
                self.facts.method_calls.insert(
                    NodeKey { file: c.file, node },
                    super::model::MethodCall {
                        scope: c.scope,
                        receiver,
                        name,
                        method: Some(method),
                        arguments,
                    },
                );
                self.visit(c, receiver, Namespace::Value)?;
                self.path_arguments(c, method)?;
                self.visit_list(c, arguments, Namespace::Value)
            }
            Kind::MethodCall {
                receiver,
                name,
                arguments,
            } => {
                let name = self.package.name(c.file, name)?;
                if let Some(method) =
                    crate::support::comparisons::Method::from_name(self.package.names.get(name))
                {
                    self.prepare_comparison_method(c, node, method)?;
                }
                self.prepare_slice_method(c, node, name)?;
                self.note_clone_name(c, node, name);
                self.facts.method_calls.insert(
                    NodeKey { file: c.file, node },
                    super::model::MethodCall {
                        scope: c.scope,
                        receiver,
                        name,
                        method: None,
                        arguments,
                    },
                );
                self.visit(c, receiver, Namespace::Value)?;
                self.visit_list(c, arguments, Namespace::Value)
            }
            Kind::Applied {
                base,
                arguments,
                output,
                ..
            } => {
                self.visit(c, base, ns)?;
                self.generic_arguments(c, arguments)?;
                self.optional(c, output, Namespace::Type)
            }
            Kind::Qualified {
                ty,
                trait_path,
                path,
            } => {
                let (_, names) = self.path_names(c, path)?;
                if let Some(&name) = names.last() {
                    self.prepare_slice_method(c, node, name)?;
                    self.note_clone_name(c, node, name);
                }
                let Some((_, segments)) =
                    self.package.unit(c.file).configured.ast.path_segments(path)
                else {
                    return Err(self.error(c, node, "qualified suffix requires parsed segments"));
                };
                self.facts.qualified_paths.insert(
                    NodeKey { file: c.file, node },
                    super::model::QualifiedPath {
                        namespace: ns,
                        scope: c.scope,
                        ty,
                        trait_path,
                        suffix: segments.iter().copied().zip(names).collect(),
                    },
                );
                self.visit(c, ty, Namespace::Type)?;
                self.optional(c, trait_path, Namespace::Type)?;
                // Associated suffixes are selected from typed receiver/trait facts later.
                self.path_arguments(c, path)
            }
            Kind::Constraint {
                target,
                value,
                bounds,
            } => {
                self.associated_binding(c, node, target)?;
                self.optional(c, value, Namespace::Type)?;
                self.visit_list(c, bounds, Namespace::Type)
            }
            Kind::FieldDefinition { ty, visibility, .. } => {
                let access = self.visibility(c, visibility)?;
                self.facts
                    .field_access
                    .insert(NodeKey { file: c.file, node }, access);
                self.visit(c, ty, Namespace::Type)
            }
            Kind::Generic(index) => {
                let generic = self.package.unit(c.file).configured.ast.generics[index as usize];
                self.optional(c, generic.ty, Namespace::Type)?;
                self.visit_list(c, generic.bounds, Namespace::Type)?;
                let first = self.facts.references.len();
                self.optional(
                    c,
                    generic.default,
                    if generic.flavor == GenericFlavor::Const {
                        Namespace::Value
                    } else {
                        Namespace::Type
                    },
                )?;
                for reference in &self.facts.references[first..] {
                    if let Target::Definition(target) = reference.target {
                        let definition = &self.facts.definitions[target.0 as usize];
                        if definition.kind == DefKind::Generic
                            && definition.scope == c.scope
                            && definition
                                .node
                                .is_some_and(|key| key.node.get() >= node.get())
                        {
                            return Err(self.error(
                                c,
                                node,
                                "generic default references a parameter not yet declared",
                            ));
                        }
                    }
                }
                Ok(())
            }
            Kind::Variant(index) => {
                let variant = self.package.unit(c.file).configured.ast.variants[index as usize];
                self.visit_list(c, variant.fields, Namespace::Type)?;
                self.optional(c, variant.discriminant, Namespace::Value)
            }
            Kind::Label { name, value } => {
                let name = self.package.name(c.file, name)?;
                self.define(
                    Definition {
                        name,
                        namespace: Namespace::Label,
                        kind: DefKind::Label,
                        scope: c.scope,
                        node: Some(NodeKey { file: c.file, node }),
                        access: Access::Within(c.module),
                        members: None,
                    },
                    false,
                )?;
                self.visit(c, value, Namespace::Value)
            }
            Kind::Break { label, value } => {
                if let Some(label) = label {
                    self.label(c, node, label)?;
                }
                self.optional(c, value, Namespace::Value)
            }
            Kind::Continue(Some(label)) => self.label(c, node, label),
            Kind::Binding { .. }
            | Kind::Pattern { .. }
            | Kind::StructPattern { .. }
            | Kind::ReferencePattern { .. } => self.bind_pattern(c, node, DefKind::Local),
            _ => {
                let mark = self.scratch.len();
                self.package
                    .unit(c.file)
                    .configured
                    .ast
                    .push_children(node, self.scratch);
                let end = self.scratch.len();
                for index in (mark..end).rev() {
                    self.visits.push((c, self.scratch[index], ns));
                }
                self.scratch.truncate(mark);
                Ok(())
            }
        }
    }
    fn parameters(&mut self, c: Context, parameters: List) -> Result<(), Error> {
        self.bind_patterns(c, parameters)
    }
    fn item_body(&mut self, c: Context, node: NodeId, index: u32) -> Result<(), Error> {
        let item = self.package.unit(c.file).configured.ast.items[index as usize];
        if let ItemKind::Trait {
            restriction: Some(clause),
            ..
        } = item.kind
        {
            self.resolve_impl_restriction(c, node, clause)?;
        }
        if matches!(item.kind, ItemKind::MacroRules { .. })
            && let Some(hygiene) = self.hygiene
        {
            hygiene.snapshot(c.module, node, c.scope, self.facts.definitions.len());
        }
        self.visit_list(c, item.generics, Namespace::Type)?;
        self.visit_list(c, item.where_clause, Namespace::Type)?;
        match item.kind {
            ItemKind::ForeignType
            | ItemKind::Module { .. }
            | ItemKind::Use(_)
            | ItemKind::ExternCrate { .. }
            | ItemKind::MacroRules { .. } => Ok(()),
            ItemKind::DeclarativeMacro { .. } => self.scoped_declared(c, node),
            ItemKind::MacroInvocation(value) => self.visit(c, value, Namespace::Value),
            ItemKind::Function {
                parameters,
                result,
                body,
                ..
            } => {
                self.parameters(c, parameters)?;
                self.optional(c, result, Namespace::Type)?;
                self.optional(c, body, Namespace::Value)
            }
            ItemKind::Const { ty, value } | ItemKind::Static { ty, value, .. } => {
                self.visit(c, ty, Namespace::Type)?;
                self.optional(c, value, Namespace::Value)
            }
            ItemKind::TraitAlias { bounds, .. } => self.visit_list(c, bounds, Namespace::Type),
            ItemKind::TypeAlias { bounds, ty } => {
                self.visit_list(c, bounds, Namespace::Type)?;
                self.optional(c, ty, Namespace::Type)
            }
            ItemKind::Struct { fields, .. } | ItemKind::Union { fields } => {
                self.visit_list(c, fields, Namespace::Type)
            }
            ItemKind::Enum { variants } => self.visit_list(c, variants, Namespace::Type),
            ItemKind::Trait { bounds, items, .. } => {
                self.visit_list(c, bounds, Namespace::Type)?;
                self.visit_list(c, items, Namespace::Value)
            }
            ItemKind::Impl {
                trait_path,
                ty,
                items,
                ..
            } => {
                self.optional(c, trait_path, Namespace::Type)?;
                self.visit(c, ty, Namespace::Type)?;
                self.visit_list(c, items, Namespace::Value)
            }
            ItemKind::ExternBlock { items, .. } => self.visit_list(c, items, Namespace::Value),
        }
        .map_err(|mut e| {
            if e.span.start == 0 {
                e.span = self.package.unit(c.file).configured.ast.node(node).span;
            }
            e
        })
    }
    pub fn reference(&mut self, c: Context, node: NodeId, ns: Namespace) -> Result<(), Error> {
        self.argument_reference(c, node, ns, false)
    }
    fn argument_reference(
        &mut self,
        c: Context,
        node: NodeId,
        mut ns: Namespace,
        const_argument: bool,
    ) -> Result<(), Error> {
        if let Kind::Name(span, name) = self.kind(c, node)
            && self.package.unit(c.file).lifetime_spelling(span)
        {
            return self.lifetime(c, node, span, name);
        }
        let (absolute, path) = self.pooled_path(c, node)?;
        let self_constructor = ns == Namespace::Value
            && !absolute
            && path.len() == 1
            && self.package.names.get(path[0]) == "Self";
        let mut builtin = false;
        for &name in &path.names {
            builtin |= self.builtin_name(name);
        }
        // Builtin preparation only reacts to the spellings `builtin_name` recognizes.
        if builtin {
            self.prepare_builtin_reference(c, node, absolute, ns, path.view())?;
        }
        // A relative ordinary single-segment type path is found exactly when its
        // lexical lookup is, so that lookup is not repeated before the full one.
        let lexical = ns == Namespace::Type
            && path.len() == 1
            && !absolute
            && !matches!(self.package.names.get(path[0]), "crate" | "self" | "super");
        let mut found = if lexical {
            Some(self.lookup_path(c, node, absolute, path.view(), LookupMode::Use(ns), None)?)
        } else {
            None
        };
        if ns == Namespace::Type && path.len() == 1 {
            // Syntactically ambiguous const generic arguments are typed after lookup.
            let offset = super::hygiene::name_offset(self.package, NodeKey { file: c.file, node });
            let missing = match &found {
                Some(found) => matches!(found, Lookup::Missing),
                None => self.hygienic_lexical(c, offset, path[0], ns)?.is_none(),
            };
            if missing
                && let Some((_, binding)) =
                    self.hygienic_lexical(c, offset, path[0], Namespace::Value)?
                && (self.facts.definitions[binding.target.0 as usize].kind == DefKind::Generic
                    || (const_argument
                        && self.facts.definitions[binding.target.0 as usize].kind
                            == DefKind::Const))
            {
                ns = Namespace::Value;
                found = None;
            }
        }
        let found = match found {
            Some(found) => found,
            None if self_constructor => self.self_constructor(c, path[0]),
            None => self.lookup_path(c, node, absolute, path.view(), LookupMode::Use(ns), None)?,
        };
        let found = self.primitive_reference(found, absolute, path.view(), ns);
        let target = self.target(c, node, &found, ns)?;
        if let Lookup::Associated(owner, start) | Lookup::Member(_, owner, start) = found {
            self.associated_path(c, node, start, &path, ns, owner)?;
        }
        self.recycle_path(path);
        self.facts.references.push(Reference {
            scope: c.scope,
            node: NodeKey { file: c.file, node },
            namespace: ns,
            target,
        });
        // Paths of plain name segments carry no generic arguments.
        let ast = &self.package.unit(c.file).configured.ast;
        if let Some((_, segments)) = ast.path_segments(node)
            && segments
                .iter()
                .all(|segment| matches!(ast.node(*segment).kind, Kind::Name(..)))
        {
            return Ok(());
        }
        self.path_arguments(c, node)
    }
    /// Ordinary module members win. A failed relative primitive-head reference
    /// then uses the same canonical associated-prefix route as a bare primitive.
    /// Import/module grammar and actual lookup errors never enter this adapter.
    fn primitive_reference(
        &self,
        found: Lookup,
        absolute: bool,
        path: super::paths::Identified<'_>,
        namespace: Namespace,
    ) -> Lookup {
        if absolute
            || !matches!(namespace, Namespace::Type | Namespace::Value)
            || (namespace == Namespace::Value && path.len() == 1)
        {
            return found;
        }
        let module = matches!(&found, Lookup::Found(binding)
            if matches!(self.facts.definitions[binding.target.0 as usize].kind,
                DefKind::Module | DefKind::ExternCrate));
        if !module && !matches!(found, Lookup::Missing) {
            return found;
        }
        let Some(primitive) = path
            .first()
            .and_then(|name| self.facts.primitives.get(name))
        else {
            return found;
        };
        if path.len() > 1 {
            Lookup::Associated(*primitive, 1)
        } else {
            Lookup::Found(super::model::Binding {
                target: *primitive,
                access: Access::Public,
                origin: super::model::Origin::Direct,
                ambiguous: false,
            })
        }
    }
    /// Demand language metadata that a reference path may name.
    fn prepare_builtin_reference(
        &mut self,
        c: Context,
        node: NodeId,
        absolute: bool,
        ns: Namespace,
        path: super::paths::Identified<'_>,
    ) -> Result<(), Error> {
        if ns == Namespace::Value {
            self.prepare_clone_path(c, node, absolute, path)?;
        }
        if ns == Namespace::Value
            && path.len() > 1
            && let Some(&name) = path.last()
        {
            self.prepare_slice_method(c, node, name)?;
            self.note_clone_name(c, node, name);
        }
        if ns == Namespace::Type
            || ns == Namespace::Value
                && path.len() > 1
                && path.last().is_some_and(|name| {
                    crate::support::comparisons::Method::from_name(self.package.names.get(*name))
                        .is_some()
                })
            || ns == Namespace::Value
                && path.last().is_some_and(|name| {
                    matches!(self.package.names.get(*name), "next" | "into_iter")
                })
            || ns == Namespace::Value
                && path.iter().any(|name| {
                    let spelling = self.package.names.get(*name);
                    LibraryType::from_name(spelling).is_some()
                        || matches!(
                            spelling,
                            "primitive"
                                | "Some"
                                | "None"
                                | "Ok"
                                | "Err"
                                | "ptr"
                                | "read_volatile"
                                | "write_volatile"
                        )
                })
        {
            self.prepare_builtin_path(c, node, absolute, ns, path)?;
        }
        Ok(())
    }
    fn associated_path(
        &mut self,
        c: Context,
        node: NodeId,
        start: usize,
        names: &[crate::support::names::NameId],
        namespace: Namespace,
        owner: DefId,
    ) -> Result<(), Error> {
        let Some((_, segments)) = self.package.unit(c.file).configured.ast.path_segments(node)
        else {
            return Err(self.error(c, node, "associated path requires parsed segments"));
        };
        let prefix = start
            .checked_sub(1)
            .and_then(|i| segments.get(i))
            .copied()
            .ok_or_else(|| self.error(c, node, "missing associated path prefix"))?;
        if segments.len() != names.len() || start >= names.len() {
            return Err(self.error(c, node, "invalid associated path suffix"));
        }
        self.facts.associated_paths.insert(
            NodeKey { file: c.file, node },
            super::model::AssociatedPath {
                owner,
                namespace,
                scope: c.scope,
                prefix,
                suffix: segments[start..]
                    .iter()
                    .copied()
                    .zip(names[start..].iter().copied())
                    .collect(),
            },
        );
        Ok(())
    }
    pub(super) fn lifetime(
        &mut self,
        c: Context,
        node: NodeId,
        span: crate::syntax::token_tree::Span,
        parsed: Option<crate::support::names::NameId>,
    ) -> Result<(), Error> {
        let name = self.package.parsed_name(c.file, span, parsed)?;
        if matches!(self.package.names.get(name), "static" | "_") {
            return Ok(());
        }
        let target = self.hygienic_name(c, node, span.start, name, Namespace::Lifetime)?;
        self.facts.references.push(Reference {
            scope: c.scope,
            node: NodeKey { file: c.file, node },
            namespace: Namespace::Lifetime,
            target,
        });
        Ok(())
    }
    fn label(
        &mut self,
        c: Context,
        node: NodeId,
        span: crate::syntax::token_tree::Span,
    ) -> Result<(), Error> {
        let name = self.package.name(c.file, span)?;
        let Some((_, binding)) = self.hygienic_lexical(c, span.start, name, Namespace::Label)?
        else {
            return Err(self.error(c, node, "unresolved label"));
        };
        self.facts.references.push(Reference {
            scope: c.scope,
            node: NodeKey { file: c.file, node },
            namespace: Namespace::Label,
            target: Target::Definition(binding.target),
        });
        Ok(())
    }
    fn associated_binding(
        &mut self,
        c: Context,
        node: NodeId,
        mut target: NodeId,
    ) -> Result<(), Error> {
        let mut arguments = None;
        loop {
            let children = match self.kind(c, target) {
                Kind::Path {
                    absolute: false,
                    segments,
                } => segments,
                Kind::Applied {
                    base,
                    arguments: list,
                    output: None,
                    parenthesized: false,
                } if arguments.is_none() => {
                    arguments = Some(list);
                    target = base;
                    continue;
                }
                Kind::Name(span, name) => {
                    let name = self.package.parsed_name(c.file, span, name)?;
                    self.facts.associated_bindings.insert(
                        NodeKey { file: c.file, node },
                        super::model::AssociatedBinding { name, arguments },
                    );
                    if let Some(arguments) = arguments {
                        self.generic_arguments(c, arguments)?;
                    }
                    return Ok(());
                }
                _ => return Ok(()),
            };
            let [child] = self.package.unit(c.file).configured.ast.list(children) else {
                return Ok(());
            };
            target = *child;
        }
    }

    fn path_arguments(&mut self, c: Context, node: NodeId) -> Result<(), Error> {
        match self.kind(c, node) {
            Kind::Path { segments, .. } => {
                for offset in segments.start..segments.end {
                    let child = self.package.unit(c.file).configured.ast.edges[offset as usize];
                    self.path_arguments(c, child)?;
                }
            }
            Kind::Applied {
                arguments, output, ..
            } => {
                self.generic_arguments(c, arguments)?;
                self.optional(c, output, Namespace::Type)?;
            }
            _ => {}
        }
        Ok(())
    }
    fn generic_arguments(&mut self, c: Context, arguments: List) -> Result<(), Error> {
        for offset in arguments.start..arguments.end {
            let node = self.package.unit(c.file).configured.ast.edges[offset as usize];
            if matches!(self.kind(c, node), Kind::Path { .. } | Kind::Name(..)) {
                self.argument_reference(c, node, Namespace::Type, true)?;
            } else {
                let ns = if matches!(
                    self.kind(c, node),
                    Kind::Block { .. } | Kind::Literal(_) | Kind::Unary { .. }
                ) {
                    Namespace::Value
                } else {
                    Namespace::Type
                };
                self.visit(c, node, ns)?;
            }
        }
        Ok(())
    }
}
