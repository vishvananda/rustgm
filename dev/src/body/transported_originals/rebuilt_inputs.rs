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
    assumed_inputs: usize,
    assume_fold_traits: bool,
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
        let provided = view.const_context()
            && matches!(
                view.inputs().environment,
                s::EnvironmentOrigin::ConstTraitDefault { .. }
            );
        let assume_fold_traits = provided && facts.imported_iterator_fold(definition)?;
        let nested_reduction_helper = view
            .rows()
            .declarations
            .get(view.raw().declaration.0 as usize)
            .and_then(|row| row.definition.parent)
            .map(|parent| {
                let parent = view.declaration(parent, steps)?;
                Ok(facts.imported_iterator_last(parent)?
                    || facts.imported_iterator_count(parent)?)
            })
            .transpose()?
            .unwrap_or(false);
        let assumed_inputs = if view.const_context()
            && (crate::sema::compiler_items::drop_glue_member(facts, definition)?
                || provided && facts.imported_clone_default(definition)?
                || provided && facts.imported_iterator_last(definition)?
                || provided && facts.imported_iterator_count(definition)?
                || nested_reduction_helper
                || assume_fold_traits)
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
            assumed_inputs,
            assume_fold_traits,
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
                    return obligation(self.facts, self.view, record.obligations[index], steps)
                        .map(Some);
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
    /// A declared result binding and a whole quantified trait clause stay in
    /// the defining environment. Each caller re-proves the same header inputs.
    /// Older profiles only carry the empty scope opened by callable sugar.
    fn declared_predicate(
        &self,
        index: usize,
        clause: Clause,
    ) -> Result<Option<Predicate>, &'static str> {
        let profile = self.view.rows().descriptor().profile();
        let body_nodes = profile.body_nodes();
        let predicate = match clause {
            Clause::Predicate(predicate) => predicate,
            Clause::ForAll(id) if body_nodes => {
                let scoped = self
                    .facts
                    .types
                    .quantified(id)
                    .ok_or("source quantified declaration debt is unknown")?;
                if scoped.bound_regions != 0 {
                    if profile.declaration_policies()
                        && matches!(
                            scoped.predicate.bound,
                            Bound::Trait(_)
                                | Bound::MaybeConstTrait(_)
                                | Bound::ConstTrait(_)
                                | Bound::ProjectionEqual(_)
                        )
                    {
                        return Ok(None);
                    }
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
        let header_premise = match predicate.bound {
            Bound::ProjectionEqual(_) => body_nodes,
            Bound::Trait(_)
            | Bound::MaybeConstTrait(_)
            | Bound::ConstTrait(_)
            | Bound::Outlives(_) => profile.declaration_policies(),
            _ => false,
        };
        // The checked defining header owns these trait assumptions, and the
        // caller pays them at the selected effect. Only the builtin debts
        // belong to the borrow contract.
        let assumed = index < self.assumed_inputs
            && (matches!(predicate.bound, Bound::MaybeConstTrait(_))
                || self.assume_fold_traits && predicate.bound.trait_parts().is_some());
        Ok((!header_premise && !assumed).then_some(predicate))
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

pub(super) struct QuantifiedAssumption {
    pub(super) index: usize,
    pub(super) bound_regions: u32,
    pub(super) predicate: Predicate,
}

pub(super) fn build(
    facts: &Declarations,
    view: &SourceOriginal<'_>,
    body: &Body,
    work: &mut ImportedOriginals,
) -> Result<(Contract, Vec<QuantifiedAssumption>), &'static str> {
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
    let mut quantified_assumptions = Vec::new();
    let mut type_assumptions = Vec::with_capacity(record.type_assumptions.len());
    for (index, input) in record.type_assumptions.iter().enumerate() {
        view.input_site(input.site, &mut work.steps)?;
        let predicate = view.predicate(&input.value, &mut work.steps)?;
        if let Some(binder) = input.binder {
            let bound_regions = view.normalization_scope(binder, &mut work.steps)?;
            quantified_assumptions.push(QuantifiedAssumption {
                index,
                bound_regions,
                predicate,
            });
        }
        type_assumptions.push(Clause::Predicate(predicate));
    }
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
        output: view.closure_output(&mut work.steps)?,
        require_result_formation: view.require_result_formation(),
    };
    if view.const_context() && view.signature().is_some() && view.closure_owner().is_none() {
        let required = crate::body::defining_cleanups::mark_imported_required(
            facts,
            view.definition(),
            &mut contract,
            &mut work.steps,
        );
        work.charge(0)?;
        required?;
    }
    Ok((contract, quantified_assumptions))
}
/// The wire admits the header's original result against the body's result by
/// shape only; their regional relation is a fresh request. The body's value
/// flows into the declared result every caller receives, so it must coerce.
fn declared_result(
    view: &SourceOriginal<'_>,
    root: ProofSite,
    steps: &mut usize,
) -> Result<Option<Obligation>, &'static str> {
    if view.closure_owner().is_some() {
        return Ok(None);
    }
    let rows = view.rows();
    if rows.descriptor().profile().declaration_policies()
        && rows
            .cold()
            .opaques
            .declarations
            .binary_search_by_key(&view.raw().declaration.0, |row| row.declaration.0)
            .is_ok()
    {
        return Ok(None);
    }
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
    facts: &Declarations,
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
        s::InputKind::Membership { region, captures }
            if view.rows().descriptor().profile().declaration_policies() =>
        {
            super::regional_records::require(view)?;
            Kind::Membership {
                region: view.region(region, steps)?,
                captures: view.membership_captures(facts, captures, steps)?,
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
            if view.rows().descriptor().profile().body_nodes() =>
        {
            let bound_regions = view
                .rows()
                .binders
                .get(frame.0 as usize)
                .filter(|row| {
                    row.count == 0 || view.rows().descriptor().profile().declaration_policies()
                })
                .ok_or("source scoped formation lost its admitted clause binder")?
                .count;
            Kind::ScopedWellFormed {
                ty: view.ty(ty, steps)?,
                bound_regions,
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
type AllocationRun = (
    crate::support::crates::generics::Range,
    std::ops::Range<u32>,
);

fn normalization(
    view: &SourceOriginal<'_>,
    index: usize,
    record: &s::NormalizationInput,
    steps: &mut usize,
) -> Result<Proof, &'static str> {
    let runs = view.normalization_intervals(index, record, steps)?;
    let start = runs.first().map_or(0, |(_, ids)| ids.start);
    let mut proof = normalization_relations(
        view,
        (&record.equalities, &record.outlives, &record.types),
        start..start,
        steps,
    )?;
    let mut run_index = 0;
    if let Some(scoped) = &record.scoped {
        append_scoped(view, &runs, scoped, &mut proof, &mut run_index, steps)?;
    }
    if let Some((_, last)) = runs.last() {
        advance_root_to(&mut proof, &runs, &mut run_index, last.end)?;
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

fn append_scoped(
    view: &SourceOriginal<'_>,
    runs: &[AllocationRun],
    scoped: &s::ScopedNormalization,
    proof: &mut Obligations,
    run_index: &mut usize,
    steps: &mut usize,
) -> Result<(), &'static str> {
    let mut ids = Vec::with_capacity(scoped.scopes.len().saturating_add(1));
    ids.push(ScopeId::ROOT);
    for row in &scoped.scopes {
        let parent = *ids
            .get(row.parent as usize)
            .ok_or("source normalization scope parent is out of order")?;
        let count = view.normalization_scope(row.frame, steps)?;
        let scope = if row.callable {
            proof.callable_scope(parent, count)?
        } else {
            proof.scope(parent, count)?
        };
        ids.push(scope);
    }
    for block in &scoped.blocks {
        let scope = *ids
            .get(block.scope as usize)
            .ok_or("source normalization block names an unknown scope")?;
        let block_range = if block.regions.start == block.regions.end {
            let pos = proof.regions.end;
            pos..pos
        } else {
            let range = map_block_range(runs, block.regions)?;
            advance_root_to(proof, runs, run_index, range.start)?;
            range
        };
        if block.universal_aliases {
            proof.append_universals(block_range, scope)?;
        } else {
            let nested = normalization_relations(
                view,
                (&block.equalities, &block.outlives, &block.types),
                block_range,
                steps,
            )?;
            proof.append_at_origin(nested, scope, block.origin)?;
        }
        while *run_index < runs.len() && proof.regions.end >= runs[*run_index].1.end {
            *run_index += 1;
        }
    }
    Ok(())
}

fn advance_root_to(
    proof: &mut Obligations,
    runs: &[AllocationRun],
    run_index: &mut usize,
    target: u32,
) -> Result<(), &'static str> {
    while proof.regions.end < target {
        let (_, ids) = runs
            .get(*run_index)
            .ok_or("source normalization allocation ran past its recorded runs")?;
        if proof.regions.end < ids.start {
            proof.resume_at(ids.start.min(target))?;
        } else {
            proof.regions.end = ids.end.min(target);
            if proof.regions.end == ids.end {
                *run_index += 1;
            }
        }
    }
    Ok(())
}

fn map_block_range(
    runs: &[AllocationRun],
    slots: crate::support::crates::generics::Range,
) -> Result<std::ops::Range<u32>, &'static str> {
    for (run_slots, ids) in runs {
        if slots.start >= run_slots.start && slots.end <= run_slots.end {
            let offset = slots.start - run_slots.start;
            let len = slots.end - slots.start;
            let start = ids
                .start
                .checked_add(offset)
                .ok_or("source normalization block interval overflow")?;
            let end = start
                .checked_add(len)
                .ok_or("source normalization block interval overflow")?;
            return Ok(start..end);
        }
    }
    Err("source normalization block spans a discontinuous allocation gap")
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
    let expected_output = view.closure_output(steps)?;
    if contract.key != body.key()
        || contract.source != view.source()?
        || contract.signature != view.signature()
        || contract.universals != view.universals()
        || !same_output(contract.output.as_deref(), expected_output.as_deref())
        || contract.require_result_formation != view.require_result_formation()
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
        let predicate = view.predicate(&record.value, steps)?;
        let matches = match (actual, record.binder) {
            (Clause::Predicate(actual), None) => actual == predicate,
            (Clause::ForAll(id), Some(binder)) => {
                let bound_regions = view.normalization_scope(binder, steps)?;
                facts.types.quantified(id).is_some_and(|quantified| {
                    quantified.bound_regions == bound_regions && quantified.predicate == predicate
                })
            }
            _ => false,
        };
        if !matches {
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
    if view.const_context() && view.signature().is_some() && view.closure_owner().is_none() {
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
fn same_output(
    actual: Option<&crate::sema::nested_outputs::Link>,
    expected: Option<&crate::sema::nested_outputs::Link>,
) -> bool {
    match (actual, expected) {
        (None, None) => true,
        (Some(a), Some(b)) => {
            a.source == b.source
                && a.parent == b.parent
                && a.revision == b.revision
                && a.universals == b.universals
                && a.creator_only_from == b.creator_only_from
                && a.bridges.len() == b.bridges.len()
                && a.bridges
                    .iter()
                    .zip(&b.bridges)
                    .all(|(x, y)| x.slot == y.slot && x.parent == y.parent && x.output == y.output)
                && match (&a.formation, &b.formation) {
                    (None, None) => true,
                    (Some(x), Some(y)) => {
                        x.identity == y.identity
                            && x.inherited == y.inherited
                            && x.callable_slots == y.callable_slots
                            && x.inputs.len() == y.inputs.len()
                            && x.inputs.iter().zip(&y.inputs).all(|(p, q)| {
                                p.root == q.root && p.child == q.child && p.parent == q.parent
                            })
                    }
                    _ => false,
                }
        }
        _ => false,
    }
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
        (
            Kind::Membership {
                region: a,
                captures: ac,
            },
            Kind::Membership {
                region: b,
                captures: bc,
            },
        ) => a == b && ac == bc,
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
    let runs = view.normalization_intervals(index, record, steps)?;
    let expected_envelope = match (runs.first(), runs.last()) {
        (Some((_, first)), Some((_, last))) => first.start..last.end,
        _ => 0..0,
    };
    if actual.proof.regions != expected_envelope
        || actual.source != view.input_site(record.site, steps)?
        || actual.sources.len() != record.sources.len()
        || !actual
            .proof
            .owned_regions()
            .eq(runs.iter().map(|(_, ids)| ids.clone()))
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
        validate_scoped_normalization(view, &runs, scoped, &actual.proof, steps)?;
    } else if actual.proof.scopes().next().is_some()
        || actual.proof.quantified_blocks().next().is_some()
    {
        return Err("source normalization acquired an unrecorded scope");
    }
    Ok(())
}

fn validate_scoped_normalization(
    view: &SourceOriginal<'_>,
    runs: &[AllocationRun],
    scoped: &s::ScopedNormalization,
    proof: &Obligations,
    steps: &mut usize,
) -> Result<(), &'static str> {
    let mut actual_scopes = proof.scopes();
    for row in &scoped.scopes {
        let actual = actual_scopes
            .next()
            .ok_or("source normalization omitted a scoped binder")?;
        if actual.parent.0 != row.parent
            || actual.count != view.normalization_scope(row.frame, steps)?
            || actual.callable != row.callable
        {
            return Err("source normalization changed its scoped binder or origin");
        }
    }
    if actual_scopes.next().is_some() {
        return Err("source normalization acquired an extra scoped binder");
    }
    let mut actual_blocks = proof.quantified_blocks();
    for row in &scoped.blocks {
        let (scope, universal_aliases, origin, block) = actual_blocks
            .next()
            .ok_or("source normalization omitted a scoped block")?;
        let regions_match = if row.regions.start == row.regions.end {
            block.regions.is_empty()
        } else {
            block.regions == map_block_range(runs, row.regions)?
        };
        if scope.0 != row.scope
            || universal_aliases != row.universal_aliases
            || origin != row.origin
            || !regions_match
        {
            return Err("source normalization changed its scoped binder or origin");
        }
        validate_normalization_relations(
            view,
            (&row.equalities, &row.outlives, &row.types),
            block,
            steps,
        )?;
    }
    if actual_blocks.next().is_some() {
        return Err("source normalization acquired an extra scoped block");
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
