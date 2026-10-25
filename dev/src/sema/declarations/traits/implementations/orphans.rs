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
                return if uncovered {
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
        for _ in 0..1_000_000 {
            match self.facts.types.kind(ty) {
                Some(Kind::Reference { referent, .. }) => ty = referent,
                Some(Kind::Adt(item, _)) => {
                    return local_nominal(self.report, item)
                        .map(|local| (local, false))
                        .map_err(|e| self.error(source, e));
                }
                Some(Kind::Dynamic { shape, .. }) => {
                    let shape = self
                        .facts
                        .types
                        .dynamic_shape(shape)
                        .ok_or_else(|| self.error(source, "invalid object ownership shape"))?;
                    let owner = shape
                        .principal
                        .and_then(|item| self.facts.associated.owner(DefId(item.0)))
                        .and_then(|owner| self.facts.associated.get(owner));
                    let local = if let Some(owner) = owner {
                        if owner.source.is_none() {
                            self.facts
                                .associated
                                .require_imported(*owner)
                                .map_err(|e| self.error(source, e))?;
                            false
                        } else {
                            matches!(
                                owner
                                    .source_owner()
                                    .map_err(|e| self.error(source, e))?
                                    .kind,
                                crate::sema::associated::OwnerKind::Trait { .. }
                            )
                        }
                    } else {
                        false
                    };
                    return Ok((local, false));
                }
                Some(Kind::Parameter(_) | Kind::Projection(_)) => return Ok((false, true)),
                Some(_) => return Ok((false, false)),
                None => return Err(self.error(source, "invalid implementation ownership type")),
            }
        }
        Err(self.error(source, "implementation ownership work budget exceeded"))
    }
}
