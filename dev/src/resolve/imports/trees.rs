//! Flatten grouped imports once, preserving each identifier and the actual star.
use super::super::{
    index::{Context, Resolver},
    model::{Access, Import, Namespace},
    names::NameId,
    package::Error,
    unit::NodeKey,
};
use crate::{
    expand::marks::ItemContext,
    syntax::ast::{Kind, NodeId},
};

enum Step {
    Tree {
        node: NodeId,
        absolute: bool,
        relevant: bool,
    },
    Truncate(usize),
}

impl Resolver<'_> {
    pub fn import(&mut self, c: Context, node: NodeId, access: Access) -> Result<(), Error> {
        if self.select_source_prelude(c, node)? {
            return Ok(());
        }
        let mut path = self.paths.pop().unwrap_or_default();
        path.truncate(0);
        let root = node;
        let mut pending = vec![Step::Tree {
            node,
            absolute: false,
            relevant: false,
        }];
        while let Some(step) = pending.pop() {
            let (node, absolute, relevant) = match step {
                Step::Tree {
                    node,
                    absolute,
                    relevant,
                } => (node, absolute, relevant),
                Step::Truncate(len) => {
                    path.truncate(len);
                    continue;
                }
            };
            let Kind::UseTree(index) = self.kind(c, node) else {
                return Err(self.error(c, node, "expected import tree"));
            };
            let tree = self.package.unit(c.file).configured.ast.use_trees[index as usize];
            if node == root
                && let Some(span) = self.legacy_import_start(c, node, tree)?
            {
                self.package.legacy_root(c.file, span, &mut path)?;
            }
            let start = path.names.len();
            let local_absolute = tree
                .prefix
                .map(|node| self.package.append_path(c.file, node, &mut path))
                .transpose()?
                .unwrap_or(false);
            let absolute = absolute || local_absolute;
            // Irrelevant prefixes need no repeated spelling scan as a group
            // deepens. Source provider/client modes never prepare bootstrap metadata.
            let relevant = !self.package.uses_source_bindings()
                && (relevant
                    || path.names[start..]
                        .iter()
                        .any(|name| self.builtin_name(*name)));
            if relevant {
                self.prepare_builtin_path(c, node, absolute, Namespace::Type, path.view())?;
            }
            if tree.children.start == tree.children.end {
                let alias = tree
                    .rename
                    .map(|node| self.import_alias(c, node))
                    .transpose()?;
                let glob_context = tree
                    .glob
                    .map(|span| self.package.item_context(c.file, span.start))
                    .transpose()?
                    .unwrap_or(ItemContext::ROOT);
                self.facts.imports.push(Import {
                    scope: c.scope,
                    node: NodeKey { file: c.file, node },
                    path: path.names.as_slice().into(),
                    contexts: path.freeze_contexts(glob_context),
                    absolute,
                    alias,
                    glob: tree.glob.is_some(),
                    access,
                });
                path.truncate(start);
            } else {
                pending.push(Step::Truncate(start));
                let ast = &self.package.unit(c.file).configured.ast;
                pending.extend(
                    ast.list(tree.children)
                        .iter()
                        .rev()
                        .map(|child| Step::Tree {
                            node: *child,
                            absolute,
                            relevant,
                        }),
                );
            }
        }
        self.recycle_path(path);
        Ok(())
    }

    /// The token whose edition decides whether a `use` path starts at the crate
    /// root: a plain first segment, or the empty prefix of a root group.
    fn legacy_import_start(
        &mut self,
        c: Context,
        node: NodeId,
        tree: crate::syntax::item::UseTree,
    ) -> Result<Option<crate::syntax::token_tree::Span>, Error> {
        let ast = &self.package.unit(c.file).configured.ast;
        let Some(prefix) = tree.prefix else {
            let nested = tree.children.start != tree.children.end;
            return Ok(nested.then(|| ast.node(node).span));
        };
        let first = match ast.node(prefix).kind {
            Kind::Path {
                absolute: false,
                segments,
            } if segments.start < segments.end => ast.edges[segments.start as usize],
            Kind::Path { .. } => return Ok(None),
            _ => prefix,
        };
        let Kind::Name(span, name) = ast.node(first).kind else {
            return Ok(None);
        };
        let name = self.package.parsed_name(c.file, span, name)?;
        Ok((!matches!(
            self.package.names.get(name),
            "crate" | "self" | "super" | "Self"
        ))
        .then_some(span))
    }

    fn import_alias(&mut self, c: Context, node: NodeId) -> Result<(NameId, ItemContext), Error> {
        let Kind::Name(span, name) = self.kind(c, node) else {
            return Err(self.error(c, node, "expected import alias name"));
        };
        let name = self.package.parsed_name(c.file, span, name)?;
        Ok((name, self.package.item_context(c.file, span.start)?))
    }
}
