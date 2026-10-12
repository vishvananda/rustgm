//! Pattern alternatives agree on binding identities before any are installed.
use super::{
    index::{Context, Resolver},
    lookup::Lookup,
    model::{Access, DefKind, Definition, Namespace, Reference, Target},
    names::NameId,
    package::Error,
    unit::NodeKey,
};
use crate::syntax::ast::{Kind, NodeId, PatternFlavor};
use std::collections::{HashMap, HashSet};
#[derive(Clone, Copy)]
struct Local {
    name: NameId,
    node: NodeId,
    mutable: bool,
    by_ref: bool,
}
impl Resolver<'_> {
    pub fn bind_pattern(
        &mut self,
        c: Context,
        node: NodeId,
        kind: DefKind,
        seen: &mut HashSet<NameId>,
    ) -> Result<(), Error> {
        let mut bindings = Vec::new();
        self.pattern(c, node, &mut bindings)?;
        for local in bindings {
            if !seen.insert(local.name) {
                return Err(self.error(
                    c,
                    local.node,
                    "duplicate binding in pattern or parameters",
                ));
            }
            self.define(
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
        }
        Ok(())
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
    ) -> Result<(), Error> {
        if !self.active(c, node) {
            return Ok(());
        }
        match self.kind(c, node) {
            Kind::Binding {
                name,
                mutable,
                by_ref,
                subpattern,
            } => {
                let name = self.package.name(c.file, name)?;
                if let Lookup::Found(binding) = self.lexical(c.scope, name, Namespace::Value)
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
                    self.pattern(c, subpattern, bindings)?;
                }
            }
            Kind::Pattern {
                flavor: PatternFlavor::Or,
                elements,
                ..
            } => {
                let mut expected = None;
                let mut first = Vec::new();
                for offset in elements.start..elements.end {
                    let child = self.package.unit(c.file).configured.ast.edges[offset as usize];
                    let mut branch = Vec::new();
                    self.pattern(c, child, &mut branch)?;
                    let mut names = HashMap::new();
                    for local in &branch {
                        if names
                            .insert(local.name, (local.mutable, local.by_ref))
                            .is_some()
                        {
                            return Err(self.error(c, local.node, "duplicate binding in pattern"));
                        }
                    }
                    if let Some(expected) = &expected {
                        if *expected != names {
                            return Err(self.error(
                                c,
                                child,
                                "or-pattern alternatives bind different names or modes",
                            ));
                        }
                    } else {
                        expected = Some(names);
                        first = branch;
                    }
                }
                bindings.extend(first);
            }
            Kind::Pattern { path, elements, .. } => {
                if let Some(path) = path {
                    self.reference(c, path, Namespace::Value)?;
                }
                for offset in elements.start..elements.end {
                    let child = self.package.unit(c.file).configured.ast.edges[offset as usize];
                    self.pattern(c, child, bindings)?;
                }
            }
            Kind::StructPattern { path, fields, .. } => {
                self.reference(c, path, Namespace::Type)?;
                for offset in fields.start..fields.end {
                    let child = self.package.unit(c.file).configured.ast.edges[offset as usize];
                    self.pattern(c, child, bindings)?;
                }
            }
            Kind::Path { .. } | Kind::Qualified { .. } => self.visit(c, node, Namespace::Value)?,
            Kind::PatternRange { start, end, .. } => {
                for bound in [start, end].into_iter().flatten() {
                    if let Kind::Binding { name, .. } = self.kind(c, bound) {
                        let name = self.package.name(c.file, name)?;
                        let target = self.target(
                            c,
                            bound,
                            &self.lexical(c.scope, name, Namespace::Value),
                            Namespace::Value,
                        )?;
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
                    .push_children(node, &mut self.scratch);
                let end = self.scratch.len();
                for index in mark..end {
                    self.pattern(c, self.scratch[index], bindings)?;
                }
                self.scratch.truncate(mark);
            }
        }
        Ok(())
    }
}
