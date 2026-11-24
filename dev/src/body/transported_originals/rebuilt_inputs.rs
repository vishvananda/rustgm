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
        lifetime_proof::{Obligations, ScopeId},
        parameters::{
            clauses::Clause,
            environment::Environment,
            predicates::{Bound, Predicate},
        },
    },
};
use std::{cell::Ref, sync::Arc};

struct Expected<'a, 'b> {
    view: &'a SourceOriginal<'b>,
    facts: &'a Declarations,
    declared: Inputs<'a>,
    glue_declared_inputs: usize,
    sized: Bound,
    root: ProofSite,
    associated: Option<Ref<'a, [Predicate]>>,
    /// The declared-result coercion, present only when the types differ.
    result: Option<Obligation>,
}
impl<'a, 'b> Expected<'a, 'b> {
    fn new(
        view: &'a SourceOriginal<'b>,
        facts: &'a Declarations,
        steps: &mut usize,
    ) -> Result<Self, &'static str> {
        let root = view.source()?.root_site();
        let definition = view.definition();
        let glue_declared_inputs = if view.const_context()
            && crate::sema::compiler_items::drop_glue_member(facts, definition)?
        {
            declaration_inputs::declared_own(&facts.generics, facts.crates.as_deref(), definition)?
                .len()
        } else {
            0
        };
        Ok(Self {
            view,
            facts,
            declared: declaration_inputs::own_view(
                &facts.generics,
                facts.crates.as_deref(),
                definition,
            )?,
            glue_declared_inputs,
            sized: view.sizing_requirement(facts, steps)?,
            root,
            associated: view.associated_goals(steps)?,
            result: declared_result(view, root, steps)?,
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
        (0..count)
            .map(move |mut index| {
                *steps = steps.saturating_add(1);
                if index < record.obligations.len() {
                    return obligation(self.view, record.obligations[index], steps).map(Some);
                }
                index -= record.obligations.len();
                let ty = if index < record.parameters.len() {
                    // A const formal has no type to form.
                    let Some(ty) = self.view.slot_type(record.parameters[index], steps)? else {
                        return Ok(None);
                    };
                    ty
                } else {
                    self.port(index - record.parameters.len(), steps)?
                };
                Ok(Some(Obligation {
                    source: self.root,
                    kind: Kind::WellFormed(ty),
                }))
            })
            .filter_map(Result::transpose)
            .chain(self.result.map(Ok))
    }
    /// Version thirteen lifts only a vacuous clause binder, which parenthesized
    /// callable sugar opens even when it binds no region. A declared result
    /// binding is not a builtin debt: each caller proves the imported header's
    /// own clauses, and the original's environment assumes it.
    fn declared_predicate(
        &self,
        index: usize,
        clause: Clause,
    ) -> Result<Option<Predicate>, &'static str> {
        let body_nodes = self.view.rows().descriptor().profile().body_nodes();
        let predicate = match clause {
            Clause::Predicate(predicate) => predicate,
            Clause::ForAll(id) if body_nodes => {
                let scoped = self
                    .facts
                    .types
                    .quantified(id)
                    .ok_or("source quantified declaration debt is unknown")?;
                if scoped.bound_regions != 0 {
                    return Err(
                        "source quantified declaration debt requires its wider original profile",
                    );
                }
                scoped.predicate
            }
            _ => {
                return Err(
                    "source quantified declaration debt requires its wider original profile",
                );
            }
        };
        let binding = matches!(predicate.bound, Bound::ProjectionEqual(_));
        // The role proof authenticated the glue's own conditional Destruct
        // premise. It is assumed here and paid by calls at their actual effect.
        let glue_premise = index < self.glue_declared_inputs
            && matches!(predicate.bound, Bound::MaybeConstTrait(_));
        Ok((!(body_nodes && binding || glue_premise)).then_some(predicate))
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
        let associated = self.associated.as_deref().unwrap_or_default();
        let ports = self.ports();
        let sized = record.parameters.len().saturating_add(ports);
        let count = builtin
            .len()
            .saturating_add(self.declared.len())
            .saturating_add(sized)
            .saturating_add(associated.len());
        (0..count)
            .map(move |mut index| {
                *steps = steps.saturating_add(1);
                if index < builtin.len() {
                    let input = &builtin[index];
                    return Ok(Some((
                        self.view.input_site(input.site, steps)?,
                        self.view.predicate(&input.value, steps)?,
                    )));
                }
                index -= builtin.len();
                if index < self.declared.len() {
                    let input = self
                        .declared
                        .get(index)
                        .ok_or("source original omitted its complete declaration debt")?;
                    if matches!(input.value, Clause::UnstableFeature(_)) {
                        // Retained in the whole imported header and its environment;
                        // callee_premises discharges it for each actual caller/instance.
                        // It is not a builtin/type Copy query.
                        return Ok(None);
                    }
                    return Ok(self
                        .declared_predicate(index, input.value)?
                        .map(|predicate| (input.source, predicate)));
                }
                index -= self.declared.len();
                if index >= sized {
                    return Ok(Some((self.root, associated[index - sized])));
                }
                let ty = if index < record.parameters.len() {
                    let reference = record.parameters[index];
                    // A const formal has no sizing goal.
                    let Some(ty) = self.view.slot_type(reference, steps)? else {
                        return Ok(None);
                    };
                    let id = self.view.parameter(reference, &self.facts.types, steps)?;
                    if !matches!(
                        self.facts.types.parameter(id).map(|p| p.kind),
                        Some(crate::types::parameters::ParameterKind::Type { sized: true })
                    ) {
                        return Ok(None);
                    }
                    ty
                } else {
                    self.port(index - record.parameters.len(), steps)?
                };
                Ok(Some((
                    self.root,
                    Predicate {
                        ty,
                        bound: self.sized,
                    },
                )))
            })
            .filter_map(Result::transpose)
    }

