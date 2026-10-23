//! Reconstruct mandatory goals from actual operations and complete declaration inputs.
use crate::{
    body::{
        ir::Body,
        regions::{BuiltinRequirements, Contract, Kind, Obligation},
    },
    sema::model::Declarations,
    support::crates::generics as g,
    types::parameters::predicates::{Bound, Predicate},
};
use std::sync::Arc;
pub(super) fn allocation(contract: &Contract) -> usize {
    use crate::support::retained::{sum, vector};
    sum([
        vector(&contract.assumptions),
        vector(&contract.parameters),
        vector(&contract.type_assumptions),
        vector(&contract.obligations),
        vector(&contract.proofs),
        contract
            .builtins
            .as_ref()
            .map_or(0, |b| 2 * size_of::<usize>() + b.retained_bytes()),
    ])
}
pub(super) fn build(
    facts: &Declarations,
    payload: &g::Payload,
    function: &g::Function,
    body: &Body,
    work: &mut super::ImportedOriginals,
) -> Result<Contract, &'static str> {
    let crate::body::ir::Owner::Function {
        definition,
        signature,
        execution: None,
    } = body.owner
    else {
        return Err("imported original is not its actual function owner");
    };
    let header = facts
        .function_header(definition)?
        .ok_or("missing imported original header")?;
    if header.body != Some(body.source) || header.signature != signature || header.universals != 0 {
        return Err("imported original contract changed its exact declaration identity");
    }
    let environment = header
        .environment
        .cloned()
        .ok_or("imported original has no declared environment")?;
    let parameters = facts.generics.parameter_ids(&facts.types, definition)?;
    let type_assumptions: Vec<crate::types::parameters::clauses::Clause> =
        crate::sema::declaration_inputs::own(&facts.generics, facts.crates.as_deref(), definition)?
            .map(|input| input.value)
            .collect();
    work.charge(
        parameters
            .len()
            .saturating_add(type_assumptions.len())
            .saturating_add(body.locals.len())
            .saturating_add(body.values.len())
            .saturating_add(body.blocks.len())
            .saturating_add(1),
    )?;
    let root = body.source.root_site();
    let mut obligations = Vec::with_capacity(
        body.locals
            .len()
            .saturating_add(2 * body.values.len())
            .saturating_add(body.blocks.len())
            .saturating_add(parameters.len())
            .saturating_add(1),
    );
    let mut predicates = Vec::with_capacity(
        body.locals.len() + body.values.len() + type_assumptions.len() + parameters.len() + 1,
    );
    work.charge(parameters.len())?;
    formal_goals(facts, definition, root, &mut obligations, &mut predicates)?;
    by_value_goals(body, &mut obligations, &mut predicates);
    // A declared input is an assumption only for the original generic proof.
    // The same goal remains pending after actual argument substitution, whose
    // ordinary transport deliberately clears this original Environment.
    for clause in &type_assumptions {
        predicates.push((root, declared_goal(facts, *clause)?));
    }
    relations(facts, payload, function, body, &mut obligations, work)?;
    super::invocations::append(
        facts,
        payload,
        function,
        body,
        &mut obligations,
        &mut predicates,
        work,
    )?;
    Ok(Contract {
        key: body.key(),
        source: body.source,
        signature: Some(signature),
        universals: 0,
        assumptions: Vec::new(),
        parameters,
        type_assumptions,
        obligations,
        proofs: Vec::new(),
        builtins: Some(Arc::new(BuiltinRequirements::imported(
            environment,
            predicates,
        ))),
        output: None,
        require_result_formation: false,
    })
}

