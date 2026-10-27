//! Associated primitive values reuse the ordinary typed-prefix scheduler.
use super::{Checker, DefId, DefKind, Error, NodeId, NodeKey, Type, Work, regions};
use crate::{resolve::model::Namespace, sema::primitive_constants::Member, types::Primitive};

#[derive(Clone, Copy)]
struct Path {
    prefix: NodeKey,
    suffix: NodeKey,
    owner: Option<DefId>,
    member: Member,
}
impl Checker<'_> {
    fn primitive_constant_path(&self, key: NodeKey) -> Option<Path> {
        if self.report.package.uses_source_bindings() {
            // Source providers own their associated declarations, including MAX.
            return None;
        }
        let (prefix, suffix, name, owner) =
            if let Some(path) = self.report.resolution.qualified_paths.get(&key) {
                if path.namespace != Namespace::Value || path.trait_path.is_some() {
                    return None;
                }
                let [(suffix, name)] = path.suffix.as_slice() else {
                    return None;
                };
                (path.ty, *suffix, *name, None)
            } else {
                let path = self.report.resolution.associated_paths.get(&key)?;
                if path.namespace != Namespace::Value {
                    return None;
                }
                let [(suffix, name)] = path.suffix.as_slice() else {
                    return None;
                };
                if !matches!(
                    self.report.resolution.definitions[path.owner.0 as usize].kind,
                    DefKind::Primitive | DefKind::TypeAlias | DefKind::SelfType
                ) {
                    return None;
                }
                (path.prefix, *suffix, *name, Some(path.owner))
            };
        let member = match self.report.package.names.get(name) {
            "MAX" => Member::Max,
            "MIN" => Member::Min,
            _ => return None,
        };
        Some(Path {
            prefix: NodeKey {
                node: prefix,
                ..key
            },
            suffix: NodeKey {
                node: suffix,
                ..key
            },
            owner,
            member,
        })
    }
    pub(super) fn schedule_primitive_constant(
        &mut self,
        key: NodeKey,
        context: regions::Context,
        steps: &mut usize,
        limit: usize,
        scan: &mut Vec<NodeId>,
    ) -> Result<bool, Error> {
        let Some(path) = self.primitive_constant_path(key) else {
            return Ok(false);
        };
        if crate::sema::primitive_constants::selected(&self.facts, key)
            .map_err(|e| self.error(key, e))?
            .is_some()
        {
            return Ok(true);
        }
        if let Some(ty) = path
            .owner
            .and_then(|id| self.facts.definitions[id.0 as usize])
            && !matches!(self.facts.types.kind(ty),Some(Type::Primitive(p)) if p.integer().is_some() || matches!(p,Primitive::F32 | Primitive::F64 | Primitive::Char))
            && !(path.owner.is_some_and(|id| {
                self.report.resolution.definitions[id.0 as usize].kind == DefKind::TypeAlias
            }) && matches!(
                self.facts.types.kind(ty),
                Some(Type::Parameter(_) | Type::Projection(_))
            ))
        {
            return Ok(false);
        }
        self.work.push(Work::MethodPrefix(key, context));
        if path.owner.is_some_and(|id| {
            self.report.resolution.definitions[id.0 as usize].kind == DefKind::TypeAlias
        }) {
            // Applied aliases must finish their ordinary scoped projection proof
            // before their actual primitive identity can select an inherent value.
            self.work.push(Work::NormalizeBound(path.prefix, context));
        }
        self.schedule_method_prefix(path.prefix, path.owner, context, steps, limit, scan)?;
        Ok(true)
    }
    pub(super) fn complete_primitive_constant(&mut self, key: NodeKey) -> Result<bool, Error> {
        let Some(path) = self.primitive_constant_path(key) else {
            return Ok(false);
        };
        let ty = self.facts.node(path.prefix).and_then(|f| f.ty);
        let Some(ty) = ty.filter(|ty| matches!(self.facts.types.kind(*ty),Some(Type::Primitive(p)) if p.integer().is_some() || matches!(p,Primitive::F32 | Primitive::F64 | Primitive::Char))) else {return Ok(false);};
        if path.member == Member::Min && ty != crate::types::TypeId::primitive(Primitive::Char) {
            return Ok(false);
        }
        if !matches!(self.facts.types.kind(ty),Some(Type::Primitive(p)) if p.integer().is_some() || p == Primitive::Char)
        {
            return Err(self.error(key, "this primitive MAX constant is not implemented"));
        }
        self.method_path_segments(key, path.prefix, path.suffix)?;
        for segment in [path.prefix, path.suffix] {
            // The ordinary alias application already checked its own arguments.
            if segment == path.prefix
                && path.owner.is_some_and(|id| {
                    self.report.resolution.definitions[id.0 as usize].kind == DefKind::TypeAlias
                })
            {
                continue;
            }
            if crate::sema::lifetimes::arguments(self.report, segment, false)?.is_some() {
                return Err(self.error(
                    segment,
                    "primitive constants do not accept generic arguments",
                ));
            }
        }
        self.facts
            .primitive_constants
            .get_or_insert_with(Box::default)
            .publish(&mut self.facts.types, key, path.prefix, ty, path.member)
            .map_err(|e| self.error(key, e))?;
        Ok(true)
    }
}
