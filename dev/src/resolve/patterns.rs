//! Pattern alternatives agree on binding identities before any are installed.
use super::{
    index::{Context, Resolver},
    lookup::Lookup,
    model::{Access, DefKind, Definition, Namespace, Reference, Target},
    names::NameId,
    package::Error,
    unit::NodeKey,
};
use crate::expand::stream::ContextId;
use crate::support::hash::{HashMap, HashSet};
use crate::syntax::ast::{Kind, NodeId, PatternFlavor};
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
        seen: &mut HashSet<super::hygiene::Ident>,
    ) -> Result<(), Error> {
        let mut bindings = Vec::new();
        let mut aliases = Vec::new();
        self.pattern(c, node, &mut bindings, &mut aliases)?;
        let mut representatives = (!aliases.is_empty()).then(HashMap::default);
        for local in bindings {
            if !seen.insert(self.pattern_identity(c, local)?) {
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
            if let Some(representatives) = &mut representatives {
                representatives.insert(local.node, definition);
            }
        }
        let Some(mut representatives) = representatives else {
            return Ok(());
        };
        // An outer alternative links representatives only after visiting its
        // nested alternatives. Reverse order resolves parents before children.
        for (alias, representative) in aliases.into_iter().rev() {
            let definition = representatives
                .get(&representative)
                .copied()
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
    fn pattern_identity(&self, c: Context, local: Local) -> Result<super::hygiene::Ident, Error> {
        let context = if self.hygiene.is_some() {
            let offset = super::hygiene::name_offset(
                self.package,
                NodeKey {
                    file: c.file,
                    node: local.node,
                },
            );
            self.package.expansion_context(c.file, offset)?
        } else {
            ContextId(0)
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
                    self.lifetime(c, node, lifetime)?;
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
                    self.pattern(c, subpattern, bindings, aliases)?;
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
                    self.pattern(c, child, &mut branch, aliases)?;
                    let mut names = HashMap::default();
                    for local in &branch {
                        if names
                            .insert(self.pattern_identity(c, *local)?, *local)
                            .is_some()
                        {
                            return Err(self.error(c, local.node, "duplicate binding in pattern"));
                        }
                    }
                    if let Some(expected) = &expected {
                        let expected: &HashMap<_, Local> = expected;
                        if expected.len() != names.len() {
                            return Err(self.error(
                                c,
                                child,
                                "or-pattern alternatives bind different names or modes",
                            ));
                        }
                        // Iterate source order; hash seeds must not order the
                        // retained identity facts consumed by later phases.
                        for local in &branch {
                            let first = expected
                                .get(&self.pattern_identity(c, *local)?)
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
            Kind::Path { .. } | Kind::Qualified { .. } => self.visit(c, node, Namespace::Value)?,
            Kind::PatternRange { start, end, .. } => {
                for bound in [start, end].into_iter().flatten() {
                    if let Kind::Binding { name: span, .. } = self.kind(c, bound) {
                        let name = self.package.name(c.file, span)?;
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
