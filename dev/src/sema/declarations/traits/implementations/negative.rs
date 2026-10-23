//! Negative headers cannot strengthen Self's nominal formation contract.
use super::{Checker, DefId, Error, NodeKey, Predicate};
use crate::{
    support::hash::HashSet,
    types::{Kind, arguments::Argument, parameters::ParameterKind},
};
impl Checker<'_> {
    pub(super) fn validate_impl_polarity_profile(
        &self,
        source: NodeKey,
        negative: bool,
    ) -> Result<(), Error> {
        if negative
            && !self
                .report
                .package
                .source_core
                .as_deref()
                .is_some_and(crate::resolve::bindings::SourceCore::permits_negative_impls)
        {
            return Err(self.error(
                source,
                "negative trait implementations require trait coherence",
            ));
        }
        Ok(())
    }
    pub(super) fn validate_negative_header(
        &mut self,
        owner: DefId,
        source: NodeKey,
        predicate: Predicate,
    ) -> Result<(), Error> {
        let error = |message| {
            Error::at(
                source.file,
                self.report
                    .package
                    .unit(source.file)
                    .configured
                    .ast
                    .node(source.node)
                    .span
                    .start,
                message,
            )
        };
        let mut formation = crate::sema::nominals::Requirements::default();
        let clauses =
            crate::sema::nominals::requirements(&mut self.facts, predicate.ty, &mut formation)
                .map_err(error)?;
        let required: HashSet<_> = clauses.into_iter().collect();
        let declared = self
            .facts
            .generics
            .declared_predicates_in_scope(owner)
            .collect::<Vec<_>>();
        self.facts.body_work.canonical_steps = self
            .facts
            .body_work
            .canonical_steps
            .checked_add(required.len() + declared.len())
            .ok_or_else(|| error("negative header legality work overflow"))?;
        if declared
            .iter()
            .any(|clause| !required.contains(&clause.value))
        {
            return Err(error(
                "negative implementation header-strength proof is not implemented",
            ));
        }
        negative_parameters(&self.facts, owner, predicate).map_err(error)
    }
}

fn negative_parameters(
    facts: &crate::sema::model::Declarations,
    owner: DefId,
    predicate: Predicate,
) -> Result<(), &'static str> {
    let Some(Kind::Adt(item, arguments)) = facts.types.kind(predicate.ty) else {
        if facts
            .generics
            .parameter_ids(&facts.types, owner)?
            .is_empty()
        {
            return Ok(());
        }
        return Err("generic non-nominal negative header legality is not implemented");
    };
    let template = facts
        .definitions
        .get(item.0 as usize)
        .copied()
        .flatten()
        .ok_or("negative header nominal template is missing")?;
    let Some(Kind::Adt(formal_item, formal_arguments)) = facts.types.kind(template) else {
        return Err("negative header nominal template changed kind");
    };
    if formal_item != item {
        return Err("negative header nominal template changed owner");
    }
    let parameters = facts
        .types
        .arguments(formal_arguments)
        .ok_or("negative header nominal template arguments are missing")?;
    let arguments = facts
        .types
        .arguments(arguments)
        .ok_or("negative header Self arguments are missing")?;
    if parameters.len() != arguments.len() {
        return Err("negative header Self arity changed");
    }
    let mut actual = HashSet::default();
    for (parameter, argument) in parameters.zip(arguments) {
        if !actual.insert(argument) {
            return Err("specialized negative header legality is not implemented");
        }
        match (parameter, argument) {
            (Argument::Type(formal), Argument::Type(actual)) => {
                negative_type(facts, owner, formal, actual)?;
            }
            (Argument::Lifetime(_), Argument::Lifetime(region))
                if matches!(region.bound_parts(), Some((0, _))) => {}
            (Argument::Const(_), Argument::Const(value)) if matches!(facts.types.const_kind(value), Some(crate::types::arguments::ConstKind::Parameter(id)) if facts.types.parameter(id).is_some_and(|p| p.owner.0 == owner.0)) =>
                {}
            _ => return Err("specialized negative header legality is not implemented"),
        }
    }
    Ok(())
}
fn negative_type(
    facts: &crate::sema::model::Declarations,
    owner: DefId,
    formal: crate::types::TypeId,
    actual: crate::types::TypeId,
) -> Result<(), &'static str> {
    let (Some(Kind::Parameter(formal)), Some(Kind::Parameter(actual))) =
        (facts.types.kind(formal), facts.types.kind(actual))
    else {
        return Err("specialized negative header legality is not implemented");
    };
    let formal = facts
        .types
        .parameter(formal)
        .ok_or("negative header nominal parameter is missing")?;
    let actual = facts
        .types
        .parameter(actual)
        .ok_or("negative header impl parameter is missing")?;
    if actual.owner.0 != owner.0 {
        return Err("negative header parameter belongs to another impl");
    }
    if matches!(actual.kind, ParameterKind::Type { sized: true })
        && !matches!(formal.kind, ParameterKind::Type { sized: true })
    {
        return Err("negative implementation strengthens an implicit Sized requirement");
    }
    Ok(())
}
