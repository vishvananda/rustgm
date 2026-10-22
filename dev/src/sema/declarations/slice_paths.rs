//! Typed slice qualification binds the inherited element without inventing a nominal.
use super::{Checker, DefKind, Error, NodeId, NodeKey, Target, Type, Work, regions};
use crate::resolve::model::Namespace;
impl Checker<'_> {
    pub(super) fn schedule_slice_alias(
        &mut self,
        key: NodeKey,
        context: regions::Context,
        steps: &mut usize,
        limit: usize,
        scan: &mut Vec<NodeId>,
    ) -> Result<bool, Error> {
        let Some(path) = self.report.resolution.associated_paths.get(&key) else {
            return Ok(false);
        };
        let [(_, name)] = path.suffix.as_slice() else {
            return Ok(false);
        };
        if path.namespace != Namespace::Value
            || !self
                .report
                .resolution
                .library_types
                .slice_length()
                .is_some_and(|length| {
                    self.report.resolution.definitions[length.definition.0 as usize].name == *name
                })
            || !matches!(
                self.report.resolution.definitions[path.owner.0 as usize].kind,
                DefKind::TypeAlias | DefKind::SelfType | DefKind::Generic
            )
        {
            return Ok(false);
        }
        let (prefix, owner) = (
            NodeKey {
                node: path.prefix,
                ..key
            },
            path.owner,
        );
        if self.facts.methods.length_prefix(key).is_some()
            && self.facts.generics.item_uses.contains_key(&key)
        {
            return Ok(true);
        }
        self.work.push(Work::MethodPrefix(key, context));
        self.schedule_method_prefix(prefix, Some(owner), context, steps, limit, scan)?;
        Ok(true)
    }
    pub(super) fn complete_slice_prefix(
        &mut self,
        key: NodeKey,
        context: regions::Context,
        steps: &mut usize,
        limit: usize,
        scan: &mut Vec<NodeId>,
    ) -> Result<bool, Error> {
        let parts = if let Some(path) = self.report.resolution.qualified_paths.get(&key) {
            if path.namespace != Namespace::Value || path.trait_path.is_some() {
                return Ok(false);
            }
            match path.suffix.as_slice() {
                [(suffix, name)] => Some((
                    NodeKey {
                        node: path.ty,
                        ..key
                    },
                    NodeKey {
                        node: *suffix,
                        ..key
                    },
                    *name,
                )),
                _ => None,
            }
        } else if let Some(path) = self.report.resolution.associated_paths.get(&key) {
            if path.namespace != Namespace::Value {
                return Ok(false);
            }
            match path.suffix.as_slice() {
                [(suffix, name)] => Some((
                    NodeKey {
                        node: path.prefix,
                        ..key
                    },
                    NodeKey {
                        node: *suffix,
                        ..key
                    },
                    *name,
                )),
                _ => None,
            }
        } else {
            None
        };
        let Some((prefix, suffix, name)) = parts else {
            return Ok(false);
        };
        if !self
            .report
            .resolution
            .library_types
            .slice_length()
            .is_some_and(|length| {
                self.report.resolution.definitions[length.definition.0 as usize].name == name
            })
        {
            return Ok(false);
        }
        let ty = self.facts.node(prefix).and_then(|fact| {
            fact.ty.or_else(|| match fact.reference {
                Some(Target::Definition(id)) => self.facts.definitions[id.0 as usize],
                _ => None,
            })
        });
        if !matches!(
            ty.and_then(|ty| self.facts.types.kind(ty)),
            Some(Type::Slice(_))
        ) {
            return Ok(false);
        }
        self.prepare_slice_methods()?;
        let length = self
            .facts
            .methods
            .length_named(name)
            .ok_or_else(|| self.error(key, "missing prepared slice method"))?;
        self.method_path_segments(key, prefix, suffix)?;
        self.facts
            .methods
            .retain_length_prefix(key, prefix)
            .map_err(|e| self.error(key, e))?;
        self.facts
            .node_mut(key)
            .ok_or_else(|| Error::at(key.file, 0, "missing slice path fact"))?
            .reference = Some(Target::Definition(length.definition));
        self.work.push(Work::MethodArguments(key, suffix));
        self.schedule_item_arguments(suffix, length.definition, context, steps, limit, scan)?;
        Ok(true)
    }
}