fn by_value_goals(
    body: &Body,
    obligations: &mut Vec<Obligation>,
    predicates: &mut Vec<(crate::body::ir::ProofSite, Predicate)>,
) {
    let root = body.source.root_site();
    // Every by-value port must be well formed and sized; metadata cannot omit
    // these goals by supplying an apparently empty encoded Contract.
    for ty in body
        .locals
        .iter()
        .map(|local| local.ty)
        .chain(body.values.iter().map(|value| value.ty))
        .chain(std::iter::once(body.result))
    {
        obligations.push(Obligation {
            source: root,
            kind: Kind::WellFormed(ty),
        });
        predicates.push((
            root,
            Predicate {
                ty,
                bound: Bound::Sized,
            },
        ));
    }
}

fn formal_goals(
    facts: &Declarations,
    definition: crate::resolve::model::DefId,
    root: crate::body::ir::ProofSite,
    obligations: &mut Vec<Obligation>,
    predicates: &mut Vec<(crate::body::ir::ProofSite, Predicate)>,
) -> Result<(), &'static str> {
    use crate::types::{ItemId, Kind as Type, arguments::Argument, parameters::ParameterKind};
    for (ordinal, formal) in facts.generics.parameters_in_scope(definition).enumerate() {
        let Argument::Type(ty) = formal.argument else {
            return Err("version-zero original has a non-type formal");
        };
        let Some(Type::Parameter(id)) = facts.types.kind(ty) else {
            return Err("imported formal lost its canonical parameter identity");
        };
        let declared = facts
            .types
            .parameter(id)
            .ok_or("unknown imported formal parameter")?;
        let ParameterKind::Type { sized } = declared.kind else {
            return Err("imported formal changed its actual parameter kind");
        };
        if declared.owner != ItemId(definition.0) || declared.index as usize != ordinal {
            return Err("imported formal belongs to another declaration or slot");
        }
        // Unused arguments are still part of the complete call contract. Their
        // original permissions disappear during actual instance substitution.
        obligations.push(Obligation {
            source: root,
            kind: Kind::WellFormed(ty),
        });
        if sized {
            predicates.push((
                root,
                Predicate {
                    ty,
                    bound: Bound::Sized,
                },
            ));
        }
    }
    Ok(())
}

fn declared_goal(
    facts: &Declarations,
    clause: crate::types::parameters::clauses::Clause,
) -> Result<Predicate, &'static str> {
    let predicate = match clause {
        crate::types::parameters::clauses::Clause::Predicate(predicate) => predicate,
        crate::types::parameters::clauses::Clause::ForAll(id) => {
            let scoped = facts
                .types
                .quantified(id)
                .ok_or("unknown imported builtin clause")?;
            if scoped.bound_regions != 0
                || facts.types.contains_regions(scoped.predicate.ty) != Some(false)
            {
                return Err("imported quantified builtin goal requires its full scoped transport");
            }
            scoped.predicate
        }
    };
    if !matches!(predicate.bound, Bound::Copy | Bound::Sized) {
        return Err("unsupported imported declared builtin goal");
    }
    Ok(predicate)
}
fn relations(
    facts: &Declarations,
    payload: &g::Payload,
    function: &g::Function,
    body: &Body,
    obligations: &mut Vec<Obligation>,
    work: &mut super::ImportedOriginals,
) -> Result<(), &'static str> {
    let types = facts
        .crates
        .as_ref()
        .ok_or("missing imported contract linkage")?
        .original_types(payload.owners().owner())?;
    for relation in payload.relations(function.body)? {
        let g::Relation {
            actual, expected, ..
        } = relation;
        work.charge(1)?;
        let crate::body::ir::BodyOrigin::Imported(origin) = body.source else {
            return Err("imported relation requires its actual body owner");
        };
        let catalog = facts
            .crates
            .as_ref()
            .and_then(|c| c.origin_catalog())
            .ok_or("imported relation has no current catalog")?;
        let site = catalog.site(origin.id(), relation.site)?;
        obligations.push(Obligation {
            source: body.source.proof_site(site)?,
            kind: Kind::Coercion {
                actual: types[actual.0 as usize],
                expected: types[expected.0 as usize],
                unsize: false,
                reference: None,
            },
        });
    }
    Ok(())
}
