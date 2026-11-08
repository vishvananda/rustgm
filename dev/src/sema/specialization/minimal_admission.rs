//! Crate-Min rules stay mandatory even when Full/token admission also applies.
use super::{
    header_conditions,
    predicate_basis::Basis,
    registry_graph::{Forest, Relation},
    retention_work::Budget,
};
use crate::{
    resolve::{expansion::Report, model::DefId},
    sema::{model::Declarations, nominals},
    support::retained::Growth,
    types::{
        GenericArgs, RegionId, TypeId,
        arguments::Argument,
        parameters::{clauses::Clause, constraints::Constraints, uses::Multiplicities},
    },
};
pub(super) fn validate(
    facts: &mut Declarations,
    report: &Report,
    forest: &Forest,
    budget: &Budget,
) -> Result<(), String> {
    for relation in &forest.parents {
        nonempty(facts, relation.child.owner())?;
        let parent = header_conditions::defining(
            facts,
            relation.parent.owner(),
            relation.parent_arguments,
            budget,
        )?;
        structural(facts, relation, budget)?;
        let child = identity_arguments(facts, relation.child.owner(), budget)?;
        let mut actual = Basis::default();
        actual.conditions(
            header_conditions::defining(facts, relation.child.owner(), child, budget)?,
            budget,
        )?;
        actual.elaborate(facts, budget)?;
        let mut permitted = Basis::default();
        permitted.conditions(parent, budget)?;
        well_formed(facts, forest, relation, &mut permitted, budget)?;
        super::applicable_rules::permit(facts, report, &actual, &mut permitted, budget)?;
        permitted.elaborate(facts, budget)?;
        budget.charge(actual.regions.len(), 0)?;
        if !actual.regions.is_subset(&permitted.regions) {
            return Err("minimal specialization introduces a lifetime requirement".into());
        }
        for clause in actual.clauses {
            budget.charge(1, 0)?;
            if !permitted.clauses.contains(&clause) {
                additional(facts, report, clause)?;
            }
        }
    }
    Ok(())
}
fn nonempty(facts: &Declarations, child: DefId) -> Result<(), &'static str> {
    let owner = facts
        .associated
        .owner(child)
        .ok_or("specialization child owner disappeared")?;
    if facts
        .associated
        .owner_members(owner)
        .is_none_or(<[_]>::is_empty)
    {
        return Err("minimal specializing implementation must define an associated item");
    }
    Ok(())
}
/// As rustc's `unconstrained_parent_impl_args`, a parent parameter that its own
/// associated equalities determine is omitted before the repeated-parameter and
/// `'static` checks: knowing the other arguments already fixes it.
fn structural(
    facts: &mut Declarations,
    relation: &Relation,
    budget: &Budget,
) -> Result<(), &'static str> {
    let parent = relation.parent.owner();
    let header = facts
        .generics
        .implementations
        .as_deref()
        .ok_or("specialization registry disappeared")?
        .checked_entry(relation.parent)?
        .predicate;
    let clauses: Vec<_> = crate::sema::declaration_inputs::declared(
        &facts.generics,
        facts.crates.as_deref(),
        parent,
    )?
    .map(|input| input.value)
    .collect();
    budget.charge(
        clauses.len(),
        clauses.len().saturating_mul(size_of::<Clause>()),
    )?;
    let determined = Constraints::projection_determined(&facts.types, header, clauses)?;
    budget.charge(determined.steps, 0)?;
    let ids = facts.generics.parameter_ids(&facts.types, parent)?;
    let values: Vec<_> = facts
        .types
        .arguments(relation.parent_arguments)
        .ok_or("specialization mapping disappeared")?
        .collect();
    let captures = values
        .len()
        .checked_sub(ids.len())
        .ok_or("specialization mapping arity changed")?;
    budget.charge(
        values.len(),
        values.len().saturating_mul(2 * size_of::<Argument>()),
    )?;
    let mut kept = Vec::with_capacity(values.len());
    for (index, argument) in values.into_iter().enumerate() {
        let omitted = match index.checked_sub(captures) {
            None => determined.lifetime(
                u16::try_from(index).map_err(|_| "specialization parent capture overflow")?,
            ),
            Some(position) => determined.value(
                *ids.get(position)
                    .ok_or("specialization mapping arity changed")?,
            ),
        };
        if !omitted {
            kept.push(argument);
        }
    }
    let arguments = facts.types.intern_arguments(&kept)?;
    structural_arguments(facts, arguments, budget)
}
pub(super) fn structural_arguments(
    facts: &Declarations,
    arguments: GenericArgs,
    budget: &Budget,
) -> Result<(), &'static str> {
    let argument_id = arguments;
    let arguments = facts
        .types
        .arguments(argument_id)
        .ok_or("specialization mapping disappeared")?;
    budget.charge(arguments.len(), 0)?;
    for argument in arguments {
        // Relation construction already rejects private free regions. At maximum
        // binder depth, this existing summary therefore detects precisely static.
        let static_ = match argument {
            Argument::Lifetime(region) => region == RegionId::STATIC,
            Argument::Type(ty) => facts
                .types
                .has_free_regions(ty, u16::MAX)
                .ok_or("invalid mapped parent type")?,
            Argument::Const(_) => false,
        };
        if static_ {
            return Err("minimal specialization introduces a static lifetime");
        }
        if let Argument::Type(ty) = argument
            && facts.types.contains_projections(ty) != Some(false)
        {
            return Err(
                "minimal specialization projection-constrained parent arguments are not implemented",
            );
        }
    }
    let paths = Multiplicities::arguments(&facts.types, argument_id, |steps, bytes| {
        budget.charge(steps, bytes)
    })?;
    if paths.repeated(|steps, bytes| budget.charge(steps, bytes))? {
        return Err("minimal specializing implementation repeats a parameter");
    }
    Ok(())
}
pub(super) fn additional(
    facts: &Declarations,
    report: &Report,
    clause: Clause,
) -> Result<(), String> {
    let Clause::Predicate(predicate) = clause else {
        return Err("higher-ranked minimal specialization clauses are not implemented".into());
    };
    if predicate.requires_definition_proof(&facts.types)? {
        return Ok(());
    }
    if let Some((application, _)) = predicate.bound.trait_parts() {
        let definition = DefId(
            facts
                .types
                .trait_application(application)
                .ok_or("invalid specialization marker application")?
                .item
                .0,
        );
        if super::marker(facts, report, definition).map_err(|error| error.to_string())? {
            let owner = facts
                .associated
                .owner(definition)
                .ok_or("specialization marker trait owner disappeared")?;
            if facts
                .associated
                .owner_members(owner)
                .is_none_or(<[_]>::is_empty)
            {
                return Ok(());
            }
            return Err(
                "unsafe specialization marker trait must not define associated items".into(),
            );
        }
    }
    Err("minimal specialization introduces a non-marker predicate".into())
}
pub(super) fn identity_arguments(
    facts: &mut Declarations,
    owner: DefId,
    budget: &Budget,
) -> Result<GenericArgs, &'static str> {
    let schema = facts
        .associated
        .schema(owner)
        .ok_or("missing child specialization schema")?;
    let count = (schema.captures as usize)
        .saturating_add(facts.generics.parameters.get(&owner).map_or(0, Vec::len));
    budget.charge(count, count.saturating_mul(size_of::<Argument>()))?;
    let mut arguments = Vec::with_capacity(count);
    for slot in 0..schema.captures {
        arguments.push(Argument::Lifetime(RegionId::bound(
            0,
            u16::try_from(slot).map_err(|_| "child specialization captures overflow")?,
        )?));
    }
    arguments.extend(
        facts
            .generics
            .parameters
            .get(&owner)
            .into_iter()
            .flatten()
            .filter_map(|parameter| {
                (!matches!(parameter.argument, Argument::Lifetime(_))).then_some(parameter.argument)
            }),
    );
    facts.types.intern_arguments(&arguments)
}
fn well_formed(
    facts: &mut Declarations,
    forest: &Forest,
    relation: &Relation,
    permitted: &mut Basis,
    budget: &Budget,
) -> Result<(), &'static str> {
    let child = forest
        .by_entry
        .get(&relation.child)
        .ok_or("missing child specialization vertex")?;
    let predicate = forest.vertices[child.index()].witness.predicate;
    well_formed_predicate(facts, predicate, permitted, budget)
}
pub(super) fn well_formed_predicate(
    facts: &mut Declarations,
    predicate: crate::types::parameters::predicates::Predicate,
    permitted: &mut Basis,
    budget: &Budget,
) -> Result<(), &'static str> {
    let mut roots = Vec::new();
    let count = predicate
        .bound
        .trait_parts()
        .map(|(id, _)| {
            facts
                .types
                .trait_application(id)
                .ok_or("invalid child trait application")
        })
        .transpose()?
        .map(|application| application.arguments);
    let additional = count
        .map(|args| {
            facts
                .types
                .arguments(args)
                .ok_or("invalid child trait inputs")
                .map(|values| values.len())
        })
        .transpose()?
        .unwrap_or(0);
    let growth = Growth::vector::<TypeId>(0, 0, additional.saturating_add(1));
    budget.charge(additional.saturating_add(1) + growth.work, growth.bytes)?;
    roots.push(predicate.ty);
    if let Some(arguments) = count {
        roots.extend(
            facts
                .types
                .arguments(arguments)
                .ok_or("missing child trait inputs")?
                .filter_map(|argument| match argument {
                    Argument::Type(ty) => Some(ty),
                    _ => None,
                }),
        );
    }
    let mut checked = nominals::FormationRequirements::default();
    for ty in roots {
        for clause in
            nominals::formation_requirements_counted(facts, ty, &mut checked, &|steps, bytes| {
                budget.charge(steps, bytes)
            })?
        {
            permitted.insert(clause, budget)?;
        }
    }
    Ok(())
}
