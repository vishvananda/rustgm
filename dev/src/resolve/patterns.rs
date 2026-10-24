//! Pattern alternatives agree on binding identities before any are installed.
use super::{
    index::{Context, Resolver},
    model::{Access, DefKind, Definition, Namespace, Reference, Target},
    names::NameId,
    package::Error,
    unit::NodeKey,
};
use crate::expand::marks::LocalContext;
use crate::support::hash::HashMap;
use crate::syntax::ast::{Kind, NodeId, PatternFlavor};
use std::hash::Hash;
#[derive(Clone, Copy)]
pub(crate) struct Local {
    name: NameId,
    node: NodeId,
    mutable: bool,
    by_ref: bool,
}
/// Patterns bind a handful of names: search them linearly and index only
/// unusually large binding sets.
pub(crate) struct SmallMap<K, V> {
    entries: Vec<(K, V)>,
    index: HashMap<K, usize>,
}
impl<K, V> Default for SmallMap<K, V> {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            index: HashMap::default(),
        }
    }
}
impl<K: Copy + Eq + Hash, V: Copy> SmallMap<K, V> {
    const LINEAR: usize = 8;
    fn position(&self, key: K) -> Option<usize> {
        if self.index.is_empty() {
            self.entries.iter().position(|(entry, _)| *entry == key)
        } else {
            self.index.get(&key).copied()
        }
    }
    pub fn get(&self, key: K) -> Option<V> {
        self.position(key).map(|index| self.entries[index].1)
    }
    /// Insert or replace, returning the previous value like `HashMap::insert`.
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        if let Some(index) = self.position(key) {
            return Some(std::mem::replace(&mut self.entries[index].1, value));
        }
        self.entries.push((key, value));
        if self.entries.len() > Self::LINEAR {
            if self.index.is_empty() {
                self.index.extend(
                    self.entries
                        .iter()
                        .enumerate()
                        .map(|(index, (entry, _))| (*entry, index)),
                );
            } else {
                self.index.insert(key, self.entries.len() - 1);
            }
        }
        None
    }
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    pub fn clear(&mut self) {
        self.entries.clear();
        self.index.clear();
    }
}
/// Reusable binding storage for one pattern or alternative. Patterns nested in
/// const arguments take another frame from the resolver's pool.
#[derive(Default)]
pub(crate) struct PatternFrame {
    locals: Vec<Local>,
    aliases: Vec<(NodeId, NodeId)>,
    names: SmallMap<super::hygiene::Ident, Local>,
    representatives: SmallMap<NodeId, super::model::DefId>,
    seen: SmallMap<super::hygiene::Ident, ()>,
}
impl Resolver<'_> {
    fn take_frame(&mut self) -> PatternFrame {
        let mut frame = self.patterns.pop().unwrap_or_default();
        frame.locals.clear();
        frame.aliases.clear();
        frame.names.clear();
        frame.representatives.clear();
        frame.seen.clear();
        frame
    }
    pub fn bind_pattern(&mut self, c: Context, node: NodeId, kind: DefKind) -> Result<(), Error> {
        let mut frame = self.take_frame();
        let result = self.bind_with(c, node, kind, &mut frame, None);
        self.patterns.push(frame);
        result
    }
    /// Bind several patterns that must not repeat an identity, such as parameters.
    pub fn bind_patterns(
        &mut self,
        c: Context,
        parameters: crate::syntax::ast::List,
    ) -> Result<(), Error> {
        let mut shared = self.take_frame();
        let result = self.parameter_patterns(c, parameters, &mut shared.seen);
        self.patterns.push(shared);
        result
    }
    fn parameter_patterns(
        &mut self,
        c: Context,
        parameters: crate::syntax::ast::List,
        seen: &mut SmallMap<super::hygiene::Ident, ()>,
    ) -> Result<(), Error> {
        for offset in parameters.start..parameters.end {
            let mut node = self.package.unit(c.file).configured.ast.edges[offset as usize];
            if !self.active(c, node) {
                continue;
            }
            while let Kind::Attributed { value, .. } = self.kind(c, node) {
                node = value;
            }
            if let Kind::Parameter { pattern, ty } = self.kind(c, node) {
                if let Some(ty) = ty {
                    self.visit(c, ty, Namespace::Type)?;
                }
                let mut frame = self.take_frame();
                let result = self.bind_with(c, pattern, DefKind::Parameter, &mut frame, Some(seen));
                self.patterns.push(frame);
                result?;
            } else {
                self.visit(c, node, Namespace::Value)?;
            }
        }
        Ok(())
    }
    fn bind_with(
        &mut self,
        c: Context,
        node: NodeId,
        kind: DefKind,
        frame: &mut PatternFrame,
        shared: Option<&mut SmallMap<super::hygiene::Ident, ()>>,
    ) -> Result<(), Error> {
        let PatternFrame {
            locals: bindings,
            aliases,
            representatives,
            seen: own,
            ..
        } = frame;
        let seen = shared.unwrap_or(own);
        self.pattern(c, node, bindings, aliases)?;
        let linked = !aliases.is_empty();
        for &local in bindings.iter() {
            if seen.insert(self.pattern_identity(c, local)?, ()).is_some() {
                return Err(self.error(
                    c,
                    local.node,
                    "duplicate binding in pattern or parameters",
                ));
            }
            let definition = self.define(
                Definition {
                    name: local.name,
                    namespace: Namespace::Value,
                    kind,
                    scope: c.scope,
                    node: Some(NodeKey {
                        file: c.file,
                        node: local.node,
                    }),
                    access: Access::Within(c.module),
                    members: None,
                },
                true,
            )?;
            if linked {
                representatives.insert(local.node, definition);
            }
        }
        if !linked {
            return Ok(());
        }
        // An outer alternative links representatives only after visiting its
        // nested alternatives. Reverse order resolves parents before children.
        for &(alias, representative) in aliases.iter().rev() {
            let definition = representatives
                .get(representative)
                .ok_or_else(|| self.error(c, alias, "missing pattern binding representative"))?;
            representatives.insert(alias, definition);
            self.facts.pattern_aliases.push((
                NodeKey {
                    file: c.file,
                    node: alias,
                },
                definition,
            ));
        }
        Ok(())
    }
    /// Every alternative binds the first alternative's identities and modes.
    fn alternatives(
        &mut self,
        c: Context,
        elements: crate::syntax::ast::List,
        first: &mut PatternFrame,
        branch: &mut PatternFrame,
        aliases: &mut Vec<(NodeId, NodeId)>,
    ) -> Result<(), Error> {
        let mut expected = false;
        for offset in elements.start..elements.end {
            let child = self.package.unit(c.file).configured.ast.edges[offset as usize];
            branch.locals.clear();
            branch.names.clear();
            self.pattern(c, child, &mut branch.locals, aliases)?;
            for &local in &branch.locals {
                if branch
                    .names
                    .insert(self.pattern_identity(c, local)?, local)
                    .is_some()
                {
                    return Err(self.error(c, local.node, "duplicate binding in pattern"));
                }
            }
            if expected {
                if first.names.len() != branch.names.len() {
                    return Err(self.error(
                        c,
                        child,
                        "or-pattern alternatives bind different names or modes",
                    ));
                }
                // Iterate source order; hash seeds must not order the
                // retained identity facts consumed by later phases.
                for &local in &branch.locals {
                    let first = first
                        .names
                        .get(self.pattern_identity(c, local)?)
                        .filter(|first| {
                            first.mutable == local.mutable && first.by_ref == local.by_ref
                        })
                        .ok_or_else(|| {
                            self.error(
                                c,
                                child,
                                "or-pattern alternatives bind different names or modes",
                            )
                        })?;
                    aliases.push((local.node, first.node));
                }
            } else {
                std::mem::swap(first, branch);
                expected = true;
            }
        }
        Ok(())
    }
    fn pattern_identity(&self, c: Context, local: Local) -> Result<super::hygiene::Ident, Error> {
        let context = if self.hygiene.is_some() {
            let offset = super::hygiene::name_offset(
                self.package,
                NodeKey {
                    file: c.file,
                    node: local.node,
                },
            );
            self.package.local_context(c.file, offset)?
        } else {
            LocalContext::ROOT
        };
        Ok(super::hygiene::Ident::new(local.name, context))
    }
    #[expect(
        clippy::too_many_lines,
        reason = "bounded pattern dispatch keeps alternative identity and reference rules together"
    )]
    fn pattern(
        &mut self,
        c: Context,
        node: NodeId,
        bindings: &mut Vec<Local>,
        aliases: &mut Vec<(NodeId, NodeId)>,
    ) -> Result<(), Error> {
        if !self.active(c, node) {
            return Ok(());
        }
        match self.kind(c, node) {
            Kind::Receiver {
                name,
                lifetime,
                mutable,
            } => {
                if lifetime.end > lifetime.start {
                    self.lifetime(c, node, lifetime, None)?;
                }
                bindings.push(Local {
                    name: self.package.name(c.file, name)?,
                    node,
                    mutable,
                    by_ref: true,
                });
            }
            Kind::Binding {
                name,
                symbol,
                mutable,
                by_ref,
                subpattern,
            } => {
                let offset = name.start;
                let name = self.package.parsed_name(c.file, name, symbol)?;
                if let Some((_, binding)) =
                    self.hygienic_lexical(c, offset, name, Namespace::Value)?
                    && matches!(
                        self.facts.definitions[binding.target.0 as usize].kind,
                        DefKind::Const | DefKind::Static | DefKind::Constructor | DefKind::Variant
                    )
                {
                    if mutable
                        || by_ref
                        || subpattern.is_some()
                        || self.facts.definitions[binding.target.0 as usize].kind == DefKind::Static
                    {
                        return Err(self.error(
                            c,
                            node,
                            "pattern binding cannot shadow a constant, static or constructor",
                        ));
                    }
                    self.facts.references.push(Reference {
                        scope: c.scope,
                        node: NodeKey { file: c.file, node },
                        namespace: Namespace::Value,
                        target: Target::Definition(binding.target),
                    });
                } else {
                    bindings.push(Local {
                        name,
                        node,
                        mutable,
                        by_ref,
                    });
                }
                if let Some(subpattern) = subpattern {
                    self.pattern(c, subpattern, bindings, aliases)?;
                }
            }
            Kind::Pattern {
                flavor: PatternFlavor::Or,
                elements,
                ..
            } => {
                let mut first = self.take_frame();
                let mut branch = self.take_frame();
                let result = self.alternatives(c, elements, &mut first, &mut branch, aliases);
                bindings.extend_from_slice(&first.locals);
                self.patterns.push(branch);
                self.patterns.push(first);
                result?;
            }
            Kind::Pattern { path, elements, .. } => {
                if let Some(path) = path {
                    self.reference(c, path, Namespace::Value)?;
                }
                for offset in elements.start..elements.end {
                    let child = self.package.unit(c.file).configured.ast.edges[offset as usize];
                    self.pattern(c, child, bindings, aliases)?;
                }
            }
            Kind::StructPattern { path, fields, .. } => {
                self.reference(c, path, Namespace::Type)?;
                for offset in fields.start..fields.end {
                    let child = self.package.unit(c.file).configured.ast.edges[offset as usize];
                    self.pattern(c, child, bindings, aliases)?;
                }
            }
            // A one-name path pattern only arises from an opaque path fragment.
            Kind::Name(..) | Kind::Path { .. } | Kind::Qualified { .. } => {
                self.visit(c, node, Namespace::Value)?;
            }
            Kind::PatternRange { start, end, .. } => {
                for bound in [start, end].into_iter().flatten() {
                    if let Kind::Binding {
                        name: span, symbol, ..
                    } = self.kind(c, bound)
                    {
                        let name = self.package.parsed_name(c.file, span, symbol)?;
                        let target =
                            self.hygienic_name(c, bound, span.start, name, Namespace::Value)?;
                        self.facts.references.push(Reference {
                            scope: c.scope,
                            node: NodeKey {
                                file: c.file,
                                node: bound,
                            },
                            namespace: Namespace::Value,
                            target,
                        });
                    } else {
                        self.visit(c, bound, Namespace::Value)?;
                    }
                }
            }
            _ => {
                let mark = self.scratch.len();
                self.package
                    .unit(c.file)
                    .configured
                    .ast
                    .push_children(node, self.scratch);
                let end = self.scratch.len();
                for index in mark..end {
                    self.pattern(c, self.scratch[index], bindings, aliases)?;
                }
                self.scratch.truncate(mark);
            }
        }
        Ok(())
    }
}
