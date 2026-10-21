//! Carry unreduced annotation identities through the same completed syntax edges.
mod arguments;
use super::{Checker, Error, Kind, List, NodeKey, Signature, Type, TypeFlavor, TypeId};
pub(super) use arguments::Arguments;

impl Checker<'_> {
    pub(super) fn unreduced_type(&self, key: NodeKey) -> Result<TypeId, Error> {
        self.facts
            .unreduced
            .node(key)
            .map_or_else(|| self.ty(key), Ok)
    }

    pub(super) fn retain_unreduced_annotation(
        &mut self,
        key: NodeKey,
        ty: TypeId,
    ) -> Result<(), Error> {
        if self.facts.unreduced.is_empty() {
            return Ok(());
        }
        let kind = self.kind(key);
        // Alias declarations retain their original by definition identity; an
        // extra entry at the item node would duplicate the RHS annotation.
        if matches!(kind, Kind::Item(_)) || self.facts.unreduced.node(key).is_some() {
            return Ok(());
        }
        let original = match kind {
            Kind::Group(node) | Kind::Attributed { value: node, .. } => {
                self.facts.unreduced.node(NodeKey { node, ..key })
            }
            Kind::Type { flavor, elements } => {
                self.unreduced_compound(key, ty, flavor, elements)?
            }
            _ => None,
        };
        if let Some(original) = original {
            self.facts.unreduced.retain_node(key, original);
        }
        Ok(())
    }

    fn unreduced_compound(
        &mut self,
        key: NodeKey,
        ty: TypeId,
        flavor: TypeFlavor,
        elements: List,
    ) -> Result<Option<TypeId>, Error> {
        let nodes = self
            .report
            .package
            .unit(key.file)
            .configured
            .ast
            .list(elements);
        if flavor == TypeFlavor::Path {
            return Ok(nodes
                .first()
                .and_then(|node| self.facts.unreduced.node(NodeKey { node: *node, ..key })));
        }
        let kind = self
            .facts
            .types
            .kind(ty)
            .ok_or_else(|| self.error(key, "missing annotation shape"))?;
        let kind = if let Type::Tuple(_) = kind {
            if !nodes.iter().any(|node| {
                self.facts
                    .unreduced
                    .node(NodeKey { node: *node, ..key })
                    .is_some()
            }) {
                return Ok(None);
            }
            let children = nodes
                .iter()
                .map(|node| self.unreduced_type(NodeKey { node: *node, ..key }))
                .collect::<Result<Vec<_>, _>>()?;
            Type::Tuple(
                self.facts
                    .types
                    .intern_list(&children)
                    .map_err(|e| self.error(key, e))?,
            )
        } else {
            let node = if matches!(kind, Type::Reference { .. }) {
                nodes.last()
            } else {
                nodes.first()
            };
            let Some(original) =
                node.and_then(|node| self.facts.unreduced.node(NodeKey { node: *node, ..key }))
            else {
                return Ok(None);
            };
            match kind {
                Type::Slice(_) => Type::Slice(original),
                Type::Array { length, .. } => Type::Array {
                    element: original,
                    length,
                },
                Type::ArrayTerm { length, .. } => Type::ArrayTerm {
                    element: original,
                    length,
                },
                Type::Pointer { mutable, .. } => Type::Pointer {
                    mutable,
                    pointee: original,
                },
                Type::Reference {
                    mutable, region, ..
                } => Type::Reference {
                    mutable,
                    region,
                    referent: original,
                },
                _ => return Err(self.error(key, "invalid unreduced compound shape")),
            }
        };
        self.intern(key, kind).map(Some)
    }

    pub(super) fn unreduced_signature(
        &mut self,
        key: NodeKey,
        ty: TypeId,
        syntax: &Signature,
    ) -> Result<TypeId, Error> {
        if self.facts.unreduced.is_empty() {
            return Ok(ty);
        }
        let unit = self.report.package.unit(key.file);
        let mut inputs = Vec::new();
        let mut changed = false;
        for &node in unit.configured.ast.list(syntax.parameters) {
            if !unit.configured.active[node.0 as usize] {
                continue;
            }
            let parameter = self.unattributed(NodeKey { node, ..key });
            if let Some(receiver) = self.facts.receivers.get(&parameter) {
                inputs.push(receiver.ty);
                continue;
            }
            let node = super::parameter_type(self.report, parameter)?;
            let node = NodeKey { node, ..key };
            changed |= self.facts.unreduced.node(node).is_some();
            inputs.push(self.unreduced_type(node)?);
        }
        let result = syntax.result.map(|node| NodeKey { node, ..key });
        changed |= result.is_some_and(|node| self.facts.unreduced.node(node).is_some());
        if !changed {
            return Ok(ty);
        }
        let result = result.map_or(Ok(TypeId::UNIT), |node| self.unreduced_type(node))?;
        let Some(Type::Function {
            bound_regions,
            unsafe_,
            convention,
            ..
        }) = self.facts.types.kind(ty)
        else {
            return Err(self.error(key, "missing completed signature"));
        };
        let parameters = self
            .facts
            .types
            .intern_list(&inputs)
            .map_err(|e| self.error(key, e))?;
        self.intern(
            key,
            Type::Function {
                parameters,
                result,
                bound_regions,
                unsafe_,
                convention,
            },
        )
    }
}
