//! Qualified value prefixes reuse the shared partial-annotation application contract.
use super::{Checker, DefId, DefKind, Error, NodeKey, regions};
use crate::{
    sema::{
        generics::{AnnotationArgument, AnnotationUse, SourceArgument},
        model::PartialAnnotation,
    },
    types::{arguments::Argument, parameters::environment::Environment},
};
use std::sync::Arc;

impl Checker<'_> {
    pub(super) fn selection_environment(
        &mut self,
        key: NodeKey,
        context: regions::Context,
    ) -> Option<Arc<Environment>> {
        let regions::Position::Local(body) = context.position else {
            return None;
        };
        let environments = self.method_environments.get_or_insert_with(|| {
            self.facts
                .functions
                .iter()
                .filter_map(|f| {
                    Some((
                        f.body?,
                        Arc::clone(self.facts.generics.environments.get(&f.definition)?),
                    ))
                })
                .collect()
        });
        environments.get(&NodeKey { node: body, ..key }).cloned()
    }

    pub(super) fn selection_prefix(
        &mut self,
        key: NodeKey,
        owner: Option<DefId>,
        context: regions::Context,
    ) -> Result<(), Error> {
        let Some(owner) = owner else { return Ok(()) };
        if !matches!(
            self.report.resolution.definitions[owner.0 as usize].kind,
            DefKind::Struct | DefKind::Enum
        ) || self.facts.generics.annotation_uses.contains_key(&key)
        {
            return Ok(());
        }
        let Some(parameters) = self.facts.generics.parameters.get(&owner).cloned() else {
            return Ok(());
        };
        let written = self
            .facts
            .generics
            .item_uses
            .get(&key)
            .ok_or_else(|| self.error(key, "missing selection prefix arguments"))?;
        let supplied = written.arguments.clone();
        let lifetimes = written.lifetimes;
        let count = parameters
            .iter()
            .take_while(|p| matches!(p.argument, Argument::Lifetime(_)))
            .count();
        let nodes =
            lifetimes.map(|list| self.report.package.unit(key.file).configured.ast.list(list));
        if nodes.is_some_and(|nodes| nodes.len() != count) {
            return Err(self.error(key, "incorrect number of lifetime arguments"));
        }
        let mut arguments = Vec::with_capacity(parameters.len());
        for (index, parameter) in parameters.iter().enumerate() {
            let argument = if matches!(parameter.argument, Argument::Lifetime(_)) {
                AnnotationArgument::Lifetime(self.region(
                    key,
                    nodes.and_then(|nodes| nodes.get(index).copied()),
                    context,
                )?)
            } else {
                match supplied.get(index - count) {
                    Some(SourceArgument::Type(key)) => AnnotationArgument::Type(*key),
                    Some(SourceArgument::Const(id)) => AnnotationArgument::Const(*id),
                    Some(SourceArgument::Infer) => AnnotationArgument::Infer,
                    None if supplied.is_empty() => AnnotationArgument::Infer,
                    None => AnnotationArgument::Default(
                        parameter
                            .default
                            .ok_or_else(|| self.error(key, "missing selection prefix default"))?,
                    ),
                }
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
        Ok(())
    }
}
