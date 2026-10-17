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
    pub(super) fn selection_trait_assumptions(
        &mut self,
        key: NodeKey,
        context: regions::Context,
        environment: Option<&Arc<Environment>>,
    ) -> Result<Option<Arc<crate::sema::traits::assumptions::Assumptions>>, Error> {
        let Some(environment) = environment else {
            return Ok(None);
        };
        if environment.trait_predicates().next().is_none() {
            return Ok(None);
        }
        let regions::Position::Local(body) = context.position else {
            return Err(self.error(key, "trait selection requires its caller scope"));
        };
        let body = NodeKey { node: body, ..key };
        if let Some(assumptions) = self
            .method_trait_environments
            .as_ref()
            .and_then(|cache| cache.get(&body))
        {
            return Ok(Some(Arc::clone(assumptions)));
        }
        let mut assumptions = crate::sema::traits::assumptions::Assumptions::default();
        for &predicate in environment.trait_predicates() {
            assumptions
                .insert(&self.facts.types, predicate)
                .map_err(|e| self.error(key, e.to_string()))?;
        }
        let assumptions = Arc::new(assumptions);
        self.method_trait_environments
            .get_or_insert_with(Default::default)
            .insert(body, Arc::clone(&assumptions));
        Ok(Some(assumptions))
    }

    pub(super) fn selection_environment(
        &self,
        context: regions::Context,
    ) -> Option<Arc<Environment>> {
        if !matches!(context.position, regions::Position::Local(_)) {
            return None;
        }
        // The declaration context already identifies its immutable caller.
        // A whole-crate snapshot here misses bodies declared after an early
        // constant query and would need repeated rebuilding as items complete.
        let owner = self.regions.owner(context.binder?)?;
        self.facts.generics.environments.get(&owner).cloned()
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
