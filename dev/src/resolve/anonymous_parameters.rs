//! Argument-position opaque syntax declares unnameable function type parameters.
use super::{
    index::{Context, Resolver},
    model::{Access, DefKind, Definition, Namespace},
    package::Error,
    unit::NodeKey,
};
use crate::syntax::{
    ast::{Kind, NodeId},
    item::{Item, ItemKind},
};

impl Resolver<'_> {
    pub(super) fn reserve_anonymous_parameters(
        &mut self,
        context: Context,
        node: NodeId,
        item: Item,
    ) -> Result<(), Error> {
        let ItemKind::Function { parameters, .. } = item.kind else {
            return Ok(());
        };
        let owner = NodeKey {
            file: context.file,
            node,
        };
        let mark = self.scratch.len();
        let result = self.anonymous_inputs(context, owner, parameters, mark);
        self.scratch.truncate(mark);
        let inputs = result?;
        if inputs.is_empty() {
            return Ok(());
        }
        for &source in &inputs {
            let name = self
                .package
                .names
                .intern(&format!("#impl-trait-{}-{}", source.file.0, source.node.0))
                .map_err(|e| self.error(context, source.node, e))?;
            u32::try_from(self.facts.definitions.len())
                .map_err(|_| self.error(context, source.node, "too many definitions"))?;
            // These identities deliberately have no lexical binding or hygiene entry.
            self.facts.definitions.push(Definition {
                name,
                namespace: Namespace::Type,
                kind: DefKind::Generic,
                scope: context.scope,
                node: Some(source),
                access: Access::Within(context.module),
                members: None,
            });
        }
        self.facts.anonymous_parameters.insert(owner, inputs);
        Ok(())
    }
    /// Find argument-position `impl Trait` types, using the shared scratch
    /// stack above `mark` as the worklist.
    fn anonymous_inputs(
        &mut self,
        context: Context,
        owner: NodeKey,
        parameters: crate::syntax::ast::List,
        mark: usize,
    ) -> Result<Vec<NodeKey>, Error> {
        let ast = &self.package.unit(context.file).configured.ast;
        self.scratch.extend(ast.list(parameters).iter().rev());
        let mut inputs = Vec::new();
        let mut steps = 0;
        while self.scratch.len() > mark {
            let Some(node) = self.scratch.pop() else {
                break;
            };
            if !self.active(context, node) {
                continue;
            }
            steps += 1;
            if steps > 1_000_000 {
                return Err(self.error(context, node, "anonymous parameter work budget exceeded"));
            }
            match ast.node(node).kind {
                Kind::TraitType {
                    opaque: true,
                    bounds,
                } => {
                    let mut trait_bound = false;
                    for &bound in ast.list(bounds) {
                        match ast.node(bound).kind {
                            Kind::PreciseCapture(_) => {
                                return Err(self.error(
                                    context,
                                    bound,
                                    "precise captures require return-position impl Trait",
                                ));
                            }
                            Kind::Bound { value, .. } => {
                                trait_bound |= !matches!(ast.node(value).kind, Kind::Name(span)
                                    if self.package.unit(context.file).spelling(span)
                                        .is_ok_and(|name| name.starts_with('\'')));
                            }
                            _ => {}
                        }
                    }
                    if !trait_bound {
                        return Err(self.error(
                            context,
                            node,
                            "impl Trait requires at least one trait bound",
                        ));
                    }
                    inputs.push(NodeKey { node, ..owner });
                }
                Kind::Parameter { ty: Some(ty), .. } => self.scratch.push(ty),
                Kind::Type { .. }
                | Kind::Path { .. }
                | Kind::Applied { .. }
                | Kind::Group(_)
                | Kind::Attributed { .. } => {
                    let start = self.scratch.len();
                    ast.push_children(node, self.scratch);
                    self.scratch[start..].reverse();
                }
                _ => {}
            }
        }
        Ok(inputs)
    }
}
