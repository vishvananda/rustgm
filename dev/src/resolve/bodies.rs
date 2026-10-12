//! Lexical traversal resolves uses in evaluation order and introduces local bindings.
use super::{
    index::{Context, Resolver},
    lookup::{Lookup, LookupMode},
    model::{Access, DefKind, Definition, Namespace, Reference, Target},
    package::{Error, ModuleId},
    unit::NodeKey,
};
use crate::syntax::{
    ast::{Kind, List, NodeId, TypeFlavor},
    item::{GenericFlavor, ItemKind},
};
use std::collections::HashSet;
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
        Ok(())
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
        match self.kind(c, node) {
            Kind::Item(index) => self.item_body(c, node, index),
            Kind::Path { .. } | Kind::Name(_) | Kind::CrateRoot(_) => self.reference(c, node, ns),
            Kind::Let {
                pattern,
                ty,
                value,
                otherwise,
            } => {
                self.optional(c, ty, Namespace::Type)?;
                self.optional(c, value, Namespace::Value)?;
                self.optional(c, otherwise, Namespace::Value)?;
                self.bind_pattern(c, pattern, DefKind::Local, &mut HashSet::new())
            }
            Kind::LetCondition { pattern, value } => {
                self.visit(c, value, Namespace::Value)?;
                self.bind_pattern(c, pattern, DefKind::Local, &mut HashSet::new())
            }
            Kind::For {
                pattern,
                iterator,
                body,
            } => {
                self.visit(c, iterator, Namespace::Value)?;
                self.bind_pattern(c, pattern, DefKind::Local, &mut HashSet::new())?;
                self.visit(c, body, Namespace::Value)
            }
            Kind::Arm {
                pattern,
                guard,
                value,
            } => {
                self.bind_pattern(c, pattern, DefKind::Local, &mut HashSet::new())?;
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
                self.bind_pattern(c, pattern, DefKind::Parameter, &mut HashSet::new())
            }
            Kind::Receiver { lifetime, .. } => {
                if lifetime.end > lifetime.start {
                    self.lifetime(c, node, lifetime)?;
                }
                let name = self
                    .package
                    .names
                    .intern("self")
                    .map_err(|e| self.error(c, node, e))?;
                self.define(
                    Definition {
                        name,
                        namespace: Namespace::Value,
                        kind: DefKind::Parameter,
                        scope: c.scope,
                        node: Some(NodeKey { file: c.file, node }),
                        access: Access::Within(c.module),
                        members: None,
                    },
                    false,
                )?;
                Ok(())
            }
            Kind::Cast { value, ty } => {
                self.visit(c, value, Namespace::Value)?;
                self.visit(c, ty, Namespace::Type)
            }
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
                self.visit(c, receiver, Namespace::Value)?;
                self.path_arguments(c, method)?;
                self.visit_list(c, arguments, Namespace::Value)
            }
            Kind::Applied {
                base,
                arguments,
                output,
            } => {
                self.visit(c, base, ns)?;
                self.visit_list(c, arguments, Namespace::Type)?;
                self.optional(c, output, Namespace::Type)
            }
            Kind::Qualified {
                ty,
                trait_path,
                path,
            } => {
                self.visit(c, ty, Namespace::Type)?;
                self.optional(c, trait_path, Namespace::Type)?;
                // Associated suffixes are selected from typed receiver/trait facts later.
                self.path_arguments(c, path)
            }
            Kind::Constraint { value, bounds, .. } => {
                self.optional(c, value, Namespace::Type)?;
                self.visit_list(c, bounds, Namespace::Type)
            }
            Kind::FieldDefinition { ty, .. } => self.visit(c, ty, Namespace::Type),
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
                            && definition.node.is_some_and(|key| key.node.0 >= node.0)
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
            Kind::Macro { .. } | Kind::UseTree(_) => Ok(()),
            Kind::Binding { .. }
            | Kind::Pattern { .. }
            | Kind::StructPattern { .. }
            | Kind::ReferencePattern { .. } => {
                self.bind_pattern(c, node, DefKind::Local, &mut HashSet::new())
            }
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
        let mut seen = HashSet::new();
        for offset in parameters.start..parameters.end {
            let mut node = self.package.unit(c.file).configured.ast.edges[offset as usize];
            if !self.active(c, node) {
                continue;
            }
            while let Kind::Attributed { value, .. } = self.kind(c, node) {
                node = value;
            }
            if let Kind::Parameter { pattern, ty } = self.kind(c, node) {
                self.optional(c, ty, Namespace::Type)?;
                self.bind_pattern(c, pattern, DefKind::Parameter, &mut seen)?;
            } else {
                self.visit(c, node, Namespace::Value)?;
            }
        }
        Ok(())
    }
    fn item_body(&mut self, c: Context, node: NodeId, index: u32) -> Result<(), Error> {
        let item = self.package.unit(c.file).configured.ast.items[index as usize];
        self.visit_list(c, item.generics, Namespace::Type)?;
        self.visit_list(c, item.where_clause, Namespace::Type)?;
        match item.kind {
            ItemKind::Module { .. }
            | ItemKind::Use(_)
            | ItemKind::ExternCrate { .. }
            | ItemKind::MacroRules { .. }
            | ItemKind::MacroInvocation(_) => Ok(()),
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
    pub fn reference(&mut self, c: Context, node: NodeId, mut ns: Namespace) -> Result<(), Error> {
        if let Kind::Name(span) = self.kind(c, node)
            && self
                .package
                .unit(c.file)
                .spelling(span)
                .map_err(|e| self.error(c, node, e.message))?
                .starts_with('\'')
        {
            return self.lifetime(c, node, span);
        }
        let (absolute, path) = self.path_names(c, node)?;
        if ns == Namespace::Type && path.len() == 1 {
            // Syntactically ambiguous const generic arguments are typed after lookup.
            if let Lookup::Missing = self.lexical(c.scope, path[0], ns)
                && let Lookup::Found(binding) = self.lexical(c.scope, path[0], Namespace::Value)
                && self.facts.definitions[binding.target.0 as usize].kind == DefKind::Generic
            {
                ns = Namespace::Value;
            }
        }
        let found = self.lookup_path(c, node, absolute, &path, LookupMode::Use(ns), None)?;
        let target = self.target(c, node, &found, ns)?;
        self.facts.references.push(Reference {
            scope: c.scope,
            node: NodeKey { file: c.file, node },
            namespace: ns,
            target,
        });
        self.path_arguments(c, node)
    }
    fn lifetime(
        &mut self,
        c: Context,
        node: NodeId,
        span: crate::syntax::token_tree::Span,
    ) -> Result<(), Error> {
        let name = self.package.name(c.file, span)?;
        if matches!(self.package.names.get(name), "static" | "_") {
            return Ok(());
        }
        let target = self.target(
            c,
            node,
            &self.lexical(c.scope, name, Namespace::Lifetime),
            Namespace::Lifetime,
        )?;
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
        let Lookup::Found(binding) = self.lexical(c.scope, name, Namespace::Label) else {
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
                self.visit_list(c, arguments, Namespace::Type)?;
                self.optional(c, output, Namespace::Type)?;
            }
            _ => {}
        }
        Ok(())
    }
}
