//! Source contracts retain record requests and operation/formal debts for fresh proof.
use super::ImportedOriginals;
use crate::{
    body::{
        ir::{Body, ProofSite},
        regions::{BuiltinRequirements, Contract, Kind, Obligation, Proof},
    },
    sema::{
        crates::SourceOriginal,
        declaration_inputs::{self, Inputs},
        model::{Declarations, SourceCopyRequirements},
    },
    support::crates::generics::source as s,
    types::{
        TypeId,
        parameters::{
            clauses::Clause,
            predicates::{Bound, Predicate},
        },
    },
};
use std::sync::Arc;

struct Expected<'a, 'b> {
    view: &'a SourceOriginal<'b>,
    facts: &'a Declarations,
    declared: Inputs<'a>,
    sized: Bound,
    root: ProofSite,
}
impl<'a, 'b> Expected<'a, 'b> {
    fn new(
        view: &'a SourceOriginal<'b>,
        facts: &'a Declarations,
        steps: &mut usize,
    ) -> Result<Self, &'static str> {
        Ok(Self {
            view,
            facts,
            declared: declaration_inputs::own_view(
                &facts.generics,
                facts.crates.as_deref(),
                view.definition(),
            )?,
            sized: view.sizing_requirement(facts, steps)?,
            root: view.source()?.root_site(),
        })
    }
    fn ports(&self) -> usize {
        self.view
            .raw()
            .locals
            .len()
            .saturating_add(self.view.raw().values.len())
            .saturating_add(1)
    }
    fn port(&self, mut index: usize, steps: &mut usize) -> Result<TypeId, &'static str> {
        let body = self.view.raw();
        let record = if index < body.locals.len() {
            body.locals[index].ty
        } else {
            index -= body.locals.len();
            if index < body.values.len() {
                body.values[index].ty
            } else {
                body.result
            }
        };
        self.view.ty(record, steps)
    }
    fn obligations<'s>(
        &'s self,
        steps: &'s mut usize,
    ) -> impl Iterator<Item = Result<Obligation, &'static str>> + 's {
        let record = self.view.inputs();
        let count = record
            .obligations
            .len()
            .saturating_add(record.parameters.len())
            .saturating_add(self.ports());
        (0..count).map(move |mut index| {
            *steps = steps.saturating_add(1);
            if index < record.obligations.len() {
                return obligation(self.view, record.obligations[index], steps);
            }
            index -= record.obligations.len();
            let ty = if index < record.parameters.len() {
                self.view.slot_type(record.parameters[index], steps)?
            } else {
                self.port(index - record.parameters.len(), steps)?
            };
            Ok(Obligation {
                source: self.root,
                kind: Kind::WellFormed(ty),
            })
        })
    }
    fn predicates<'s>(
        &'s self,
        steps: &'s mut usize,
    ) -> impl Iterator<Item = Result<(ProofSite, Predicate), &'static str>> + 's {
        let record = self.view.inputs();
        let builtin = record
            .builtins
            .as_ref()
            .map_or(&[][..], |builtins| builtins.predicates.as_slice());
        let count = builtin
            .len()
            .saturating_add(self.declared.len())
            .saturating_add(record.parameters.len())
            .saturating_add(self.ports());
        (0..count).map(move |mut index| {
            *steps = steps.saturating_add(1);
            if index < builtin.len() {
                let input = builtin[index];
                return Ok(Some((self.view.input_site(input.site, steps)?, self.view.predicate(input.value, steps)?)));
            }
            index -= builtin.len();
            if index < self.declared.len() {
                let input = self.declared.get(index).ok_or("source original omitted its complete declaration debt")?;
                let Clause::Predicate(predicate) = input.value else {
                    return Err("source quantified declaration debt requires its wider original profile");
                };
                return Ok(Some((input.source, predicate)));
            }
            index -= self.declared.len();
            let ty = if index < record.parameters.len() {
                let id = self.view.parameter(record.parameters[index], &self.facts.types, steps)?;
                if !matches!(self.facts.types.parameter(id).map(|p| p.kind), Some(crate::types::parameters::ParameterKind::Type { sized: true })) {
                    return Ok(None);
                }
                self.view.slot_type(record.parameters[index], steps)?
            } else { self.port(index - record.parameters.len(), steps)? };
            Ok(Some((self.root, Predicate { ty, bound: self.sized })))
        }).filter_map(Result::transpose)
    }
}