    fn builtin_environment(&self) -> Result<Option<&Arc<Environment>>, &'static str> {
        let record = self.view.inputs();
        match record
            .builtins
            .as_ref()
            .map(|builtins| builtins.environment)
        {
            None | Some(s::EnvironmentOrigin::Absent) => {
                if self.associated.as_deref().is_none_or(<[_]>::is_empty) {
                    Ok(None)
                } else {
                    self.view.environment().map(Some)
                }
            }
            Some(origin) if origin == record.environment => self.view.environment().map(Some),
            _ => Err("source original builtin environment changed its actual provenance"),
        }
    }
}

pub(super) fn build(
    facts: &Declarations,
    view: &SourceOriginal<'_>,
    body: &Body,
    work: &mut ImportedOriginals,
) -> Result<Contract, &'static str> {
    super::non_type_calls::check(facts, view, body, &mut work.steps)?;
    super::safety_requests::check(facts, view, body, work)?;
    super::encoded_assembly::check(facts, view, body, work)?;
    view.check_const_policies(&mut work.steps)?;
    let expected = Expected::new(view, facts, &mut work.steps)?;
    let record = view.inputs();
    let parameters = view.type_parameters(&facts.types, &mut work.steps)?;
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
                view.predicate(&input.value, &mut work.steps)?,
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
        .enumerate()
        .map(|(index, record)| normalization(view, index, record, &mut work.steps))
        .collect::<Result<_, _>>()?;
    let environment = expected.builtin_environment()?.cloned();
    let source_copy = record
        .builtins
        .as_ref()
        .and_then(|b| b.source_copy.as_ref())
        .map(|record| copy_queries(view, facts, record, &mut work.steps))
        .transpose()?;
    let mut contract = Contract {
        key: body.key(),
        source: body.source,
        signature: view.signature(),
        universals: view.universals(),
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
    };
    if view.const_context() && view.signature().is_some() {
        let required = crate::body::defining_cleanups::mark_imported_required(
            facts,
            view.definition(),
            &mut contract,
            &mut work.steps,
        );
        work.charge(0)?;
        required?;
    }
    Ok(contract)
}
/// The wire admits the header's original result against the body's result by
/// shape only; their regional relation is a fresh request. The body's value
/// flows into the declared result every caller receives, so it must coerce.
fn declared_result(
    view: &SourceOriginal<'_>,
    root: ProofSite,
    steps: &mut usize,
) -> Result<Option<Obligation>, &'static str> {
    let rows = view.rows();
    let result = match rows.header(view.raw().declaration)? {
        Some(s::HeaderRef::Function(index)) => rows.functions[index as usize].original.result,
        Some(s::HeaderRef::Constant(index)) => rows.constants()[index as usize].ty,
        _ => return Err("source original lost its actual defining header"),
    };
    let declared = view.ty(result, steps)?;
    let actual = view.ty(view.raw().result, steps)?;
    Ok((declared != actual).then_some(Obligation {
        source: root,
        kind: Kind::Coercion {
            actual,
            expected: declared,
            unsize: false,
            reference: None,
        },
    }))
}
fn parameter_order(
    facts: &Declarations,
    view: &SourceOriginal<'_>,
    parameters: &[crate::types::parameters::ParameterId],
    steps: &mut usize,
) -> Result<(), &'static str> {
    use crate::types::{Kind, arguments::Argument};
    if view.numbered() {
        let raw = view.type_parameters(&facts.types, steps)?;
        if parameters.len() != raw.len() {
            return Err("selected original changed its complete raw formal domain");
        }
        if raw != parameters {
            return Err("selected original changed its actual full formal order");
        }
        return Ok(());
    }
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
            Argument::Type(ty) => match facts.types.kind(ty) {
                Some(Kind::Parameter(id)) => id,
                _ => return Err("source original lost its canonical rigid type formal"),
            },
            // `type_parameters` checked each const formal's rigid identity.
            Argument::Lifetime(_) | Argument::Const(_) => continue,
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
        .map(|goal| {
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
        s::InputKind::Variable {
            slot,
            needs_provenance,
        } => {
            super::regional_records::require(view)?;
            Kind::Variable {
                id: view.variable(slot, steps)?,
                needs_provenance,
            }
        }
        s::InputKind::LocalBorrow(region) => {
            super::regional_records::require(view)?;
            Kind::LocalBorrow(view.region(region, steps)?)
        }
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
        s::InputKind::ScopedWellFormed { ty, frame }
            if view.rows().descriptor().profile().body_nodes()
                && view
                    .rows()
                    .binders
                    .get(frame.0 as usize)
                    .is_some_and(|binder| binder.count == 0) =>
        {
            // A vacuous clause scope from parenthesized callable sugar.
            Kind::ScopedWellFormed {
                ty: view.ty(ty, steps)?,
                bound_regions: 0,
            }
        }
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
    index: usize,
    record: &s::NormalizationInput,
    steps: &mut usize,
) -> Result<Proof, &'static str> {
    let regions = view.normalization_interval(index, record, steps)?;
    let mut proof = normalization_relations(
        view,
        (&record.equalities, &record.outlives, &record.types),
        regions,
        steps,
    )?;
    if let Some(scoped) = &record.scoped {
        let count = view.normalization_scope(scoped.frame, steps)?;
        let scope = if scoped.callable {
            proof.callable_scope(ScopeId::ROOT, count)?
        } else {
            proof.scope(ScopeId::ROOT, count)?
        };
        let end = proof.regions.end;
        let nested = normalization_relations(
            view,
            (&scoped.equalities, &scoped.outlives, &scoped.types),
            end..end,
            steps,
        )?;
        proof.append_at_origin(nested, scope, scoped.origin)?;
    }
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

type RelationRows<'a> = (
    &'a [s::RegionBound],
    &'a [s::RegionBound],
    &'a [s::TypeOutlives],
);

