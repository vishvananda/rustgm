//! Canonical library trait headers require a local type before uncovered parameters.
use super::{Checker, DefId, DefKind, Error, NodeKey, Predicate};
use crate::{
    resolve::{expansion::Report, model::BuiltinTrait},
    types::{ItemId, Kind, TypeId, arguments::Argument},
};

// Canonical library and source-free nominal declarations remain foreign.
// A local nominal keeps its real declaration origin through aliases.
fn local_nominal(report: &Report, item: ItemId) -> Result<bool, &'static str> {
    let definition = DefId(item.0);
    if report
        .resolution
        .library_types
        .identify(definition)
        .is_some()
    {
        return Ok(false);
    }
    let definition = report
        .resolution
        .definitions
        .get(item.0 as usize)
        .ok_or("invalid nominal ownership identity")?;
    if !matches!(definition.kind, DefKind::Struct | DefKind::Enum) {
        return Err("nominal ownership requires a nominal declaration identity");
    }
    Ok(definition.node.is_some())
}

impl Checker<'_> {
    pub(super) fn validate_language_orphan(
        &self,
        source: NodeKey,
        trait_definition: DefId,
        predicate: Predicate,
    ) -> Result<(), Error> {
        let imported = self
            .report
            .resolution
            .definitions
            .get(trait_definition.0 as usize)
            .ok_or_else(|| self.error(source, "invalid implementation trait ownership identity"))?
            .node
            .is_none()
            && self.facts.uses_source_copy();
        if imported {
            let owner = self
                .facts
                .associated
                .owner(trait_definition)
                .and_then(|id| self.facts.associated.get(id))
                .ok_or_else(|| {
                    self.error(source, "imported orphan check lacks its actual trait owner")
                })?;
            self.facts
                .associated
                .require_imported(*owner)
                .map_err(|e| self.error(source, e))?;
        }
        if !imported
            && !matches!(
                self.report
                    .resolution
                    .builtin_traits
                    .identify(trait_definition),
                Some(
                    BuiltinTrait::Language(_)
                        | BuiltinTrait::Comparison(_)
                        | BuiltinTrait::Iteration(_)
                )
            )
        {
            return Ok(());
        }
        let Some((application, _)) = predicate.bound.trait_parts() else {
            return Err(self.error(source, "missing language trait application"));
        };
        let arguments = self
            .facts
            .types
            .trait_application(application)
            .map(|application| application.arguments)
            .ok_or_else(|| self.error(source, "missing language trait application"))?;
        let mut uncovered = false;
        for ty in std::iter::once(predicate.ty).chain(
            self.facts
                .types
                .arguments(arguments)
                .into_iter()
                .flatten()
                .filter_map(|argument| {
                    if let Argument::Type(ty) = argument {
                        Some(ty)
                    } else {
                        None
                    }
                }),
        ) {
            let (local, parameter) = self.orphan_type(source, ty)?;
            if local {
                return if uncovered || parameter {
                    Err(self.error(
                        source,
                        "uncovered parameter precedes the first local implementation type",
                    ))
                } else {
                    Ok(())
                };
            }
            uncovered |= parameter;
        }
        Err(self.error(source, "library trait implementation requires a local type"))
    }

    fn orphan_type(&self, source: NodeKey, mut ty: TypeId) -> Result<(bool, bool), Error> {
        let mut pending = Vec::new();
        let mut visited = crate::support::hash::HashSet::default();
        let mut uncovered = false;
        for _ in 0..1_000_000 {
            // Local/nonfundamental roots retain their allocation-free path.
            // A foreign fundamental DAG visits each shared subtree once. Any
            // earlier local result already returned; repeated uncovered facts
            // are idempotent, preserving left-to-right orphan ordering.
            if !visited.is_empty() && !self.orphan_visit(source, &mut visited, ty)? {
                if let Some(next) = pending.pop() {
                    ty = next;
                    continue;
                }
                return Ok((false, uncovered));
            }
            match self.facts.types.kind(ty) {
                Some(Kind::Reference { referent, .. }) => {
                    ty = referent;
                    continue;
                }
                Some(Kind::Adt(item, arguments)) => {
                    if local_nominal(self.report, item).map_err(|e| self.error(source, e))? {
                        return Ok((true, uncovered));
                    }
                    if crate::sema::header_attributes::fundamental_policy(
                        &self.facts,
                        DefId(item.0),
                    )
                    .map_err(|e| self.error(source, e))?
                    {
                        if visited.is_empty() {
                            self.orphan_visit(source, &mut visited, ty)?;
                        }
                        let arguments = self.facts.types.arguments(arguments).ok_or_else(|| {
                            self.error(source, "fundamental nominal lost its arguments")
                        })?;
                        let growth = crate::support::retained::Growth::vector::<TypeId>(
                            pending.len(),
                            pending.capacity(),
                            arguments.len(),
                        );
                        crate::sema::header_attributes::charge_source_retention(
                            &self.facts,
                            growth.work + arguments.len(),
                            growth.bytes,
                        )?;
                        pending.try_reserve(arguments.len()).map_err(|_| {
                            self.error(source, "fundamental orphan traversal allocation failed")
                        })?;
                        let first = pending.len();
                        pending.extend(arguments.filter_map(|a| match a {
                            Argument::Type(ty) => Some(ty),
                            _ => None,
                        }));
                        pending[first..].reverse();
                    }
                }
                Some(Kind::Dynamic { shape, .. }) => {
                    if self.orphan_dynamic(source, shape)? {
                        return Ok((true, uncovered));
                    }
                }
                Some(Kind::Parameter(_) | Kind::Projection(_)) => uncovered = true,
                Some(_) => (),
                None => return Err(self.error(source, "invalid implementation ownership type")),
            }
            if let Some(next) = pending.pop() {
                ty = next;
            } else {
                return Ok((false, uncovered));
            }
        }
        Err(self.error(source, "implementation ownership work budget exceeded"))
    }

    fn orphan_visit(
        &self,
        source: NodeKey,
        visited: &mut crate::support::hash::HashSet<TypeId>,
        ty: TypeId,
    ) -> Result<bool, Error> {
        let growth = crate::support::retained::Growth::map::<TypeId, ()>(
            visited.len(),
            visited.capacity(),
            usize::from(!visited.contains(&ty)),
        );
        crate::sema::header_attributes::charge_source_retention(
            &self.facts,
            growth.work + 1,
            growth.bytes,
        )?;
        if visited.contains(&ty) {
            return Ok(false);
        }
        visited
            .try_reserve(1)
            .map_err(|_| self.error(source, "fundamental orphan visited allocation failed"))?;
        visited.insert(ty);
        Ok(true)
    }

    fn orphan_dynamic(
        &self,
        source: NodeKey,
        shape: crate::types::dynamic::ShapeId,
    ) -> Result<bool, Error> {
        let shape = self
            .facts
            .types
            .dynamic_shape(shape)
            .ok_or_else(|| self.error(source, "invalid object ownership shape"))?;
        let owner = shape
            .principal
            .and_then(|item| self.facts.associated.owner(DefId(item.0)))
            .and_then(|owner| self.facts.associated.get(owner));
        let Some(owner) = owner else {
            return Ok(false);
        };
        if owner.source.is_none() {
            self.facts
                .associated
                .require_imported(*owner)
                .map_err(|e| self.error(source, e))?;
            return Ok(false);
        }
        Ok(matches!(
            owner
                .source_owner()
                .map_err(|e| self.error(source, e))?
                .kind,
            crate::sema::associated::OwnerKind::Trait { .. }
        ))
    }
}