pub(super) fn build(
    facts: &Declarations,
    view: &SourceOriginal<'_>,
    body: &Body,
    work: &mut ImportedOriginals,
) -> Result<Contract, &'static str> {
    let expected = Expected::new(view, facts, &mut work.steps)?;
    let record = view.inputs();
    let parameters = record
        .parameters
        .iter()
        .map(|&id| view.parameter(id, &facts.types, &mut work.steps))
        .collect::<Result<Vec<_>, _>>()?;
    parameter_order(facts, view, &parameters, &mut work.steps)?;
    let assumptions = record
        .assumptions
        .iter()
        .map(|&p| pair(view, p, &mut work.steps))
        .collect::<Result<_, _>>()?;
    let type_assumptions = record
        .type_assumptions
        .iter()
        .map(|input| {
            view.input_site(input.site, &mut work.steps)?;
            Ok(Clause::Predicate(
                view.predicate(input.value, &mut work.steps)?,
            ))
        })
        .collect::<Result<_, &'static str>>()?;
    let obligations = expected
        .obligations(&mut work.steps)
        .collect::<Result<_, _>>()?;
    let predicates = expected
        .predicates(&mut work.steps)
        .collect::<Result<_, _>>()?;
    let proofs = record
        .normalizations
        .iter()
        .map(|record| normalization(view, record, &mut work.steps))
        .collect::<Result<_, _>>()?;
    let environment = match record.builtins.as_ref().map(|b| b.environment) {
        None | Some(s::EnvironmentOrigin::Absent) => None,
        Some(origin) if origin == record.environment => Some(Arc::clone(view.environment()?)),
        _ => return Err("source original builtin environment changed its actual provenance"),
    };
    let source_copy = record
        .builtins
        .as_ref()
        .and_then(|b| b.source_copy.as_ref())
        .map(|record| copy_queries(view, facts, record, &mut work.steps))
        .transpose()?;
    Ok(Contract {
        key: body.key(),
        source: body.source,
        signature: Some(view.signature()),
        universals: 0,
        assumptions,
        parameters,
        type_assumptions,
        obligations,
        proofs,
        builtins: Some(Arc::new(BuiltinRequirements::unpaid_imported(
            environment,
            predicates,
            source_copy,
        ))),
        output: None,
        require_result_formation: false,
    })
}
fn parameter_order(
    facts: &Declarations,
    view: &SourceOriginal<'_>,
    parameters: &[crate::types::parameters::ParameterId],
    steps: &mut usize,
) -> Result<(), &'static str> {
    use crate::types::{Kind, arguments::Argument};
    let mut actual = parameters.iter();
    for parameter in facts
        .generics
        .parameters
        .get(&view.definition())
        .into_iter()
        .flatten()
    {
        *steps = steps.saturating_add(1);
        let id = match parameter.argument {
            Argument::Lifetime(_) => continue,
            Argument::Type(ty) => match facts.types.kind(ty) {
                Some(Kind::Parameter(id)) => id,
                _ => return Err("source original lost its canonical rigid type formal"),
            },
            Argument::Const(_) => {
                return Err("source const formals require their wider original profile");
            }
        };
        if actual.next() != Some(&id) {
            return Err("source original contract changed its full canonical formal order");
        }
    }
    if actual.next().is_some() {
        return Err("source original contract added an unowned canonical formal");
    }
    Ok(())
}
fn copy_queries(
    view: &SourceOriginal<'_>,
    facts: &Declarations,
    record: &s::CopyQueries,
    steps: &mut usize,
) -> Result<Arc<SourceCopyRequirements>, &'static str> {
    let declaration = view.copy_identity(facts, record.declaration, steps)?;
    if declaration.is_none() && !record.goals.is_empty() {
        return Err("missing source Copy cannot carry a different query family");
    }
    let goals = record
        .goals
        .iter()
        .map(|&goal| {
            let predicate = view.predicate(goal, steps)?;
            let Bound::Trait(id) = predicate.bound else {
                return Err("source Copy query changed its ordinary trait effect");
            };
            let actual = facts
                .types
                .trait_application(id)
                .ok_or("source Copy query has an unknown application")?;
            if Some(crate::resolve::model::DefId(actual.item.0)) != declaration
                || facts
                    .types
                    .arguments(actual.arguments)
                    .is_none_or(|a| a.len() != 0)
            {
                return Err("source Copy query changed its actual declared trait/application");
            }
            Ok(predicate)
        })
        .collect::<Result<_, _>>()?;
    Ok(Arc::new(SourceCopyRequirements { declaration, goals }))
}
fn obligation(
    view: &SourceOriginal<'_>,
    record: s::Obligation,
    steps: &mut usize,
) -> Result<Obligation, &'static str> {
    let kind = match record.kind {
        s::InputKind::Equality(p) => Kind::Equality(pair(view, p, steps)?),
        s::InputKind::Outlives(p) => Kind::Outlives(pair(view, p, steps)?),
        s::InputKind::WellFormed(ty) => Kind::WellFormed(view.ty(ty, steps)?),
        s::InputKind::TypeOutlives { ty, shorter } => Kind::TypeOutlives {
            ty: view.ty(ty, steps)?,
            shorter: view.region(shorter, steps)?,
        },
        s::InputKind::Coercion {
            actual,
            expected,
            unsize,
            reference,
        } => Kind::Coercion {
            actual: view.ty(actual, steps)?,
            expected: view.ty(expected, steps)?,
            unsize,
            reference: reference.map(|r| crate::sema::model::ReferenceConversion {
                dereferences: r.dereferences,
                unsize: r.unsize,
                two_phase: r.two_phase,
            }),
        },
        _ => return Err("source allocation/scoped obligation requires its wider original profile"),
    };
    Ok(Obligation {
        source: view.input_site(record.site, steps)?,
        kind,
    })
}
fn pair(
    view: &SourceOriginal<'_>,
    record: s::RegionBound,
    steps: &mut usize,
) -> Result<crate::types::outlives::Pair, &'static str> {
    Ok((
        view.region(record.longer, steps)?,
        view.region(record.shorter, steps)?,
    ))
}
fn normalization(
    view: &SourceOriginal<'_>,
    record: &s::NormalizationInput,
    steps: &mut usize,
) -> Result<Proof, &'static str> {
    if !record.owned.is_empty() {
        return Err("source normalization allocations require fresh wider owner mapping");
    }
    let mut proof = crate::types::lifetime_proof::Obligations::new(0);
    proof.equalities = record
        .equalities
        .iter()
        .map(|&p| pair(view, p, steps))
        .collect::<Result<_, _>>()?;
    proof.outlives = record
        .outlives
        .iter()
        .map(|&p| pair(view, p, steps))
        .collect::<Result<_, _>>()?;
    proof.types = record
        .types
        .iter()
        .map(|p| Ok((view.ty(p.ty, steps)?, view.region(p.shorter, steps)?)))
        .collect::<Result<_, &'static str>>()?;
    Ok(Proof {
        source: view.input_site(record.site, steps)?,
        sources: record
            .sources
            .iter()
            .map(|&s| view.input_site(s, steps))
            .collect::<Result<_, _>>()?,
        proof,
    })
}

