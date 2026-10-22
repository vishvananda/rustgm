//! A conditional inline proof needs the real original parents, never a source-shaped permit.
use super::*;
use crate::{
    body::{ir::BodyKey, regions::Contract},
    sema::model::Declarations,
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
struct Component {
    facts: Declarations,
    templates: Templates,
    contracts: Vec<Contract>,
}
fn component(source: &str) -> Result<Component> {
    let report = crate::test_source::resolve(source, true)?;
    let context = crate::pipeline::execution::Context::default();
    let facts = crate::sema::declarations::analyze_with_executor(&report, context.executor())?;
    // Inspect the complete original facts before the current single-body inline guard.
    let mut facts = crate::pipeline::analyze(&report, facts)?.facts;
    let registry = facts.inline_constants.as_deref().ok_or("inline registry")?;
    let inline = (0..registry.len())
        .filter_map(|index| registry.at(index))
        .find(|inline| facts.closures.get(inline.parent_body).is_some())
        .ok_or("inline closure child")?;
    let child_source = inline.source;
    let closure = *facts.closures.get(inline.parent_body).ok_or("closure")?;
    let owner = closure.owner.ok_or("closure owner")?;
    let function = *facts
        .function_indices
        .get(&owner)
        .and_then(|&index| facts.functions.get(index))
        .ok_or("actual function owner")?;
    let links = facts.nested_outputs.as_deref().ok_or("output links")?;
    assert_eq!(
        links.get(&child_source).ok_or("child link")?.parent,
        closure.source
    );
    assert_eq!(
        links.get(&closure.source).ok_or("closure link")?.parent,
        function.body.ok_or("actual function body")?,
    );
    let mut prepared = Prepared::default();
    let mut child = prepared.anonymous(&report, &facts, child_source)?;
    let mut child_contract =
        crate::body::regions::Collector::default().expression(&facts, child_source)?;
    crate::body::dispatch::original_anonymous(&mut facts, &mut child.body, &mut child_contract)?;
    let mut templates = Templates {
        bodies: vec![
            child,
            prepared.closure(&report, &facts, &closure, owner)?,
            prepared.function(&report, &facts, &function)?,
        ],
    };
    let mut promotions = crate::body::promote::Scratch::default();
    for template in &mut templates.bodies {
        crate::body::promote::instance_reusing(
            &report,
            &mut facts,
            &mut template.body,
            &template.destruction,
            &mut promotions,
        )?;
    }
    let contracts = crate::body::regions::collect(&facts)?
        .bodies
        .into_iter()
        .filter(|contract| {
            templates
                .bodies
                .iter()
                .any(|t| t.body.key() == contract.key)
        })
        .map(|contract| {
            if contract.key == BodyKey::Anonymous(child_source) {
                child_contract.clone()
            } else {
                contract
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(contracts.len(), 3);
    for contract in &contracts {
        contract.prepare_outlives(&mut facts.types, &mut facts.lifetimes.nominal_requirements)?;
    }
    Ok(Component {
        facts,
        templates,
        contracts,
    })
}
fn verify(component: &Component, templates: &Templates, contracts: Vec<Contract>) -> Result {
    let facts = &component.facts;
    let contracts = crate::body::regions::Contracts {
        bodies: contracts,
        callables: &facts.lifetimes.callables,
        nominal_requirements: &facts.lifetimes.nominal_requirements,
    };
    let custom = facts
        .destructors
        .implementations()
        .iter()
        .map(|i| i.item)
        .collect::<Vec<_>>();
    let mut copies = crate::types::copy::Copies::default();
    let checked = crate::borrow::templates::verify(
        &mut crate::borrow::Environment {
            types: &facts.types,
            definitions: &facts.aggregates,
            derives: &facts.derives,
            copies: &mut copies,
        },
        &facts.lifetimes.variances,
        &contracts,
        templates,
        &custom,
        |ty, field| crate::body::drops::nominal_position(facts, ty, field),
    )?;
    assert_eq!(checked.regions().bodies.len(), templates.bodies.len());
    assert_eq!(checked.loans().len(), templates.bodies.len());
    Ok(())
}
fn fixture(name: &str) -> Result<String> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../pa25/tests/spec/inline-query-context")
        .join(format!("{name}.rs"));
    Ok(std::fs::read_to_string(path)?)
}
#[test]
fn conditional_inline_originals_require_the_exact_complete_parent_component() -> Result {
    for name in ["closure", "paid-lifetime"] {
        let component = component(&fixture(name)?)?;
        verify(
            &component,
            &component.templates,
            component.contracts.clone(),
        )?;
        for missing in [1, 2] {
            let templates = Templates {
                bodies: component
                    .templates
                    .bodies
                    .iter()
                    .enumerate()
                    .filter(|(index, _)| *index != missing)
                    .map(|(_, body)| body.clone())
                    .collect(),
            };
            let error = verify(&component, &templates, component.contracts.clone())
                .err()
                .ok_or("missing parent was accepted")?;
            eprintln!("{name} missing parent {missing}: {error}");
            assert_eq!(
                error.to_string(),
                "nested output requires its checked parent contract"
            );
        }
        let mut contracts = component.contracts.clone();
        let child = contracts
            .iter_mut()
            .find(|c| c.key == component.templates.bodies[0].body.key())
            .ok_or("child")?;
        let link = child.output.as_ref().ok_or("child output")?;
        assert!(link.formation.is_none());
        child.output = Some(Arc::new(crate::sema::nested_outputs::Link {
            source: link.source,
            parent: component.templates.bodies[2].body.source,
            revision: link.revision,
            universals: link.universals,
            bridges: link.bridges.clone(),
            formation: None,
        }));
        let error = verify(&component, &component.templates, contracts)
            .err()
            .ok_or("foreign parent was accepted")?;
        eprintln!("{name} foreign parent: {error}");
        assert_eq!(error.to_string(), "invalid lifetime constraint identity");
    }
    Ok(())
}
#[test]
fn conditional_inline_component_does_not_supply_an_unpaid_outer_lifetime_relation() -> Result {
    let source = fixture("paid-lifetime")?.replace(" where 'a:'b", "");
    let component = component(&source)?;
    let error = verify(
        &component,
        &component.templates,
        component.contracts.clone(),
    )
    .err()
    .ok_or("unpaid relation was accepted")?;
    eprintln!("unpaid outer relation: {error}");
    assert_eq!(
        error.to_string(),
        "required lifetime does not outlive its target"
    );
    Ok(())
}

#[test]
fn context_parents_prove_originals_without_entering_runtime_lookup() -> Result {
    let component = component(&fixture("paid-lifetime")?)?;
    let facts = &component.facts;
    let templates = Templates {
        bodies: vec![component.templates.bodies[0].clone()],
    };
    let contexts = crate::borrow::templates::Contexts {
        bodies: Templates {
            bodies: component.templates.bodies[1..].to_vec(),
        },
        source: Vec::new(),
    };
    let inputs = crate::body::regions::Contracts {
        bodies: component.contracts.clone(),
        callables: &facts.lifetimes.callables,
        nominal_requirements: &facts.lifetimes.nominal_requirements,
    };
    let custom = facts
        .destructors
        .implementations()
        .iter()
        .map(|i| i.item)
        .collect::<Vec<_>>();
    let mut copies = crate::types::copy::Copies::default();
    let mut environment = crate::borrow::Environment {
        types: &facts.types,
        definitions: &facts.aggregates,
        derives: &facts.derives,
        copies: &mut copies,
    };
    let checked = crate::borrow::templates::verify_with_context(
        &mut environment,
        &facts.lifetimes.variances,
        &inputs,
        &templates,
        &contexts,
        &custom,
        |ty, field| crate::body::drops::nominal_position(facts, ty, field),
    )?;
    assert_eq!(checked.regions().bodies.len(), 1);
    assert_eq!(checked.loans().len(), 1);
    for parent in &contexts.bodies.bodies {
        match parent.body.owner {
            crate::body::ir::Owner::Function { definition, .. } => {
                assert!(checked.get(crate::types::ItemId(definition.0)).is_none());
            }
            crate::body::ir::Owner::Closure { identity, .. } => {
                assert!(checked.closure(identity).is_none());
            }
            _ => return Err("unexpected original parent".into()),
        }
    }
    let mut caller = crate::types::relations::Constraints::new(
        &facts.types,
        &facts.lifetimes.variances,
        2,
        10_000,
    )?;
    caller.outlives(
        crate::types::RegionId::bound(0, 0)?,
        crate::types::RegionId::bound(0, 1)?,
        0,
    )?;
    assert!(
        caller.solve().is_err(),
        "original parent proof cannot pay another caller's debt"
    );
    drop(checked);
    let mut duplicated = crate::borrow::templates::Contexts {
        bodies: Templates {
            bodies: contexts.bodies.bodies.clone(),
        },
        source: Vec::new(),
    };
    duplicated.bodies.bodies.push(templates.bodies[0].clone());
    assert!(
        crate::borrow::templates::verify_with_context(
            &mut environment,
            &facts.lifetimes.variances,
            &inputs,
            &templates,
            &duplicated,
            &custom,
            |ty, field| crate::body::drops::nominal_position(facts, ty, field)
        )
        .is_err()
    );
    Ok(())
}

#[test]
fn provider_originals_feed_actual_parent_proof_before_execution() -> Result {
    for name in ["closure", "paid-lifetime"] {
        let mut report = crate::test_source::resolve(&fixture(name)?, true)?;
        let analysis = crate::pipeline::analyze_source(&report)?;
        assert!(analysis.prepared.original_contexts.is_some());
        let mut facts = analysis.facts;
        let entry = crate::sema::entry::select(&mut report, &facts)?;
        crate::lower::instances::build_prepared(&report, &mut facts, entry, analysis.prepared)?;
        // Original context-only rows still cannot pay a foreign caller: the
        // neighboring invariant checks that independently of this full demand.
    }
    let report =
        crate::test_source::resolve(&fixture("paid-lifetime")?.replace(" where 'a:'b", ""), true)?;
    let error = crate::pipeline::analyze_source(&report)
        .err()
        .ok_or("unpaid provider parent was accepted")?;
    assert!(
        error
            .to_string()
            .contains("required lifetime does not outlive its target"),
        "{error}"
    );
    Ok(())
}

#[test]
fn source_and_provider_component_debts_are_both_mandatory() -> Result {
    let component = component(&fixture("paid-lifetime")?)?;
    let facts = &component.facts;
    let templates = Templates {
        bodies: vec![component.templates.bodies[0].clone()],
    };
    for side in 0..2 {
        let mut contexts = crate::borrow::templates::Contexts {
            bodies: Templates {
                bodies: component.templates.bodies[1..].to_vec(),
            },
            source: component.contracts.clone(),
        };
        let mut inputs = crate::body::regions::Contracts {
            bodies: component.contracts.clone(),
            callables: &facts.lifetimes.callables,
            nominal_requirements: &facts.lifetimes.nominal_requirements,
        };
        let rows = if side == 0 {
            &mut contexts.source
        } else {
            &mut inputs.bodies
        };
        let parent = rows
            .iter_mut()
            .find(|row| row.key == component.templates.bodies[2].body.key())
            .ok_or("function contract")?;
        parent.obligations.push(crate::body::regions::Obligation {
            source: parent.source,
            kind: crate::body::regions::Kind::Outlives((
                crate::types::RegionId::bound(0, 0)?,
                crate::types::RegionId::STATIC,
            )),
        });
        let mut copies = crate::types::copy::Copies::default();
        let custom = facts
            .destructors
            .implementations()
            .iter()
            .map(|i| i.item)
            .collect::<Vec<_>>();
        let error = crate::borrow::templates::verify_with_context(
            &mut crate::borrow::Environment {
                types: &facts.types,
                definitions: &facts.aggregates,
                derives: &facts.derives,
                copies: &mut copies,
            },
            &facts.lifetimes.variances,
            &inputs,
            &templates,
            &contexts,
            &custom,
            |ty, field| crate::body::drops::nominal_position(facts, ty, field),
        )
        .err()
        .ok_or("component debt was erased")?;
        assert_eq!(
            error.to_string(),
            "required lifetime does not outlive its target"
        );
    }
    Ok(())
}
