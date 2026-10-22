//! Typed slice qualification binds the inherited element without inventing a nominal.
use super::{Checker, DefKind, Error, NodeId, NodeKey, Target, Type, Work, regions};
use crate::resolve::model::Namespace;
impl Checker<'_> {
    fn slice_definition(&self, name: crate::resolve::names::NameId) -> Option<super::DefId> {
        self.report
            .resolution
            .library_types
            .slice_length()
            .filter(|l| self.report.resolution.definitions[l.definition.0 as usize].name == name)
            .map(|l| l.definition)
            .or_else(|| {
                self.report
                    .resolution
                    .builtin_traits
                    .collections()
                    .map(|c| {
                        c.declarations[crate::support::iteration::Collection::SliceMethod as usize]
                    })
                    .filter(|c| {
                        self.report.resolution.definitions[c.method.0 as usize].name == name
                    })
                    .map(|c| c.method)
            })
    }
    pub(super) fn collection_prefix(&self, key: NodeKey) -> Option<NodeKey> {
        self.facts
            .iteration
            .as_deref()?
            .collections
            .as_deref()?
            .prefixes
            .get(&key)
            .copied()
    }

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
            || (self.slice_definition(*name).is_none()
                && super::super::methods::pointers::named(self.report, *name, false).is_none())
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
        if (self.facts.methods.length_prefix(key).is_some()
            || self.facts.methods.pointer_prefix(key).is_some()
            || self.collection_prefix(key).is_some())
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
        let pointer_name =
            super::super::methods::pointers::named(self.report, name, false).is_some();
        let definition = self.slice_definition(name);
        if !pointer_name && definition.is_none() {
            return Ok(false);
        };
        let ty = self.facts.node(prefix).and_then(|fact| {
            fact.ty.or_else(|| match fact.reference {
                Some(Target::Definition(id)) => self.facts.definitions[id.0 as usize],
                _ => None,
            })
        });
        let head = ty.and_then(|ty| self.facts.types.kind(ty));
        if pointer_name {
            let Some(Type::Pointer { mutable, .. }) = head else {
                return Ok(false);
            };
            let definition = super::super::methods::pointers::named(self.report, name, mutable)
                .ok_or_else(|| self.error(key, "missing canonical pointer method"))?;
            self.complete_pointer_prefix(
                (key, prefix, suffix),
                definition,
                context,
                steps,
                limit,
                scan,
            )?;
            return Ok(true);
        }
        if !matches!(head, Some(Type::Slice(_))) {
            return Ok(false);
        }
        let definition =
            definition.ok_or_else(|| self.error(key, "missing canonical slice method"))?;
        self.method_path_segments(key, prefix, suffix)?;
        if self
            .report
            .resolution
            .library_types
            .slice_length()
            .is_some_and(|l| l.definition == definition)
        {
            self.prepare_slice_methods()?;
            if self
                .facts
                .methods
                .length_named(name)
                .is_none_or(|l| l.definition != definition)
            {
                return Err(self.error(key, "slice method name differs from canonical declaration"));
            }
            self.facts
                .methods
                .retain_length_prefix(key, prefix)
                .map_err(|e| self.error(key, e))?;
        } else {
            self.activate_collection(definition)?;
            self.facts
                .iteration
                .as_deref_mut()
                .and_then(|r| r.collections.as_deref_mut())
                .ok_or_else(|| Error::at(key.file, 0, "missing collection prefix registry"))?
                .prefixes
                .insert(key, prefix);
        }
        self.facts
            .node_mut(key)
            .ok_or_else(|| Error::at(key.file, 0, "missing slice path fact"))?
            .reference = Some(Target::Definition(definition));
        self.work.push(Work::MethodArguments(key, suffix));
        self.schedule_item_arguments(suffix, definition, context, steps, limit, scan)?;
        Ok(true)
    }
}