pub(super) fn validate(
    facts: &Declarations,
    view: &SourceOriginal<'_>,
    body: &Body,
    contract: &Contract,
    steps: &mut usize,
) -> Result<(), &'static str> {
    let expected = Expected::new(view, facts, steps)?;
    let record = view.inputs();
    if contract.key != body.key()
        || contract.source != view.source()?
        || contract.signature != Some(view.signature())
        || contract.universals != 0
        || contract.output.is_some()
        || contract.require_result_formation
        || contract.parameters.len() != record.parameters.len()
        || contract.assumptions.len() != record.assumptions.len()
        || contract.type_assumptions.len() != record.type_assumptions.len()
        || contract.proofs.len() != record.normalizations.len()
    {
        return Err("source original changed its complete record contract/header domains");
    }
    for (&actual, &record) in contract.parameters.iter().zip(&record.parameters) {
        if actual != view.parameter(record, &facts.types, steps)? {
            return Err("source original changed its canonical parameter order");
        }
    }
    for (&actual, &record) in contract.assumptions.iter().zip(&record.assumptions) {
        if actual != pair(view, record, steps)? {
            return Err("source original changed a record lifetime assumption");
        }
    }
    for (&actual, record) in contract
        .type_assumptions
        .iter()
        .zip(&record.type_assumptions)
    {
        view.input_site(record.site, steps)?;
        if actual != Clause::Predicate(view.predicate(record.value, steps)?) {
            return Err("source original changed a record regional assumption");
        }
    }
    let mut obligations = expected.obligations(steps);
    for actual in &contract.obligations {
        let wanted = obligations
            .next()
            .ok_or("source original added an unowned obligation")??;
        if actual.source != wanted.source || !same_kind(actual.kind, wanted.kind) {
            return Err("source original changed a complete typed obligation");
        }
    }
    if obligations.next().transpose()?.is_some() {
        return Err("source original omitted an actual mandatory obligation");
    }
    drop(obligations);
    validate_builtins(&expected, contract, steps)?;
    for (record, actual) in record.normalizations.iter().zip(&contract.proofs) {
        validate_normalization(view, record, actual, steps)?;
    }
    Ok(())
}
fn validate_builtins(
    expected: &Expected<'_, '_>,
    contract: &Contract,
    steps: &mut usize,
) -> Result<(), &'static str> {
    let view = expected.view;
    let record = view.inputs();
    let facts = expected.facts;
    let builtin = contract
        .builtins
        .as_deref()
        .ok_or("source original omitted its unpaid builtin input block")?;
    if !builtin.is_unprepared_imported() {
        return Err("source original cannot import prepared permission");
    }
    let environment = record.builtins.as_ref().map(|b| b.environment);
    match (environment, builtin.environment.as_ref()) {
        (None | Some(s::EnvironmentOrigin::Absent), None) => {}
        (Some(origin), Some(actual))
            if origin == record.environment && Arc::ptr_eq(actual, view.environment()?) => {}
        _ => return Err("source original borrowed a different record environment origin"),
    }
    let mut predicates = expected.predicates(steps);
    for &(site, predicate) in &builtin.predicates {
        let wanted = predicates
            .next()
            .ok_or("source original added an unowned predicate")??;
        if (site, predicate) != wanted {
            return Err("source original changed a complete pending predicate");
        }
    }
    if predicates.next().transpose()?.is_some() {
        return Err("source original omitted a mandatory pending predicate");
    }
    drop(predicates);
    match (
        record
            .builtins
            .as_ref()
            .and_then(|b| b.source_copy.as_ref()),
        &builtin.source_copy,
    ) {
        (None, None) => {}
        (Some(record), Some(actual)) => {
            if view.copy_identity(facts, record.declaration, steps)? != actual.declaration
                || record.goals.len() != actual.goals.len()
            {
                return Err("source original changed its active Copy query inputs");
            }
            for (&record, &actual) in record.goals.iter().zip(&actual.goals) {
                if view.predicate(record, steps)? != actual {
                    return Err("source original changed a retained Copy query");
                }
            }
        }
        _ => return Err("source original changed absent-versus-present Copy queries"),
    }
    Ok(())
}
fn same_kind(actual: Kind, expected: Kind) -> bool {
    match (actual, expected) {
        (Kind::Equality(actual), Kind::Equality(expected))
        | (Kind::Outlives(actual), Kind::Outlives(expected)) => actual == expected,
        (Kind::WellFormed(actual), Kind::WellFormed(expected)) => actual == expected,
        (
            Kind::TypeOutlives {
                ty: actual,
                shorter: actual_region,
            },
            Kind::TypeOutlives {
                ty: expected,
                shorter: expected_region,
            },
        ) => actual == expected && actual_region == expected_region,
        (
            Kind::Coercion {
                actual,
                expected,
                unsize,
                reference,
            },
            Kind::Coercion {
                actual: other_actual,
                expected: other_expected,
                unsize: other_unsize,
                reference: other_reference,
            },
        ) => {
            actual == other_actual
                && expected == other_expected
                && unsize == other_unsize
                && reference.map(|value| (value.dereferences, value.unsize, value.two_phase))
                    == other_reference
                        .map(|value| (value.dereferences, value.unsize, value.two_phase))
        }
        _ => false,
    }
}
fn validate_normalization(
    view: &SourceOriginal<'_>,
    record: &s::NormalizationInput,
    actual: &Proof,
    steps: &mut usize,
) -> Result<(), &'static str> {
    if !record.owned.is_empty()
        || actual.proof.has_quantified()
        || actual.proof.regions != (0..0)
        || actual.source != view.input_site(record.site, steps)?
        || actual.sources.len() != record.sources.len()
        || actual.proof.equalities.len() != record.equalities.len()
        || actual.proof.outlives.len() != record.outlives.len()
        || actual.proof.types.len() != record.types.len()
    {
        return Err("source original changed its complete record normalization domain");
    }
    for (&record, &actual) in record.sources.iter().zip(&actual.sources) {
        if view.input_site(record, steps)? != actual {
            return Err("source normalization changed an actual diagnostic origin");
        }
    }
    for (rows, mapped) in [
        (&record.equalities, &actual.proof.equalities),
        (&record.outlives, &actual.proof.outlives),
    ] {
        for (&record, &actual) in rows.iter().zip(mapped) {
            if pair(view, record, steps)? != actual {
                return Err("source normalization changed a record relation");
            }
        }
    }
    for (record, &actual) in record.types.iter().zip(&actual.proof.types) {
        if (
            view.ty(record.ty, steps)?,
            view.region(record.shorter, steps)?,
        ) != actual
        {
            return Err("source normalization changed a record type-outlives input");
        }
    }
    Ok(())
}
