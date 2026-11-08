//! Negative headers cannot strengthen Self's nominal formation contract.
use super::{Checker, DefId, Error, NodeKey, Predicate};
use crate::{
    support::hash::HashSet,
    types::{
        Kind,
        arguments::Argument,
        parameters::{ParameterKind, clauses::Clause, predicates::Bound},
    },
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
                .requested_negative_impls()
                .map_err(|error| self.error(source, error))?
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
        let automatic = matches!(
            predicate.bound,
            crate::types::parameters::predicates::Bound::Auto(_)
        ) || predicate
            .bound
            .trait_parts()
            .and_then(|(app, _)| self.facts.types.trait_application(app))
            .and_then(|app| self.facts.associated.owner(DefId(app.item.0)))
            .and_then(|id| self.facts.associated.get(id))
            .is_some_and(|owner| {
                matches!(
                    owner.kind,
                    crate::sema::associated::OwnerKind::Trait { auto_: true, .. }
                )
            });
        if !automatic {
            return Ok(());
        }
        // The actual unstable language deliberately permits conditional
        // non-ADT negative auto headers. Their conditions still belong to the
        // ordinary implementation proof; only ADTs require coverage here.
        if !matches!(self.facts.types.kind(predicate.ty), Some(Kind::Adt(..)))
            && self
                .report
                .package
                .requested_auto_traits()
                .map_err(|e| self.error(source, e))?
        {
            return Ok(());
        }
        self.negative_coverage(owner, predicate, false)
            .map_err(|message| {
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
            })
    }

    /// Version thirteen: an imported negative impl of its own packet's auto
    /// trait. Declaring that trait required the provider's `auto_traits`
    /// feature, which is exactly what lets a non-ADT negative header skip the
    /// coverage proof; a nominal Self still re-proves it here.
    pub(super) fn validate_imported_negative_header(
        &mut self,
        owner: DefId,
        predicate: Predicate,
    ) -> Result<(), Error> {
        if !matches!(self.facts.types.kind(predicate.ty), Some(Kind::Adt(..))) {
            return Ok(());
        }
        self.negative_coverage(owner, predicate, true)
            .map_err(Error::unlocated)
    }

    /// The impl's declared inputs, source or imported, are each a formation
    /// requirement of its nominal Self, whose arguments are its distinct own
    /// parameters.
    fn negative_coverage(
        &mut self,
        owner: DefId,
        predicate: Predicate,
        imported: bool,
    ) -> Result<(), &'static str> {
        let clauses = if imported {
            imported_nominal_inputs(&mut self.facts, predicate.ty)?
        } else {
            let mut formation = crate::sema::nominals::Requirements::default();
            crate::sema::nominals::requirements(&mut self.facts, predicate.ty, &mut formation)?
        };
        let required: HashSet<_> = clauses.into_iter().collect();
        let declared = crate::sema::declaration_inputs::declared(
            &self.facts.generics,
            self.facts.crates.as_deref(),
            owner,
        )?
        .map(|input| input.value)
        .collect::<Vec<_>>();
        self.facts.body_work.canonical_steps = self
            .facts
            .body_work
            .canonical_steps
            .checked_add(required.len() + declared.len())
            .ok_or("negative header legality work overflow")?;
        if declared.iter().any(|clause| !required.contains(clause)) {
            return Err("negative implementation header-strength proof is not implemented");
        }
        negative_parameters(&self.facts, owner, predicate)
    }
}

/// A source client installs no imported nominal's predicates. Its declared
/// and implied inputs are borrowed from the installed header and instantiated
/// at Self's arguments; outlives inputs are omitted as formation omits them.
fn imported_nominal_inputs(
    facts: &mut crate::sema::model::Declarations,
    ty: crate::types::TypeId,
) -> Result<Vec<Clause>, &'static str> {
    let Some(Kind::Adt(item, arguments)) = facts.types.kind(ty) else {
        return Err("imported negative coverage requires its nominal Self");
    };
    let nominal = DefId(item.0);
    let inputs =
        crate::sema::declaration_inputs::all(&facts.generics, facts.crates.as_deref(), nominal)?
            .map(|input| input.value)
            .filter(|clause| {
                !matches!(
                    clause,
                    Clause::Predicate(Predicate {
                        bound: Bound::Outlives(_),
                        ..
                    })
                )
            })
            .collect::<Vec<_>>();
    inputs
        .into_iter()
        .map(|clause| {
            facts
                .generics
                .substitute_clause(&mut facts.types, nominal, arguments, clause)
        })
        .collect()
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
