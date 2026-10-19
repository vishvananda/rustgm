//! Projection applications reuse generic argument preparation and resolved member identities.
use super::{Checker, DefId, Error, GenericArgs, ItemId, Namespace, NodeKey, regions};
use crate::resolve::names::NameId;

impl Checker<'_> {
    pub(super) fn partial_projection_path(
        &mut self,
        key: NodeKey,
        path: crate::sema::generics::AnnotationProjection,
        context: regions::Context,
    ) -> Result<bool, Error> {
        let owner = self
            .facts
            .associated
            .trait_owner(path.declaration)
            .ok_or_else(|| self.error(key, "missing projection trait owner"))?;
        let parent = self.partial_generic_path(path.parent, owner, context)?;
        let member = self.partial_generic_path(path.member, path.declaration, context)?;
        if !parent && !member && !self.facts.partial_annotations.contains_key(&path.subject) {
            return Ok(false);
        }
        self.retain_annotation_arguments(path.parent, owner, context)?;
        if self
            .facts
            .generics
            .parameters
            .contains_key(&path.declaration)
        {
            self.retain_annotation_arguments(path.member, path.declaration, context)?;
        } else {
            self.projection_arguments(path.member, path.declaration, context)?;
            self.facts.generics.annotation_uses.insert(
                path.member,
                crate::sema::generics::AnnotationUse {
                    owner: path.declaration,
                    arguments: Vec::new(),
                },
            );
        }
        self.facts.generics.annotation_projections.insert(key, path);
        self.facts
            .partial_annotations
            .insert(key, crate::sema::model::PartialAnnotation::Shape);
        Ok(true)
    }

    pub(super) fn qualified_projection_member(
        &self,
        key: NodeKey,
        owner: DefId,
        name: NameId,
    ) -> Result<DefId, Error> {
        let scope = self.report.resolution.definitions[owner.0 as usize]
            .members
            .ok_or_else(|| self.error(key, "missing trait member scope"))?;
        self.report.resolution.scopes[scope.0 as usize]
            .bindings
            .get(&(name, Namespace::Type))
            .map(|b| b.target)
            .filter(|id| {
                self.facts.associated.trait_owner(*id) == Some(owner)
                    && self.facts.types.projection_schema(ItemId(id.0)).is_some()
            })
            .ok_or_else(|| self.error(key, "qualified trait has no prepared associated type"))
    }

    pub(super) fn projection_arguments(
        &mut self,
        key: NodeKey,
        member: DefId,
        context: regions::Context,
    ) -> Result<GenericArgs, Error> {
        if self.facts.generics.parameters.contains_key(&member) {
            return self.application_arguments(key, member, context);
        }
        if let Some(arguments) = crate::sema::lifetimes::arguments(self.report, key, false)?
            && arguments.start != arguments.end
        {
            // Preserve PA20's diagnostic for arguments on a non-parameterized member.
            return Err(self.error(
                key,
                "associated type arguments require advanced generic relationships",
            ));
        }
        Ok(GenericArgs::EMPTY)
    }
}
