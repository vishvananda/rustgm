//! Canonical library trait headers require a local type before uncovered parameters.
use super::{Checker, DefId, DefKind, Error, NodeKey, Predicate};
use crate::{
    resolve::{expansion::Report, model::BuiltinTrait},
    types::{ItemId, Kind, TypeId, arguments::Argument},
};

// This is the ownership seam for future nominal metadata. The current external
// profile transports scalar functions/constants, never nominal declarations.
// Canonical library types are explicitly foreign; resolved source nominal IDs
// retain their ownership through aliases, without diagnostic-source inspection.
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
    Ok(true)
}

impl Checker<'_> {
    pub(super) fn validate_language_orphan(
        &self,
        source: NodeKey,
        trait_definition: DefId,
        predicate: Predicate,
    ) -> Result<(), Error> {
        if !matches!(
            self.report
                .resolution
                .builtin_traits
                .identify(trait_definition),
            Some(
                BuiltinTrait::Language(_)
                    | BuiltinTrait::Comparison(_)
                    | BuiltinTrait::Iteration(_)
            )
        ) {
            return Ok(());
        }
        let crate::types::parameters::predicates::Bound::Trait(application) = predicate.bound
        else {
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
                    let local = shape
                        .principal
                        .and_then(|item| self.facts.associated.owner(DefId(item.0)))
                        .and_then(|owner| self.facts.associated.get(owner))
                        .is_some_and(|owner| {
                            matches!(owner.kind, crate::sema::associated::OwnerKind::Trait { .. })
                        });
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
