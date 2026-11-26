//! Always-applicable traits extend Min's predicate basis only with checked impls.
use super::{
    annotation_reads::{applicable_flag, imported_policy},
    annotation_store::LocalMarkers,
    header_conditions, minimal_admission,
    predicate_basis::Basis,
    retention_work::Budget,
};
use crate::{
    resolve::{expansion::Report, model::DefId, specialization_authority::Requests, unit::NodeKey},
    sema::model::Declarations,
    support::bindings::LangItem,
    types::{
        arguments::Argument,
        parameters::{
            clauses::Clause,
            predicates::{Bound, Predicate},
        },
    },
};
#[cfg(test)]
#[path = "../../../tests/invariants/applicable_headers.rs"]
mod tests;

fn policy(facts: &Declarations, report: &Report, clause: Clause) -> Result<bool, String> {
    let Clause::Predicate(predicate) = clause else {
        return Ok(false);
    };
    if predicate.bound == Bound::Sized {
        // Bootstrap's fixed Sized is marked in its pinned defining source. A
        // source provider instead supplies its actual trait and policy witness.
        let Some(catalog) = facts.source_bindings.as_deref() else {
            return Ok(true);
        };
        let role = catalog
            .role(LangItem::Sized)?
            .ok_or("missing actual source Sized")?;
        return super::applicable(facts, report, role.definition).map_err(|e| e.to_string());
    }
    let Some((application, _)) = predicate.bound.trait_parts() else {
        return Ok(false);
    };
    let definition = facts
        .types
        .trait_application(application)
        .ok_or("missing always-applicable trait application")?
        .item;
    super::applicable(facts, report, DefId(definition.0)).map_err(|e| e.to_string())
}

pub(super) fn permit(
    facts: &Declarations,
    report: &Report,
    actual: &Basis,
    permitted: &mut Basis,
    budget: &Budget,
) -> Result<(), String> {
    for &clause in &actual.clauses {
        budget.charge(1, 0)?;
        if policy(facts, report, clause)? {
            permitted.insert(clause, budget)?;
        }
    }
    Ok(())
}

pub(in crate::sema) fn verify(
    facts: &mut Declarations,
    report: &Report,
    headers: &[(DefId, Option<NodeKey>, Predicate)],
) -> Result<(), String> {
    let local = facts
        .must_use_contracts
        .as_deref()
        .and_then(|owner| owner.specialization.as_deref())
        .filter(|table| table.has_applicable);
    let imported = facts.associated.imported_owner().is_some();
    if local.is_none() && !imported {
        return Ok(());
    }
    if local.is_some_and(|table| !table.complete) {
        return Err("applicable trait policy inventory is unfinished".into());
    }
    let local_policies = local.is_some();
    let budget = local.map_or_else(|| Budget::new(0), LocalMarkers::budget)?;
    let requests = Requests::new(report).map_err(|e| e.to_string())?;
    let mut allowances = report.macro_allowances();
    for &(owner, source, predicate) in headers {
        budget.charge(1, 0)?;
        // A provider-owned implementation was admitted in its producer.
        if source.is_none() {
            continue;
        }
        let Some((application, _)) = predicate.bound.trait_parts() else {
            continue;
        };
        let definition = facts
            .types
            .trait_application(application)
            .ok_or("applicable implementation lost its trait")?
            .item;
        if !implemented_trait_policy(facts, report, DefId(definition.0), local_policies)? {
            continue;
        }
        let admission = requests
            .capture(report, &mut allowances, owner, |steps, bytes| {
                budget.charge(steps, bytes)
            })
            .map_err(|e| e.to_string())?;
        if !admission.may_specialize() {
            return Err("implementing an always-applicable trait requires specialization or min_specialization".into());
        }
        if requests.min_checks() {
            implementation(facts, report, owner, predicate, &budget)?;
        }
        admission
            .validate(report, &requests, |steps, bytes| {
                budget.charge(steps, bytes)
            })
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn implemented_trait_policy(
    facts: &Declarations,
    report: &Report,
    definition: DefId,
    local_policies: bool,
) -> Result<bool, String> {
    let actual = report
        .resolution
        .definitions
        .get(definition.0 as usize)
        .ok_or("applicable implementation lost its actual declaration")?;
    if actual.node.is_some() {
        return local_policies
            .then(|| super::applicable(facts, report, definition))
            .transpose()
            .map(|allowed| allowed.unwrap_or(false))
            .map_err(|error| error.to_string());
    }
    imported_policy(facts, definition)
        .map(|flags| flags.is_some_and(applicable_flag))
        .map_err(|error| error.to_string())
}

fn implementation(
    facts: &mut Declarations,
    report: &Report,
    owner: DefId,
    predicate: Predicate,
    budget: &Budget,
) -> Result<(), String> {
    let (application, _) = predicate
        .bound
        .trait_parts()
        .ok_or("missing applicable trait")?;
    let inputs = facts
        .types
        .trait_application(application)
        .ok_or("missing applicable inputs")?
        .arguments;
    let count = facts
        .types
        .arguments(inputs)
        .ok_or("invalid applicable arguments")?
        .len();
    budget.charge(count + 1, (count + 1) * size_of::<Argument>())?;
    let mut arguments = Vec::with_capacity(count + 1);
    arguments.push(Argument::Type(predicate.ty));
    arguments.extend(
        facts
            .types
            .arguments(inputs)
            .ok_or("missing applicable arguments")?,
    );
    let arguments = facts.types.intern_arguments(&arguments)?;
    minimal_admission::structural_arguments(facts, arguments, budget)?;
    let identity = minimal_admission::identity_arguments(facts, owner, budget)?;
    let mut actual = Basis::default();
    actual.conditions(
        header_conditions::defining(facts, owner, identity, budget)?,
        budget,
    )?;
    actual.elaborate(facts, budget)?;
    let mut permitted = Basis::default();
    minimal_admission::well_formed_predicate(facts, predicate, &mut permitted, budget)?;
    permit(facts, report, &actual, &mut permitted, budget)?;
    permitted.elaborate(facts, budget)?;
    budget.charge(actual.regions.len(), 0)?;
    if !actual.regions.is_subset(&permitted.regions) {
        return Err("always-applicable implementation introduces a lifetime requirement".into());
    }
    for clause in actual.clauses {
        budget.charge(1, 0)?;
        if !permitted.clauses.contains(&clause) {
            minimal_admission::additional(facts, report, clause)?;
        }
    }
    Ok(())
}