fn normalization_relations(
    view: &SourceOriginal<'_>,
    (equalities, outlives, types): RelationRows<'_>,
    regions: std::ops::Range<u32>,
    steps: &mut usize,
) -> Result<Obligations, &'static str> {
    let mut proof = Obligations::new(regions.start);
    proof.regions = regions;
    proof.equalities = equalities
        .iter()
        .map(|&row| pair(view, row, steps))
        .collect::<Result<_, _>>()?;
    proof.outlives = outlives
        .iter()
        .map(|&row| pair(view, row, steps))
        .collect::<Result<_, _>>()?;
    proof.types = types
        .iter()
        .map(|row| Ok((view.ty(row.ty, steps)?, view.region(row.shorter, steps)?)))
        .collect::<Result<_, &'static str>>()?;
    Ok(proof)
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
    let parameters = view.type_parameters(&facts.types, steps)?;
    if contract.key != body.key()
        || contract.source != view.source()?
        || contract.signature != view.signature()
        || contract.universals != view.universals()
        || contract.output.is_some()
        || contract.require_result_formation
        || contract.parameters.len() != parameters.len()
        || contract.assumptions.len() != record.assumptions.len()
        || contract.type_assumptions.len() != record.type_assumptions.len()
        || contract.proofs.len() != record.normalizations.len()
    {
        return Err("source original changed its complete record contract/header domains");
    }
    if contract.parameters != parameters {
        return Err("source original changed its canonical parameter order");
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
        if actual != Clause::Predicate(view.predicate(&record.value, steps)?) {
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
    for (index, (record, actual)) in record
        .normalizations
        .iter()
        .zip(&contract.proofs)
        .enumerate()
    {
        validate_normalization(view, index, record, actual, steps)?;
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
    if view.const_context() && view.signature().is_some() {
        crate::body::defining_cleanups::validate_imported_required(
            facts,
            view.definition(),
            contract,
            steps,
        )?;
    } else if builtin.defining().is_some() {
        return Err("ordinary source original cannot import defining cleanup requests");
    }
    match (
        expected.builtin_environment()?,
        builtin.environment.as_ref(),
    ) {
        (None, None) => {}
        (Some(expected), Some(actual)) if Arc::ptr_eq(actual, expected) => {}
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
            for (record, &actual) in record.goals.iter().zip(&actual.goals) {
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
        (
            Kind::Variable {
                id: a,
                needs_provenance: ap,
            },
            Kind::Variable {
                id: b,
                needs_provenance: bp,
            },
        ) => a == b && ap == bp,
        (Kind::LocalBorrow(a), Kind::LocalBorrow(b)) => a == b,
        (Kind::Equality(actual), Kind::Equality(expected))
        | (Kind::Outlives(actual), Kind::Outlives(expected)) => actual == expected,
        (Kind::WellFormed(actual), Kind::WellFormed(expected)) => actual == expected,
        (
            Kind::ScopedWellFormed {
                ty: actual,
                bound_regions: actual_regions,
            },
            Kind::ScopedWellFormed {
                ty: expected,
                bound_regions: expected_regions,
            },
        ) => actual == expected && actual_regions == expected_regions,
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
    index: usize,
    record: &s::NormalizationInput,
    actual: &Proof,
    steps: &mut usize,
) -> Result<(), &'static str> {
    let regions = view.normalization_interval(index, record, steps)?;
    if actual.proof.regions != regions
        || actual.source != view.input_site(record.site, steps)?
        || actual.sources.len() != record.sources.len()
    {
        return Err("source original changed its complete record normalization domain");
    }
    for (&record, &actual) in record.sources.iter().zip(&actual.sources) {
        if view.input_site(record, steps)? != actual {
            return Err("source normalization changed an actual diagnostic origin");
        }
    }
    validate_normalization_relations(
        view,
        (&record.equalities, &record.outlives, &record.types),
        &actual.proof,
        steps,
    )?;
    if let Some(scoped) = &record.scoped {
        let evidence = actual
            .proof
            .single_normalization_scope()
            .ok_or("source normalization lost its actual single scope")?;
        if evidence.regions != view.normalization_scope(scoped.frame, steps)?
            || evidence.callable != scoped.callable
            || evidence.origin != scoped.origin
        {
            return Err("source normalization changed its scoped binder or origin");
        }
        validate_normalization_relations(
            view,
            (&scoped.equalities, &scoped.outlives, &scoped.types),
            evidence.block,
            steps,
        )?;
    } else if actual.proof.has_quantified() {
        return Err("source normalization acquired an unrecorded scope");
    }
    Ok(())
}

fn validate_normalization_relations(
    view: &SourceOriginal<'_>,
    (equalities, outlives, types): RelationRows<'_>,
    proof: &Obligations,
    steps: &mut usize,
) -> Result<(), &'static str> {
    if equalities.len() != proof.equalities.len()
        || outlives.len() != proof.outlives.len()
        || types.len() != proof.types.len()
    {
        return Err("source original changed its complete record normalization relations");
    }
    for (rows, mapped) in [(equalities, &proof.equalities), (outlives, &proof.outlives)] {
        for (&record, &actual) in rows.iter().zip(mapped) {
            if pair(view, record, steps)? != actual {
                return Err("source normalization changed a record relation");
            }
        }
    }
    for (record, &actual) in types.iter().zip(&proof.types) {
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
