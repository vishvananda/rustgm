//! Local generic annotation holes retain written binders and argument provenance.
use super::{Argument, Checker, DefId, Error, NodeKey, regions};
use crate::sema::{
    generics::{AnnotationArgument, AnnotationUse},
    model::PartialAnnotation,
};

impl Checker<'_> {
    pub(in super::super) fn inferred_expression_alias(
        &mut self,
        key: NodeKey,
        owner: DefId,
        context: regions::Context,
    ) -> Result<bool, Error> {
        if self.report.resolution.definitions[owner.0 as usize].kind
            != crate::resolve::model::DefKind::TypeAlias
            || !self.facts.generics.parameters.contains_key(&owner)
        {
            return Ok(false);
        }
        if self.facts.generics.annotation_uses.contains_key(&key) {
            return Ok(true);
        }
        let (lifetimes, supplied) = self.item_argument_lists(key, owner)?;
        if supplied.is_some() {
            return Ok(false);
        }
        let parameters = self.facts.generics.parameters[&owner].clone();
        let nodes =
            lifetimes.map(|list| self.report.package.unit(key.file).configured.ast.list(list));
        let count = parameters
            .iter()
            .take_while(|p| matches!(p.argument, Argument::Lifetime(_)))
            .count();
        if nodes.is_some_and(|nodes| nodes.len() != count) {
            return Err(self.error(key, "incorrect number of lifetime arguments"));
        }
        let mut arguments = Vec::with_capacity(parameters.len());
        for (index, parameter) in parameters.iter().enumerate() {
            let argument = if matches!(parameter.argument, Argument::Lifetime(_)) {
                let region = self.region(
                    key,
                    nodes.and_then(|nodes| nodes.get(index).copied()),
                    context,
                )?;
                self.regions.record(context, region);
                AnnotationArgument::Lifetime(region)
            } else {
                // A value path with no type/const arguments infers every slot,
                // including parameters with defaults. A type annotation does not.
                AnnotationArgument::Infer
            };
            arguments.push(argument);
        }
        self.facts
            .generics
            .annotation_uses
            .insert(key, AnnotationUse { owner, arguments });
        self.facts
            .partial_annotations
            .insert(key, PartialAnnotation::Shape);
        Ok(true)
    }

    pub(in super::super) fn partial_generic_path(
        &mut self,
        key: NodeKey,
        owner: DefId,
        context: regions::Context,
    ) -> Result<bool, Error> {
        if !self.facts.generics.parameters.contains_key(&owner)
            || self.report.resolution.definitions[owner.0 as usize].kind
                == crate::resolve::model::DefKind::SelfType
        {
            return Ok(false);
        }
        let (supplied, omitted) = self.supplied_arguments(key, owner)?;
        let ast = &self.report.package.unit(key.file).configured.ast;
        let nodes = supplied.map_or(&[][..], |list| ast.list(list));
        if !nodes.iter().any(|node| {
            self.facts
                .partial_annotations
                .contains_key(&NodeKey { node: *node, ..key })
        }) {
            return Ok(false);
        }
        let parameters = self.facts.generics.parameters[&owner].clone();
        let mut arguments = Vec::with_capacity(parameters.len());
        for (index, parameter) in parameters.iter().enumerate() {
            let node = index
                .checked_sub(omitted)
                .and_then(|index| nodes.get(index))
                .copied();
            let argument = match (parameter.argument, node) {
                (Argument::Lifetime(_), node) => {
                    let region = self.region(key, node, context)?;
                    self.regions.record(context, region);
                    AnnotationArgument::Lifetime(region)
                }
                (_, Some(node))
                    if matches!(
                        self.facts.partial_annotations.get(&NodeKey { node, ..key }),
                        Some(PartialAnnotation::Hole)
                    ) =>
                {
                    AnnotationArgument::Infer
                }
                (Argument::Type(_), Some(node)) => {
                    AnnotationArgument::Type(NodeKey { node, ..key })
                }
                (Argument::Const(id), Some(node)) => {
                    let ty = self
                        .facts
                        .types
                        .const_type(id)
                        .ok_or_else(|| self.error(key, "missing const argument type"))?;
                    AnnotationArgument::Const(self.generic_constant(NodeKey { node, ..key }, ty)?)
                }
                (_, None) => AnnotationArgument::Default(
                    parameter
                        .default
                        .ok_or_else(|| self.error(key, "missing generic default"))?,
                ),
            };
            arguments.push(argument);
        }
        self.facts
            .generics
            .annotation_uses
            .insert(key, AnnotationUse { owner, arguments });
        self.facts
            .partial_annotations
            .insert(key, PartialAnnotation::Shape);
        Ok(true)
    }
}
